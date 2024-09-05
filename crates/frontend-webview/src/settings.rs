use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{Config, DefaultProfileType, ProfilesInfo};

use crate::{
    communication_services::{
        control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
        SystemInfoSyncType,
    },
    helpers::{
        components::{Dropdown, ValueBindDropdown},
        toggleable_components::ToggleableDropdown,
        toggleable_types::ToggleableString,
    },
};

#[derive(Clone, Default, PartialEq)]
struct SettingsForm {
    ac_profile: Signal<String>,
    bat_profile: Signal<String>,
    profile_override: ToggleableString,
    profiles: Signal<Vec<String>>,
}

impl SettingsForm {
    pub fn new(config: &Config) -> SettingsForm {
        let mut ret = SettingsForm::default();
        ret.set_values(config);
        ret
    }

    pub fn set_values(&mut self, config: &Config) {
        self.ac_profile.set(config.ac_profile.clone());
        self.bat_profile.set(config.bat_profile.clone());
        self.profile_override.from(config.profile_override.clone());
        self.profiles.set(config.profiles.clone());
    }
}

#[component]
pub fn SettingsMenu(
    settings_opened: Signal<bool>,
    config: Signal<Option<Config>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));
    use_hook(|| {
        control_routine.send((ControlAction::GetConfig, None));
    });

    if config.as_ref().is_none() || profiles_info.as_ref().is_none() {
        return rsx! { "Obtaining configuration..." };
    }

    let config = config().unwrap();
    let profiles_info = profiles_info().unwrap();

    let mut form_used_config = use_signal(|| config.clone());
    let mut form = use_hook(|| SettingsForm::new(&config));
    if form_used_config() != config {
        form.set_values(&config);
        form_used_config.set(config.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let awaiting_reset = use_signal(|| false);
    let awaiting_remove = use_signal(|| false);
    let awaiting_create = use_signal(|| false);
    let awaiting_move_up = use_signal(|| false);
    let awaiting_move_down = use_signal(|| false);
    let awaiting_rename = use_signal(|| false);

    let mut awaiting_reset_idx = use_signal(|| 0);
    let mut awaiting_remove_idx = use_signal(|| 0);
    let mut awaiting_move_up_idx = use_signal(|| 0);
    let mut awaiting_move_down_idx = use_signal(|| 0);
    let mut awaiting_rename_idx = use_signal(|| 0);

    let new_profile_type = use_signal(|| DefaultProfileType::Balanced.get_name());
    let profile_types = use_hook(|| {
        vec![
            DefaultProfileType::Superpowersave.get_name(),
            DefaultProfileType::Powersave.get_name(),
            DefaultProfileType::Balanced.get_name(),
            DefaultProfileType::Performance.get_name(),
            DefaultProfileType::Ultraperformance.get_name(),
        ]
    });

    let onsubmit = {
        let config = config.clone();
        move || {
            let mut config = config.clone();

            config.ac_profile = form.ac_profile.cloned();
            config.bat_profile = form.bat_profile.cloned();

            config.profile_override = form.profile_override.into_base();

            control_routine_send_multiple(
                control_routine,
                &[
                    ControlAction::UpdateConfig(config.into()),
                    ControlAction::GetConfig,
                ],
                Some(awaiting_completion),
            );
        }
    };

    rsx! {
        button {
            position: "absolute",
            top: "10px",
            left: "10px",
            display: "flex",
            padding: "5px",

            onclick: move |_| {
                settings_opened.set(false);
            },
            img { src: "assets/icons/cross.svg" }
        }

        div {
            display: "flex",
            position: "absolute",
            top: "50px",
            justify_content: "center",
            width: "100%",

            form {
                id: "settings-form",

                width: "80%",
                onchange: move |_| {
                    changed.set(true);
                },
                onsubmit: move |_| {
                    onsubmit();
                    changed.set(false);
                },

                div { class: "option-group",
                    div { class: "option",
                        label { "AC Profile" }
                        Dropdown {
                            items: form.profiles.cloned(),
                            selected: form.ac_profile,
                            disabled: false,
                            oninput: move |e| {
                                form.ac_profile.set(e);
                            }
                        }
                    }
                    div { class: "option",
                        label { "Battery Profile" }
                        Dropdown {
                            items: form.profiles.cloned(),
                            selected: form.bat_profile,
                            disabled: false,
                            oninput: move |e| {
                                form.bat_profile.set(e);
                            }
                        }
                    }
                }

                div { class: "option-group",
                    div { class: "option",
                        ToggleableDropdown {
                            name: "Persisting override",
                            items: form.profiles.cloned(),
                            value: form.profile_override
                        }
                    }
                }

                br {}

                div { class: "option-group",
                    div { class: "option",
                        label { "New Profile Type" }
                        ValueBindDropdown { disabled: false, items: profile_types, value: new_profile_type }
                    }
                    if awaiting_create() {
                        div {
                            width: "100%",
                            display: "flex",
                            justify_content: "center",
                            div { class: "spinner" }
                        }
                    } else {
                        button {
                            width: "100%",
                            r#type: "button",
                            onclick: move |_| {
                                control_routine_send_multiple(
                                    control_routine,
                                    &[
                                        ControlAction::CreateProfile(
                                            DefaultProfileType::from_name(new_profile_type()).unwrap(),
                                        ),
                                        ControlAction::GetConfig,
                                        ControlAction::GetProfilesInfo,
                                    ],
                                    Some(awaiting_create),
                                );
                            },
                            "Create Profile"
                        }
                    }
                }

                br {}

                label { "Profiles" }

                table { max_width: "600px",
                    tr {
                        th { "" }
                        th { "" }
                        th { "" }
                        th { "" }
                        th { "" }
                    }

                    for (idx , name) in form.profiles.cloned().into_iter().enumerate() {
                        tr {
                            td {
                                if awaiting_rename() && awaiting_rename_idx() == idx {
                                    div { class: "spinner" }
                                } else {
                                    input {
                                        r#type: "text",
                                        class: "editable-label",
                                        onchange: move |v| {
                                            v.stop_propagation();
                                            awaiting_rename_idx.set(idx);
                                            control_routine_send_multiple(
                                                control_routine,
                                                &[
                                                    ControlAction::RenameProfile(idx as u32, v.value()),
                                                    ControlAction::GetConfig,
                                                    ControlAction::GetProfileOverride,
                                                    ControlAction::GetProfilesInfo,
                                                ],
                                                Some(awaiting_rename),
                                            );
                                        },
                                        initial_value: "{name}"
                                    }
                                }
                            }

                            td { width: "20px",
                                if awaiting_move_down() && awaiting_move_down_idx() == idx {
                                    div { class: "spinner" }
                                } else if idx < form.profiles.len() - 1 {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            awaiting_move_down_idx.set(idx);
                                            control_routine_send_multiple(
                                                control_routine,
                                                &[
                                                    ControlAction::SwapProfiles(idx as u32, (idx + 1) as u32),
                                                    ControlAction::GetConfig,
                                                    ControlAction::GetProfilesInfo,
                                                ],
                                                Some(awaiting_move_down),
                                            );
                                        },
                                        img { src: "assets/icons/icon-down.svg" }
                                    }
                                }
                            }

                            td { width: "20px",
                                if awaiting_move_up() && awaiting_move_up_idx() == idx {
                                    div { class: "spinner" }
                                } else if idx > 0 {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            awaiting_move_up_idx.set(idx);
                                            control_routine_send_multiple(
                                                control_routine,
                                                &[
                                                    ControlAction::SwapProfiles(idx as u32, (idx - 1) as u32),
                                                    ControlAction::GetConfig,
                                                    ControlAction::GetProfilesInfo,
                                                ],
                                                Some(awaiting_move_up),
                                            );
                                        },
                                        img { src: "assets/icons/icon-up.svg" }
                                    }
                                }
                            }

                            td { width: "20px",
                                if awaiting_reset() && awaiting_reset_idx() == idx {
                                    div { class: "spinner" }
                                } else {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            awaiting_reset_idx.set(idx);
                                            control_routine_send_multiple(
                                                control_routine,
                                                &[ControlAction::ResetProfile(idx as u32), ControlAction::GetProfilesInfo],
                                                Some(awaiting_reset),
                                            );
                                        },
                                        "Reset"
                                    }
                                }
                            }

                            td { width: "20px",
                                if awaiting_remove() && awaiting_remove_idx() == idx {
                                    div { class: "spinner" }
                                } else if !awaiting_remove() && profiles_info.active_profile != idx
                                    && profiles_info.profiles.len() != 1
                                {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            awaiting_remove_idx.set(idx);
                                            control_routine_send_multiple(
                                                control_routine,
                                                &[
                                                    ControlAction::RemoveProfile(idx as u32),
                                                    ControlAction::GetConfig,
                                                    ControlAction::GetProfilesInfo,
                                                    ControlAction::GetProfilesInfo,
                                                ],
                                                Some(awaiting_remove),
                                            );
                                        },
                                        "Remove"
                                    }
                                }
                            }
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
                            form.set_values(&config);
                            settings_opened.set(false);
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
}
