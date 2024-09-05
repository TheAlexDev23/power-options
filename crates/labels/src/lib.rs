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

pub const DIS_NMI_TITLE: &str = "Disable NMI watchdog";
pub const DIS_NMI_TT: &str = "Disables the Kernel's NMI watchdog. A logging tool often used in Kernel development/debugging, it is often recommended to disable this feature";

pub const VM_WR_TITLE: &str = "VM writeback in seconds";
pub const VM_WR_TT: &str = "In modern operating systems data is first written to memory and then written to disk in predefined intervals for performance and power saving reasons. This option defines the interval for memory flushing to disk.";

pub const LAPTOP_MODE_TITLE: &str = "Laptop Mode";
pub const LAPTOP_MODE_TT: &str =
    "Overrides VM Writeback and other Kernel parameters when the system is in laptop-mode, i.e. off the wall. Requires ACPI to be installed.";

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
