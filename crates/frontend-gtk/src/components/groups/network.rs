use std::time::Duration;

use adw::prelude::*;
use power_daemon::{NetworkSettings, Profile};
use relm4::{
    binding::{Binding, BoolBinding},
    prelude::*,
    RelmObjectExt,
};

use crate::{
    communications::{daemon_control, system_info},
    helpers::extra_bindings::AdjustmentBinding,
    AppInput, AppSyncUpdate, RootRequest,
};

#[derive(Debug, Clone)]
pub enum NetworkInput {
    RootRequest(RootRequest),
    Changed,
}

impl From<RootRequest> for NetworkInput {
    fn from(value: RootRequest) -> Self {
        Self::RootRequest(value)
    }
}

#[derive(Debug, Default)]
pub struct NetworkGroup {
    settings_obtained: bool,

    disable_ethernet: BoolBinding,

    disable_wifi_7: BoolBinding,
    disable_wifi_6: BoolBinding,
    disable_wifi_5: BoolBinding,

    enable_power_save: BoolBinding,
    enable_uapsd: BoolBinding,

    power_level: AdjustmentBinding,
    power_scheme: AdjustmentBinding,

    last_network_settings: Option<NetworkSettings>,
    active_profile: Option<(usize, Profile)>,

    supports_wifi_drivers: bool,
    supports_ifconfig: bool,
}

impl NetworkGroup {
    #[allow(clippy::wrong_self_convention)]
    fn from_network_settings(&mut self, network_settings: &NetworkSettings) {
        *self.disable_ethernet.guard() = network_settings.disable_ethernet.unwrap_or_default();
        *self.disable_wifi_7.guard() = network_settings.disable_wifi_7.unwrap_or_default();
        *self.disable_wifi_6.guard() = network_settings.disable_wifi_6.unwrap_or_default();
        *self.disable_wifi_5.guard() = network_settings.disable_wifi_5.unwrap_or_default();
        *self.enable_power_save.guard() = network_settings.enable_power_save.unwrap_or_default();
        *self.enable_uapsd.guard() = network_settings.enable_uapsd.unwrap_or_default();

        let power_scheme = self.power_scheme.guard();
        power_scheme.set_upper(3.0);
        power_scheme.set_lower(1.0);
        power_scheme.set_step_increment(1.0);
        power_scheme.set_value(network_settings.power_scheme.unwrap_or_default() as f64);

        let power_level = self.power_level.guard();
        power_level.set_upper(5.0);
        power_level.set_lower(0.0);
        power_level.set_step_increment(1.0);
        power_level.set_value(network_settings.power_level.unwrap_or_default() as f64);
    }

