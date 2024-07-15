use std::fs;

use crate::helpers::{file_content_to_bool, file_content_to_list, file_content_to_u32};

pub struct SystemInfo {
    pub cpu_info: CPUInfo,
    pub aspm_info: ASPMInfo,
}

impl SystemInfo {
    pub fn obtain() -> SystemInfo {
        SystemInfo {
            cpu_info: CPUInfo::obtain(),
            aspm_info: ASPMInfo::obtain(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum CPUFreqDriver {
    Intel,
    Amd,
    Other,
}

pub struct CPUInfo {
    pub driver: CPUFreqDriver,

    pub governors: Vec<String>,
    pub energy_performance_preferences: Vec<String>,

    pub scaling_cur_frequency: u32,
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
        let driver = if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
            CPUFreqDriver::Intel
        } else if fs::metadata("/sys/devices/system/cpu/amd_pstate").is_ok() {
            CPUFreqDriver::Amd
        } else {
            CPUFreqDriver::Other
        };

        CPUInfo {
            driver: driver.clone(),

            governors: file_content_to_list(
                "/sys/devices/system/cpu0/cpufreq/scaling_available_governors",
            ),
            energy_performance_preferences: file_content_to_list(
                "/sys/devices/system/cpu0/cpufreq/energy_performance_available_preferences",
            ),

            scaling_cur_frequency: file_content_to_u32(
                "/sys/devices/sytem/cpu0/cpufreq/scaling_cur_freq",
            ),
            scaling_min_frequency: file_content_to_u32(
                "/sys/devices/sytem/cpu0/cpufreq/scaling_min_freq",
            ),
            scaling_max_frequency: file_content_to_u32(
                "/sys/devices/sytem/cpu0/cpufreq/scaling_max_freq",
            ),

            total_min_frequency: file_content_to_u32(
                "/sys/devices/sytem/cpu0/cpufreq/cpuinfo_min_freq",
            ),
            total_max_frequency: file_content_to_u32(
                "/sys/devices/sytem/cpu0/cpufreq/cpuinfo_max_freq",
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

pub struct ASPMInfo {
    pub supported_modes: Option<Vec<String>>,
}

impl ASPMInfo {
    pub fn obtain() -> ASPMInfo {
        ASPMInfo {
            supported_modes: if fs::metadata("path/sys/module/pcie_aspm/parameters/policy").is_err()
            {
                None
            } else {
                Some(
                    file_content_to_list("path/sys/module/pcie_aspm/parameters/policy")
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
