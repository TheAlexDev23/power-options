use std::time::Duration;

use dioxus::prelude::*;
use power_daemon::{AmdGpuInfo, GpuSettings, ProfilesInfo, ReducedUpdate, SystemInfo};

use crate::communication_services::{
    control_routine_send_multiple, ControlAction, ControlRoutine, SystemInfoRoutine,
    SystemInfoSyncType,
};
use crate::helpers::toggleable_components::{ToggleableDropdown, ToggleableNumericField};
use crate::helpers::toggleable_types::{ToggleableInt, ToggleableString};

#[derive(Debug, Clone, PartialEq, Default)]
struct GpuForm {
    pub intel_min: ToggleableInt,
    pub intel_max: ToggleableInt,
    pub intel_boost: ToggleableInt,

    pub amd_perf_level: ToggleableString,
    pub amd_perf_state: ToggleableString,
    pub amd_power_profile: ToggleableString,
}

impl GpuForm {
    pub fn new(gpu_settings: &GpuSettings) -> GpuForm {
        let mut ret = GpuForm::default();
        ret.set_values(gpu_settings);
        ret
    }

    pub fn set_values(&mut self, gpu_settings: &GpuSettings) {
        self.intel_min.from_u32(gpu_settings.intel_min);
        self.intel_max.from_u32(gpu_settings.intel_max);
        self.intel_boost.from_u32(gpu_settings.intel_boost);

        self.amd_perf_level
            .from(gpu_settings.amd_dpm_perf_level.clone());
        self.amd_perf_state
            .from(gpu_settings.amd_dpm_power_state.clone());
        self.amd_power_profile
            .from(gpu_settings.amd_power_profile.clone());
    }
}

