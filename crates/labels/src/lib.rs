pub const SUSPEND_TITLE: &str = "Minutes of inactivity before the system goes into suspend mode.";
pub const SCREEN_TURN_OFF_TITLE: &str = "Minutes of inactivity before the display turns off.";

pub const SUSPEND_XAUTOLOCK_MISSING: &str =
    "System suspend settings require xautolock to be installed.";
pub const SCREEN_TURN_OFF_XSET_MISSING: &str =
    "Screen turn off settings require xset to be installed.";

pub const DRIVER_OPMODE_TITLE: &str = "Scaling driver operation mode";
pub const DRIVER_OPMODE_TT: &str = "The operation mode of the CPU frequency scaling driver. Passive will give control of frequency scaling to the CPUFreq kernel driver. In active mode the CPU manages frequencies.";
pub const DRIVER_OPMODE_UNAVAILABLE_TT: &str = "The ability to change the scaling driver operation mode is only available on intel_pstate and amd_pstate drivers.";

pub const EPP_TITLE: &str = "Energy to Performance Ratio";
pub const EPP_GOV_PERF_TT: &str = "EPP/EPB will be locked to the highest setting by the kernel when the governor is set to performance.";
pub const EPP_TT: &str = "Configures either the EPB or EPP features of the CPU. These features allow the user to select a preferable proportion between energy expense and performance.";
pub const EPP_UNAVAILABLE_TT: &str =
    "EPP/EPB is only available on Intel Core i-series 2nd gen or newer and AMD Zen 2 or newer.";

pub const GOV_TITLE: &str = "Scaling governor";
pub const GOV_TT: &str = "Parameter of the scaling driver that configures its behaviour. The available governors differ depending on whether the scaling driver is in passive/active operation mode.";

pub const MIN_FREQ_MHZ_TITLE: &str = "Minimum frequency (MHz)";
pub const MAX_FREQ_MHZ_TITLE: &str = "Maximum frequency (MHz)";

pub const MIN_PERF_PCT: &str = "Minimum performance percentage";
pub const MAX_PERF_PCT: &str = "Maximum performance percentage";
pub const MIN_PERF_PCT_TT: &str = "Configures the minimum allowed p-state as a percentage.";
pub const MAX_PERF_PCT_TT: &str = "Configures the maximum allowed p-state as a percentage.";
pub const PERF_PCT_UNAVAILABLE_TT: &str = "Configuring minimum/maximum p-state percentage is only available in Intel Core i-series 2nd gen or newer.";

pub const BOOST_TITLE: &str = "Boost technology";
pub const BOOST_TT: &str = "Enable/Disable Intel's Turbo Boost or AMD's Turbo Core features. Note that enabling this feature does not activate it, it just allows the CPU to boost when the load demands for it.";
pub const BOOST_UNAVAILABLE_TT: &str =
    "Boost technologies are only available in AMD and Intel CPU's.";

pub const HWP_DYN_BOOST_TITLE: &str = "HWP Dynamic Boost.";
pub const HWP_DYN_BOOST_TT: &str = "May improve performance by dynamically increasing the P-state limit whenever a task previously waiting for IO is selected to run.";
pub const HWP_DYN_BOOST_MODE_ACTIVE_TT: &str =
    "HWP Dynamic Boost is only supported when the scaling driver operation mode is set to active.";
pub const HWP_DYN_BOOST_UNAVAILABLE_TT: &str =
    "HWP Dynamic Boost is only available on Intel Core i-series 6th gen or newer.";

pub const DIS_ETH_TITLE: &str = "Disable Ethernet";
pub const DIS_ETH_TT: &str = "Some tools such as powertop report that the ethernet port uses 2-3 watts when not connected. While these values may be incorrect, disabling ethernet completely if not in use is common in users looking for best power savings in their devices.";

pub const IWLWIFI_POWERSAVING_TITLE: &str = "Enable WiFi driver powersaving";
pub const IWLWIFI_POWERSAVING_TT: &str =
    "Configures the power_save parameter in the iwlwifi network driver.";

pub const UAPSD_TITLE: &str = "Enable U-APSD";
pub const UAPSD_TT: &str = "U-APSD stands for Unscheduled Automatic Power Save Delivery and is a part of the 802.11e standard. It allows the Network card to go into standby mode when no packets are being received. May cause performance loss.";

pub const WIFI_POWERLEVEL_TITLE: &str = "WiFi driver power level (0-5)";
pub const WIFI_POWERLEVEL_TT: &str = "Configures the power_level parameter in the iwlwifi Kernel driver. A bigger value means less battery savings and more performance.";

pub const WIFI_POWERSCHEME_TITLE: &str = "WiFi driver power scheme (1-3)";
pub const WIFI_POWERSCHEME_TT: &str = "Configures the power_scheme parameter in the iwlmvm Kernel driver. 1 - means always on, 2 - balanced and 3 - lower power. If the the iwldvm driver is being used instead, the force_cam=0 parameter will be set if this value is equal to 3.";

pub const ASPM_TITLE: &str = "ASPM Operation Mode";
pub const ASPM_TT: &str = "PCIe Active State Power Management is a PCIe feature that allows a PCI link to be disabled if there is no traffic across it.";
pub const ASPM_UNSUPPORTED_TT: &str = "Your system does not PCIe Active State Power Management";

