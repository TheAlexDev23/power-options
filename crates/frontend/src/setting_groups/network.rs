use dioxus::prelude::*;
use power_daemon::{NetworkSettings, ProfilesInfo, ReducedUpdate};

use crate::communication_services::{ControlAction, ControlRoutine};
use crate::helpers::{ToggleableNumericField, ToggleableToggle};

use super::{ToggleableBool, ToggleableInt};

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
            .0
            .set(network_settings.disable_ethernet.is_some());
        self.disable_ethernet
            .1
            .set(network_settings.disable_ethernet.unwrap_or_default());

        self.disable_wifi_7
            .0
            .set(network_settings.disable_wifi_7.is_some());
        self.disable_wifi_7
            .1
            .set(network_settings.disable_wifi_7.unwrap_or_default());

        self.disable_wifi_6
            .0
            .set(network_settings.disable_wifi_6.is_some());
        self.disable_wifi_6
            .1
            .set(network_settings.disable_wifi_6.unwrap_or_default());

        self.disable_wifi_5
            .0
            .set(network_settings.disable_wifi_5.is_some());
        self.disable_wifi_5
            .1
            .set(network_settings.disable_wifi_5.unwrap_or_default());

        self.enable_power_save
            .0
            .set(network_settings.enable_power_save.is_some());
        self.enable_power_save
            .1
            .set(network_settings.enable_power_save.unwrap_or_default());

        self.enable_uapsd
            .0
            .set(network_settings.enable_uapsd.is_some());
        self.enable_uapsd
            .1
            .set(network_settings.enable_uapsd.unwrap_or_default());

        self.power_level
            .0
            .set(network_settings.power_level.is_some());
        self.power_level
            .1
            .set(network_settings.power_level.unwrap_or_default() as i32);

        self.power_scheme
            .0
            .set(network_settings.power_scheme.is_some());
        self.power_scheme
            .1
            .set(network_settings.power_scheme.unwrap_or_default() as i32);
    }
}

#[component]
pub fn NetworkGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
) -> Element {
    if profiles_info.read().is_none() {
        return rsx! { "Connecting to daemon.." };
    }

    let network_settings = profiles_info
        .read()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .network_settings
        .clone();

    let mut form_used_settings = use_signal(|| network_settings.clone());
    let mut form = use_hook(|| NetworkForm::new(&network_settings));

    if *form_used_settings.read() != network_settings {
        form.set_values(&network_settings);
        form_used_settings.set(network_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let active_profile_idx = profiles_info.read().as_ref().unwrap().active_profile;
        let mut active_profile = profiles_info
            .read()
            .as_ref()
            .unwrap()
            .get_active_profile()
            .clone();

        active_profile.network_settings = NetworkSettings {
            disable_ethernet: if form.disable_ethernet.0.cloned() {
                Some(form.disable_ethernet.1.cloned())
            } else {
                None
            },
            disable_wifi_7: if form.disable_wifi_7.0.cloned() {
                Some(form.disable_wifi_7.1.cloned())
            } else {
                None
            },
            disable_wifi_6: if form.disable_wifi_6.0.cloned() {
                Some(form.disable_wifi_6.1.cloned())
            } else {
                None
            },
            disable_wifi_5: if form.disable_wifi_5.0.cloned() {
                Some(form.disable_wifi_5.1.cloned())
            } else {
                None
            },
            enable_power_save: if form.enable_power_save.0.cloned() {
                Some(form.enable_power_save.1.cloned())
            } else {
                None
            },
            power_level: if form.power_level.0.cloned() {
                Some(form.power_level.1.cloned() as u8)
            } else {
                None
            },
            power_scheme: if form.power_scheme.0.cloned() {
                Some(form.power_scheme.1.cloned() as u8)
            } else {
                None
            },
            enable_uapsd: if form.enable_uapsd.0.cloned() {
                Some(form.enable_uapsd.1.cloned())
            } else {
                None
            },
        };

        control_routine.send((
            ControlAction::SetReducedUpdate(ReducedUpdate::Network),
            Some(awaiting_completion),
        ));
        control_routine.send((
            ControlAction::UpdateProfile(active_profile_idx as u32, active_profile),
            Some(awaiting_completion),
        ));
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
                    disabled: !changed.cloned() || *awaiting_completion.read(),
                    if *awaiting_completion.read() {
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
        }
    }
}
