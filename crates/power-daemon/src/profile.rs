use std::fs;

use serde::{Deserialize, Serialize};

use crate::helpers::run_command;

pub fn find_profile_index_by_name(vec: &Vec<Profile>, name: &str) -> usize {
    vec.iter().position(|p| p.profile_name == name).unwrap()
}

#[derive(Default)]
pub struct ProfilesInfo {
    pub active_profile: usize,
    pub profiles: Vec<Profile>,
}

impl ProfilesInfo {
    pub fn get_active_profile(&self) -> &Profile {
        &self.profiles[self.active_profile]
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Profile {
    #[serde(skip_deserializing, skip_serializing)]
    /// Name of the profile. Should match the profile filename
    pub profile_name: String,

    pub cpu_settings: CPUSettings,
    pub cpu_core_settings: CPUCoreSettings,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CPUSettings {
    pub governor: String,
    pub energy_performance_preference: String,

    pub min_frequency: Option<u32>,
    pub max_frequency: Option<u32>,

    // Performance boosting cpu tech. intel turbo or amd precission boost
    pub boost: bool,
}

impl CPUSettings {
    pub fn apply(&self) {
        run_command(&format!(
            "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference",
            self.energy_performance_preference
        ));

        run_command(&format!(
            "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor",
            self.governor
        ));

        if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/intel_pstate/no_turbo",
                if self.boost { '0' } else { '1' }
            ));
        } else if fs::metadata("/sys/devices/system/cpu/cpufreq/boost").is_ok() {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpufreq/boost",
                if self.boost { '1' } else { '0' }
            ));
        }

        if let Some(min_frequency) = self.min_frequency {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq",
                min_frequency
            ));
        }
        if let Some(max_frequency) = self.max_frequency {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq",
                max_frequency
            ));
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CPUCoreSettings {
    pub cores: Option<Vec<CoreSetting>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CoreSetting {
    pub max_frequency: Option<u32>,
    pub min_frequency: Option<u32>,
    pub governor: Option<String>,
    pub energy_performance_preference: Option<String>,
}

impl CPUCoreSettings {
    pub fn apply(&self) {
        if self.cores.is_none() {
            return;
        }

        for (id, core) in self.cores.as_ref().unwrap().iter().enumerate() {
            if let Some(ref epp) = core.energy_performance_preference {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
                    epp, id,
                ));
            }

            if let Some(ref governor) = core.governor {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
                    governor, id,
                ));
            }

            if let Some(min_frequency) = core.min_frequency {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq",
                    min_frequency, id,
                ));
            }
            if let Some(max_frequency) = core.max_frequency {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq",
                    max_frequency, id
                ));
            }
        }
    }
}
