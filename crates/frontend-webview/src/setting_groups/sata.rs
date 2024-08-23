use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, ReducedUpdate, SATASettings, SystemInfo};

use crate::communication_services::{
    ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
};
use crate::helpers::toggleable_components::ToggleableDropdown;
use crate::helpers::toggleable_types::ToggleableString;

#[component]
pub fn SATAGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(5.0), SystemInfoSyncType::SATA));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let sata_info = system_info().as_ref().unwrap().sata_info.clone();

    let sata_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .sata_settings
        .clone();

    let mut used_settings = use_signal(|| sata_settings.clone());
    let mut active_link_pm_policty = ToggleableString(
        use_signal(|| sata_settings.active_link_pm_policy.is_some()),
        use_signal(|| {
            sata_settings
                .active_link_pm_policy
                .clone()
                .unwrap_or("med_power_with_dipm".to_string())
        }),
    );

    if used_settings() != sata_settings {
        active_link_pm_policty.from(sata_settings.active_link_pm_policy.clone());
        used_settings.set(sata_settings.clone());
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

        active_profile.sata_settings = SATASettings {
            active_link_pm_policy: active_link_pm_policty.into_base(),
        };

        control_routine.send((
            ControlAction::UpdateProfileReduced(
                active_profile_idx as u32,
                active_profile,
                ReducedUpdate::SATA,
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

            p { "{sata_info.hosts} SATA hosts present" }

            div { class: "option-group",
                div { class: "option",
                    ToggleableDropdown {
                        name: "SATA active link power management",
                        items: vec![
                            "max_performance".to_string(),
                            "medium_power".to_string(),
                            "med_power_with_dipm".to_string(),
                            "min_power".to_string(),
                        ],
                        value: active_link_pm_policty
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
                        active_link_pm_policty.from(sata_settings.active_link_pm_policy.clone());
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
