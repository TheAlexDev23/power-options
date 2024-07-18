use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{ProfilesInfo, SystemInfo};

use crate::{
    communication_services::{ControlAction, SystemInfoSyncType},
    helpers::{Dropdown, OptInToggle},
};

#[component]
pub fn CPUGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: Coroutine<ControlAction>,
    system_info_routine: Coroutine<(Duration, SystemInfoSyncType)>,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(3.0), SystemInfoSyncType::CPU));
    if profiles_info.read().is_none() && system_info.read().is_none() {
        return rsx! {"Connecting to the daemon..."};
    }

    let profiles_info = profiles_info.read();
    let system_info = system_info.read();

    let current_profile = profiles_info.as_ref().unwrap().get_active_profile();
    let system_info = system_info.as_ref().unwrap();

    let mut epps = vec![String::from("Don't overwrite")];
    let mut governors = vec![String::from("Don't overwrite")];

    epps.extend(
        system_info
            .cpu_info
            .energy_performance_preferences
            .iter()
            .cloned(),
    );
    governors.extend(system_info.cpu_info.governors.iter().cloned());

    rsx! {
        form {
            class: "cpu-form",
            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Set EPP for all"
                    }
                    Dropdown {
                        items: epps,
                        selected: if let Some(ref epp) = current_profile.cpu_settings.energy_performance_preference {
                            epp.clone()
                        } else {
                            String::from("Don't overwrite")
                        }
                    }
                }
                div {
                    class: "option",
                    label {
                        "Set governor for all"
                    }
                    Dropdown {
                        items: governors,
                        selected: if let Some(ref epp) = current_profile.cpu_settings.energy_performance_preference {
                            epp.clone()
                        } else {
                            String::from("Don't overwrite")
                        }
                    }
                }
            }

            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Minimum frequency (MHz)"
                    }
                    input {
                        class: "numeric-input",
                        value: current_profile.cpu_settings.min_frequency.map(|v|format!("{v}")).unwrap_or(String::from("-")),
                        r#type: "text",
                    }
                }
                div {
                    class: "option",
                    label {
                        "Maximum frequency (MHz)"
                    }
                    input {
                        class: "numeric-input",
                        value: current_profile.cpu_settings.max_frequency.map(|v|format!("{v}")).unwrap_or(String::from("-")),
                        r#type: "text",
                    }
                }
            }

            div {
                class: "option-group",
                div {
                    class: "option",
                    label {
                        "Allow boost"
                    }
                    if system_info.cpu_info.boost.is_none() {
                        p {
                            "Your driver does not seem to support the boost feature"
                        }

                    } else {
                        OptInToggle {
                            overwriting: current_profile.cpu_settings.boost.is_some(),
                            value: if let Some(boost) = current_profile.cpu_settings.boost {
                                boost
                            } else {
                                system_info.cpu_info.boost.unwrap()
                            }
                        }
                    }
                }
            }

            div {
                class: "confirm-buttons",
                input {
                    r#type: "submit",
                    value: "Apply"
                }
                input {
                    r#type: "reset",
                    value: "Cancel"
                }
            }
        }
    }
}
