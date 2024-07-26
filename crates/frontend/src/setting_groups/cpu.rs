use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{CPUSettings, CoreSetting, Profile, ProfilesInfo, ReducedUpdate, SystemInfo};

use crate::communication_services::{ControlAction, SystemInfoSyncType};

use crate::helpers::{Dropdown, ToggleableDropdown, ToggleableNumericField, ToggleableToggle};

type ToggleableString = (Signal<bool>, Signal<String>);
type ToggleableInt = (Signal<bool>, Signal<i32>);
type ToggleableBool = (Signal<bool>, Signal<bool>);

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
        self.mode.0.set(cpu_settings.mode.is_some());
        self.mode
            .1
            .set(cpu_settings.mode.clone().unwrap_or(String::from("passive")));

        self.epp
            .0
            .set(cpu_settings.energy_performance_preference.is_some());
        self.epp.1.set(
            cpu_settings
                .energy_performance_preference
                .clone()
                .unwrap_or_default(),
        );

        self.governor.0.set(cpu_settings.governor.is_some());
        self.governor
            .1
            .set(cpu_settings.governor.clone().unwrap_or_default());

        self.min_freq.0.set(cpu_settings.min_frequency.is_some());
        self.min_freq
            .1
            .set(cpu_settings.min_frequency.unwrap_or_default() as i32);

        self.max_freq.0.set(cpu_settings.max_frequency.is_some());
        self.max_freq
            .1
            .set(cpu_settings.max_frequency.unwrap_or_default() as i32);

        self.min_perf_pct.0.set(cpu_settings.min_perf_pct.is_some());
        self.min_perf_pct
            .1
            .set(cpu_settings.min_perf_pct.unwrap_or_default() as i32);

        self.max_perf_pct.0.set(cpu_settings.max_perf_pct.is_some());
        self.max_perf_pct
            .1
            .set(cpu_settings.max_perf_pct.unwrap_or_default() as i32);

        self.boost.0.set(cpu_settings.boost.is_some());
        self.boost.1.set(cpu_settings.boost.unwrap_or_default());

        self.hwp_dyn_boost
            .0
            .set(cpu_settings.hwp_dyn_boost.is_some());
        self.hwp_dyn_boost
            .1
            .set(cpu_settings.hwp_dyn_boost.unwrap_or_default());
    }
}

#[component]
pub fn CPUGroup(
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: Coroutine<ControlAction>,
    system_info_routine: Coroutine<(Duration, SystemInfoSyncType)>,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(0.5), SystemInfoSyncType::CPU));
    if profiles_info.read().is_none() || system_info.read().is_none() {
        return rsx! { "Connecting to the daemon..." };
    }

    let profiles_info = profiles_info.read().as_ref().unwrap().clone();
    let system_info = system_info.read().as_ref().unwrap().clone();

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
            control_routine
        }
    }
}

