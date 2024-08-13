use std::fs;

use log::{debug, error, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    helpers::{file_content_to_string, run_command, run_command_with_output, WhiteBlackList},
    profiles_generator::{self, DefaultProfileType},
    ReducedUpdate, SystemInfo,
};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct ProfilesInfo {
    pub active_profile: usize,
    pub profiles: Vec<Profile>,
}

impl ProfilesInfo {
    pub fn get_active_profile(&self) -> &Profile {
        &self.profiles[self.active_profile]
    }
    pub fn find_profile_index_by_name(&self, name: &str) -> usize {
        self.profiles
            .iter()
            .position(|p| p.profile_name == name)
            .unwrap()
    }
    pub fn try_find_profile_index_by_name(&self, name: &str) -> Option<usize> {
        self.profiles.iter().position(|p| p.profile_name == name)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Profile {
    /// Name of the profile. Should match the profile filename
    pub profile_name: String,
    pub base_profile: DefaultProfileType,

    pub cpu_settings: CPUSettings,
    pub cpu_core_settings: CPUCoreSettings,
    pub screen_settings: ScreenSettings,
    pub radio_settings: RadioSettings,
    pub network_settings: NetworkSettings,
    pub aspm_settings: ASPMSettings,
    pub pci_settings: PCISettings,
    pub usb_settings: USBSettings,
    pub sata_settings: SATASettings,
    pub kernel_settings: KernelSettings,
}

impl Profile {
    pub fn apply_all(&self) {
        info!("Applying profile: {}", self.profile_name);

        let settings_functions: Vec<Box<dyn FnOnce() + Send>> = vec![
            Box::new(|| self.cpu_settings.apply()),
            Box::new(|| self.cpu_core_settings.apply()),
            Box::new(|| self.screen_settings.apply()),
            Box::new(|| self.radio_settings.apply()),
            Box::new(|| self.network_settings.apply()),
            Box::new(|| self.aspm_settings.apply()),
            Box::new(|| self.pci_settings.apply()),
            Box::new(|| self.usb_settings.apply()),
            Box::new(|| self.sata_settings.apply()),
            Box::new(|| self.kernel_settings.apply()),
        ];

        settings_functions.into_par_iter().for_each(|f| f());
    }

    pub fn apply_reduced(&self, reduced_update: &ReducedUpdate) {
        debug!("Applying reduced amount of settings: {reduced_update:?}");

        match reduced_update {
            ReducedUpdate::CPU => self.cpu_settings.apply(),
            ReducedUpdate::CPUCores => self.cpu_core_settings.apply(),
            ReducedUpdate::SingleCPUCore(idx) => {
                if let Some(ref cores) = self.cpu_core_settings.cores {
                    cores[*idx as usize].apply()
                }
            }
            ReducedUpdate::MultipleCPUCores(tochange) => {
                if let Some(ref cores) = self.cpu_core_settings.cores {
                    for idx in tochange.iter() {
                        cores[*idx as usize].apply()
                    }
                }
            }
            ReducedUpdate::Screen => self.screen_settings.apply(),
            ReducedUpdate::Radio => self.radio_settings.apply(),
            ReducedUpdate::Network => self.network_settings.apply(),
            ReducedUpdate::ASPM => self.aspm_settings.apply(),
            ReducedUpdate::PCI => self.pci_settings.apply(),
            ReducedUpdate::USB => self.usb_settings.apply(),
            ReducedUpdate::SATA => self.sata_settings.apply(),
            ReducedUpdate::Kernel => self.kernel_settings.apply(),
        }
    }

    pub fn get_original_values(&self, system_info: &SystemInfo) -> Profile {
        profiles_generator::create_default(
            &self.profile_name,
            self.base_profile.clone(),
            system_info,
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CPUSettings {
    // Scaling driver mode (active, passive) for intel_pstate (active, passive, guided) for amd_pstate
    pub mode: Option<String>,

    pub governor: Option<String>,
    pub epp: Option<String>,

    pub min_freq: Option<u32>,
    pub max_freq: Option<u32>,

    // Minimum allowed P-state scalling as percentage
    // Only supported on intel
    pub min_perf_pct: Option<u8>,
    // Maximum allowed P-state scalling as percentage
    // Only supported on intel
    pub max_perf_pct: Option<u8>,

    // Performance boosting cpu tech. intel turbo or amd precission boost
    pub boost: Option<bool>,
    // Intel only. Won't work in passive mode
    pub hwp_dyn_boost: Option<bool>,
}

impl CPUSettings {
    pub fn apply(&self) {
        info!("Applying CPU settings on {:?}", std::thread::current().id());

        if let Some(ref mode) = self.mode {
            if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/intel_pstate/status",
                    mode
                ))
            } else if fs::metadata("/sys/devices/system/cpu/amd_pstate").is_ok() {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/amd_pstate/status",
                    mode
                ))
            } else {
                error!("Scaling driver operation mode is only supported on intel_pstate and amd_pstate drivers.")
            }
        }

        // Governor and hwp_dynaamic_boost needs to run before epp options because those determine if epp is changable
        if let Some(hwp_dynamic_boost) = self.hwp_dyn_boost {
            let value = if hwp_dynamic_boost { "1" } else { "0" };
            if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost",
                    value
                ))
            } else {
                error!("HWP dynamic boost is currently only supported for intel CPUs with intel_pstate");
            }
        }

        if let Some(ref governor) = self.governor {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor",
                governor
            ));
        }

        if let Some(ref epp) = self.epp {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference",
                epp
            ));
        }

        if let Some(boost) = self.boost {
            if fs::metadata("/sys/devices/system/cpu/intel_pstate/no_turbo").is_ok() {
                // using intel turbo
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/intel_pstate/no_turbo",
                    if boost { '0' } else { '1' }
                ));
            } else if fs::metadata("/sys/devices/system/cpu/cpufreq/boost").is_ok() {
                // using amd precission boost
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpufreq/boost",
                    if boost { '1' } else { '0' }
                ));
            } else {
                error!("CPU boost technology is unsupported by your CPU/driver")
            }
        }

        if let Some(min_frequency) = self.min_freq {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq",
                min_frequency * 1000
            ));
        }
        if let Some(max_frequency) = self.max_freq {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq",
                max_frequency * 1000
            ));
        }

        if let Some(min_perf_pct) = self.min_perf_pct {
            if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/intel_pstate/min_perf_pct",
                    min_perf_pct
                ))
            } else {
                error!("Min/Max scaling perf percentage is currently only supported for intel CPUs with intel_pstate");
            }
        }
        if let Some(max_perf_pct) = self.max_perf_pct {
            if fs::metadata("/sys/devices/system/cpu/intel_pstate").is_ok() {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/intel_pstate/max_perf_pct",
                    max_perf_pct
                ))
            } else {
                error!("Min/Max scaling perf percentage is currently only supported for intel CPUs with intel_pstate");
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct CPUCoreSettings {
    pub cores: Option<Vec<CoreSetting>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct CoreSetting {
    pub cpu_id: u32,
    pub online: Option<bool>,
    pub max_frequency: Option<u32>,
    pub min_frequency: Option<u32>,
    pub governor: Option<String>,
    pub epp: Option<String>,
}

impl CPUCoreSettings {
    pub fn apply(&self) {
        info!(
            "Applying CPU core settings on {:?}",
            std::thread::current().id()
        );

        // In the UI, when disabling a core and then resetting the core override self.online would be set to None
        // But the user likely would have meant to return cpu back to the default values in the profile.
        // Given the way per-core settings work (first apply settings to all cores then individual overrides),
        // it's logical to also remove all the core-disabling overrides first and then maybe disable individual cores
        // Could this be fixed in the UI? Yes. Would it be better architecture-wise? Yes. But it's way easier to just to this
        run_command("echo 1 > /sys/devices/system/cpu/cpu*/online");

        if self.cores.is_none() {
            return;
        }

        for core in self.cores.as_ref().unwrap().iter() {
            core.apply();
        }
    }
}

impl CoreSetting {
    pub fn apply(&self) {
        if let Some(online) = self.online {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu{}/online",
                if online { "1" } else { "0" },
                self.cpu_id,
            ));
        }

        if let Some(ref epp) = self.epp {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
                epp, self.cpu_id,
            ));
        }

        if let Some(ref governor) = self.governor {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
                governor, self.cpu_id,
            ));
        }

        if let Some(min_frequency) = self.min_frequency {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq",
                min_frequency * 1000,
                self.cpu_id,
            ));
        }
        if let Some(max_frequency) = self.max_frequency {
            run_command(&format!(
                "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq",
                max_frequency * 1000,
                self.cpu_id
            ));
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct ScreenSettings {
    pub resolution: Option<String>,
    pub refresh_rate: Option<String>,
    pub brightness: Option<u32>,
}

impl ScreenSettings {
    pub fn apply(&self) {
        info!(
            "Applying Screen settings on {:?}",
            std::thread::current().id()
        );

        if let Some(ref resolution) = self.resolution {
            run_command(&format!("xrandr --mode {}", resolution));
        }
        if let Some(ref refresh_rate) = self.refresh_rate {
            run_command(&format!("xrandr -r {}", refresh_rate));
        }
        if let Some(brightness) = self.brightness {
            run_command(&format!("brightnessctl s {}%", brightness));
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RadioSettings {
    pub block_wifi: Option<bool>,
    pub block_nfc: Option<bool>,
    pub block_bt: Option<bool>,
}

impl RadioSettings {
    pub fn apply(&self) {
        info!(
            "Applying Radio settings on {:?}",
            std::thread::current().id()
        );

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
        if let Some(bt) = self.block_bt {
            run_command(&format!(
                "rfkill {} bluetooth",
                if bt { "block" } else { "unblock" },
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NetworkSettings {
    pub disable_ethernet: Option<bool>,

    pub disable_wifi_7: Option<bool>,
    pub disable_wifi_6: Option<bool>,
    pub disable_wifi_5: Option<bool>,

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
        info!(
            "Applying Network settings on {:?}",
            std::thread::current().id()
        );

        if self.disable_ethernet.unwrap_or_default() {
            Self::disable_all_ethernet_cards()
        }

        let uses_iwlmvm = if run_command_with_output("lsmod | grep '^iwl.vm'")
            .0
            .contains("iwlmvm")
        {
            true
        } else {
            false
        };

        let firmware_parameters = if let Some(power_scheme) = self.power_scheme {
            if uses_iwlmvm {
                &format!("power_scheme={}", power_scheme)
            } else if power_scheme == 3 {
                "force_cam=0"
            } else {
                ""
            }
        } else {
            ""
        };

        let mut driver_parameters = String::new();

        if let Some(val) = self.disable_wifi_5 {
            driver_parameters.push_str(&format!("disable_11ac={} ", if val { "1" } else { "0" }));
        }
        if let Some(val) = self.disable_wifi_6 {
            driver_parameters.push_str(&format!("disable_11ax={} ", if val { "1" } else { "0" }));
        }
        if let Some(val) = self.disable_wifi_7 {
            driver_parameters.push_str(&format!("disable_11be={} ", if val { "1" } else { "0" }));
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]

pub struct ASPMSettings {
    pub mode: Option<String>,
}

impl ASPMSettings {
    pub fn apply(&self) {
        info!(
            "Applying ASPM settings on {:?}",
            std::thread::current().id()
        );

        if let Some(ref mode) = self.mode {
            run_command(&format!(
                "echo {} > /sys/module/pcie_aspm/parameters/policy",
                mode
            ));
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PCISettings {
    pub enable_power_management: Option<bool>,
    // whitelist or blacklist device to exlude/include.
    // Should be the name of the device under /sys/bus/pci/devices excluding the beggining 0000:
    pub whiteblacklist: Option<WhiteBlackList>,
}

impl PCISettings {
    pub fn apply(&self) {
        info!(
            "Applying PCI PM settings on {:?}",
            std::thread::current().id()
        );

        if self.enable_power_management.is_none() {
            return;
        }

        let entries = fs::read_dir("/sys/bus/pci/devices").expect("Could not read sysfs directory");

        for entry in entries {
            let entry = entry.expect("Could not read sysfs entry");
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
                self.enable_power_management.unwrap(),
            );

            run_command(&format!(
                "echo {} > {}",
                if enable_pm { "auto" } else { "on" },
                path.join("power/control").display()
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct USBSettings {
    pub enable_pm: Option<bool>,
    pub autosuspend_delay_ms: Option<u32>,
    // whitelist or blacklist to exlude/include vendor_id:product_id
    pub whiteblacklist: Option<WhiteBlackList>,
}

impl USBSettings {
    pub fn apply(&self) {
        info!("Applying USB settings on {:?}", std::thread::current().id());

        let entries = fs::read_dir("/sys/bus/usb/devices").expect("Could not read sysfs directory");

        if self.enable_pm.is_none() {
            return;
        }

        for entry in entries {
            let entry = entry.expect("Could not read sysfs entry");
            let path = entry.path();

            // Those are hubs I believe, and they do not have product/vendor info so we skip them
            if path.file_name().unwrap().to_string_lossy().contains(":") {
                continue;
            }

            let vendor_id = file_content_to_string(path.join("idVendor"));
            let product_id = file_content_to_string(path.join("idProduct"));

            if let Some(enable_power_management) = self.enable_pm {
                let enable_pm = WhiteBlackList::should_enable_item(
                    &self.whiteblacklist,
                    &format!("{vendor_id}:{product_id}"),
                    enable_power_management,
                );

                run_command(&format!(
                    "echo {} > {}",
                    if enable_pm { "auto" } else { "on" },
                    path.join("power/control").display()
                ));

                if enable_pm {
                    if let Some(auto_suspend_ms) = self.autosuspend_delay_ms {
                        run_command(&format!(
                            "echo {auto_suspend_ms} > {}",
                            path.join("power/autosuspend_delay_ms").display()
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SATASettings {
    pub active_link_pm_policy: Option<String>,
}

impl SATASettings {
    pub fn apply(&self) {
        info!(
            "Applying SATA settings on {:?}",
            std::thread::current().id()
        );

        if self.active_link_pm_policy.is_none() {
            return;
        }

        let pm_policy = self.active_link_pm_policy.as_ref().unwrap();

        let entries =
            fs::read_dir("/sys/class/scsi_host/").expect("Could not read sysfs directory");

        for entry in entries {
            let entry = entry.expect("Could not read sysfs entry");
            let path = entry.path();

            run_command(&format!(
                "echo {pm_policy} > {}",
                path.join("link_power_management_policy").display()
            ))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct KernelSettings {
    pub disable_nmi_watchdog: Option<bool>,
    pub vm_writeback: Option<u32>,
    pub laptop_mode: Option<u32>,
}

impl KernelSettings {
    pub fn apply(&self) {
        info!(
            "Applying Kernel settings on {:?}",
            std::thread::current().id()
        );

        if let Some(disable_wd) = self.disable_nmi_watchdog {
            run_command(&format!(
                "echo {} > /proc/sys/kernel/nmi_watchdog",
                if disable_wd { "0" } else { "1" }
            ))
        }
        if let Some(vm_writeback) = self.vm_writeback {
            run_command(&format!(
                "echo {} > /proc/sys/vm/dirty_writeback_centisecs",
                vm_writeback * 100
            ))
        }
        if let Some(lm) = self.laptop_mode {
            run_command(&format!("echo {} > /proc/sys/vm/laptop_mode", lm))
        }
    }
}
