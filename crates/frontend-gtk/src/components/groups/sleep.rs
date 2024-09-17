use std::time::Duration;

use adw::prelude::*;
use power_daemon::{OptionalFeaturesInfo, Profile, SleepSettings};
use relm4::{
    binding::{Binding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    AppInput, AppSyncUpdate, RootRequest,
};

use lazy_static::lazy_static;

lazy_static! {
    static ref SCREEN_TURN_OFF_TIMES: Vec<Option<u32>> = vec![
        None,
        Some(1),
        Some(5),
        Some(10),
        Some(15),
        Some(20),
        Some(30),
        Some(45),
        Some(60),
        Some(90),
        Some(180),
        Some(270),
    ];
    static ref SCREEN_TURN_OFF_LABELS: Vec<&'static str> = vec![
        "Never",
        "1 Minute",
        "5 Minutes",
        "10 Minutes",
        "15 Minutes",
        "20 Minutes",
        "30 Minutes",
        "45 Minutes",
        "1 hour",
        "1.5 hours",
        "2 hours",
        "3 hours",
    ];
    static ref SUSPEND_TIMES: Vec<Option<u32>> = vec![
        None,
        Some(1),
        Some(5),
        Some(10),
        Some(15),
        Some(20),
        Some(30),
        Some(45),
        Some(60),
    ];
    static ref SUSPEND_LABELS: Vec<&'static str> = vec![
        "Never",
        "1 Minute",
        "5 Minutes",
        "10 Minutes",
        "15 Minutes",
        "20 Minutes",
        "30 Minutes",
        "45 Minutes",
        "1 hour",
    ];
}

#[derive(Debug, Clone)]
pub enum SleepInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for SleepInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[tracker::track]
#[derive(Debug, Default)]
pub struct SleepGroup {
    settings_obtained: bool,

    supports_xautolock: bool,
    supports_xset: bool,

    #[do_not_track]
    suspend: U32Binding,
    #[do_not_track]
    turn_off_screen: U32Binding,

    #[do_not_track]
    last_sleep_settings: Option<SleepSettings>,
    #[do_not_track]
    active_profile: Option<(usize, Profile)>,
}

impl SleepGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_sleep_settings(&mut self, sleep_settings: &SleepSettings) {
        *self.suspend.guard() = Self::transform_time(sleep_settings.suspend_after, &SUSPEND_TIMES);
        *self.turn_off_screen.guard() =
            Self::transform_time(sleep_settings.turn_off_screen_after, &SCREEN_TURN_OFF_TIMES);
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_opt_info(&mut self, opt_info: &OptionalFeaturesInfo) {
        self.set_supports_xautolock(opt_info.supports_xautolock);
        self.set_supports_xset(opt_info.supports_xset);
    }

    fn to_sleep_settings(&self) -> SleepSettings {
        SleepSettings {
            suspend_after: SUSPEND_TIMES[self.suspend.value() as usize],
            turn_off_screen_after: SCREEN_TURN_OFF_TIMES[self.turn_off_screen.value() as usize],
        }
    }

    fn transform_time(value: Option<u32>, items: &[Option<u32>]) -> u32 {
        assert!(items.contains(&None));

        if let Some(idx) = items.iter().position(|v| *v == value) {
            return idx as u32;
        } else {
            // value cannot be none as that is within items
            let time = value.unwrap();
            if time == 0 {
                return 0;
            } else if time >= items.last().unwrap().unwrap() {
                return (items.len() - 1) as u32;
            } else {
                for (idx, val) in items.iter().skip(1).enumerate() {
                    let val = val.unwrap();
                    if time <= val {
                        return (idx + 1) as u32;
                    }
                }
            }
        }

        unreachable!();
    }
}

#[relm4::component(pub)]
impl SimpleComponent for SleepGroup {
    type Input = SleepInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_homogeneous: true,
            set_expand: true,
            if !model.settings_obtained {
                gtk::Box {
                    set_align: gtk::Align::Center,
                    gtk::Label::new(Some("Connecting to the daemon...")),
                    gtk::Spinner {
                        set_spinning: true,
                        set_visible: true,
                    }
                }
            } else {
                adw::PreferencesPage {
                    set_expand: true,
                    set_title: "Sleep settings",
                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: labels::SCREEN_TURN_OFF_TITLE,
                            #[track(model.changed(SleepGroup::supports_xset()))]
                            set_sensitive: model.supports_xset,
                            #[track(model.changed(SleepGroup::supports_xset()))]
                            set_tooltip_text: if !model.supports_xset {
                                Some(labels::SCREEN_TURN_OFF_XSET_MISSING)
                            } else {
                                None
                            },
                            set_model: Some(&gtk::StringList::new(&SCREEN_TURN_OFF_LABELS)),
                            add_binding: (&model.turn_off_screen, "selected"),
                            connect_selected_item_notify => SleepInput::Changed,
                        },
                        adw::ComboRow {
                            set_title: labels::SUSPEND_TITLE,
                            #[track(model.changed(SleepGroup::supports_xautolock()))]
                            set_sensitive: model.supports_xautolock,
                            #[track(model.changed(SleepGroup::supports_xautolock()))]
                            set_tooltip_text: if !model.supports_xautolock {
                                Some(labels::SUSPEND_XAUTOLOCK_MISSING)
                            } else {
                                None
                            },
                            set_model: Some(&gtk::StringList::new(&SUSPEND_LABELS)),
                            add_binding: (&model.suspend, "selected"),
                            connect_selected_item_notify => SleepInput::Changed,
                        },
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SleepGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SleepInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_sleep_settings(&profile.sleep_settings);
                            self.settings_obtained = true;
                            self.last_sleep_settings = Some(self.to_sleep_settings());
                        }
                    }
                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.from_opt_info(&system_info.opt_features_info);
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(10.0),
                    system_info::SystemInfoSyncType::Opt,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.sleep_settings = self.to_sleep_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Sleep,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            SleepInput::Changed => {
                if let Some(ref last_settings) = self.last_sleep_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_sleep_settings(),
                            crate::SettingsGroup::Sleep,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
