use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, ReducedUpdate, ScreenSettings};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::toggleable_components::{ToggleableNumericField, ToggleableTextField};
use crate::helpers::toggleable_types::{ToggleableInt, ToggleableString};

#[derive(PartialEq, Clone, Default)]
struct ScreenForm {
    pub brightness: ToggleableInt,
    pub resolution: ToggleableString,
    pub refresh_rate: ToggleableString,
}

impl ScreenForm {
    pub fn new(screen_settings: &ScreenSettings) -> ScreenForm {
        let mut ret = ScreenForm::default();
        ret.set_values(screen_settings);
        ret
    }

    pub fn set_values(&mut self, screen_settings: &ScreenSettings) {
        self.brightness.from_u32(screen_settings.brightness);
        self.resolution.from(screen_settings.resolution.clone());
        self.refresh_rate.from(screen_settings.refresh_rate.clone());
    }
}

#[component]
pub fn ScreenGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));

    if profiles_info().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let screen_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .screen_settings
        .clone();

    let mut form_used_settings = use_signal(|| screen_settings.clone());
    let mut form = use_hook(|| ScreenForm::new(&screen_settings));
    if screen_settings != form_used_settings() {
        form.set_values(&screen_settings);
        form_used_settings.set(screen_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().as_ref().unwrap().clone();

        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.screen_settings = ScreenSettings {
            brightness: form.brightness.into_u32(),
            refresh_rate: form.refresh_rate.into_base(),
            resolution: form.resolution.into_base(),
        };

        control_routine.send((
            ControlAction::UpdateProfileReduced(
                active_profile_idx as u32,
                active_profile,
                ReducedUpdate::Screen,
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
                    ToggleableNumericField { name: "Set brightness percentage", value: form.brightness }
                }
            }
            div { class: "option-group",
                div { class: "option",
                    ToggleableTextField { name: "Set resolution", value: form.resolution }
                }
                div { class: "option",
                    ToggleableTextField { name: "Set refresh rate", value: form.refresh_rate }
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
                        form.set_values(&screen_settings);
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