#[component]
pub fn GpuGroup(
    profiles_info: Signal<Option<ProfilesInfo>>,
    system_info: Signal<Option<SystemInfo>>,
    system_info_routine: SystemInfoRoutine,
    control_routine: ControlRoutine,
) -> Element {
    system_info_routine.send((Duration::from_secs_f32(5.0), SystemInfoSyncType::Gpu));

    if profiles_info().is_none() || system_info().is_none() {
        return rsx! { "Connecting to the daemon.." };
    }

    let gpu_settings = profiles_info()
        .as_ref()
        .unwrap()
        .get_active_profile()
        .gpu_settings
        .clone();

    let system_info = system_info().unwrap();

    let intel_gpu_supported = system_info.gpu_info.intel_info.is_some();

    let mut supports_amd_gpu = false;
    let mut supports_amd_perf_level = false;
    let mut supports_amd_power_state = false;
    let mut supports_amd_power_profiles = false;

    if let Some(ref info) = system_info.gpu_info.amd_info {
        supports_amd_gpu = true;
        if matches!(info, AmdGpuInfo::AmdGpu { dpm_perf: _ }) {
            supports_amd_perf_level = true;
            supports_amd_power_state = false;
            supports_amd_power_profiles = false;
        }
        if matches!(
            info,
            AmdGpuInfo::Radeon {
                dpm_perf: _,
                dpm_state: _
            }
        ) {
            supports_amd_perf_level = true;
            supports_amd_power_state = true;
            supports_amd_power_profiles = false;
        }
        if matches!(info, AmdGpuInfo::Legacy { power_profile: _ }) {
            supports_amd_perf_level = false;
            supports_amd_power_state = false;
            supports_amd_power_profiles = true;
        }
    }

    let mut form_used_settings = use_signal(|| gpu_settings.clone());
    let mut form = use_hook(|| GpuForm::new(&gpu_settings));

    if form_used_settings() != gpu_settings {
        form.set_values(&gpu_settings);
        form_used_settings.set(gpu_settings.clone());
    }

    let mut changed = use_signal(|| false);
    let awaiting_completion = use_signal(|| false);

    let onsubmit = move || {
        let profiles_info = profiles_info().unwrap();
        let active_profile_idx = profiles_info.active_profile;
        let mut active_profile = profiles_info.get_active_profile().clone();

        active_profile.gpu_settings = GpuSettings {
            intel_min: if intel_gpu_supported {
                form.intel_min.into_u32()
            } else {
                None
            },
            intel_max: if intel_gpu_supported {
                form.intel_max.into_u32()
            } else {
                None
            },
            intel_boost: if intel_gpu_supported {
                form.intel_boost.into_u32()
            } else {
                None
            },
            amd_dpm_perf_level: if supports_amd_gpu && supports_amd_perf_level {
                form.amd_perf_level.into_base()
            } else {
                None
            },
            amd_dpm_power_state: if supports_amd_gpu && supports_amd_power_state {
                form.amd_perf_state.into_base()
            } else {
                None
            },
            amd_power_profile: if supports_amd_gpu && supports_amd_power_profiles {
                form.amd_power_profile.into_base()
            } else {
                None
            },
        };

        control_routine_send_multiple(
            control_routine,
            &[
                ControlAction::UpdateProfileReduced(
                    active_profile_idx as u32,
                    active_profile.into(),
                    ReducedUpdate::Gpu,
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
                        name: labels::INTEL_GPU_MIN,
                        disabled: !intel_gpu_supported,
                        tooltip: if !intel_gpu_supported {
                            Some(labels::INTEL_GPU_MISSING_TT.to_string())
                        } else {
                            None
                        },
                        value: form.intel_min
                    }
                }
                div { class: "option",
                    ToggleableNumericField {
                        name: labels::INTEL_GPU_MAX,
                        disabled: !intel_gpu_supported,
                        tooltip: if !intel_gpu_supported {
                            Some(labels::INTEL_GPU_MISSING_TT.to_string())
                        } else {
                            None
                        },
                        value: form.intel_max
                    }
                }
                div { class: "option",
                    ToggleableNumericField {
                        name: labels::INTEL_GPU_BOOST,
                        disabled: !intel_gpu_supported,
                        tooltip: if !intel_gpu_supported {
                            Some(labels::INTEL_GPU_MISSING_TT.to_string())
                        } else {
                            None
                        },
                        value: form.intel_boost
                    }
                }
            }

            div { class: "option-group",
                div { class: "option",
                    ToggleableDropdown {
                        name: labels::AMD_GPU_PERF_LEVEL,
                        value: form.amd_perf_level,
                        items: vec!["low".to_string(), "auto".to_string(), "high".to_string()],
                        disabled: !supports_amd_gpu || !supports_amd_perf_level,
                        tooltip: if !supports_amd_gpu {
                            Some(labels::AMD_GPU_MISSING_TT.to_string())
                        } else if !supports_amd_perf_level {
                            Some(labels::AMD_GPU_PERF_LEVEL_UNAVAILABLE.to_string())
                        } else {
                            Some(labels::AMD_GPU_PERF_LEVEL_TT.to_string())
                        }
                    }
                }
                div { class: "option",
                    ToggleableDropdown {
                        name: labels::AMD_GPU_STATE,
                        value: form.amd_perf_state,
                        items: vec!["battery".to_string(), "balanced".to_string(), "performance".to_string()],
                        disabled: !supports_amd_gpu || !supports_amd_power_state,
                        tooltip: if !supports_amd_gpu {
                            Some(labels::AMD_GPU_MISSING_TT.to_string())
                        } else if !supports_amd_power_state {
                            Some(labels::AMD_GPU_STATE_UNAVAILABLE.to_string())
                        } else {
                            Some(labels::AMD_GPU_STATE_TT.to_string())
                        }
                    }
                }
                div { class: "option",
                    ToggleableDropdown {
                        name: labels::AMD_GPU_POWER_PROFILE,
                        value: form.amd_power_profile,
                        items: vec![
                            "low".to_string(),
                            "mid".to_string(),
                            "high".to_string(),
                            "auto".to_string(),
                            "default".to_string(),
                        ],
                        disabled: !supports_amd_gpu || !supports_amd_power_profiles,
                        tooltip: if !supports_amd_gpu {
                            Some(labels::AMD_GPU_MISSING_TT.to_string())
                        } else if !supports_amd_power_profiles {
                            Some(labels::AMD_GPU_POWER_PROFILE_UNAVAILABLE.to_string())
                        } else {
                            Some(labels::AMD_GPU_POWER_PROFILE_TT.to_string())
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
                        form.set_values(&gpu_settings);
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
