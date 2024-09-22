use std::time::Duration;

use adw::prelude::*;
use power_daemon::{FirmwareInfo, FirmwareSettings, Profile};
use relm4::{
    binding::{Binding, U32Binding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::StringListBinding,
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum FirmwareInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for FirmwareInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct FirmwareGroup {
    initialized: bool,

    settings: Option<FirmwareSettings>,
    info: Option<FirmwareInfo>,

    has_platform_profile: bool,

    platform_profile: U32Binding,
    available_platform_profiles: StringListBinding,

    last_firmware_settings: Option<FirmwareSettings>,
    active_profile: Option<(usize, Profile)>,
}

impl FirmwareGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_firmware_settings_and_info(&mut self) {
        assert!(self.settings.is_some() && self.info.is_some());

        let info = self.info.clone().unwrap();
        let settings = self.settings.clone().unwrap();

        if let Some(ref profiles) = info.platform_profiles {
            self.has_platform_profile = true;

            *self.available_platform_profiles.guard() =
                gtk::StringList::new(&profiles.iter().map(|p| p.as_str()).collect::<Vec<_>>());
            *self.platform_profile.guard() = profiles
                .iter()
                .position(|p| {
                    *p == *settings
                        .platform_profile
                        .as_ref()
                        .unwrap_or(&"balanced".to_string())
                })
                .unwrap() as u32;
        } else {
            self.has_platform_profile = false;
        }

        self.initialized = true;
    }

    fn to_firmware_settings(&self) -> FirmwareSettings {
        FirmwareSettings {
            platform_profile: Some(
                self.available_platform_profiles
                    .value()
                    .string(self.platform_profile.value())
                    .unwrap()
                    .into(),
            ),
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for FirmwareGroup {
    type Input = FirmwareInput;

    type Output = AppInput;

    type Init = ();

    view! {
        gtk::Box {
            set_homogeneous: true,
            set_expand: true,
            if !model.initialized {
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
                    set_title: "Firmware settings",
                    adw::PreferencesGroup {
                        adw::ComboRow {
                            set_title: labels::ACPI_PLATFORM_PROFILE_TITLE,
                            #[watch]
                            set_sensitive: model.has_platform_profile,
                            #[watch]
                            set_tooltip_text: if !model.has_platform_profile {
                                Some(labels::ACPI_PLATFORM_PROFILE_MISSING_TT)
                            } else {
                                Some(labels::ACPI_PLATFORM_PROFILE_TT)
                            },
                            add_binding: (&model.platform_profile, "selected"),
                            add_binding: (&model.available_platform_profiles, "model"),
                            connect_selected_item_notify => FirmwareInput::Changed,
                        }
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
        let model = FirmwareGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FirmwareInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.settings = profiles_info
                                .get_active_profile()
                                .firmware_settings
                                .clone()
                                .into();
                            self.initialized = false;
                        }
                    }

                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.info = system_info.firmware_info.clone().into();
                        }
                    }

                    if !self.initialized && self.settings.is_some() && self.info.is_some() {
                        self.from_firmware_settings_and_info();
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(10.0),
                    system_info::SystemInfoSyncType::Firmware,
                ),
                RootRequest::Apply => {
                    if !self.initialized {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.firmware_settings = self.to_firmware_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Firmware,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            FirmwareInput::Changed => {
                if let Some(ref last_settings) = self.last_firmware_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_firmware_settings(),
                            crate::SettingsGroup::Firmware,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
