use log::debug;
use power_daemon::{
    ASPMInfo, ASPMSettings, AudioModule, AudioSettings, CPUInfo, CPUSettings, GpuInfo, GpuSettings,
    KernelSettings, NetworkSettings, PCISettings, RadioSettings, SATASettings, USBSettings,
};

use power_daemon::FirmwareInfo;
use power_daemon::FirmwareSettings;
use power_daemon::OptionalFeaturesInfo;

use crate::communications::{self, daemon_control, SYSTEM_INFO};

/// Iterates through all profiles and removes all possible None options. Except
/// for those that the system does not support and need to be set to None.
pub async fn remove_all_none_options() -> bool {
    debug!("Updating profiles to not have None values, unless those settings are unsupported.");

    if SYSTEM_INFO.is_none().await {
        communications::system_info::obtain_full_info_once().await;
    }

    assert!(!communications::SYSTEM_INFO.is_none().await);
    assert!(!communications::PROFILES_INFO.is_none().await);

    let info = communications::SYSTEM_INFO.get().await.clone().unwrap();

    let mut changed_any = false;

    for (idx, mut profile) in communications::PROFILES_INFO
        .get()
        .await
        .as_ref()
        .unwrap()
        .profiles
        .clone()
        .into_iter()
        .enumerate()
    {
        let initial = profile.clone();

        default_cpu_settings(&mut profile.cpu_settings, &info.cpu_info);

        // The CPU core settings component works by reading system info not
        // profiles info, so there is no need to update the individual core settings. As
        // those will be updated on demand by the component.

        default_radio_settings(&mut profile.radio_settings);
        default_network_settings(&mut profile.network_settings, &info.opt_features_info);
        default_pci_settings(&mut profile.pci_settings);
        default_aspm_settings(&mut profile.aspm_settings, &info.pci_info.aspm_info);
        default_usb_settings(&mut profile.usb_settings);
        default_sata_settings(&mut profile.sata_settings);
        default_kernel_settings(&mut profile.kernel_settings);
        default_firmware_settings(&mut profile.firmware_settings, &info.firmware_info);
        default_audio_settings(&mut profile.audio_settings, &info.opt_features_info);
        default_gpu_settings(&mut profile.gpu_settings, &info.gpu_info);

        if initial != profile {
            changed_any = true;
            daemon_control::update_profile_full(idx as u32, profile).await;
        }
    }

    if changed_any {
        daemon_control::get_profiles_info().await;
    }

    changed_any
}

fn default_cpu_settings(settings: &mut CPUSettings, cpu_info: &CPUInfo) {
    if settings.mode.is_none() {
        // cpu info mode will be none if unsupported so we won't be overriding
        // unsupported settings
        settings.mode = cpu_info.mode.clone();
    }

    if settings.governor.is_none() {
        // Available in both passive and active, the safest option
        settings.governor = String::from("powersave").into();
    }
    if settings.energy_perf_ratio.is_none() && cpu_info.has_epp {
        settings.energy_perf_ratio = String::from("default").into();
    }

    if settings.min_freq.is_none() {
        settings.min_freq = cpu_info.total_min_frequency.into();
    }
    if settings.max_freq.is_none() {
        settings.max_freq = cpu_info.total_max_frequency.into();
    }

    if settings.min_perf_pct.is_none() && cpu_info.has_perf_pct_scaling {
        settings.min_perf_pct = 0.into();
    }
    if settings.max_perf_pct.is_none() && cpu_info.has_perf_pct_scaling {
        settings.max_perf_pct = 100.into();
    }

    if settings.boost.is_none() {
        settings.boost = cpu_info.boost;
    }
    if settings.hwp_dyn_boost.is_none() {
        settings.boost = cpu_info.hwp_dynamic_boost;
    }
}

fn default_radio_settings(settings: &mut RadioSettings) {
    if settings.block_bt.is_none() {
        settings.block_bt = Some(false);
    }
    if settings.block_wifi.is_none() {
        settings.block_wifi = Some(false);
    }
    if settings.block_nfc.is_none() {
        settings.block_nfc = Some(false);
    }
}

