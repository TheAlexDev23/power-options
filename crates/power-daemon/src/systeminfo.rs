use std::{collections::HashMap, fs};

use log::trace;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    helpers::{
        file_content_to_bool, file_content_to_list, file_content_to_string, file_content_to_u32,
    },
    CPUCoreSettings,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SystemInfo {
    pub cpu_info: CPUInfo,
    pub aspm_info: ASPMInfo,
}

impl SystemInfo {
    pub fn obtain() -> SystemInfo {
        trace!("Obtaining system info");

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CPUInfo {
    pub driver: CPUFreqDriver,
    pub mode: Option<String>,

    pub has_epp: bool,
    pub has_perf_pct_scaling: bool,

    pub hybrid: bool,
    pub cores: Vec<CoreInfo>,

    pub total_min_frequency: u32,
    pub total_max_frequency: u32,

    // None if unsupported
    pub boost: Option<bool>,
    // None if unsupported
    pub hwp_dynamic_boost: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CoreInfo {
    // Some CPU's status cannot be changed, and this value would be None
    pub online: Option<bool>,

    pub physical_core_id: u32,
    pub logical_cpu_id: u32,

    pub current_frequency: u32,
    pub base_frequency: u32,

    pub min_frequency: u32,
    pub max_frequency: u32,

    pub scaling_min_frequency: u32,
    pub scaling_max_frequency: u32,

    pub is_performance_core: Option<bool>,

    pub governor: String,
    // None if unsupported
    pub epp: Option<String>,
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

        let well_known_driver_name = if driver == CPUFreqDriver::Intel {
            "intel_pstate"
        } else if driver == CPUFreqDriver::Amd {
            "amd_pstate"
        } else {
            ""
        };

        let mut ret = CPUInfo {
            driver: driver.clone(),

            mode: if driver == CPUFreqDriver::Other {
                None
            } else {
                Some(file_content_to_string(&format!(
                    "/sys/devices/system/cpu/{}/status",
                    well_known_driver_name
                )))
            },

            has_epp: fs::metadata(
                "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences",
            )
            .is_ok(),
            // This feature is exclusive to intel
            has_perf_pct_scaling: fs::metadata(&format!(
                "/sys/devices/system/cpu/intel_pstate/min_perf_pct"
            ))
            .is_ok(),

            hybrid: false,
            cores: Vec::default(),

            total_min_frequency: 0,
            total_max_frequency: 0,

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
        };

        ret.obtain_core_info();

        ret.total_max_frequency = ret.cores.iter().map(|c| c.max_frequency).max().unwrap();
        ret.total_min_frequency = ret.cores.iter().map(|c| c.min_frequency).min().unwrap();

        ret
    }

    pub fn update_core_info_with_initial(&mut self, initial: &CPUInfo) {
        for core in &mut self.cores {
            if core.online.unwrap_or(true) {
                continue;
            }

            let core_initial = initial
                .cores
                .iter()
                .find(|c| c.logical_cpu_id == core.logical_cpu_id)
                .unwrap();

            if !core_initial.online.unwrap_or(true) {
                continue;
            }

            core.physical_core_id = core_initial.physical_core_id;
            core.base_frequency = core_initial.base_frequency;
            core.min_frequency = core_initial.min_frequency;
            core.max_frequency = core_initial.max_frequency;
            core.is_performance_core = core_initial.is_performance_core;
        }
    }

    fn obtain_core_info(&mut self) {
        let mut base_frequency_variations: HashMap<u32, Vec<usize>> = HashMap::new();

        let cpu_pattern = Regex::new(r"cpu\d+").unwrap();

        let mut cores = Vec::new();

        let mut count = 0;

        let mut entries: Vec<_> = fs::read_dir("/sys/devices/system/cpu/")
            .expect("Could not read sysfs directory")
            .filter_map(Result::ok)
            .collect();

        entries.sort_by(|a, b| {
            natord::compare(a.path().to_str().unwrap(), b.path().to_str().unwrap())
        });

        for entry in entries {
            if !cpu_pattern.is_match(entry.file_name().as_os_str().to_str().unwrap()) {
                continue;
            }
            let logical_cpu_id = count;
            count += 1;

            let online_path = entry.path().join("online");
            let online = if fs::metadata(&online_path).is_ok() {
                Some(file_content_to_bool(online_path))
            } else {
                None
            };

            // If online is missing the cpu cannot be taken offline so we should treat it as online
            if online.unwrap_or(true) {
                let physical_core_id = file_content_to_u32(entry.path().join("topology/core_id"));

                let cpufreq_path = entry.path().join("cpufreq/");

                let base_frequency =
                    file_content_to_u32(cpufreq_path.join("base_frequency")) / 1000;
                let current_frequency =
                    file_content_to_u32(cpufreq_path.join("scaling_cur_freq")) / 1000;

                let min_frequency =
                    file_content_to_u32(cpufreq_path.join("cpuinfo_min_freq")) / 1000;
                let max_frequency =
                    file_content_to_u32(cpufreq_path.join("cpuinfo_max_freq")) / 1000;

                let scaling_min_frequency =
                    file_content_to_u32(cpufreq_path.join("scaling_min_freq")) / 1000;
                let scaling_max_frequency =
                    file_content_to_u32(cpufreq_path.join("scaling_max_freq")) / 1000;

                let epp = if self.has_epp {
                    Some(file_content_to_string(
                        cpufreq_path.join("energy_performance_preference"),
                    ))
                } else {
                    None
                };

                let governor = file_content_to_string(cpufreq_path.join("scaling_governor"));

                if let Some(val) = base_frequency_variations.get_mut(&base_frequency) {
                    val.push(cores.len());
                } else {
                    base_frequency_variations.insert(base_frequency, vec![cores.len()]);
                }

                cores.push(CoreInfo {
                    online,

                    physical_core_id,
                    logical_cpu_id,

                    base_frequency,
                    current_frequency,

                    min_frequency,
                    max_frequency,

                    scaling_min_frequency,
                    scaling_max_frequency,

                    epp,
                    governor,

                    // These would be set later
                    is_performance_core: None,
                })
            } else {
                cores.push(CoreInfo {
                    online,
                    logical_cpu_id,
                    ..Default::default()
                })
            }
        }

        let variations = base_frequency_variations.keys().count();

        let hybrid = match variations {
            1 => false,
            2 => true,
            _ => panic!("Unexpected CPU architecture. Has {variations} sets of possible base frequencies, program only supports single core type and hybrid architectures with 2 cpu types")
        };

        if hybrid {
            let pcore_freq = base_frequency_variations.keys().max().unwrap();
            for cpu in cores.iter_mut() {
                cpu.is_performance_core = Some(cpu.base_frequency == *pcore_freq);
            }
        }

        cores.sort_by(|a, b| a.physical_core_id.partial_cmp(&b.physical_core_id).unwrap());

        let mut previous_core_id = cores[0].physical_core_id;
        cores[0].physical_core_id = 0;
        let mut core_id = 0;
        for core in cores.iter_mut() {
            let iter_core_id = core.physical_core_id;
            if iter_core_id == previous_core_id {
                core.physical_core_id = core_id;
            } else {
                core_id += 1;
                core.physical_core_id = core_id;
            }
            previous_core_id = iter_core_id;
        }

        self.hybrid = hybrid;
        self.cores = cores;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ASPMInfo {
    pub supported_modes: Option<Vec<String>>,
}

impl ASPMInfo {
    pub fn obtain() -> ASPMInfo {
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
