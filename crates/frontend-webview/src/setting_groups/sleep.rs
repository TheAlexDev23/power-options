use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, ReducedUpdate, SleepSettings, SystemInfo};

use crate::{
    communication_services::{
        control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
        SystemInfoSyncType,
    },
    helpers::{toggleable_components::ToggleableNumericField, toggleable_types::ToggleableInt},
};

#[derive(Default, PartialEq, Clone, Debug)]
struct SleepForm {
    pub turn_off_screen: ToggleableInt,
    pub suspend: ToggleableInt,
}

impl SleepForm {
    pub fn new(sleep_settings: &SleepSettings) -> SleepForm {
        let mut form = SleepForm::default();
        form.set_values(sleep_settings);
        form
    }

    pub fn set_values(&mut self, sleep_settings: &SleepSettings) {
        self.turn_off_screen
            .from_u32(sleep_settings.turn_off_screen_after);
        self.suspend.from_u32(sleep_settings.suspend_after);
    }
}

#[component]
pub fn SleepGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(5.0), SystemInfoSyncType::Opt));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let opt_info = system_info().as_ref().unwrap().opt_features_info.clone();

    let sleep_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .sleep_settings
        .clone();

    let mut form_used_settings = use_signal(|| sleep_settings.clone());
    let mut form = use_hook(|| SleepForm::new(&sleep_settings));
    if form_used_settings() != sleep_settings {
        form.set_values(&sleep_settings);
        form_used_settings.set(sleep_settings.clone());
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

        active_profile.sleep_settings = SleepSettings {
            turn_off_screen_after: form.turn_off_screen.into_u32(),
            suspend_after: form.suspend.into_u32(),
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Sleep,
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
                if opt_info.supports_xautolock {
                    div { class: "option",
                        ToggleableNumericField { name: labels::SUSPEND_TITLE, value: form.suspend }
                    }
                } else {
                    p { "To set proper suspend time, xautolock needs to be installed in your system" }
                }
                if opt_info.supports_xset {
                    div { class: "option",
                        ToggleableNumericField { name: labels::SCREEN_TURN_OFF_TITLE, value: form.turn_off_screen }
                    }
                } else {
                    p { "To set proper suspend time, xset needs to be installed in your system" }
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
                        form.set_values(&sleep_settings);
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
