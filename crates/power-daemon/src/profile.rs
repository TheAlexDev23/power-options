use std::{fs, io, sync::Mutex};

use log::{debug, error, info, warn};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    helpers::{
        command_exists, file_content_to_string, run_command, run_graphical_command,
        run_graphical_command_in_background, WhiteBlackList,
    },
    profiles_generator::{self, DefaultProfileType},
    ReducedUpdate, SystemInfo,
};

use lazy_static::lazy_static;

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

    pub sleep_settings: SleepSettings,
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
            Box::new(|| self.sleep_settings.apply()),
            Box::new(|| {
                self.cpu_settings.apply();
                self.cpu_core_settings.apply();
            }),
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
            ReducedUpdate::None => {}
            ReducedUpdate::Sleep => {
                self.sleep_settings.apply();
            }
            ReducedUpdate::CPU => {
                self.cpu_settings.apply();
                self.cpu_core_settings.apply();
            }
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

    pub fn parse_or_default(contents: &str, profile_name: &str) -> Profile {
        match toml::from_str(contents) {
            Ok(p) => p,
            Err(_) => {
                #[derive(Deserialize)]
                struct ProfileTypeOnly {
                    pub base_profile: DefaultProfileType,
                }

                warn!("Could not parse profile {profile_name}. Attempting to migrate to newer version.");

                let profile_type: DefaultProfileType = toml::from_str::<ProfileTypeOnly>(contents)
                    .expect("Could not parse base profile type")
                    .base_profile;

                let base_profile = toml::to_string(&profiles_generator::create_default(
                    profile_name,
                    profile_type,
                    &SystemInfo::obtain(),
                ))
                .expect("Could not merge default profile and user profile");

                let merged = serde_toml_merge::merge(
                    base_profile.parse().unwrap(),
                    contents.parse().unwrap(),
                )
                .unwrap();

                debug!("Merged profile {merged:?}");

                let profile =
                    Profile::deserialize(merged).expect("Could not parse updated profile");

                profile
            }
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
pub struct SleepSettings {
    /// Time to turn off screen after N minutes of inactivity
    pub turn_off_screen_after: Option<u32>,
    /// Time to suspend the device after N mintues of inactivity
    pub suspend_after: Option<u32>,
}

lazy_static! {
    pub static ref AUTOLOCK_INSTANCE: Mutex<Option<std::process::Child>> = Mutex::new(None);
}

impl SleepSettings {
    pub fn apply(&self) {
        info!(
            "Applying Sleep settings on {:?}",
            std::thread::current().id()
        );

        if let Some(turn_off_screen_after) = self.turn_off_screen_after {
            if command_exists("xset") {
                let time_in_secs = turn_off_screen_after * 60;
                run_graphical_command(&format!(
                    "xset dpms {time_in_secs} {time_in_secs} {time_in_secs}"
                ));
            } else {
                error!("Attempted to set screen turn off timeout when xset is not installed");
            }
        } else if command_exists("xset") {
            run_graphical_command("xset -dpms");
        }

        if let Some(suspend_after) = self.suspend_after {
            Self::kill_previous_autolock_instance();

            if command_exists("xautolock") {
                info!("Running autolock");
                *AUTOLOCK_INSTANCE.lock().unwrap() = run_graphical_command_in_background(&format!(
                    "xautolock -time {suspend_after} -locker 'systemctl suspend'"
                ))
                .into();
            } else {
                error!("Attempted to set suspend time when xautolock is not installed");
            }
        } else {
            Self::kill_previous_autolock_instance();
        }
    }

    fn kill_previous_autolock_instance() {
        debug!("Killing previous autolock instance");

        if command_exists("xautolock") {
            run_graphical_command("xautolock -exit");
        }

        let mut instance_lock = AUTOLOCK_INSTANCE.lock().unwrap();

        if let Some(instance) = instance_lock.as_mut() {
            instance
                .wait()
                .expect("Could not wait for xautolock process to exit after killing.");
        }

        *instance_lock = None;
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CPUSettings {
    // Scaling driver mode (active, passive) for intel_pstate (active, passive, guided) for amd_pstate
    pub mode: Option<String>,

    pub governor: Option<String>,

    /// The CPU EPP value as a string (digits are unsupported), if the system
    /// does not have EPP the EPB value will be set, however, this property
    /// needs to be set to the equivalent EPP value. Take a look at the
    /// [translation function](CPUSettings::translate_epp_to_epb) for more info
    pub energy_perf_ratio: Option<String>,

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

        if let Some(ref epp) = self.energy_perf_ratio {
            if fs::metadata("/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
                .is_ok()
            {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference",
                    epp
                ));
            } else if fs::metadata("/sys/devices/system/cpu/cpu0/power/energy_perf_bias").is_ok() {
                debug!(
                    "System does not have EPP but EPB is present, translating and setting EPB..."
                );
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu*/power/energy_perf_bias",
                    Self::translate_epp_to_epb(epp)
                ));
            } else {
                warn!("System does not have EPP or EPB but configuration attempted to set anyways. Ignoring...");
            }
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

    pub fn translate_epp_to_epb(epp: &str) -> String {
        match epp {
            "performance" => "performance",
            "balance_performance" => "balance-performance",
            "default" => "normal",
            "balance_power" => "balance-power",
            "power" => "power",
            _ => "normal",
        }
        .to_string()
    }

    pub fn translate_epb_to_epp(epb: &str) -> String {
        match epb {
            "0" => "performance",
            "4" => "balance_performance",
            "6" => "default",
            "8" => "balance_power",
            "15" => "power",

            "performance" => "performance",
            "balance-performance" => "balance_performance",
            "normal" => "default",
            "balance-power" => "balance_power",
            "power" => "power",
            _ => "default",
        }
        .to_string()
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
    /// The CPU EPP value as a string (digits are unsupported), if the system
    /// does not have EPP the EPB value will be set, however, this property
    /// needs to be set to the equivalent EPP value. Take a look at the
    /// [translation function](CPUSettings::translate_epp_to_epb) for more info
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
            if fs::metadata("/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
                .is_ok()
            {
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
                    epp, self.cpu_id,
                ));
            } else if fs::metadata("/sys/devices/system/cpu/cpu0/power/energy_perf_bias").is_ok() {
                debug!(
                    "System does not have EPP but EPB is present, translating and setting EPB..."
                );
                run_command(&format!(
                    "echo {} > /sys/devices/system/cpu/cpu{}/power/energy_perf_bias",
                    CPUSettings::translate_epp_to_epb(epp),
                    self.cpu_id
                ));
            } else {
                warn!("System does not have EPP or EPB but configuration attempted to set anyways. Ignoring...");
            }
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
            Self::try_run_xrandr(&format!("xrandr --mode {}", resolution));
        }
        if let Some(ref refresh_rate) = self.refresh_rate {
            Self::try_run_xrandr(&format!("xrandr -r {}", refresh_rate));
        }
        if let Some(brightness) = self.brightness {
            Self::try_set_brightness(&format!("brightnessctl s {}%", brightness));
        }
    }

    pub fn try_run_xrandr(command: &str) {
        if command_exists("xrandr") {
            run_graphical_command(command);
        } else {
            error!("xrandr is not present in the system. Ignoring settings utilizing it...");
        }
    }

    pub fn try_set_brightness(command: &str) {
        if command_exists("brightnessctl") {
            run_command(command);
        } else {
            error!("brightnessctl is not present in the system. Install it if you want brightness configuration. Ignoring settings utilizing it...");
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

        if let Some(disable_ethernet) = self.disable_ethernet {
            Self::toggle_all_ethernet_cards(disable_ethernet);
        }

        if !self.all_kernel_module_settings_are_none() {
            self.apply_kernel_module_settings();
        }
    }

    fn toggle_all_ethernet_cards(disable: bool) {
        if !command_exists("ifconfig") {
            error!("ifconfig is not present in the system, ignoring ethernet settings...");
        }

        let entries = fs::read_dir("/sys/class/net").expect("Could not read sysfs path");
        let eth_pattern = regex::Regex::new(r"^(eth|enp|ens|eno)").unwrap();

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if eth_pattern.is_match(&name_str) {
                run_command(&format!(
                    "ifconfig {} {}",
                    &name_str,
                    if disable { "down" } else { "up" }
                ))
            }
        }
    }

    fn all_kernel_module_settings_are_none(&self) -> bool {
        self.power_scheme.is_none()
            && self.disable_wifi_7.is_none()
            && self.disable_wifi_6.is_none()
            && self.disable_wifi_5.is_none()
            && self.enable_power_save.is_none()
            && self.power_level.is_none()
            && self.enable_uapsd.is_none()
    }

    fn apply_kernel_module_settings(&self) {
        let uses_iwlmvm = if fs::metadata("/sys/module/iwlmvm").is_ok() {
            debug!("Identified that the system uses iwlmvm");
            true
        } else if fs::metadata("/sys/module/iwldvm").is_ok() {
            debug!("Identified that the system uses iwldvm");
            false
        } else {
            error!("Could not identify spuported wifi firmware module. Expected either iwlmvm or iwldvm, neither found. Ignoring network kernel module settings...");
            return;
        };

        let firmware_parameters = if let Some(power_scheme) = self.power_scheme {
            if uses_iwlmvm {
                format!("power_scheme={}", power_scheme)
            } else if power_scheme == 3 {
                "force_cam=0".to_string()
            } else {
                String::default()
            }
        } else {
            String::default()
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
    // Should be the name of the device under /sys/bus/pci/devices
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

            let enable_pm = WhiteBlackList::should_enable_item(
                &self.whiteblacklist,
                path.file_name().unwrap().to_str().unwrap(),
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

        let entries = match fs::read_dir("/sys/class/scsi_host/") {
            Ok(itr) => itr,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return,
            Err(e) => panic!("Could not read sysfs directory: {e:?}"),
        };

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
