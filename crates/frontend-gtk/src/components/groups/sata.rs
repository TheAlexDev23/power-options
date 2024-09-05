use std::time::Duration;

use adw::prelude::*;
use power_daemon::{Profile, SATASettings};
use relm4::{
    binding::{Binding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

lazy_static::lazy_static! {
    static ref SATA_POLICIES: Vec<&'static str> = vec!["max_performance", "medium_power", "med_power_with_dipm", "min_power"];
}

use crate::{
    communications::{daemon_control, system_info},
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum SATAInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for SATAInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct SATAGroup {
    settings_obtained: bool,

    active_link_pm_policy: U32Binding,

    last_sata_settings: Option<SATASettings>,
    active_profile: Option<(usize, Profile)>,
}

impl SATAGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_sata_settings(&mut self, sata_settings: &SATASettings) {
        *self.active_link_pm_policy.guard() = SATA_POLICIES
            .iter()
            .position(|v| *v == sata_settings.active_link_pm_policy.as_ref().unwrap())
            .unwrap() as u32;
    }

    fn to_sata_settings(&self) -> SATASettings {
        SATASettings {
            active_link_pm_policy: SATA_POLICIES
                .get(self.active_link_pm_policy.value() as usize)
                .unwrap()
                .to_string()
                .into(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for SATAGroup {
    type Input = SATAInput;

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
                    set_title: "SATA settings",
                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: labels::SATA_ACTIVE_LINK_TITLE,
                            set_tooltip_text: Some(labels::SATA_ACTIVE_LINK_TT),
                            set_model: Some(&gtk::StringList::new(&SATA_POLICIES)),
                            add_binding: (&model.active_link_pm_policy, "selected"),
                            connect_selected_item_notify => SATAInput::Changed,
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
        let model = SATAGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SATAInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_sata_settings(&profile.sata_settings);
                            self.settings_obtained = true;
                            self.last_sata_settings = Some(self.to_sata_settings());
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(10.0),
                    system_info::SystemInfoSyncType::None,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.sata_settings = self.to_sata_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::SATA,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            SATAInput::Changed => {
                if let Some(ref last_settings) = self.last_sata_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_sata_settings(),
                            crate::SettingsGroup::SATA,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
