use std::{collections::HashMap, fs, io};

use log::{error, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::helpers::{
    command_exists, file_content_to_bool, file_content_to_list, file_content_to_string,
    file_content_to_u32, run_command_with_output,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SystemInfo {
    pub cpu_info: CPUInfo,
    pub pci_info: PCIInfo,
    pub usb_info: USBInfo,
    pub sata_info: SATAInfo,
    pub firmware_info: FirmwareInfo,
    pub opt_features_info: OptionalFeaturesInfo,
}

impl SystemInfo {
    pub fn obtain() -> SystemInfo {
        trace!("Obtaining system info");

        SystemInfo {
            cpu_info: CPUInfo::obtain(),
            pci_info: PCIInfo::obtain(),
            usb_info: USBInfo::obtain(),
            sata_info: SATAInfo::obtain(),
            firmware_info: FirmwareInfo::obtain(),
            opt_features_info: OptionalFeaturesInfo::obtain(),
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
    pub has_epb: bool,

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

    pub total_min_frequency: u32,
    pub total_max_frequency: u32,

    pub scaling_min_frequency: u32,
    pub scaling_max_frequency: u32,

    pub is_performance_core: Option<bool>,

    pub governor: String,

    pub epp: Option<String>,
    pub epb: Option<String>,
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
                Some(file_content_to_string(format!(
                    "/sys/devices/system/cpu/{}/status",
                    well_known_driver_name
                )))
            },

            has_epp: fs::metadata(
                "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences",
            )
            .is_ok(),
            has_epb: fs::metadata("/sys/devices/system/cpu/cpu0/power/energy_perf_bias").is_ok(),

            // This feature is exclusive to intel
            has_perf_pct_scaling: fs::metadata("/sys/devices/system/cpu/intel_pstate/min_perf_pct")
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

        ret.total_max_frequency = ret
            .cores
            .iter()
            .map(|c| c.total_max_frequency)
            .max()
            .unwrap();
        ret.total_min_frequency = ret
            .cores
            .iter()
            .map(|c| c.total_min_frequency)
            .min()
            .unwrap();

        ret
    }

    pub fn sync_core_info(&mut self, secondary: &mut CPUInfo) {
        // A cpu won't stop being hybrid so if there's a mismatch it just means that a version could not capture the fact that the cpu is hybrid
        if secondary.hybrid != self.hybrid {
            secondary.hybrid = true;
            self.hybrid = true;
        }

        for core in &mut self.cores {
            let core_secondary = secondary
                .cores
                .iter_mut()
                .find(|c| c.logical_cpu_id == core.logical_cpu_id)
                .unwrap();

            if self.hybrid {
                if core.is_performance_core.is_none()
                    && core_secondary.is_performance_core.is_some()
                {
                    core.is_performance_core = core_secondary.is_performance_core;
                } else if core.is_performance_core.is_some()
                    && core_secondary.is_performance_core.is_none()
                {
                    core_secondary.is_performance_core = core.is_performance_core;
                }
            }
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

                let base_frequency_path = cpufreq_path.join("base_frequency");

                // Some cpu's straight up do not have base_frequency and it's
                // not mentioned anywhere in kernel docs
                let base_frequency = if fs::metadata(&base_frequency_path).is_ok() {
                    file_content_to_u32(base_frequency_path) / 1000
                } else {
                    0
                };

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
                let epb = if self.has_epb {
                    Some(file_content_to_string(
                        cpufreq_path.join(entry.path().join("power/energy_perf_bias")),
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

                    total_min_frequency: min_frequency,
                    total_max_frequency: max_frequency,

                    scaling_min_frequency,
                    scaling_max_frequency,

                    epp,
                    epb,

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

        cores.sort_by(|a, b| a.logical_cpu_id.cmp(&b.logical_cpu_id));

        let variations = base_frequency_variations.keys().count();

        let hybrid = match variations {
            1 => false,
            2 => true,
            _ => {
                error!("Unexpected CPU architecture. This program identifies hybrid cores by their base frequency and your CPU contains more than 2 possible base frequencies. Assuming CPU is not hybrid...");
                false
            }
        };

        if hybrid {
            let pcore_freq = base_frequency_variations.keys().max().unwrap();
            for cpu in cores.iter_mut() {
                if !cpu.online.unwrap_or(true) {
                    continue;
                }

                cpu.is_performance_core = Some(cpu.base_frequency == *pcore_freq);
            }
        }

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
pub struct PCIInfo {
    pub pci_devices: Vec<PCIDeviceInfo>,
    pub aspm_info: ASPMInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PCIDeviceInfo {
    pub display_name: String,
    pub pci_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ASPMInfo {
    pub supported_modes: Option<Vec<String>>,
}

impl PCIInfo {
    pub fn obtain() -> PCIInfo {
        let mut entries: Vec<_> = fs::read_dir("/sys/bus/pci/devices")
            .expect("Could not read sysfs directory")
            .filter_map(Result::ok)
            .collect();

        entries.sort_by(|a, b| {
            natord::compare(a.path().to_str().unwrap(), b.path().to_str().unwrap())
        });

        let mut pci_devices = Vec::new();

        for device in entries {
            let display_name = run_command_with_output(&format!(
                "lspci -s \"{}\" | sed -E 's/^[0-9a-f]+:[0-9a-f]+.[0-9] //; s/ \\(rev.*\\)//'",
                device.file_name().into_string().unwrap()
            ))
            .0;

            let pci_address = device.file_name().into_string().unwrap();

            pci_devices.push(PCIDeviceInfo {
                display_name,
                pci_address,
            })
        }
        PCIInfo {
            aspm_info: ASPMInfo::obtain(),
            pci_devices,
        }
    }
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
                            if let Some(stripped) = stripped {
                                String::from(stripped.strip_suffix("]").unwrap())
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct USBInfo {
    pub usb_devices: Vec<USBDeviceInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct USBDeviceInfo {
    pub display_name: String,
    pub id: String,
}

impl USBInfo {
    pub fn obtain() -> USBInfo {
        let mut usb_devices = Vec::new();
        let lsusb = run_command_with_output("lsusb").0;

        let re = Regex::new(r"ID (\w+:\w+) (.+)").unwrap();
        for line in lsusb.lines() {
            let captures = re.captures(line).unwrap();
            let name = &captures[2];
            let id = &captures[1];

            usb_devices.push(USBDeviceInfo {
                display_name: name.to_string(),
                id: id.to_string(),
            });
        }

        USBInfo { usb_devices }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SATAInfo {
    pub hosts: u32,
}

impl SATAInfo {
    pub fn obtain() -> SATAInfo {
        let raw = fs::read_dir("/sys/class/scsi_host/");
        match raw {
            Ok(itr) => SATAInfo {
                hosts: itr.filter_map(Result::ok).count() as u32,
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => SATAInfo { hosts: 0 },
            Err(e) => {
                panic!("Could not read sysfs dir: {e:?}")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FirmwareInfo {
    /// None if unsupported
    pub platform_profiles: Option<Vec<String>>,
}

impl FirmwareInfo {
    pub fn obtain() -> FirmwareInfo {
        let supports_acpi_profiles = fs::metadata("/sys/firmware/acpi/platform_profile").is_ok();
        FirmwareInfo {
            platform_profiles: if supports_acpi_profiles {
                file_content_to_list("/sys/firmware/acpi/platform_profile_choices").into()
            } else {
                None
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OptionalFeaturesInfo {
    pub supports_xautolock: bool,
    pub supports_xset: bool,
    pub supports_xrandr: bool,
    pub supports_brightnessctl: bool,

    pub supports_wifi_drivers: bool,
    pub supports_ifconfig: bool,

    pub audio_module: AudioModule,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AudioModule {
    SndHdaIntel,
    SndAc9Codec,
    Other,
}

impl OptionalFeaturesInfo {
    pub fn obtain() -> OptionalFeaturesInfo {
        OptionalFeaturesInfo {
            supports_xautolock: command_exists("xautolock"),
            supports_xset: command_exists("xset"),
            supports_xrandr: command_exists("xrandr"),
            supports_brightnessctl: command_exists("brightnessctl"),
            supports_wifi_drivers: fs::metadata("/sys/module/iwlwifi").is_ok()
                && (fs::metadata("/sys/module/iwlmvm").is_ok()
                    || fs::metadata("/sys/module/iwldvm").is_ok()),
            supports_ifconfig: command_exists("ifconfig"),
            audio_module: if fs::metadata("/sys/module/snd_hda_intel/").is_ok() {
                AudioModule::SndHdaIntel
            } else if fs::metadata("/sys/module/snd_ac97_codec/").is_ok() {
                AudioModule::SndAc9Codec
            } else {
                AudioModule::Other
            },
        }
    }
}
