use std::{
    fs::{self, File},
    io::Read,
};

use serde::{Deserialize, Serialize};

use crate::helpers::{run_command, run_command_with_output_unchecked};

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
    pub screen_settings: ScreenSettings,
    pub radio_settings: RadioSettings,
    pub network_settings: NetworkSettings,
    pub aspm_settings: ASPMSettings,
    pub pci_settings: PCISettings,
    pub usb_settings: USBSettings,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CPUSettings {
    pub governor: Option<String>,
    pub energy_performance_preference: Option<String>,

    pub min_frequency: Option<u32>,
    pub max_frequency: Option<u32>,

    // Performance boosting cpu tech. intel turbo or amd precission boost
    pub boost: bool,
}

impl CPUSettings {
    pub fn apply(&self) {
        if let Some(ref epp) = self.energy_performance_preference {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference",
                epp
            ));
        }

        if let Some(ref governor) = self.governor {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor",
                governor
            ));
        }

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
    pub cpu_id: u32,
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

        for core in self.cores.as_ref().unwrap().iter() {
            if let Some(ref epp) = core.energy_performance_preference {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
                    epp, core.cpu_id,
                ));
            }

            if let Some(ref governor) = core.governor {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
                    governor, core.cpu_id,
                ));
            }

            if let Some(min_frequency) = core.min_frequency {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq",
                    min_frequency, core.cpu_id,
                ));
            }
            if let Some(max_frequency) = core.max_frequency {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq",
                    max_frequency, core.cpu_id
                ));
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScreenSettings {
    pub resolution: Option<String>,
    pub refresh_rate: Option<String>,
    pub brightness: Option<u32>,
}

