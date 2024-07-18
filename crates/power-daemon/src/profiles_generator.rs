use std::{fs::File, io::Write, path::PathBuf, str::FromStr};

use log::{debug, trace};

use crate::{
    profile::{
        ASPMSettings, CPUCoreSettings, CPUSettings, KernelSettings, NetworkSettings, PCISettings,
        Profile, RadioSettings, SATASettings, ScreenSettings, USBSettings,
    },
    systeminfo::{CPUFreqDriver, SystemInfo},
};

#[derive(Debug)]
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
}

pub fn create_profile_file(
    directory_path: &str,
    profile_type: DefaultProfileType,
    system_info: &SystemInfo,
) {
    debug!("Creating profile of type {profile_type:?}");

    let name = profile_type.get_name();

    let profile = create_default(&name, profile_type, system_info);

    let path = PathBuf::from_str(directory_path)
        .unwrap()
        .join(format!("{name}.toml"));

    let mut file = File::create(path).expect("Could not create profile file");

    let content = toml::to_string_pretty(&profile).unwrap();

    trace!("{content}");

    file.write(content.as_bytes())
        .expect("Could not write to profile file");
}

fn create_default(
    name: &str,
    profile_type: DefaultProfileType,
    system_info: &SystemInfo,
) -> Profile {
    Profile {
        profile_name: String::from(name),
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
            energy_performance_preference: if widespread_driver {
                Some(String::from("power"))
            } else {
                None
            },
            min_frequency: None,
            max_frequency: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(70),
            boost: if widespread_driver { Some(false) } else { None },
            hwp_dynamic_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Powersave => CPUSettings {
            mode,
            governor: Some(String::from(if widespread_driver {
                "powersave"
            } else {
                // Like ondemand but more gradual
                "conservative"
            })),
            energy_performance_preference: if widespread_driver {
                Some(String::from("balance_power"))
            } else {
                None
            },
            min_frequency: None,
            max_frequency: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(false) } else { None },
            hwp_dynamic_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Balanced => CPUSettings {
            mode,
            governor: Some(String::from(if widespread_driver {
                "powersave"
            } else {
                "ondemand"
            })),
            energy_performance_preference: if widespread_driver {
                Some(String::from("default"))
            } else {
                None
            },
            min_frequency: None,
            max_frequency: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dynamic_boost: if intel { Some(false) } else { None },
        },
        DefaultProfileType::Performance => CPUSettings {
            mode,
            governor: Some(String::from("performance")),
            energy_performance_preference: if widespread_driver {
                Some(String::from("balance_performance"))
            } else {
                None
            },
            min_frequency: None,
            max_frequency: None,
            min_perf_pct: Some(0),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dynamic_boost: if intel { Some(true) } else { None },
        },
        DefaultProfileType::Ultraperformance => CPUSettings {
            mode,
            governor: Some(String::from("performance")),
            energy_performance_preference: if widespread_driver {
                Some(String::from("performance"))
            } else {
                None
            },
            min_frequency: None,
            max_frequency: None,
            min_perf_pct: Some(30),
            max_perf_pct: Some(100),
            boost: if widespread_driver { Some(true) } else { None },
            hwp_dynamic_boost: if intel { Some(true) } else { None },
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
            block_bluetooth: Some(true),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => RadioSettings {
            block_wifi: None,
            block_nfc: None,
            block_bluetooth: None,
        },
    }
}

pub fn network_settings_default(profile_type: &DefaultProfileType) -> NetworkSettings {
    match profile_type {
        DefaultProfileType::Superpowersave => NetworkSettings {
            disable_ethernet: true,
            disable_wifi_5: false,
            disable_wifi_6: true,
            disable_wifi_7: true,
            enable_power_save: Some(true),
            power_level: Some(0),
            power_scheme: Some(3),
            enable_uapsd: Some(true),
        },
        DefaultProfileType::Powersave => NetworkSettings {
            disable_ethernet: true,
            disable_wifi_5: false,
            disable_wifi_6: false,
            disable_wifi_7: true,
            enable_power_save: Some(true),
            power_level: Some(1),
            power_scheme: Some(3),
            enable_uapsd: Some(false),
        },
        DefaultProfileType::Balanced => NetworkSettings {
            disable_ethernet: false,
            disable_wifi_5: false,
            disable_wifi_6: false,
            disable_wifi_7: false,
            enable_power_save: Some(true),
            power_level: Some(3),
            power_scheme: Some(2),
            enable_uapsd: Some(false),
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => NetworkSettings {
            disable_ethernet: false,
            disable_wifi_5: false,
            disable_wifi_6: false,
            disable_wifi_7: false,
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
    if system_info.aspm_info.supported_modes.is_none() {
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
            enable_power_management: true,
            whiteblacklist: None,
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => PCISettings {
            enable_power_management: false,
            whiteblacklist: None,
        },
    }
}

pub fn usb_settings_default(profile_type: &DefaultProfileType) -> USBSettings {
    match profile_type {
        DefaultProfileType::Superpowersave
        | DefaultProfileType::Powersave
        | DefaultProfileType::Balanced => USBSettings {
            enable_power_management: true,
            whiteblacklist: None,
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => USBSettings {
            enable_power_management: false,
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
            whiteblacklist: None,
        },
        DefaultProfileType::Performance | DefaultProfileType::Ultraperformance => SATASettings {
            active_link_pm_policy: Some(String::from("max_performance")),
            whiteblacklist: None,
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
