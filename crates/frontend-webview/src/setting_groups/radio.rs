use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, RadioSettings, ReducedUpdate};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::toggleable_components::ToggleableToggle;
use crate::helpers::toggleable_types::ToggleableBool;

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
        self.nfc.from(radio_settings.block_nfc);
        self.wifi.from(radio_settings.block_wifi);
        self.bt.from(radio_settings.block_bt);
    }
}

#[component]
pub fn RadioGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info().is_none() {
        return rsx! { "Connecting to the daemon..." };
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
            block_nfc: form.nfc.into_base(),
            block_wifi: form.wifi.into_base(),
            block_bt: form.bt.into_base(),
        };

        control_routine.send((
            ControlAction::UpdateProfileReduced(
                active_profile_idx as u32,
                active_profile,
                ReducedUpdate::Radio,
            ),
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
                    disabled: !changed() || awaiting_completion(),
                    if awaiting_completion() {
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

            br {}
            br {}
            br {}
        }
    }
}
