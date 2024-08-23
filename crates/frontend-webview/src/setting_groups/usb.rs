use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, ReducedUpdate, SystemInfo, USBSettings};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::toggleable_components::{
    ToggleableNumericField, ToggleableToggle, ToggleableWhiteBlackListDisplay,
};
use crate::helpers::toggleable_types::{ToggleableBool, ToggleableInt, ToggleableWhiteBlackList};

#[derive(PartialEq, Clone, Default)]
struct USBForm {
    pub enable_pm: ToggleableBool,
    pub autosuspend_delay_ms: ToggleableInt,
    pub whiteblacklist: ToggleableWhiteBlackList,
}

impl USBForm {
    pub fn new(usb_settings: &USBSettings) -> USBForm {
        let mut ret = USBForm::default();
        ret.set_values(usb_settings);
        ret
    }

    pub fn set_values(&mut self, usb_settings: &USBSettings) {
        self.enable_pm.from(usb_settings.enable_pm);
        self.whiteblacklist
            .from(usb_settings.whiteblacklist.clone());
        self.autosuspend_delay_ms
            .from_u32(usb_settings.autosuspend_delay_ms);
    }
}

#[component]
pub fn USBGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(2.0), SystemInfoSyncType::USB));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let usb_info = system_info().unwrap().usb_info;
    let usb_settings = profiles_info()
        .unwrap()
        .get_active_profile()
        .clone()
        .usb_settings;

    let mut form_used_settings = use_signal(|| usb_settings.clone());
    let mut form = use_hook(|| USBForm::new(&usb_settings));
    if usb_settings != form_used_settings() {
        form.set_values(&usb_settings);
        form_used_settings.set(usb_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().as_ref().unwrap().clone();

        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.usb_settings = USBSettings {
            enable_pm: form.enable_pm.into_base(),
            whiteblacklist: form.whiteblacklist.into_base(),
            autosuspend_delay_ms: form.autosuspend_delay_ms.into_u32(),
        };

        control_routine.send((
            ControlAction::UpdateProfileReduced(
                active_profile_idx as u32,
                active_profile,
                ReducedUpdate::USB,
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
                    ToggleableToggle { name: "Enable runtime power management", value: form.enable_pm }
                }
                div { class: "option",
                    ToggleableNumericField { name: "Autosuspend delay in miliseconds", value: form.autosuspend_delay_ms }
                }
            }

            if form.enable_pm.1() {
                ToggleableWhiteBlackListDisplay {
                    value: form.whiteblacklist,
                    columns: ["ID".to_string(), "Device Name".to_string()],
                    rows: usb_info
                        .usb_devices
                        .iter()
                        .map(|d| [d.id.clone(), d.display_name.clone()])
                        .collect::<Vec<_>>(),
                    identifying_column: 0
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
                        form.set_values(&usb_settings);
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