#[component]
fn CPUSettingsForm(
    system_info: SystemInfo,
    profiles_info: ProfilesInfo,
    control_routine: Coroutine<ControlAction>,
) -> Element {
    let cpu_settings = profiles_info.get_active_profile().cpu_settings.clone();

    let cpu_info = system_info.clone().cpu_info;

    let mut changed = use_signal(|| false);

    let mode_supported = cpu_info.mode.is_some();
    let epp_supported = cpu_info.has_epp;
    let perf_pct_scaling_supported = cpu_info.has_perf_pct_scaling;
    let boost_supported = cpu_info.boost.is_some();
    let hwp_dyn_boost_supported = cpu_info.hwp_dynamic_boost.is_some();

    // The CPUSettings used to configure the form if these change, it means that the daemon settings changed so we would neet to refresh.
    let mut form_used_settings = use_signal(|| cpu_settings.clone());

    let mut form = use_hook(|| CPUForm::new(&cpu_settings));

    if cpu_settings != *form_used_settings.read() {
        form.set_values(&cpu_settings);
        form_used_settings.set(cpu_settings.clone());
    }

    let onsubmit = move || {
        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.cpu_settings = CPUSettings {
            mode: if mode_supported && form.mode.0.cloned() {
                Some(form.mode.1.cloned())
            } else {
                None
            },
            governor: if form.governor.0.cloned() {
                Some(form.governor.1.cloned())
            } else {
                None
            },
            energy_performance_preference: if epp_supported && form.epp.0.cloned() {
                Some(form.epp.1.cloned())
            } else {
                None
            },
            min_frequency: if form.min_freq.0.cloned() {
                Some(form.min_freq.1.cloned() as u32 * 1000)
            } else {
                None
            },
            max_frequency: if form.max_freq.0.cloned() {
                Some(form.max_freq.1.cloned() as u32 * 1000)
            } else {
                None
            },
            min_perf_pct: if form.min_perf_pct.0.cloned() {
                Some(form.min_perf_pct.1.cloned() as u8)
            } else {
                None
            },
            max_perf_pct: if form.max_perf_pct.0.cloned() {
                Some(form.max_perf_pct.1.cloned() as u8)
            } else {
                None
            },

            boost: if form.boost.0.cloned() {
                Some(form.boost.1.cloned())
            } else {
                None
            },

            hwp_dyn_boost: if *form.mode.1.read() == "active"
                && hwp_dyn_boost_supported
                && form.hwp_dyn_boost.0.cloned()
            {
                Some(form.hwp_dyn_boost.1.cloned())
            } else {
                None
            },
        };

        control_routine.send(ControlAction::SetReducedUpdate(ReducedUpdate::CPU));
        control_routine.send(ControlAction::UpdateProfile(
            active_profile_idx as u32,
            active_profile,
        ));
    };

    use_effect(move || {
        // If the mode overwriting is disabled we set it to reflect the system current opmode
        // The reasoning is: the user does not set an explicit override so the opmode is not guaranteed, therefore we will assume the value is what the system is currently at
        // And even though the current value of the system does not reflect the users selection, it still won't be set by the daemon as the override is disabled
        if !*form.mode.0.read() {
            if let Some(ref mode) = cpu_info.mode {
                form.mode.1.set(mode.clone());
            }
        }
    });

    let epps = get_epps();
    let governors = get_governors(&*form.mode.1.read());

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
                            name: String::from("Scaling driver operation mode"),
                            items: vec![String::from("active"), String::from("passive")],
                            value: form.mode
                        }
                    }
                }
            }

            div { class: "option-group",
                if epp_supported && *form.governor.1.read() != "performance" {
                    div { class: "option",
                        ToggleableDropdown {
                            name: String::from("Energy Performance Preference"),
                            items: epps,
                            value: form.epp
                        }
                    }
                }
                div { class: "option",
                    ToggleableDropdown { name: String::from("Governor"), items: governors, value: form.governor }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableNumericField { name: String::from("Minimum frequency (MHz)"), value: form.min_freq }
                }
                div { class: "option",
                    ToggleableNumericField { name: String::from("Maximum frequency (MHz)"), value: form.max_freq }
                }
            }

            if perf_pct_scaling_supported {
                div { class: "option-group",
                    div { class: "option",
                        ToggleableNumericField {
                            name: String::from("Minimum performance percentage"),
                            value: form.min_perf_pct
                        }
                    }
                    div { class: "option",
                        ToggleableNumericField {
                            name: String::from("Maximum performance percentage"),
                            value: form.max_perf_pct
                        }
                    }
                }
            }

            div { class: "option-group",
                if boost_supported {
                    div { class: "option",
                        ToggleableToggle { name: String::from("Boost technology"), value: form.boost }
                    }
                }

                if hwp_dyn_boost_supported && *form.mode.1.read() == "active" {
                    div { class: "option",
                        ToggleableToggle { name: String::from("HWP Dynamic Boost"), value: form.hwp_dyn_boost }
                    }
                }
            }

            div { class: "confirm-buttons",
                input {
                    r#type: "submit",
                    value: "Apply",
                    disabled: !changed.cloned()
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
    control_routine: Coroutine<ControlAction>,
) -> Element {
    let mut cpu_info = system_info.cpu_info.clone();
    let cpu_info_intial = use_hook(|| cpu_info.clone());
    cpu_info.update_core_info_with_initial(&cpu_info_intial);

    let cpu_core_settings = &profiles_info.get_active_profile().cpu_core_settings;

    let epps = get_epps();
    let governors = get_governors(&cpu_info.mode.clone().unwrap_or(String::from("passive")));

    let current_profile = profiles_info.get_active_profile().clone();
    let profile_id = profiles_info.active_profile as u32;

    rsx! {
        table { id: "cpu-cores-table",
            tr {
                th { "" }
                th { "On" }

                th { "CPU" }

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
                    class: if let Some(ref cores) = cpu_core_settings.cores {
                        if cores.iter().any(|c| c.cpu_id == logical_cpu_id as u32) {
                            "overriden"
                        } else {
                            ""
                        }
                    } else {
                        ""
                    },

                    td {
                        button {
                            onclick: {
                                let mut current_profile = current_profile.clone();
                                move |_| {
                                    reset_core_settings(
                                        &mut current_profile,
                                        profile_id,
                                        logical_cpu_id,
                                        control_routine,
                                    )
                                }
                            },
                            "Reset"
                        }
                    }

                    td {
                        if core.online.is_some() {
                            input {
                                onchange: {
                                    let mut current_profile = current_profile.clone();
                                    move |v| {
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            logical_cpu_id,
                                            move |core_settings| {
                                                core_settings.online = Some(v.value() == "true");
                                            },
                                            control_routine,
                                        );
                                    }
                                },
                                checked: "{core.online.unwrap()}",
                                r#type: "checkbox"
                            }
                        }
                    }

                    if cpu_info.hybrid {
                        td {
                            if core.is_performance_core.unwrap() {
                                "P ({core.physical_core_id}-{logical_cpu_id})"
                            } else {
                                "E ({core.physical_core_id}-{logical_cpu_id})"
                            }
                        }
                    } else {
                        td { "{core.physical_core_id}" }
                    }

                    td { "{core.base_frequency}" }
                    if core.online.unwrap_or(true) {
                        td { "{core.current_frequency}" }
                    } else {
                        td { "" }
                    }

                    td { "{core.min_frequency}-{core.max_frequency}" }

                    if core.online.unwrap_or(true) {
                        td {
                            Dropdown {
                                id: "governor-dd-cpu{logical_cpu_id}",
                                selected: "{core.governor}",
                                items: governors.clone(),
                                disabled: false,
                                onchange: {
                                    let mut current_profile = current_profile.clone();
                                    move |v: String| {
                                        update_core_settings(
                                            &mut current_profile,
                                            profile_id,
                                            logical_cpu_id,
                                            move |core_settings| {
                                                core_settings.governor = Some(v.clone());
                                            },
                                            control_routine,
                                        );
                                    }
                                }
                            }
                        }
                        if cpu_info.has_epp {
                            td {
                                Dropdown {
                                    id: "epp-dd-cpu{logical_cpu_id}",
                                    selected: "{core.epp.clone().unwrap()}",
                                    items: epps.clone(),
                                    disabled: cpu_info.mode.as_ref().unwrap() != "active",
                                    onchange: {
                                        let mut current_profile = current_profile.clone();
                                        move |v: String| {
                                            update_core_settings(
                                                &mut current_profile,
                                                profile_id,
                                                logical_cpu_id,
                                                move |core_settings| {
                                                    core_settings.epp = Some(v.clone());
                                                },
                                                control_routine,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        td { "" }
                        if cpu_info.has_epp {
                            td { "" }
                        }
                    }
                }
            }
        }

        br {}
        br {}
        br {}
    }
}

fn update_core_settings<F>(
    profile: &mut Profile,
    profile_id: u32,
    cpu_id: u32,
    mut update: F,
    control_routine: Coroutine<ControlAction>,
) where
    F: FnMut(&mut CoreSetting),
{
    let (core_setting, idx) = if let Some(ref mut cores) = profile.cpu_core_settings.cores {
        if let Some(idx) = cores.iter().position(|c| c.cpu_id == cpu_id) {
            (&mut cores[idx], idx)
        } else {
            cores.push(CoreSetting {
                cpu_id,
                ..Default::default()
            });
            let idx = cores.len() - 1;
            (cores.last_mut().unwrap(), idx)
        }
    } else {
        profile.cpu_core_settings.cores = Some(vec![CoreSetting {
            cpu_id,
            ..Default::default()
        }]);
        (&mut profile.cpu_core_settings.cores.as_mut().unwrap()[0], 0)
    };

    update(core_setting);

    control_routine.send(ControlAction::SetReducedUpdate(
        ReducedUpdate::SingleCPUCore(idx as u32),
    ));
    control_routine.send(ControlAction::UpdateProfile(profile_id, profile.clone()));
    control_routine.send(ControlAction::GetProfilesInfo);
}

fn reset_core_settings(
    profile: &mut Profile,
    profile_id: u32,
    cpu_id: u32,
    control_routine: Coroutine<ControlAction>,
) {
    if let Some(ref mut cores) = profile.cpu_core_settings.cores {
        if let Some(pos) = cores.iter().position(|c| c.cpu_id == cpu_id) {
            cores.remove(pos);
            control_routine.send(ControlAction::SetReducedUpdate(ReducedUpdate::CPUCores));
            control_routine.send(ControlAction::UpdateProfile(profile_id, profile.clone()));
            control_routine.send(ControlAction::GetProfilesInfo);
        }
    };
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