impl ScreenSettings {
    pub fn apply(&self) {
        if let Some(ref resolution) = self.resolution {
            run_command(&format!("xrandr --mode {}", resolution));
        }
        if let Some(ref refresh_rate) = self.refresh_rate {
            run_command(&format!("xrandr -r {}", refresh_rate));
        }
        if let Some(brightness) = self.brightness {
            run_command(&format!("brightnessctl -s {}%", brightness));
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RadioSettings {
    pub block_wifi: Option<bool>,
    pub block_nfc: Option<bool>,
    pub block_bluetooth: Option<bool>,
}

impl RadioSettings {
    pub fn apply(&self) {
        if let Some(wifi) = self.block_wifi {
            run_command(&format!(
                "rfkill {} wifi",
                if wifi { "block" } else { "unblock" },
            ))
        }
        if let Some(nfc) = self.block_nfc {
            run_command(&format!(
                "rfkill {} nfc",
                if nfc { "block" } else { "unblock" },
            ))
        }
        if let Some(bt) = self.block_bluetooth {
            run_command(&format!(
                "rfkill {} bluetooth",
                if bt { "block" } else { "unblock" },
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkSettings {
    pub disable_ethernet: bool,

    pub disable_wifi_7: bool,
    pub disable_wifi_6: bool,
    pub disable_wifi_5: bool,

    pub enable_power_save: Option<bool>,
    // Ranges from 0-5, the bigger the value the more performance and less battery savings
    pub power_level: Option<u8>,

    // Will set power_scheme in iwlmvm. If the firmware controller is not iwlmvm will set force_cam = 0 in iwldvm for values == 3
    // 1 - active | 2 - balanced | 3 - low power
    pub power_scheme: Option<u8>,

    // Can tank performance if enabled
    pub enable_uapsd: Option<bool>,
}

impl NetworkSettings {
    pub fn apply(&self) {
        if self.disable_ethernet {
            Self::disable_all_ethernet_cards()
        }

        let uses_iwlmvm = if run_command_with_output_unchecked("lsmod | grep '^iwl.vm'")
            .0
            .contains("iwlmvm")
        {
            true
        } else {
            false
        };

        let firmware_parameters = if let Some(power_level) = self.power_level {
            if uses_iwlmvm {
                &format!("power_scheme = {}", power_level)
            } else if power_level == 3 {
                "force_cam = 0"
            } else {
                ""
            }
        } else {
            ""
        };

        let mut driver_parameters = String::new();

        if self.disable_wifi_5 {
            driver_parameters.push_str("disable_11ac=1 ")
        }
        if self.disable_wifi_6 {
            driver_parameters.push_str("disable_11ax=1 ")
        }
        if self.disable_wifi_7 {
            driver_parameters.push_str("disable_11be=1 ")
        }

        if let Some(enable_powersave) = self.enable_power_save {
            driver_parameters.push_str(&format!(
                "power_save={} ",
                if enable_powersave { "1" } else { "0" }
            ))
        }
        if let Some(power_level) = self.power_level {
            driver_parameters.push_str(&format!("power_level={power_level} "))
        }
        if let Some(enable_uapsd) = self.enable_uapsd {
            driver_parameters.push_str(&format!(
                "uapsd_disable={} ",
                if enable_uapsd { "0" } else { "1" }
            ))
        }

        let firmware_name = if uses_iwlmvm { "iwlmvm" } else { "iwldvm" };

        run_command(&format!(
            "modprobe -r {firmware_name} && modprobe -r iwlwifi && modprobe {firmware_name} {} && modprobe iwlwifi {}", firmware_parameters, driver_parameters,
        ));
    }

    fn disable_all_ethernet_cards() {
        let entries = fs::read_dir("/sys/class/net").expect("Could not read sysfs path");
        let eth_pattern = regex::Regex::new(r"^(eth|enp|ens|eno)").unwrap();

        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if eth_pattern.is_match(&name_str) {
                    run_command(&format!("ifconfig {} down", &name_str))
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]

pub struct ASPMSettings {
    pub mode: Option<String>,
}

impl ASPMSettings {
    pub fn apply(&self) {
        if let Some(ref mode) = self.mode {
            run_command(&format!(
                "echo {} > /sys/module/pcie_aspm/parameters/policy",
                mode
            ));
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum WhiteBlackList<T: PartialEq> {
    Whitelist(Vec<T>),
    Blacklist(Vec<T>),
}

impl<T: PartialEq> WhiteBlackList<T> {
    // If enable = true and no list provided, will return true for all items
    // If enable = true, will return true for items in whitelist or only for items outside of blacklist
    // If enable = false, will return false for all items
    pub fn should_enable_item(whiteblacklist: &Option<Self>, item: &T, enable: bool) -> bool {
        if !enable {
            // Always disable
            false
        } else if let Some(ref whiteblacklist) = whiteblacklist {
            match whiteblacklist {
                // If on whitelist always enable, otherwise always disable
                WhiteBlackList::Whitelist(ref list) => {
                    if list.iter().any(|i| i == item) {
                        true
                    } else {
                        false
                    }
                }
                // If on blacklist always disable, otherwise always enable
                WhiteBlackList::Blacklist(ref list) => {
                    if list.iter().any(|i| i == item) {
                        false
                    } else {
                        true
                    }
                }
            }
        } else {
            // No list, always enable
            true
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PCISettings {
    pub enable_power_management: bool,
    // whitelist or blacklist device to exlude/include.
    // Should be the name of the device under /sys/bus/pci/devices excluding the beggining 0000:
    pub whiteblacklist: Option<WhiteBlackList<String>>,
}

impl PCISettings {
    pub fn apply(&self) {
        let entries = fs::read_dir("/sys/bus/pci/devices")
            .expect("Could not read pci devices sysfs directory");

        for entry in entries {
            let entry = entry.expect("Could not read usb device sysfs entry");
            let path = entry.path();

            let device_name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_prefix("0000:")
                .unwrap();

            let enable_pm = WhiteBlackList::should_enable_item(
                &self.whiteblacklist,
                &device_name.to_string(),
                self.enable_power_management,
            );

            run_command(&format!(
                "echo {} > {}",
                if enable_pm { "auto" } else { "on" },
                path.join("power/control").to_str().unwrap()
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct USBSettings {
    pub enable_power_management: bool,
    // whitelist or blacklist to exlude/include (vendor id, product id)
    pub whiteblacklist: Option<WhiteBlackList<(String, String)>>,
}

impl USBSettings {
    pub fn apply(&self) {
        let entries = fs::read_dir("/sys/bus/usb/devices")
            .expect("Could not read usb devices sysfs directory");

        for entry in entries {
            let entry = entry.expect("Could not read usb device sysfs entry");
            let path = entry.path();

            let mut vendor_id = String::new();
            File::open(path.join("idVendor"))
                .expect("Could not read usb device vendor id")
                .read_to_string(&mut vendor_id)
                .expect("Could not read usb device vendor id");

            let mut product_id = String::new();
            File::open(path.join("idProduct"))
                .expect("Could not read usb device product id")
                .read_to_string(&mut product_id)
                .expect("Could not read usb device product id");

            let enable_pm = WhiteBlackList::should_enable_item(
                &self.whiteblacklist,
                &(vendor_id, product_id),
                self.enable_power_management,
            );

            run_command(&format!(
                "echo {} > {}",
                if enable_pm { "auto" } else { "on" },
                path.join("power/control").to_str().unwrap()
            ))
        }
    }
}
