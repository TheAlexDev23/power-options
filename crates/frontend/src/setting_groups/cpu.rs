use std::{rc::Rc, time::Duration};

use dioxus::prelude::*;
use power_daemon::{CPUSettings, ProfilesInfo, SystemInfo};

use crate::{
    communication_services::{ControlAction, SystemInfoSyncType},
    helpers::{Dropdown, Toggle},
};

#[component]
pub fn CPUGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: Coroutine<ControlAction>,
    system_info_routine: Coroutine<(Duration, SystemInfoSyncType)>,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(10.0), SystemInfoSyncType::CPU));
    use_context_provider(|| 0);
    if profiles_info.read().is_none() || system_info.read().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let cpu_settings = profiles_info
        .read()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .cpu_settings
        .clone();

    let system_info = system_info.read().as_ref().unwrap().clone();

    let mut changed = use_signal(|| false);

    let mode_supported = system_info.cpu_info.active_mode.is_some();
    let epp_supported = system_info.cpu_info.has_epp;
    let perf_pct_scaling_supported = system_info.cpu_info.has_perf_pct_scaling;
    let boost_supported = system_info.cpu_info.boost.is_some();
    let hwp_dyn_boost_supported = system_info.cpu_info.hwp_dynamic_boost.is_some();

    let toggle_mode_initial = cpu_settings.mode.is_some();
    let toggle_epp_initial = cpu_settings.energy_performance_preference.is_some();
    let toggle_governor_initial = cpu_settings.governor.is_some();
    let toggle_min_freq_initial = cpu_settings.min_frequency.is_some();
    let toggle_max_freq_initial = cpu_settings.max_frequency.is_some();
    let toggle_min_perf_pct_initial = cpu_settings.min_perf_pct.is_some();
    let toggle_max_perf_pct_initial = cpu_settings.max_perf_pct.is_some();
    let toggle_boost_initial = cpu_settings.boost.is_some();
    let toggle_hwp_dyn_boost_initial = cpu_settings.hwp_dynamic_boost.is_some();

    let mut toggle_mode = use_signal(|| toggle_mode_initial);
    let mut toggle_epp = use_signal(|| toggle_epp_initial);
    let mut toggle_governor = use_signal(|| toggle_governor_initial);
    let mut toggle_min_freq = use_signal(|| toggle_min_freq_initial);
    let mut toggle_max_freq = use_signal(|| toggle_max_freq_initial);
    let mut toggle_min_perf_pct = use_signal(|| toggle_min_perf_pct_initial);
    let mut toggle_max_perf_pct = use_signal(|| toggle_max_perf_pct_initial);
    let mut toggle_boost = use_signal(|| toggle_boost_initial);
    let mut toggle_hwp_dyn_boost = use_signal(|| toggle_hwp_dyn_boost_initial);

    // Required for reactivity for elements that depend on it. Will not be accurate with the actual selection,
    // And therefore should not be used for form parsing
    let mut current_mode = use_signal(|| {
        if let Some(ref mode) = cpu_settings.mode {
            mode.clone()
        } else {
            String::from("passive")
        }
    });

    let onsubmit = move |f: Rc<FormData>| {
        let f = f.values();
        let active_profile_idx = profiles_info.read().as_ref().unwrap().active_profile;
        let mut active_profile = profiles_info
            .read()
            .as_ref()
            .unwrap()
            .get_active_profile()
            .clone();

        active_profile.cpu_settings = CPUSettings {
            mode: if mode_supported && toggle_mode.cloned() {
                Some(f.get("mode").unwrap().0[0].clone())
            } else {
                None
            },
            governor: if toggle_governor.cloned() {
                Some(f.get("governor").unwrap().0[0].clone())
            } else {
                None
            },
            energy_performance_preference: if *current_mode.read() == "active"
                && epp_supported
                && toggle_epp.cloned()
            {
                Some(f.get("epp").unwrap().0[0].clone())
            } else {
                None
            },
            min_frequency: if toggle_min_freq.cloned() {
                Some(f.get("min_frequency").unwrap().0[0].parse().unwrap())
            } else {
                None
            },
            max_frequency: if toggle_max_freq.cloned() {
                Some(f.get("max_frequency").unwrap().0[0].parse().unwrap())
            } else {
                None
            },
            min_perf_pct: if toggle_min_perf_pct.cloned() {
                Some(f.get("min_perf_pct").unwrap().0[0].parse().unwrap())
            } else {
                None
            },
            max_perf_pct: if toggle_max_perf_pct.cloned() {
                Some(f.get("max_perf_pct").unwrap().0[0].parse().unwrap())
            } else {
                None
            },

            boost: if toggle_boost.cloned() {
                Some(f.get("boost").unwrap().0[0] == "on")
            } else {
                None
            },

            hwp_dynamic_boost: if *current_mode.read() == "active"
                && hwp_dyn_boost_supported
                && toggle_hwp_dyn_boost.cloned()
            {
                Some(f.get("hwp_dyn_boost").unwrap().0[0] == "on")
            } else {
                None
            },
        };

        control_routine.send(ControlAction::UpdateProfile(
            active_profile_idx as u32,
            active_profile,
        ));
    };

    use_effect(move || {
        if !*toggle_mode.read() {
            if let Some(mode) = system_info.cpu_info.active_mode.clone() {
                current_mode.set(if mode {
                    String::from("active")
                } else {
                    String::from("passive")
                });
            }
        }
    });

    // No need to check wether EPP is available as when it's not it won't even be rendered
    let epps = vec![
        "performance",
        "balance_performance",
        "default",
        "balance_power",
        "power",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let governors = if *current_mode.read() == "active" {
        vec!["performance", "powersave"]
    } else {
        vec![
            "conservative",
            "ondemand",
            "userspace",
            "powersave",
            "performance",
            "schedutil",
        ]
    }
    .into_iter()
    .map(String::from)
    .collect();

    rsx! {
        form {
            id: "cpu-form",
            onchange: move |_| {
                changed.set(true);
            },
            onsubmit: move |f| {
                onsubmit(f.data());
                changed.set(false);
            },
            onreset: move |f| {
                println!("A: {}", f.value());
                println!("B: {:?}", f.values());
            },
            if mode_supported {
                div { class: "option-group",
                    div { class: "option",
                        div {
                            Toggle { val: toggle_mode, initial: toggle_mode_initial }
                            label { "Scaling driver operation mode" }
                        }
                        Dropdown {
                            name: String::from("mode"),
                            onchange: move |f: String| {
                                current_mode.set(f);
                            },
                            items: vec![String::from("active"), String::from("passive")],
                            selected: current_mode.read().clone(),
                            disabled: Some(!toggle_mode.cloned())
                        }
                    }
                }
            }

            div { class: "option-group",
                // EPP might be unsupported on some CPUs and on those that are, it might not take effect if the mode is passive
                if epp_supported && *current_mode.read() == "active" {
                    div { class: "option",
                        div {
                            Toggle { val: toggle_epp, initial: toggle_epp_initial }
                            label { "Set EPP for all" }
                        }
                        Dropdown {
                            name: String::from("epp"),
                            items: epps,
                            selected: if let Some(ref epp) = cpu_settings.energy_performance_preference {
                                epp.clone()
                            } else {
                                String::new()
                            },
                            disabled: Some(!toggle_epp.cloned())
                        }
                    }
                }
                div { class: "option",
                    div {
                        Toggle { val: toggle_governor, initial: toggle_governor_initial }
                        label { "Set governor for all" }
                    }
                    Dropdown {
                        name: String::from("governor"),
                        items: governors,
                        selected: if let Some(ref epp) = cpu_settings.energy_performance_preference {
                            epp.clone()
                        } else {
                            String::new()
                        },
                        disabled: Some(!toggle_governor.cloned())
                    }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    div {
                        Toggle { val: toggle_min_freq, initial: toggle_min_freq_initial }
                        label { "Minimum frequency (MHz)" }
                    }
                    input {
                        class: "numeric-input",
                        name: "min_frequency",
                        initial_value: cpu_settings.min_frequency.map(|v| format!("{v}")).unwrap_or(String::new()),
                        disabled: !toggle_min_freq.cloned(),
                        r#type: "text"
                    }
                }
                div { class: "option",
                    div {
                        Toggle { val: toggle_max_freq, initial: toggle_max_freq_initial }
                        label { "Maximum frequency (MHz)" }
                    }
                    input {
                        class: "numeric-input",
                        name: "max_frequency",
                        initial_value: cpu_settings.max_frequency.map(|v| format!("{v}")).unwrap_or(String::new()),
                        disabled: !toggle_max_freq.cloned(),
                        r#type: "text"
                    }
                }
            }

            if perf_pct_scaling_supported {
                div { class: "option-group",
                    div { class: "option",
                        div {
                            Toggle { val: toggle_min_perf_pct, initial: toggle_min_perf_pct_initial }
                            label { "Minimum performance percentage" }
                        }
                        input {
                            class: "numeric-input",
                            name: "min_perf_pct",
                            initial_value: cpu_settings.min_perf_pct.map(|v| format!("{v}")).unwrap_or(String::new()),
                            disabled: !toggle_min_perf_pct.cloned(),
                            r#type: "text"
                        }
                    }
                    div { class: "option",
                        div {
                            Toggle { val: toggle_max_perf_pct, initial: toggle_max_perf_pct_initial }
                            label { "Maximum performance percentage" }
                        }
                        input {
                            class: "numeric-input",
                            name: "max_perf_pct",
                            initial_value: cpu_settings.max_perf_pct.map(|v| format!("{v}")).unwrap_or(String::new()),
                            disabled: !toggle_max_perf_pct.cloned(),
                            r#type: "text"
                        }
                    }
                }
            }

            div { class: "option-group",
                if boost_supported {
                    div { class: "option",
                        div {
                            Toggle { val: toggle_boost, initial: toggle_boost_initial }
                            label { "Allow boost" }
                        }
                        input {
                            name: "boost",
                            initial_checked: if let Some(val) = cpu_settings.boost { val } else { false },
                            disabled: !toggle_boost.cloned(),
                            r#type: "checkbox"
                        }
                    }
                }

                if hwp_dyn_boost_supported && *current_mode.read() == "active" {
                    div { class: "option",
                        div {
                            Toggle { val: toggle_hwp_dyn_boost, initial: toggle_hwp_dyn_boost_initial }
                            label { "Allow HWP dynamic boost" }
                        }
                        input {
                            name: "hwp_dyn_boost",
                            initial_checked: if let Some(val) = cpu_settings.hwp_dynamic_boost { val } else { false },
                            disabled: !toggle_hwp_dyn_boost.cloned(),
                            r#type: "checkbox"
                        }
                    }
                }
            }

            br {}
            br {}

            div { class: "confirm-buttons",
                input {
                    r#type: "submit",
                    value: "Apply",
                    disabled: !changed.cloned()
                }
                input {
                    onclick: move |_| {
                        if let Some(mode) = system_info.cpu_info.active_mode.clone() {
                            current_mode
                                .set(
                                    if mode { String::from("active") } else { String::from("passive") },
                                );
                        }
                        toggle_mode.set(toggle_mode_initial);
                        toggle_epp.set(toggle_epp_initial);
                        toggle_governor.set(toggle_governor_initial);
                        toggle_min_freq.set(toggle_min_freq_initial);
                        toggle_max_freq.set(toggle_max_freq_initial);
                        toggle_min_perf_pct.set(toggle_min_perf_pct_initial);
                        toggle_max_perf_pct.set(toggle_max_perf_pct_initial);
                        toggle_boost.set(toggle_boost_initial);
                        toggle_hwp_dyn_boost.set(toggle_hwp_dyn_boost_initial);
                    },
                    r#type: "reset",
                    value: "Cancel"
                }
            }
        }
    }
}
