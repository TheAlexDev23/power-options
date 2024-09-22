use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{KernelSettings, ProfilesInfo, ReducedUpdate};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};
use crate::helpers::toggleable_components::{ToggleableNumericField, ToggleableToggle};
use crate::helpers::toggleable_types::{ToggleableBool, ToggleableInt};

#[derive(PartialEq, Clone, Default)]
struct KernelForm {
    pub disable_nmi_watchdog: ToggleableBool,
    pub vm_writeback: ToggleableInt,
    pub laptop_mode: ToggleableInt,
}

impl KernelForm {
    pub fn new(kernel_settings: &KernelSettings) -> KernelForm {
        let mut ret = KernelForm::default();
        ret.set_values(kernel_settings);
        ret
    }

    pub fn set_values(&mut self, kernel_settings: &KernelSettings) {
        self.disable_nmi_watchdog
            .from(kernel_settings.disable_nmi_watchdog);
        self.vm_writeback.from_u32(kernel_settings.vm_writeback);
        self.laptop_mode.from_u32(kernel_settings.laptop_mode);
    }
}

#[component]
pub fn KernelGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let kernel_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .kernel_settings
        .clone();

    let mut form_used_settings = use_signal(|| kernel_settings.clone());
    let mut form = use_hook(|| KernelForm::new(&kernel_settings));
    if kernel_settings != form_used_settings() {
        form.set_values(&kernel_settings);
        form_used_settings.set(kernel_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().as_ref().unwrap().clone();

        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.kernel_settings = KernelSettings {
            disable_nmi_watchdog: form.disable_nmi_watchdog.into_base(),
            vm_writeback: form.vm_writeback.into_u32(),
            laptop_mode: form.laptop_mode.into_u32(),
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Kernel,
                ),
                ControlAction::GetProfilesInfo,
            ],
            Some(awaiting_completion),
        );
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
                    ToggleableToggle {
                        name: labels::DIS_NMI_TITLE,
                        tooltip: labels::DIS_NMI_TT,
                        value: form.disable_nmi_watchdog
                    }
                }
            }
            div { class: "option-group",
                div { class: "option",
                    ToggleableNumericField {
                        name: labels::VM_WR_TITLE,
                        tooltip: labels::VM_WR_TT,
                        value: form.vm_writeback
                    }
                }
                div { class: "option",
                    ToggleableNumericField {
                        name: labels::LAPTOP_MODE_TITLE,
                        tooltip: labels::LAPTOP_MODE_TT,
                        value: form.laptop_mode
                    }
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
                        form.set_values(&kernel_settings);
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