    fn to_network_settings(&self) -> NetworkSettings {
        NetworkSettings {
            disable_ethernet: if self.supports_ifconfig {
                self.disable_ethernet.value().into()
            } else {
                None
            },
            disable_wifi_7: if self.supports_wifi_drivers {
                self.disable_wifi_7.value().into()
            } else {
                None
            },
            disable_wifi_6: if self.supports_wifi_drivers {
                self.disable_wifi_6.value().into()
            } else {
                None
            },
            disable_wifi_5: if self.supports_wifi_drivers {
                self.disable_wifi_5.value().into()
            } else {
                None
            },
            enable_power_save: if self.supports_wifi_drivers {
                self.enable_power_save.value().into()
            } else {
                None
            },
            enable_uapsd: if self.supports_wifi_drivers {
                self.enable_uapsd.value().into()
            } else {
                None
            },
            power_level: if self.supports_wifi_drivers {
                (self.power_level.value().value() as u8).into()
            } else {
                None
            },
            power_scheme: if self.supports_wifi_drivers {
                (self.power_scheme.value().value() as u8).into()
            } else {
                None
            },
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for NetworkGroup {
    type Input = NetworkInput;

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
                    set_title: "Network settings",
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: labels::DIS_ETH_TITLE,
                            #[watch]
                            set_sensitive: model.supports_ifconfig,
                            #[watch]
                            set_tooltip_text: if !model.supports_ifconfig {
                                Some(labels::NO_IFCONFIG_TT)
                            } else {
                                Some(labels::DIS_ETH_TT)
                            },
                            add_binding: (&model.disable_ethernet, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: "Disable WiFi 7",
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.disable_wifi_7, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                        adw::SwitchRow {
                            set_title: "Disable WiFi 6",
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.disable_wifi_6, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                        adw::SwitchRow {
                            set_title: "Disable WiFi 5",
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                None
                            },
                            add_binding: (&model.disable_wifi_5, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::SwitchRow {
                            set_title: labels::IWLWIFI_POWERSAVING_TITLE,
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                Some(labels::IWLWIFI_POWERSAVING_TT)
                            },
                            add_binding: (&model.enable_power_save, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                        adw::SwitchRow {
                            set_title: labels::UAPSD_TITLE,
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                Some(labels::UAPSD_TT)
                            },
                            add_binding: (&model.enable_uapsd, "active"),
                            connect_active_notify => NetworkInput::Changed,
                        },
                    },
                    adw::PreferencesGroup {
                        adw::SpinRow {
                            set_title: labels::WIFI_POWERLEVEL_TITLE,
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                Some(labels::WIFI_POWERLEVEL_TT)
                            },
                            add_binding: (&model.power_level, "adjustment"),
                            connect_value_notify => NetworkInput::Changed,
                        },
                        adw::SpinRow {
                            set_title: labels::WIFI_POWERSCHEME_TITLE,
                            #[watch]
                            set_sensitive: model.supports_wifi_drivers,
                            #[watch]
                            set_tooltip_text: if !model.supports_wifi_drivers {
                                Some(labels::NO_WIFI_DRIVER_TT)
                            } else {
                                Some(labels::WIFI_POWERSCHEME_TT)
                            },
                            add_binding: (&model.power_scheme, "adjustment"),
                            connect_value_notify => NetworkInput::Changed,
                        }
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
        let model = NetworkGroup::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NetworkInput::RootRequest(request) => match request {
                RootRequest::ReactToUpdate(message) => {
                    if let AppSyncUpdate::ProfilesInfo(ref profiles_info) = message {
                        if let Some(profiles_info) = profiles_info.as_ref() {
                            let profile = profiles_info.get_active_profile();
                            self.active_profile =
                                Some((profiles_info.active_profile, profile.clone()));
                            self.from_network_settings(&profile.network_settings);
                            self.settings_obtained = true;
                            self.last_network_settings = Some(self.to_network_settings());
                        }
                    }
                    if let AppSyncUpdate::SystemInfo(ref system_info) = message {
                        if let Some(system_info) = system_info.as_ref() {
                            self.supports_ifconfig =
                                system_info.opt_features_info.supports_ifconfig;
                            self.supports_wifi_drivers =
                                system_info.opt_features_info.supports_wifi_drivers;
                        }
                    }
                }
                RootRequest::ConfigureSystemInfoSync => system_info::set_system_info_sync(
                    Duration::from_secs_f32(15.0),
                    system_info::SystemInfoSyncType::Opt,
                ),
                RootRequest::Apply => {
                    if !(self.settings_obtained && self.active_profile.is_some()) {
                        return;
                    }

                    sender.output(AppInput::SetUpdating(true)).unwrap();

                    let mut active_profile = self.active_profile.clone().unwrap();
                    active_profile.1.network_settings = self.to_network_settings();

                    tokio::spawn(async move {
                        daemon_control::update_profile_reduced(
                            active_profile.0 as u32,
                            active_profile.1,
                            power_daemon::ReducedUpdate::Network,
                        )
                        .await;

                        daemon_control::get_profiles_info().await;

                        sender.output(AppInput::SetUpdating(false)).unwrap();
                    });
                }
            },
            NetworkInput::Changed => {
                if let Some(ref last_settings) = self.last_network_settings {
                    sender
                        .output(AppInput::SetChanged(
                            *last_settings != self.to_network_settings(),
                            crate::SettingsGroup::Network,
                        ))
                        .unwrap()
                }
            }
        }
    }
}
