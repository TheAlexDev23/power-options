use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    profile::{
        ASPMSettings, CPUCoreSettings, CPUSettings, KernelSettings, NetworkSettings, PCISettings,
        Profile, RadioSettings, SATASettings, ScreenSettings, USBSettings,
    },
    systeminfo::{CPUFreqDriver, SystemInfo},
    FirmwareSettings, SleepSettings,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum DefaultProfileType {
    Superpowersave,
    Powersave,
    Balanced,
    Performance,
    Ultraperformance,
}

impl DefaultProfileType {
    pub fn get_name_of_all() -> Vec<String> {
        use DefaultProfileType::*;
        vec![
            Superpowersave.get_name(),
            Powersave.get_name(),
            Balanced.get_name(),
            Performance.get_name(),
            Ultraperformance.get_name(),
        ]
    }

    pub fn get_name(&self) -> String {
        String::from(match self {
            DefaultProfileType::Superpowersave => "Powersave++",
            DefaultProfileType::Powersave => "Powersave",
            DefaultProfileType::Balanced => "Balanced",
            DefaultProfileType::Performance => "Performance",
            DefaultProfileType::Ultraperformance => "Performance++",
        })
    }

    pub fn from_name(name: String) -> Option<Self> {
        match name.as_str() {
            "Powersave++" => DefaultProfileType::Superpowersave.into(),
            "Powersave" => DefaultProfileType::Powersave.into(),
            "Balanced" => DefaultProfileType::Balanced.into(),
            "Performance" => DefaultProfileType::Performance.into(),
            "Performance++" => DefaultProfileType::Ultraperformance.into(),
            _ => None,
        }
    }
}

pub fn create_profile_file<P: AsRef<Path>>(
    directory_path: P,
    profile_type: DefaultProfileType,
    system_info: &SystemInfo,
) {
    debug!("Creating profile of type {profile_type:?}");

    let name = profile_type.get_name();

    create_profile_file_with_name(name, directory_path, profile_type, system_info);
}

pub fn create_empty_profile_file_with_name<P: AsRef<Path>>(directory_path: P, name: &str) {
    debug!("Generating empty profile");

    let profile = create_empty(name);

    let path = PathBuf::from(directory_path.as_ref()).join(format!("{name}.toml"));

    let mut file = File::create(path).expect("Could not create profile file");

    let content = toml::to_string_pretty(&profile).unwrap();

    trace!("{content}");

    file.write_all(content.as_bytes())
        .expect("Could not write to profile file");
}

pub fn create_profile_file_with_name<P: AsRef<Path>>(
    name: String,
    directory_path: P,
    profile_type: DefaultProfileType,
    system_info: &SystemInfo,
) {
    let mut profile = create_default(&name, profile_type, system_info);
    profile.profile_name = name.clone();

    let path = PathBuf::from(directory_path.as_ref()).join(format!("{name}.toml"));

    let mut file = File::create(path).expect("Could not create profile file");

    let content = toml::to_string_pretty(&profile).unwrap();

    trace!("{content}");

    file.write_all(content.as_bytes())
        .expect("Could not write to profile file");
}

pub fn create_default(
    name: &str,
    profile_type: DefaultProfileType,
    system_info: &SystemInfo,
) -> Profile {
    Profile {
        profile_name: String::from(name),
        base_profile: profile_type.into(),

        sleep_settings: sleep_settings_default(&profile_type),
        cpu_settings: cpu_settings_default(&profile_type, system_info),
        cpu_core_settings: CPUCoreSettings::default(),
        screen_settings: ScreenSettings::default(),
        radio_settings: radio_settings_default(&profile_type),
        network_settings: network_settings_default(&profile_type),
        aspm_settings: aspm_settings_default(&profile_type, system_info),
        pci_settings: pci_settings_default(&profile_type),
        usb_settings: usb_settings_default(&profile_type),
        sata_settings: sata_settings_default(&profile_type),
        kernel_settings: kernel_settings_default(&profile_type),
        firmware_settings: firmware_settings_default(&profile_type, system_info),
    }
}

pub fn create_empty(name: &str) -> Profile {
    Profile {
        profile_name: String::from(name),
        base_profile: None,

        ..Default::default()
    }
}

