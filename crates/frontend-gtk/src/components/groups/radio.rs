use std::time::Duration;

use adw::prelude::*;
use power_daemon::{Profile, RadioSettings};
use relm4::{
    binding::{Binding, BoolBinding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    AppInput, AppSyncUpdate,
};

#[derive(Debug, Clone)]
pub enum RadioInput {
    Sync(AppSyncUpdate),
    ReactivityUpdate,
    ConfigureSysinfo,
    Changed,
    Apply,
}

impl From<AppSyncUpdate> for RadioInput {
    fn from(value: AppSyncUpdate) -> Self {
        Self::Sync(value)
    }
}

#[derive(Debug, Default)]
pub struct RadioGroup {
    settings_obtained: bool,

    block_wifi: BoolBinding,
    block_nfc: BoolBinding,
    block_bt: BoolBinding,

    last_radio_settings: Option<RadioSettings>,
    active_profile: Option<(usize, Profile)>,
}

impl RadioGroup {
    fn from_radio_settings(&mut self, radio_settings: &RadioSettings) {
        *self.block_wifi.guard() = radio_settings.block_wifi.unwrap_or_default();
        *self.block_nfc.guard() = radio_settings.block_nfc.unwrap_or_default();
        *self.block_bt.guard() = radio_settings.block_bt.unwrap_or_default();
    }

    fn to_radio_settings(&self) -> RadioSettings {
        RadioSettings {
            block_wifi: self.block_wifi.value().into(),
            block_nfc: self.block_nfc.value().into(),
            block_bt: self.block_bt.value().into(),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for RadioGroup {
    type Input = RadioInput;

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
                    set_title: "Radio settings",
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Disable WiFi",
                            add_binding: (&model.block_wifi, "active"),
                            connect_active_notify => RadioInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Disable NFC",
                            add_binding: (&model.block_nfc, "active"),
                            connect_active_notify => RadioInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Disable Bluetooth",
                            add_binding: (&model.block_bt, "active"),
                            connect_active_notify => RadioInput::Changed,
                        },
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RadioGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RadioInput::ReactivityUpdate => {}
            RadioInput::Changed => {
                if let Some(ref last_settings) = self.last_radio_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_radio_settings(),
                        ))
                        .unwrap()
                }
            }
            RadioInput::Sync(message) => {
                if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                    if let Some(profiles_info) = profiles_info.as_ref() {
                        let profile = profiles_info.get_active_profile();
                        self.active_profile = Some((profiles_info.active_profile, profile.clone()));
                        self.from_radio_settings(&profile.radio_settings);
                        self.settings_obtained = true;
                        self.last_radio_settings = Some(self.to_radio_settings());
                    }
                }
            }
            RadioInput::Apply => {
                if !(self.settings_obtained && self.active_profile.is_some()) {
                    return;
                }

                sender.output(AppInput::SetUpdating(true)).unwrap();

                let mut active_profile = self.active_profile.clone().unwrap();
                active_profile.1.radio_settings = self.to_radio_settings();

                tokio::spawn(async move {
                    daemon_control::set_reduced_update(power_daemon::ReducedUpdate::Radio).await;

                    daemon_control::update_profile(active_profile.0 as u32, active_profile.1).await;

                    daemon_control::get_profiles_info().await;

                    sender.output(AppInput::SetUpdating(false)).unwrap();
                });
            }
            RadioInput::ConfigureSysinfo => system_info::set_system_info_sync(
                Duration::from_secs_f32(10.0),
                system_info::SystemInfoSyncType::None,
            ),
        }
    }
}
