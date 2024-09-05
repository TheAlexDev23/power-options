use std::collections::HashMap;
use std::time::Duration;

use dioxus::desktop::tao::keyboard::ModifiersState;
use dioxus::desktop::{use_wry_event_handler, WindowEvent};
use dioxus::prelude::Event;
use dioxus::prelude::*;

use labels::DRIVER_OPMODE_TT;
use power_daemon::{CPUSettings, CoreSetting, Profile, ProfilesInfo, ReducedUpdate, SystemInfo};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};

use crate::helpers::{
    components::Dropdown,
    toggleable_components::{ToggleableDropdown, ToggleableNumericField, ToggleableToggle},
    toggleable_types::{ToggleableBool, ToggleableInt, ToggleableString},
    TooltipDirection,
};

#[derive(Default, Debug, Clone)]
struct CPUForm {
    pub mode: ToggleableString,

    pub epp: ToggleableString,
    pub governor: ToggleableString,

    pub min_freq: ToggleableInt,
    pub max_freq: ToggleableInt,

    pub min_perf_pct: ToggleableInt,
    pub max_perf_pct: ToggleableInt,

    pub boost: ToggleableBool,
    pub hwp_dyn_boost: ToggleableBool,
}

impl CPUForm {
    pub fn new(cpu_settings: &CPUSettings) -> CPUForm {
        let mut form = CPUForm::default();
        form.set_values(cpu_settings);
        form
    }

    pub fn set_values(&mut self, cpu_settings: &CPUSettings) {
        self.mode
            .from_or(cpu_settings.mode.clone(), String::from("passive"));

        self.epp.from(cpu_settings.energy_perf_ratio.clone());

        self.governor.from(cpu_settings.governor.clone());

        self.min_freq.from_u32(cpu_settings.min_freq);
        self.max_freq.from_u32(cpu_settings.max_freq);

        self.min_perf_pct.from_u8(cpu_settings.min_perf_pct);
        self.max_perf_pct.from_u8(cpu_settings.max_perf_pct);

        self.boost.from(cpu_settings.boost);
        self.hwp_dyn_boost.from(cpu_settings.hwp_dyn_boost);
    }
}

#[component]
pub fn CPUGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: SystemInfoRoutine,
) -> Element {
    use_hook(|| {
        system_info_routine.send((Duration::from_secs_f32(0.5), SystemInfoSyncType::CPU));
    });

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let profiles_info = profiles_info().as_ref().unwrap().clone();
    let system_info = system_info().as_ref().unwrap().clone();

    rsx! {
        CPUSettingsForm {
            system_info: system_info.clone(),
            profiles_info: profiles_info.clone(),
            control_routine
        }

        br {}

        h2 { "Per-core settings" }

        CoreSettings {
            system_info: system_info.clone(),
            profiles_info: profiles_info.clone(),
            control_routine,
            system_info_routine
        }

        br {}
        br {}
        br {}
    }
}

