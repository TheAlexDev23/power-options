use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::Config;

use crate::{
    communication_services::{
        ControlAction, ControlRoutine, SystemInfoRoutine, SystemInfoSyncType,
    },
    helpers::components::Dropdown,
    helpers::toggleable_components::ToggleableDropdown,
    helpers::toggleable_types::ToggleableString,
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
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(15.0), SystemInfoSyncType::None));
    control_routine.send((ControlAction::GetConfig, None));

    if config.as_ref().is_none() {
        return rsx! { "Obtaining configuration..." };
    }

    let config = config().unwrap();
    let mut form_used_config = use_signal(|| config.clone());
    let mut form = use_hook(|| SettingsForm::new(&config));
    if form_used_config() != config {
        form.set_values(&config);
        form_used_config.set(config.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let awaiting_reset = use_signal(|| false);
    let mut awaiting_reset_idx = use_signal(|| 0);

    let onsubmit = {
        let config = config.clone();
        move || {
            let mut config = config.clone();

            config.profiles = form.profiles.cloned();
            config.ac_profile = form.ac_profile.cloned();
            config.bat_profile = form.bat_profile.cloned();

            config.profile_override = form.profile_override.into_base();

            control_routine.send((
                ControlAction::UpdateConfig(config),
                Some(awaiting_completion),
            ));
            control_routine.send((ControlAction::GetConfig, Some(awaiting_completion)));
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
            img { src: "icons/cross.svg" }
        }

        div {
            display: "flex",
            position: "absolute",
            top: "50px",
            justify_content: "center",
            width: "100%",

            form {
                width: "80%",
                onchange: move |_| {
                    changed.set(true);
                },
                onsubmit: move |_| {
                    onsubmit();
                    changed.set(false);
                },

                label { "Profile order" }

                table { max_width: "400px",
                    tr {
                        th { "" }
                        th { "" }
                        th { "" }
                        th { "" }
                    }

                    for (idx , name) in form.profiles.cloned().into_iter().enumerate() {
                        tr {
                            td { "{name}" }
                            if idx < form.profiles.len() - 1 {
                                td { width: "20px",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            form.profiles.write().swap(idx, idx + 1);
                                            changed.set(true);
                                        },
                                        img { src: "icons/icon-down.svg" }
                                    }
                                }
                            } else {
                                td { "" }
                            }

                            if idx > 0 {
                                td { width: "20px",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            form.profiles.write().swap(idx, idx - 1);
                                            changed.set(true);
                                        },
                                        img { src: "icons/icon-up.svg" }
                                    }
                                }
                            } else {
                                td { "" }
                            }

                            td {
                                if awaiting_reset() && awaiting_reset_idx() == idx {
                                    div { class: "spinner" }
                                } else if !awaiting_reset() {
                                    button {
                                        onclick: move |_| {
                                            awaiting_reset_idx.set(idx);
                                            control_routine
                                                .send((ControlAction::ResetProfile(idx as u32), Some(awaiting_reset)));
                                            control_routine.send((ControlAction::GetProfilesInfo, Some(awaiting_reset)));
                                        },
                                        r#type: "button",
                                        "Reset to defaults"
                                    }
                                }
                            }
                        }
                    }
                }

                br {
                }

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
