use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{AudioModule, AudioSettings, ProfilesInfo, ReducedUpdate, SystemInfo};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};
use crate::helpers::toggleable_components::ToggleableNumericField;
use crate::helpers::toggleable_types::{ToggleableBool, ToggleableInt};

#[derive(Debug, Clone, PartialEq, Default)]
struct AudioForm {
    pub disable_ethernet: ToggleableBool,

    pub idle_timeout: ToggleableInt,
}

impl AudioForm {
    pub fn new(audio_settings: &AudioSettings) -> AudioForm {
        let mut ret = AudioForm::default();
        ret.set_values(audio_settings);
        ret
    }

    pub fn set_values(&mut self, audio_settings: &AudioSettings) {
        self.idle_timeout.from_u32(audio_settings.idle_timeout);
    }
}

#[component]
pub fn AudioGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info: Signal<Option<SystemInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::Opt));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let info = system_info().as_ref().unwrap().opt_features_info.clone();

    let audio_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .audio_settings
        .clone();

    let mut form_used_settings = use_signal(|| audio_settings.clone());
    let mut form = use_hook(|| AudioForm::new(&audio_settings));

    if form_used_settings() != audio_settings {
        form.set_values(&audio_settings);
        form_used_settings.set(audio_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().unwrap();
        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.audio_settings = AudioSettings {
            idle_timeout: form.idle_timeout.into_u32(),
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Audio,
                ),
                ControlAction::GetProfilesInfo,
            ],
            Some(awaiting_completion),
        )
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
                    ToggleableNumericField {
                        name: labels::AUDIO_IDLE_TIMEOUT_TITLE,
                        disabled: info.audio_module == AudioModule::Other,
                        tooltip: if info.audio_module == AudioModule::Other {
                            labels::AUDIO_IDLE_TIMEOUT_MODULE_UNSPORTED_TT
                        } else {
                            labels::AUDIO_IDLE_TIMEOUT_TT
                        },
                        value: form.idle_timeout
                    }
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
                        form.set_values(&audio_settings);
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