#[component]
fn CPUSettingsForm(
    system_info: SystemInfo,
    profiles_info: ProfilesInfo,
    control_routine: ControlRoutine,
) -> Element {
    let cpu_settings = profiles_info.get_active_profile().cpu_settings.clone();

    let cpu_info = system_info.clone().cpu_info;

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let mode_supported = cpu_info.mode.is_some();
    let epp_supported = cpu_info.has_epp;
    let epb_supported = cpu_info.has_epb;
    let perf_pct_scaling_supported = cpu_info.has_perf_pct_scaling;
    let boost_supported = cpu_info.boost.is_some();
    let hwp_dyn_boost_supported = cpu_info.hwp_dynamic_boost.is_some();

    let mut form_used_settings = use_signal(|| cpu_settings.clone());
    let mut form = use_hook(|| CPUForm::new(&cpu_settings));
    if cpu_settings != form_used_settings() {
        form.set_values(&cpu_settings);
        form_used_settings.set(cpu_settings.clone());
    }

    let onsubmit = move || {
        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.cpu_settings = CPUSettings {
            mode: if !mode_supported {
                None
            } else {
                form.mode.into_base()
            },

            energy_perf_ratio: if !epp_supported {
                None
            } else {
                form.epp.into_base()
            },

            governor: form.governor.into_base(),

            min_freq: form.min_freq.into_u32(),
            max_freq: form.max_freq.into_u32(),

            min_perf_pct: form.min_perf_pct.into_u8(),
            max_perf_pct: form.max_perf_pct.into_u8(),

            boost: form.boost.into_base(),
            hwp_dyn_boost: if !(form.mode.1() == "active" || hwp_dyn_boost_supported) {
                None
            } else {
                form.hwp_dyn_boost.into_base()
            },
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::CPU,
                ),
                ControlAction::GetProfilesInfo,
            ],
            Some(awaiting_completion),
        );
    };

    use_effect(move || {
        // If the mode overwriting is disabled we set it to reflect the system current opmode
        // The reasoning is: the user does not set an explicit override so the opmode is not guaranteed, therefore we will assume the value is what the system is currently at
        // And even though the current value of the system does not reflect the users selection, it still won't be set by the daemon as the override is disabled
        if !form.mode.0() {
            if let Some(ref mode) = cpu_info.mode {
                form.mode.1.set(mode.clone());
            }
        }
    });

    let epps = get_epps();
    let governors = get_governors(&form.mode.1());

    rsx! {
        form {
            id: "cpu-form",
            onchange: move |_| {
                changed.set(true);
            },
            onsubmit: move |_| {
                onsubmit();
                changed.set(false);
            },
            if mode_supported {
                div { class: "option-group",
                    div { class: "option",
                        ToggleableDropdown {
                            name: labels::DRIVER_OPMODE_TITLE,
                            dropdown_tooltip: labels::DRIVER_OPMODE_TT,
                            items: vec![String::from("active"), String::from("passive")],
                            value: form.mode
                        }
                    }
                }
            }

            div { class: "option-group",
                if epp_supported || epb_supported {
                    div { class: "option",
                        ToggleableDropdown {
                            name: labels::EPP_TITLE,
                            items: epps,
                            value: form.epp,
                            disabled: form.mode.1() == "active" && form.governor.1() == "performance",
                            dropdown_tooltip: if form.governor.1() == "performance" {
                                Some(labels::EPP_GOV_PERF_TT.to_string())
                            } else {
                                Some(labels::EPP_TT.to_string())
                            }
                        }
                    }
                }
                div { class: "option",
                    ToggleableDropdown {
                        name: labels::GOV_TITLE,
                        dropdown_tooltip: Some(labels::GOV_TT.to_string()),
                        items: governors,
                        value: form.governor
                    }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableNumericField { name: labels::MIN_FREQ_MHZ_TITLE, value: form.min_freq }
                }
                div { class: "option",
                    ToggleableNumericField { name: labels::MAX_FREQ_MHZ_TITLE, value: form.max_freq }
                }
            }

            if perf_pct_scaling_supported {
                div { class: "option-group",
                    div { class: "option",
                        ToggleableNumericField {
                            name: labels::MIN_PERF_PCT,
                            tooltip: labels::MIN_PERF_PCT_TT,
                            value: form.min_perf_pct
                        }
                    }
                    div { class: "option",
                        ToggleableNumericField {
                            name: labels::MAX_PERF_PCT,
                            tooltip: labels::MAX_PERF_PCT_TT,
                            value: form.max_perf_pct
                        }
                    }
                }
            }

            div { class: "option-group",
                if boost_supported {
                    div { class: "option",
                        ToggleableToggle {
                            name: labels::BOOST_TITLE.to_string(),
                            toggle_tooltip: labels::BOOST_TT.to_string(),
                            value: form.boost
                        }
                    }
                }

                if hwp_dyn_boost_supported {
                    div { class: "option",
                        ToggleableToggle {
                            name: labels::HWP_DYN_BOOST_TITLE,
                            value: form.hwp_dyn_boost,
                            disabled: form.mode.1() != "active",
                            toggle_tooltip: if form.mode.1() != "active" {
                                labels::HWP_DYN_BOOST_MODE_ACTIVE_TT
                            } else {
                                labels::HWP_DYN_BOOST_TT
                            }
                        }
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
                        form.set_values(&cpu_settings);
                        changed.set(false);
                    },
                    r#type: "button",
                    value: "Cancel"
                }
            }
        }
    }
}

