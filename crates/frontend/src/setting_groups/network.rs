use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{NetworkSettings, ProfilesInfo, ReducedUpdate};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::{ToggleableBool, ToggleableInt};
use crate::helpers::{ToggleableNumericField, ToggleableToggle};

#[derive(Debug, Clone, PartialEq, Default)]
struct NetworkForm {
    pub disable_ethernet: ToggleableBool,

    pub disable_wifi_7: ToggleableBool,
    pub disable_wifi_6: ToggleableBool,
    pub disable_wifi_5: ToggleableBool,

    pub enable_power_save: ToggleableBool,
    pub enable_uapsd: ToggleableBool,

    pub power_level: ToggleableInt,
    pub power_scheme: ToggleableInt,
}

impl NetworkForm {
    pub fn new(network_settings: &NetworkSettings) -> NetworkForm {
        let mut ret = NetworkForm::default();
        ret.set_values(network_settings);
        ret
    }

    pub fn set_values(&mut self, network_settings: &NetworkSettings) {
        self.disable_ethernet
            .from(network_settings.disable_ethernet);

        self.disable_wifi_7.from(network_settings.disable_wifi_7);
        self.disable_wifi_6.from(network_settings.disable_wifi_6);
        self.disable_wifi_5.from(network_settings.disable_wifi_5);

        self.enable_power_save
            .from(network_settings.enable_power_save);
        self.enable_uapsd.from(network_settings.enable_uapsd);

        self.power_level.from_u8(network_settings.power_level);
        self.power_scheme.from_u8(network_settings.power_scheme);
    }
}

#[component]
pub fn NetworkGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let network_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .network_settings
        .clone();

    let mut form_used_settings = use_signal(|| network_settings.clone());
    let mut form = use_hook(|| NetworkForm::new(&network_settings));

    if form_used_settings() != network_settings {
        form.set_values(&network_settings);
        form_used_settings.set(network_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().unwrap();
        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.network_settings = NetworkSettings {
            disable_ethernet: form.disable_ethernet.into_base(),

            disable_wifi_7: form.disable_wifi_7.into_base(),
            disable_wifi_6: form.disable_wifi_6.into_base(),
            disable_wifi_5: form.disable_wifi_5.into_base(),

            enable_power_save: form.enable_power_save.into_base(),
            enable_uapsd: form.enable_uapsd.into_base(),

            power_level: form.power_level.into_u8(),
            power_scheme: form.power_scheme.into_u8(),
        };

        control_routine.send((
            ControlAction::SetReducedUpdate(ReducedUpdate::Network),
            Some(awaiting_completion),
        ));
        control_routine.send((
            ControlAction::UpdateProfile(active_profile_idx as u32, active_profile),
            Some(awaiting_completion),
        ));
        control_routine.send((ControlAction::GetProfilesInfo, Some(awaiting_completion)));
    };

    rsx! {
        form {
            onchange: move |_| {
                changed.set(true);
            },
            onsubmit: move |_| {
                onsubmit();
                changed.set(false);
            },
            div { class: "option-group",
                div { class: "option",
                    ToggleableToggle { name: "Disable ethernet", value: form.disable_ethernet }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableToggle { name: "Disable WiFi 7", value: form.disable_wifi_7 }
                }
                div { class: "option",
                    ToggleableToggle { name: "Disable WiFi 6", value: form.disable_wifi_6 }
                }
                div { class: "option",
                    ToggleableToggle { name: "Disable WiFi 5", value: form.disable_wifi_5 }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableToggle { name: "Enable power save", value: form.enable_power_save }
                }
                div { class: "option",
                    ToggleableToggle { name: "Enable U-APSD", value: form.enable_uapsd }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableNumericField { name: "Power level (0-5)", value: form.power_level }
                }
                div { class: "option",
                    ToggleableNumericField { name: "Power scheme (1-3)", value: form.power_scheme }
                }
            }

            div { class: "confirm-buttons",
                button {
                    r#type: "submit",
                    disabled: !changed.cloned() || awaiting_completion(),
                    if awaiting_completion() {
                        div { class: "spinner" }
                    }
                    label { "Apply" }
                }
                input {
                    onclick: move |_| {
                        form.set_values(&network_settings);
                        changed.set(false);
                    },
                    r#type: "button",
                    value: "Cancel"
                }
            }

            br {}
            br {}
            br {}
        }
    }
}