fn default_network_settings(settings: &mut NetworkSettings, info: &OptionalFeaturesInfo) {
    if info.supports_ifconfig && settings.disable_ethernet.is_none() {
        settings.disable_ethernet = false.into();
    }

    if info.supports_wifi_drivers {
        if settings.disable_wifi_7.is_none() {
            settings.disable_wifi_7 = false.into();
        }
        if settings.disable_wifi_6.is_none() {
            settings.disable_wifi_6 = false.into();
        }
        if settings.disable_wifi_5.is_none() {
            settings.disable_wifi_5 = false.into();
        }
        if settings.enable_power_save.is_none() {
            settings.enable_power_save = false.into();
        }
        if settings.enable_uapsd.is_none() {
            settings.enable_uapsd = false.into();
        }
        if settings.power_level.is_none() {
            settings.power_level = 2.into();
        }
        if settings.power_scheme.is_none() {
            settings.power_scheme = 2.into();
        }
    }
}

fn default_pci_settings(settings: &mut PCISettings) {
    if settings.enable_power_management.is_none() {
        settings.enable_power_management = Some(false);
    }
    if settings.whiteblacklist.is_none() {
        settings.whiteblacklist = Some(power_daemon::WhiteBlackList {
            items: Vec::new(),
            list_type: power_daemon::WhiteBlackListType::Blacklist,
        })
    }
}

fn default_aspm_settings(settings: &mut ASPMSettings, info: &ASPMInfo) {
    if settings.mode.is_none() && info.supported_modes.is_some() {
        settings.mode = Some(info.supported_modes.as_ref().unwrap()[0].clone());
    }
}

fn default_usb_settings(settings: &mut USBSettings) {
    if settings.enable_pm.is_none() {
        settings.enable_pm = Some(false);
    }
    if settings.autosuspend_delay_ms.is_none() {
        settings.autosuspend_delay_ms = Some(10000);
    }
    if settings.whiteblacklist.is_none() {
        settings.whiteblacklist = Some(power_daemon::WhiteBlackList {
            items: Vec::new(),
            list_type: power_daemon::WhiteBlackListType::Blacklist,
        })
    }
}

fn default_sata_settings(settings: &mut SATASettings) {
    if settings.active_link_pm_policy.is_none() {
        settings.active_link_pm_policy = Some("med_power_with_dipm".to_string());
    }
}

fn default_kernel_settings(settings: &mut KernelSettings) {
    if settings.disable_nmi_watchdog.is_none() {
        settings.disable_nmi_watchdog = Some(true);
    }
    if settings.vm_writeback.is_none() {
        settings.vm_writeback = Some(10);
    }
    if settings.laptop_mode.is_none() {
        settings.laptop_mode = Some(5);
    }
}

fn default_firmware_settings(settings: &mut FirmwareSettings, info: &FirmwareInfo) {
    if settings.platform_profile.is_none() && info.platform_profiles.is_some() {
        settings.platform_profile = info.platform_profiles.as_ref().unwrap()
            [info.platform_profiles.as_ref().unwrap().len() / 2]
            .to_string()
            .into();
    }
}

fn default_audio_settings(settings: &mut AudioSettings, info: &OptionalFeaturesInfo) {
    if settings.idle_timeout.is_none() && info.audio_module != AudioModule::Other {
        // Usually the default
        settings.idle_timeout = Some(10);
    }
}

fn default_gpu_settings(settings: &mut GpuSettings, info: &GpuInfo) {
    if let Some(ref info) = info.intel_info {
        if settings.intel_min.is_none() {
            settings.intel_min = Some(info.min_frequency);
        }
        if settings.intel_max.is_none() {
            settings.intel_max = Some(info.max_frequency);
        }
        if settings.intel_boost.is_none() {
            settings.intel_boost = Some(info.boost_frequency);
        }
    }

    if let Some(ref info) = info.amd_info {
        match info {
            power_daemon::AmdGpuInfo::AmdGpu { dpm_perf } => {
                if settings.amd_dpm_perf_level.is_none() {
                    settings.amd_dpm_perf_level = dpm_perf.clone().into();
                }
            }
            power_daemon::AmdGpuInfo::Radeon {
                dpm_perf,
                dpm_state,
            } => {
                if settings.amd_dpm_perf_level.is_none() {
                    settings.amd_dpm_perf_level = dpm_perf.clone().into();
                }
                if settings.amd_dpm_power_state.is_none() {
                    settings.amd_dpm_power_state = dpm_state.clone().into();
                }
            }
            power_daemon::AmdGpuInfo::Legacy { power_profile } => {
                if settings.amd_power_profile.is_none() {
                    settings.amd_power_profile = power_profile.clone().into();
                }
            }
        }
    }
}