fn sleep_settings_default(profile_type: &DefaultProfileType) -> SleepSettings {
    match profile_type {
        DefaultProfileType::Superpowersave => SleepSettings {
            turn_off_screen_after: Some(10),
            suspend_after: Some(15),
        },
        DefaultProfileType::Powersave => SleepSettings {
            turn_off_screen_after: Some(15),
            suspend_after: Some(20),
        },
        DefaultProfileType::Balanced => SleepSettings {
            turn_off_screen_after: Some(20),
            suspend_after: Some(30),
        },
        DefaultProfileType::Performance => SleepSettings {
            turn_off_screen_after: Some(30),
            suspend_after: Some(45),
        },
        DefaultProfileType::Ultraperformance => SleepSettings {
            turn_off_screen_after: Some(45),
            suspend_after: Some(60),
        },
    }
}

pub fn cpu_settings_default(
    profile_type: &DefaultProfileType,
    system_info: &SystemInfo,
) -> CPUSettings {
    let cpu_info = &system_info.cpu_info;
    let intel = cpu_info.driver == CPUFreqDriver::Intel;
    let widespread_driver =
        cpu_info.driver == CPUFreqDriver::Amd || cpu_info.driver == CPUFreqDriver::Intel;

    let mode = if widespread_driver {
        Some(String::from("active"))
    } else {
        None
    };

    match profile_type {
        DefaultProfileType::Superpowersave => CPUSettings {
            mode,
            governor: Some(String::from("powersave")),
            energy_perf_ratio: if widespread_driver {
                Some(String::from("power"))
            } else {
                None
            },
            min_freq: None,
            max_freq: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(70),
            boost: if widespread_driver { Some(false) } else { None },
            hwp_dyn_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Powersave => CPUSettings {
            mode,
            governor: Some(String::from(if widespread_driver {
                "powersave"
            } else {
                // Like ondemand but more gradual
                "conservative"
            })),
            energy_perf_ratio: if widespread_driver {
                Some(String::from("balance_power"))
            } else {
                None
            },
            min_freq: None,
            max_freq: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(false) } else { None },
            hwp_dyn_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Balanced => CPUSettings {
            mode,
            governor: Some(String::from(if widespread_driver {
                "powersave"
            } else {
                "ondemand"
            })),
            energy_perf_ratio: if widespread_driver {
                Some(String::from("default"))
            } else {
                None
            },
            min_freq: None,
            max_freq: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dyn_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Performance => CPUSettings {
            mode,
            governor: Some(String::from(if widespread_driver {
                // To set EPP balance_performance we cannot set governor to performance. idk why the scaling driver behaves like that
                "powersave"
            } else {
                "performance"
            })),
            energy_perf_ratio: if widespread_driver {
                Some(String::from("balance_performance"))
            } else {
                None
            },
            min_freq: None,
            max_freq: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dyn_boost: if intel { Some(true) } else { None },
        },
        DefaultProfileType::Ultraperformance => CPUSettings {
            mode,
            governor: Some(String::from("performance")),
            energy_perf_ratio: if widespread_driver {
                Some(String::from("performance"))
            } else {
                None
            },
            min_freq: None,
            max_freq: None,
            min_perf_pct: Some(30),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dyn_boost: if intel { Some(true) } else { None },
        },
    }
}

pub fn radio_settings_default(profile_type: &DefaultProfileType) -> RadioSettings {
    match profile_type {
        DefaultProfileType::Superpowersave
        | DefaultProfileType::Powersave
        | DefaultProfileType::Balanced => RadioSettings {
            block_wifi: None,
            block_nfc: Some(true),
            block_bt: Some(true),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => RadioSettings {
            block_wifi: None,
            block_nfc: None,
            block_bt: None,
        },
    }
}

pub fn network_settings_default(profile_type: &DefaultProfileType) -> NetworkSettings {
    match profile_type {
        DefaultProfileType::Superpowersave => NetworkSettings {
            disable_ethernet: Some(true),
            disable_wifi_5: Some(false),
            disable_wifi_6: Some(true),
            disable_wifi_7: Some(true),
            enable_power_save: Some(true),
            power_level: Some(0),
            power_scheme: Some(3),
            enable_uapsd: Some(true),
        },
        DefaultProfileType::Powersave => NetworkSettings {
            disable_ethernet: Some(true),
            disable_wifi_5: Some(false),
            disable_wifi_6: Some(false),
            disable_wifi_7: Some(true),
            enable_power_save: Some(true),
            power_level: Some(1),
            power_scheme: Some(3),
            enable_uapsd: Some(false),
        },
        DefaultProfileType::Balanced => NetworkSettings {
            disable_ethernet: Some(false),
            disable_wifi_5: Some(false),
            disable_wifi_6: Some(false),
            disable_wifi_7: Some(false),
            enable_power_save: Some(true),
            power_level: Some(3),
            power_scheme: Some(2),
            enable_uapsd: Some(false),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => NetworkSettings {
            disable_ethernet: Some(false),
            disable_wifi_5: Some(false),
            disable_wifi_6: Some(false),
            disable_wifi_7: Some(false),
            enable_power_save: Some(false),
            power_level: Some(5),
            power_scheme: Some(1),
            enable_uapsd: Some(false),
        },
    }
}

pub fn aspm_settings_default(
    profile_type: &DefaultProfileType,
    system_info: &SystemInfo,
) -> ASPMSettings {
    if system_info.pci_info.aspm_info.supported_modes.is_none() {
        ASPMSettings { mode: None }
    } else {
        ASPMSettings {
            mode: Some(String::from(match profile_type {
                DefaultProfileType::Superpowersave => "powersupersave",
                DefaultProfileType::Powersave => "powersave",
                DefaultProfileType::Balanced => "default",
                DefaultProfileType::Performance => "performance",
                DefaultProfileType::Ultraperformance => "performance",
            })),
        }
    }
}

pub fn pci_settings_default(profile_type: &DefaultProfileType) -> PCISettings {
    match profile_type {
        DefaultProfileType::Superpowersave
        | DefaultProfileType::Powersave
        | DefaultProfileType::Balanced => PCISettings {
            enable_power_management: Some(true),
            whiteblacklist: None,
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => PCISettings {
            enable_power_management: Some(false),
            whiteblacklist: None,
        },
    }
}

pub fn usb_settings_default(profile_type: &DefaultProfileType) -> USBSettings {
    match profile_type {
        DefaultProfileType::Superpowersave
        | DefaultProfileType::Powersave
        | DefaultProfileType::Balanced => USBSettings {
            autosuspend_delay_ms: None,
            enable_pm: Some(true),
            whiteblacklist: None,
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => USBSettings {
            autosuspend_delay_ms: None,
            enable_pm: Some(false),
            whiteblacklist: None,
        },
    }
}

pub fn sata_settings_default(profile_type: &DefaultProfileType) -> SATASettings {
    match profile_type {
        DefaultProfileType::Superpowersave
        | DefaultProfileType::Powersave
        | DefaultProfileType::Balanced => SATASettings {
            active_link_pm_policy: Some(String::from("med_power_with_dipm")),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => SATASettings {
            active_link_pm_policy: Some(String::from("max_performance")),
        },
    }
}

pub fn kernel_settings_default(profile_type: &DefaultProfileType) -> KernelSettings {
    match profile_type {
        DefaultProfileType::Superpowersave => KernelSettings {
            disable_nmi_watchdog: Some(true),
            vm_writeback: Some(60),
            laptop_mode: Some(5),
        },
        DefaultProfileType::Powersave => KernelSettings {
            disable_nmi_watchdog: Some(true),
            vm_writeback: Some(45),
            laptop_mode: Some(5),
        },
        DefaultProfileType::Balanced => KernelSettings {
            disable_nmi_watchdog: Some(true),
            vm_writeback: Some(30),
            laptop_mode: Some(5),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => KernelSettings {
            disable_nmi_watchdog: Some(true),
            vm_writeback: Some(15),
            laptop_mode: Some(2),
        },
    }
}

fn firmware_settings_default(
    profile_type: &DefaultProfileType,
    system_info: &SystemInfo,
) -> FirmwareSettings {
    let firmware_info = &system_info.firmware_info;
    if let Some(ref profiles) = firmware_info.platform_profiles {
        let step = (profiles.len() - 1) as f64 / (5 - 1) as f64;
        let equally_spaced_values: Vec<String> = (0..5)
            .map(|i| profiles[(i as f64 * step) as usize].clone())
            .collect();

        FirmwareSettings {
            platform_profile: equally_spaced_values[*profile_type as usize].clone().into(),
        }
    } else {
        FirmwareSettings {
            platform_profile: None,
        }
    }
}
