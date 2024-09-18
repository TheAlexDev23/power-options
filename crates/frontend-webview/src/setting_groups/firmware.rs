use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{FirmwareSettings, ProfilesInfo, ReducedUpdate, SystemInfo};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};
use crate::helpers::toggleable_components::ToggleableDropdown;
use crate::helpers::toggleable_types::ToggleableString;

#[component]
pub fn FirmwareGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(5.0), SystemInfoSyncType::Firmware));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let firmware_info = system_info().as_ref().unwrap().firmware_info.clone();

    let firmware_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .firmware_settings
        .clone();

    let mut used_settings = use_signal(|| firmware_settings.clone());
    let mut platform_profile = ToggleableString(
        use_signal(|| firmware_settings.platform_profile.is_some()),
        use_signal(|| {
            firmware_settings
                .platform_profile
                .clone()
                .unwrap_or("default".to_string())
        }),
    );

    if used_settings() != firmware_settings {
        platform_profile.from(firmware_settings.platform_profile.clone());
        used_settings.set(firmware_settings.clone());
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

        active_profile.firmware_settings = FirmwareSettings {
            platform_profile: platform_profile.1().into(),
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Firmware,
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
                    ToggleableDropdown {
                        name: labels::ACPI_PLATFORM_PROFILE_TITLE,
                        disabled: firmware_info.platform_profiles.is_none(),
                        tooltip: if firmware_info.platform_profiles.is_some() {
                            Some(labels::ACPI_PLATFORM_PROFILE_TT.to_string())
                        } else {
                            Some(labels::ACPI_PLATFORM_PROFILE_MISSING_TT.to_string())
                        },
                        items: firmware_info.platform_profiles.unwrap_or(vec!["default".to_string()]),
                        value: platform_profile
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
                        platform_profile.from(firmware_settings.platform_profile.clone());
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