#[component]
fn CoreSettings(
    system_info: SystemInfo,
    profiles_info: ProfilesInfo,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    let mut cpu_info = system_info.cpu_info.clone();
    let mut cpu_info_secondary = use_signal(|| cpu_info.clone());
    let mut secondary = cpu_info_secondary().clone();
    cpu_info.sync_core_info(&mut secondary);
    if cpu_info_secondary() != secondary {
        cpu_info_secondary.set(secondary);
    }

    let current_profile = profiles_info.get_active_profile().clone();

    let epps = get_epps();
    let governors = get_governors(&cpu_info.mode.clone().unwrap_or(String::from("passive")));

    let profile_id = profiles_info.active_profile as u32;

    let mut cores_awaiting_update_signals = HashMap::new();
    for core in &cpu_info.cores {
        cores_awaiting_update_signals.insert(core.logical_cpu_id, use_signal(|| false));
    }

    let mut ctrl_pressed = use_signal(|| false);
    let mut shift_pressed = use_signal(|| false);

    use_wry_event_handler(move |event, _| {
        if let dioxus::desktop::tao::event::Event::WindowEvent {
            event: WindowEvent::ModifiersChanged(state),
            ..
        } = event
        {
            if state.contains(ModifiersState::CONTROL) {
                ctrl_pressed.set(true);
            }
            if state.contains(ModifiersState::SHIFT) {
                shift_pressed.set(true);
            }

            if state.is_empty() {
                ctrl_pressed.set(false);
                shift_pressed.set(false);
            }
        }
    });

    #[allow(clippy::redundant_closure)]
    let mut selected: Signal<Vec<u32>> = use_signal(|| Vec::new());
    let mut shift_selection_pinpoint = use_signal(|| None);

    rsx! {
        table { id: "cpu-cores-table",
            tr {
                th { "" }
                th { "On" }

                th { "CPU" }

                th { "Current Min" }
                th { "Current Max" }

                th { "Base" }

                th { "Current" }

                th { "Range" }

                th { "Governor" }
                if cpu_info.has_epp {
                    th { "EPP" }
                }
            }

            for (logical_cpu_id , core) in cpu_info.cores.into_iter().map(|c| (c.logical_cpu_id, c)) {
                tr {
                    class: if selected().iter().any(|s| *s == logical_cpu_id) { "selected" },

                    onclick: move |_| {
                        let ctrl = ctrl_pressed();
                        let shift = shift_pressed();
                        if !ctrl && !shift {
                            selected.set(vec![logical_cpu_id]);
                            shift_selection_pinpoint.set(Some(logical_cpu_id));
                        } else if ctrl && !shift {
                            let len = selected().len();
                            selected.retain(|s| *s != logical_cpu_id);
                            if len == selected().len() {
                                selected.push(logical_cpu_id);
                            }
                            shift_selection_pinpoint.set(Some(logical_cpu_id));
                        } else if shift && !ctrl {
                            if shift_selection_pinpoint().is_some() {
                                let a = shift_selection_pinpoint().unwrap();
                                let b = logical_cpu_id;
                                selected.set((a.min(b)..=b.max(a)).collect());
                            } else {
                                shift_selection_pinpoint.set(Some(logical_cpu_id));
                                selected.set(vec![logical_cpu_id]);
                            }
                        } else if shift && ctrl {
                            if selected.is_empty() {
                                selected.set(vec![logical_cpu_id]);
                            } else {
                                let range = (selected()
                                    .into_iter()
                                    .min()
                                    .unwrap()
                                    .min(
                                        logical_cpu_id,
                                    )..=selected().into_iter().max().unwrap().max(logical_cpu_id))
                                    .collect();
                                selected.set(range);
                            }
                        }
                    },

                    td {
                        div { class: if !cores_awaiting_update_signals.get(&logical_cpu_id).unwrap()() { "hidden" },
                            div { class: "spinner" }
                        }
                    }

                    // Online
                    td {
                        if core.online.is_some() {
                            input {
                                onclick: move |e| {
                                    if selected().iter().any(|c| *c == logical_cpu_id) {
                                        e.stop_propagation();
                                    }
                                },
                                oninput: {
                                    let mut current_profile = current_profile.clone();
                                    let awaiting_signal = *cores_awaiting_update_signals
                                        .get(&logical_cpu_id)
                                        .unwrap();
                                    move |v| {
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            &selected(),
                                            move |core_settings| {
                                                core_settings.online = Some(v.value() == "true");
                                            },
                                            control_routine,
                                            awaiting_signal,
                                        );
                                    }
                                },
                                checked: "{core.online.unwrap()}",
                                r#type: "checkbox"
                            }
                        }
                    }
                    // Online

                    if core.online.unwrap_or(true) {
                        // Identification
                        if cpu_info.hybrid && core.is_performance_core.is_some() {
                            td {
                                if core.is_performance_core.unwrap() {
                                    "P ({core.physical_core_id} - {logical_cpu_id})"
                                } else {
                                    "E ({core.physical_core_id} - {logical_cpu_id})"
                                }
                            }
                        } else {
                            td { "{logical_cpu_id}" }
                        }
                        // Identification

                        // Min frqeuency
                        td {
                            input {
                                r#type: "number",
                                value: core.scaling_min_frequency as f64,
                                min: (core.total_min_frequency) as f64,
                                max: (core.total_max_frequency.min(core.scaling_max_frequency)) as f64,
                                onfocusin: move |_| {
                                    system_info_routine
                                        .send((Duration::from_secs_f32(5.0), SystemInfoSyncType::None));
                                },
                                onfocusout: move |_| {
                                    system_info_routine
                                        .send((Duration::from_secs_f32(0.5), SystemInfoSyncType::CPU));
                                },
                                onchange: {
                                    let mut current_profile = current_profile.clone();
                                    let awaiting_signal = *cores_awaiting_update_signals
                                        .get(&logical_cpu_id)
                                        .unwrap();
                                    move |v: Event<FormData>| {
                                        let v = v.value();
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            &selected(),
                                            move |core_settings| {
                                                if let Ok(v) = v.parse() {
                                                    core_settings.min_frequency = Some(v);
                                                }
                                            },
                                            control_routine,
                                            awaiting_signal,
                                        );
                                    }
                                }
                            }
                        }
                        // Min frqeuency

                        // Max frqeuency
                        td {
                            input {
                                r#type: "number",
                                value: core.scaling_max_frequency as f64,
                                min: (core.total_min_frequency.max(core.scaling_min_frequency)) as f64,
                                max: (core.total_max_frequency) as f64,
                                onfocusin: move |_| {
                                    system_info_routine
                                        .send((Duration::from_secs_f32(5.0), SystemInfoSyncType::None));
                                },
                                onfocusout: move |_| {
                                    system_info_routine
                                        .send((Duration::from_secs_f32(0.5), SystemInfoSyncType::CPU));
                                },
                                onchange: {
                                    let mut current_profile = current_profile.clone();
                                    let awaiting_signal = *cores_awaiting_update_signals
                                        .get(&logical_cpu_id)
                                        .unwrap();
                                    move |v: Event<FormData>| {
                                        let v = v.value();
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            &selected(),
                                            move |core_settings| {
                                                if let Ok(v) = v.parse() {
                                                    core_settings.max_frequency = Some(v);
                                                }
                                            },
                                            control_routine,
                                            awaiting_signal,
                                        );
                                    }
                                }
                            }
                        }
                        // Max frqeuency

                        td { "{core.base_frequency}" }

                        td { "{core.current_frequency}" }

                        td { "{core.total_min_frequency}-{core.total_max_frequency}" }

                        // Gov
                        td {
                            Dropdown {
                                selected: "{core.governor}",
                                items: governors.clone(),
                                disabled: false,
                                oninput: {
                                    let mut current_profile = current_profile.clone();
                                    let awaiting_signal = *cores_awaiting_update_signals
                                        .get(&logical_cpu_id)
                                        .unwrap();
                                    move |v: String| {
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            &selected(),
                                            move |core_settings| {
                                                core_settings.governor = Some(v.clone());
                                            },
                                            control_routine,
                                            awaiting_signal,
                                        );
                                    }
                                },
                                onclick: move |e: MouseEvent| {
                                    if selected().iter().any(|c| *c == logical_cpu_id) {
                                        e.stop_propagation();
                                    }
                                }
                            }
                        }
                        // Gov

                        // Epp
                        if cpu_info.has_epp || cpu_info.has_epb {
                            td {
                                Dropdown {
                                    selected: if cpu_info.has_epp {
                                        core.epp.clone().unwrap()
                                    } else {
                                        CPUSettings::translate_epb_to_epp(&core.epb.clone().unwrap())
                                    },
                                    items: epps.clone(),
                                    disabled: cpu_info.mode.clone().unwrap_or_default() == "active"
                                        && core.governor == "performance",
                                    tooltip: if cpu_info.mode.clone().unwrap_or_default() == "active"
                                        && core.governor == "performance"
                                    {
                                        Some((TooltipDirection::Top, labels::EPP_GOV_PERF_TT.to_string()))
                                    } else {
                                        None
                                    },
                                    oninput: {
                                        let mut current_profile = current_profile.clone();
                                        let awaiting_signal = *cores_awaiting_update_signals
                                            .get(&logical_cpu_id)
                                            .unwrap();
                                        move |v: String| {
                                            update_core_settings(
                                                &mut current_profile,
                                                profile_id,
                                                &selected(),
                                                move |core_settings| {
                                                    core_settings.epp = Some(v.clone());
                                                },
                                                control_routine,
                                                awaiting_signal,
                                            );
                                        }
                                    },
                                    onclick: move |e: MouseEvent| {
                                        if selected().iter().any(|c| *c == logical_cpu_id) {
                                            e.stop_propagation();
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        if cpu_info.hybrid && core.is_performance_core.is_some() {
                            td {
                                if core.is_performance_core.unwrap() {
                                    "P (n.a - {logical_cpu_id})"
                                } else {
                                    "E (n.a - {logical_cpu_id})"
                                }
                            }
                        } else {
                            td { "{logical_cpu_id}" }
                        }

                        td { "" }
                        td { "" }
                        td { "" }
                        td { "" }
                        td { "" }
                        td { "" }
                        td { "" }
                        td { "" }

                        if cpu_info.has_epp {
                            td { "" }
                        }
                    }
                }
            }
        }
    }
}

fn update_core_settings<F>(
    profile: &mut Profile,
    profile_id: u32,
    cpu_ids: &[u32],
    mut update: F,
    control_routine: ControlRoutine,
    awaiting_signal: Signal<bool>,
) where
    F: FnMut(&mut CoreSetting),
{
    let mut indices = Vec::new();
    for cpu_id in cpu_ids {
        let (core_setting, idx) = if let Some(ref mut cores) = profile.cpu_core_settings.cores {
            if let Some(idx) = cores.iter().position(|c| c.cpu_id == *cpu_id) {
                (&mut cores[idx], idx)
            } else {
                cores.push(CoreSetting {
                    cpu_id: *cpu_id,
                    ..Default::default()
                });
                let idx = cores.len() - 1;
                (cores.last_mut().unwrap(), idx)
            }
        } else {
            profile.cpu_core_settings.cores = Some(vec![CoreSetting {
                cpu_id: *cpu_id,
                ..Default::default()
            }]);
            (&mut profile.cpu_core_settings.cores.as_mut().unwrap()[0], 0)
        };

        indices.push(idx as u32);

        update(core_setting);
    }

    control_routine_send_multiple(
        control_routine,
        &[
            ControlAction::UpdateProfileReduced(
                profile_id,
                profile.clone().into(),
                ReducedUpdate::MultipleCPUCores(indices),
            ),
            ControlAction::GetProfilesInfo,
        ],
        Some(awaiting_signal),
    );
}

fn get_epps() -> Vec<String> {
    [
        "performance",
        "balance_performance",
        "default",
        "balance_power",
        "power",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn get_governors(mode: &str) -> Vec<String> {
    if mode == "active" {
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
    .collect()
}