pub const SATA_ACTIVE_LINK_TITLE: &str = "Set SATA Active Link Power Management";
pub const SATA_ACTIVE_LINK_TT: &str = "SATA Active Link PM is a feature of the SATA Protocol which allows different performance to powersaving rates. med_power_with_dipm has been shown to save 1.0 to 1.5 watts and is enabled by default on most linux distributions. Be careful with min_power, it could cause data loss.";

pub const NO_IFCONFIG_TT: &str =
    "This option requires ifconfig. Install net-tools with you system's package manager.";
pub const NO_WIFI_DRIVER_TT: &str = "This option is unsupported for you network card and/or driver. Only Intel WiFi cards with the iwlwifi module, and iwldvm/iwlmvm firmware modules are supported.";
pub const NO_XRANDR_TT: &str = "This option requires xrandr to be installed in your system. Install xorg-xrandr with you system's package manager.";
pub const NO_BRIGHTNESSCTL_TT: &str =
    "This option requires brightnessctl to be installed in your system. Install brightnessctl with your system's package manager.";

pub const DIS_NMI_TITLE: &str = "Disable NMI watchdog";
pub const DIS_NMI_TT: &str = "Disables the Kernel's NMI watchdog. A logging tool often used in Kernel development/debugging, it is often recommended to disable this feature";

pub const VM_WR_TITLE: &str = "VM writeback in seconds";
pub const VM_WR_TT: &str = "In modern operating systems data is first written to memory and then written to disk in predefined intervals for performance and power saving reasons. This option defines the interval for memory flushing to disk.";

pub const LAPTOP_MODE_TITLE: &str = "Laptop Mode";
pub const LAPTOP_MODE_TT: &str =
    "Overrides VM Writeback and other Kernel parameters when the system is in laptop-mode, i.e. off the wall. Requires ACPI to be installed.";

pub const ACPI_PLATFORM_PROFILE_TITLE: &str = "ACPI platform profile";
pub const ACPI_PLATFORM_PROFILE_TT: &str = "Platform profiles is a feature in some Lenovo and newer laptops that controls characteristics around power-performance ratio, thermals and fan speed.";
pub const ACPI_PLATFORM_PROFILE_MISSING_TT: &str =
    "Platform profiles are not available in your system.";

pub const AUDIO_IDLE_TIMEOUT_TITLE: &str = "Audio Module Timeout in Seconds";
pub const AUDIO_IDLE_TIMEOUT_TT: &str =
    "Time in seconds of audio inactivity for the audio driver to go into idle mode.";
pub const AUDIO_IDLE_TIMEOUT_MODULE_UNSPORTED_TT: &str =
    "Audio Module Idle Timeout is only available on systems with the snd_hda_intel and snd_ac97_codec audio drivers.";

pub const INTEL_GPU_MIN: &str = "Minimum frequency of Intel GPU";
pub const INTEL_GPU_MAX: &str = "Maximum frequency of Intel GPU";
pub const INTEL_GPU_BOOST: &str = "Boost frequency of Intel GPU";
pub const INTEL_GPU_MISSING_TT: &str =
    "This setting is only available on Intel GPUs with the i915 kernel module.";

pub const AMD_GPU_PERF_LEVEL: &str = "AMD GPU DPM Performance Level";
pub const AMD_GPU_STATE: &str = "AMD GPU DPM State";
pub const AMD_GPU_POWER_PROFILE: &str = "AMD GPU Power Profile";

pub const AMD_GPU_MISSING_TT: &str = "This setting is only available on AMD GPUs.";

pub const AMD_GPU_PERF_LEVEL_TT: &str = "Standard setting for AMD GPU power management.";
pub const AMD_GPU_STATE_TT: &str = "Dynamic Power Management method. Available on Radeon module.";
pub const AMD_GPU_POWER_PROFILE_TT: &str = "Configures AMD GPU graphics clock speed. Only available in AMD GPU legacy Radeon module where other options are unsupported.";

pub const AMD_GPU_PERF_LEVEL_UNAVAILABLE: &str = "AMD GPU DPM Performance Levels are only available on AMD GPUs with AMDGPU or non-legacy Radeon modules.";
pub const AMD_GPU_STATE_UNAVAILABLE: &str =
    "AMD GPU DPM States are only available on AMD GPUs with non-legacy Radeon module.";
pub const AMD_GPU_POWER_PROFILE_UNAVAILABLE: &str =
    "AMD GPU Power Profiles are only available on AMD GPUs with legacy Radeon module.";

pub const RAPL_LONG_TERM_TITLE: &str = "Long Term Power Limit (Watts)";
pub const RAPL_SHORT_TERM_TITLE: &str = "Short Term Power Limit (Watts)";
pub const RAPL_PEAK_POWER_TITLE: &str = "Peak Power Limit (Watts)";

pub const RAPL_LONG_TERM_TT: &str =
    "The power limit that the CPU will be outside of bursts, most of the time.";
pub const RAPL_SHORT_TERM_TT: &str =
    "The power limit that the CPU will shortly be within on longer bursts.";
pub const RAPL_PEAK_POWER_TT: &str =
    "Maximum CPU burst power, can only be sustained for fractions of a second.";

pub const RAPL_CONSTRAINT_UNSUPPORTED: &str = "This RAPL constraint is not supported on your CPU";
pub const RAPL_INTERFACE_UNSUPPORTED: &str = "This RAPL interface is not supported on your CPU";
pub const RAPL_UNSUPPORTED: &str = "Intel RAPL is not supported on your CPU";
