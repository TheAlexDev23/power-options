use std::{
    fs::{self, read_link, DirEntry},
    path::PathBuf,
};

use super::{
    reading::{file_content_to_string, file_content_to_u32},
    writing::{write_str, write_u32},
};

pub struct IntelGpu {
    pub min_frequency: u32,
    pub max_frequency: u32,
    pub boost_frequency: u32,

    path: PathBuf,
}

impl IntelGpu {
    pub fn from_dir(entry: DirEntry) -> IntelGpu {
        IntelGpu {
            path: entry.path(),
            min_frequency: file_content_to_u32(entry.path().join("gt_min_freq_mhz")),
            max_frequency: file_content_to_u32(entry.path().join("gt_max_freq_mhz")),
            boost_frequency: file_content_to_u32(entry.path().join("gt_boost_freq_mhz")),
        }
    }

    pub fn set_min(&self, min: u32) {
        write_u32(self.path.join("gt_min_freq_mhz"), min);
    }
    pub fn set_max(&self, max: u32) {
        write_u32(self.path.join("gt_max_freq_mhz"), max);
    }
    pub fn set_boost(&self, boost: u32) {
        write_u32(self.path.join("gt_boost_freq_mhz"), boost);
    }
}

pub struct AmdGpu {
    pub driver: AmdGpuDriver,

    path: PathBuf,
}

pub enum AmdGpuDriver {
    AmdGpu { dpm_perf: String },
    Radeon { dpm_perf: String, dpm_state: String },
    Legacy { power_profile: String },
}

impl AmdGpu {
    pub fn from_dir(entry: DirEntry) -> AmdGpu {
        let driver_link = entry.path().join("device/driver");
        let driver_resolved = read_link(&driver_link).unwrap_or(driver_link);
        let driver = driver_resolved.to_string_lossy();

        AmdGpu {
            path: entry.path(),
            driver: if driver.contains("amdgpu") {
                AmdGpuDriver::AmdGpu {
                    dpm_perf: file_content_to_string(
                        entry
                            .path()
                            .join("device/power_dpm_force_performance_level"),
                    ),
                }
            } else {
                let dpm_performance_level_path = entry
                    .path()
                    .join("device/power_dpm_force_performance_level");

                let dpm_performance_state_path = entry
                    .path()
                    .join("device/power_dpm_force_performance_level");

                if fs::metadata(&dpm_performance_level_path).is_ok()
                    && fs::metadata(&dpm_performance_state_path).is_ok()
                {
                    AmdGpuDriver::Radeon {
                        dpm_perf: file_content_to_string(&dpm_performance_level_path),
                        dpm_state: file_content_to_string(&dpm_performance_state_path),
                    }
                } else {
                    AmdGpuDriver::Legacy {
                        power_profile: file_content_to_string(entry.path().join("power_profile")),
                    }
                }
            },
        }
    }

    pub fn set_dpm_perf_level(&self, perf_level: &str) {
        match &self.driver {
            AmdGpuDriver::AmdGpu { dpm_perf: _ } => {
                write_str(
                    self.path.join("device/power_dpm_force_performance_level"),
                    perf_level,
                );
            }
            AmdGpuDriver::Radeon {
                dpm_perf: _,
                dpm_state: _,
            } => {
                write_str(
                    self.path.join("device/power_dpm_force_performance_level"),
                    perf_level,
                );
            }
            AmdGpuDriver::Legacy { power_profile: _ } => {}
        }
    }

    pub fn set_dpm_power_state(&self, power_state: &str) {
        match &self.driver {
            AmdGpuDriver::AmdGpu { dpm_perf: _ } => {}
            AmdGpuDriver::Radeon {
                dpm_perf: _,
                dpm_state: _,
            } => {
                write_str(self.path.join("device/power_dpm_state"), power_state);
            }
            AmdGpuDriver::Legacy { power_profile: _ } => {}
        }
    }

    pub fn set_power_profile(&self, power_profile: &str) {
        match &self.driver {
            AmdGpuDriver::AmdGpu { dpm_perf: _ } => {}
            AmdGpuDriver::Radeon {
                dpm_perf: _,
                dpm_state: _,
            } => {}
            AmdGpuDriver::Legacy { power_profile: _ } => {
                write_str(self.path.join("device/power_method"), "profile");
                write_str(self.path.join("power_profile"), power_profile);
            }
        }
    }
}

pub fn iterate_intel_gpus() -> impl IntoIterator<Item = IntelGpu> {
    fs::read_dir("/sys/class/drm")
        .expect("Could not read sysfs drm directory")
        .flatten()
        .filter(|entry| {
            if !entry.file_name().into_string().unwrap().starts_with("card") {
                false
            } else {
                let driver_link = entry.path().join("device/driver");
                let driver_resolved = read_link(&driver_link).unwrap_or(driver_link);
                let driver = driver_resolved.to_string_lossy();
                driver.contains("i915")
            }
        })
        .map(IntelGpu::from_dir)
}

pub fn iterate_amd_gpus() -> impl IntoIterator<Item = AmdGpu> {
    fs::read_dir("/sys/class/drm")
        .expect("Could not read sysfs drm directory")
        .flatten()
        .filter(|entry| {
            if !entry.file_name().into_string().unwrap().starts_with("card") {
                false
            } else {
                let driver_link = entry.path().join("device/driver");
                let driver_resolved = read_link(&driver_link).unwrap_or(driver_link);
                let driver = driver_resolved.to_string_lossy();
                driver.contains("amdgpu") || driver.contains("radeon")
            }
        })
        .map(AmdGpu::from_dir)
}
