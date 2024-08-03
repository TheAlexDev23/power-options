use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, RadioSettings, ReducedUpdate};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::ToggleableToggle;

use super::ToggleableBool;

#[derive(Default, PartialEq, Clone, Debug)]
struct RadioForm {
    pub nfc: ToggleableBool,
    pub wifi: ToggleableBool,
    pub bt: ToggleableBool,
}

impl RadioForm {
    pub fn new(radio_settings: &RadioSettings) -> RadioForm {
        let mut form = RadioForm::default();
        form.set_values(radio_settings);
        form
    }

    pub fn set_values(&mut self, radio_settings: &RadioSettings) {
        self.nfc.0.set(radio_settings.block_nfc.is_some());
        self.nfc.1.set(radio_settings.block_nfc.unwrap_or_default());

        self.wifi.0.set(radio_settings.block_wifi.is_some());
        self.wifi
            .1
            .set(radio_settings.block_wifi.unwrap_or_default());

        self.bt.0.set(radio_settings.block_bt.is_some());
        self.bt.1.set(radio_settings.block_bt.unwrap_or_default());
    }
}

#[component]
pub fn RadioGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info.read().is_none() {
        return rsx! { "Connecting to daemon..." };
    }

    let radio_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .radio_settings
        .clone();

    let mut form_used_settings = use_signal(|| radio_settings.clone());
    let mut form = use_hook(|| RadioForm::new(&radio_settings));
    if form_used_settings() != radio_settings {
        form.set_values(&radio_settings);
        form_used_settings.set(radio_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let active_profile_idx = profiles_info().as_ref().unwrap().active_profile;
        let mut active_profile = profiles_info()
            .as_ref()
            .unwrap()
            .get_active_profile()
            .clone();

        active_profile.radio_settings = RadioSettings {
            block_nfc: if form.nfc.0.cloned() {
                Some(form.nfc.1.cloned())
            } else {
                None
            },
            block_wifi: if form.wifi.0.cloned() {
                Some(form.wifi.1.cloned())
            } else {
                None
            },
            block_bt: if form.bt.0.cloned() {
                Some(form.bt.1.cloned())
            } else {
                None
            },
        };
        control_routine.send((
            ControlAction::SetReducedUpdate(ReducedUpdate::Radio),
            Some(awaiting_completion),
        ));
        control_routine.send((
            ControlAction::UpdateProfile(active_profile_idx as u32, active_profile),
            Some(awaiting_completion),
        ));
        control_routine.send((ControlAction::GetProfilesInfo, Some(awaiting_completion)));
    };

    let var_name = rsx! {
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
                    ToggleableToggle { name: "Block NFC", value: form.nfc }
                }
                div { class: "option",
                    ToggleableToggle { name: "Block WiFi", value: form.wifi }
                }
                div { class: "option",
                    ToggleableToggle { name: "Block Bluetooth", value: form.bt }
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
                        form.set_values(&radio_settings);
                        changed.set(false);
                    },
                    r#type: "button",
                    value: "Cancel"
                }
            }
        }
    };
    var_name
}
