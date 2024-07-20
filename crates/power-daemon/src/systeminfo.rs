use std::fs;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::helpers::{
    file_content_to_bool, file_content_to_list, file_content_to_string, file_content_to_u32,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
    pub cpu_info: CPUInfo,
    pub aspm_info: ASPMInfo,
}

impl SystemInfo {
    pub fn obtain() -> SystemInfo {
        debug!("Obtaining system info");

        SystemInfo {
            cpu_info: CPUInfo::obtain(),
            aspm_info: ASPMInfo::obtain(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CPUFreqDriver {
    Intel,
    Amd,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CPUInfo {
    pub driver: CPUFreqDriver,
    pub active_mode: Option<bool>,

    pub has_epp: bool,
    pub has_perf_pct_scaling: bool,

    pub scaling_min_frequency: u32,
    pub scaling_max_frequency: u32,

    pub total_min_frequency: u32,
    pub total_max_frequency: u32,

    // None if unsupported
    pub boost: Option<bool>,
    // None if unsupported
    pub hwp_dynamic_boost: Option<bool>,
}

impl CPUInfo {
    pub fn obtain() -> CPUInfo {
        debug!("Obtaining CPU info");

        let driver = if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
            CPUFreqDriver::Intel
        } else if fs::metadata("/sys/devices/system/cpu/amd_pstate").is_ok() {
            CPUFreqDriver::Amd
        } else {
            CPUFreqDriver::Other
        };

        let well_known_driver_name = if driver == CPUFreqDriver::Intel {
            "intel_pstate"
        } else if driver == CPUFreqDriver::Amd {
            "amd_pstate"
        } else {
            ""
        };

        CPUInfo {
            driver: driver.clone(),

            active_mode: if driver == CPUFreqDriver::Other {
                None
            } else {
                Some(
                    file_content_to_string(&format!(
                        "/sys/devices/system/cpu/{}/status",
                        well_known_driver_name
                    )) == "active",
                )
            },

            has_epp: fs::metadata(
                "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences",
            )
            .is_ok(),
            // This feature is exclusive to intel, but there's no need to check whether we use intel because if we do not then the file won't exist anyways
            has_perf_pct_scaling: fs::metadata(&format!(
                "/sys/devices/system/cpu/intel_pstate/min_perf_pct"
            ))
            .is_ok(),

            scaling_min_frequency: file_content_to_u32(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq",
            ),
            scaling_max_frequency: file_content_to_u32(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
            ),

            total_min_frequency: file_content_to_u32(
                "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq",
            ),
            total_max_frequency: file_content_to_u32(
                "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
            ),

            boost: match driver {
                CPUFreqDriver::Intel => Some(!file_content_to_bool(
                    "/sys/devices/system/cpu/intel_pstate/no_turbo",
                )),
                CPUFreqDriver::Amd => Some(file_content_to_bool(
                    "/sys/devices/system/cpu/cpufreq/boost",
                )),
                CPUFreqDriver::Other => None,
            },
            hwp_dynamic_boost: if let CPUFreqDriver::Intel = driver {
                Some(file_content_to_bool(
                    "/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost",
                ))
            } else {
                None
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ASPMInfo {
    pub supported_modes: Option<Vec<String>>,
}

impl ASPMInfo {
    pub fn obtain() -> ASPMInfo {
        debug!("Obtaining ASPM info");

        ASPMInfo {
            supported_modes: if fs::metadata("/sys/module/pcie_aspm/parameters/policy").is_err() {
                None
            } else {
                Some(
                    file_content_to_list("/sys/module/pcie_aspm/parameters/policy")
                        .into_iter()
                        .map(|s| {
                            // The current enabled mode is written [mode_name] when reading the sysfs entry
                            let stripped = s.strip_prefix("[");
                            if stripped.is_some() {
                                String::from(stripped.unwrap().strip_suffix("]").unwrap())
                            } else {
                                s
                            }
                        })
                        .collect(),
                )
            },
        }
    }
}
