#![allow(deprecated)]

pub mod communications;
pub mod components;
pub mod helpers;

use std::time::Duration;

use power_daemon::{CPUInfo, CPUSettings, RadioSettings};
use relm4::prelude::*;

use communications::{daemon_control, system_info::SystemInfoSyncType};
use components::*;

#[tokio::main]
async fn main() {
    communications::daemon_control::setup_control_client().await;

    tokio::join!(
        communications::daemon_control::get_profiles_info(),
        communications::daemon_control::get_profile_override(),
        communications::daemon_control::get_config(),
    );

    communications::system_info::start_system_info_sync_routine();
    communications::system_info::set_system_info_sync(
        Duration::from_secs_f32(5.0),
        SystemInfoSyncType::None,
    );

    communications::system_info::obtain_full_info_once().await;

    remove_all_none_options().await;

    RelmApp::new("io.github.thealexdev23.power-options.frontend").run_async::<App>(());
}

/// Iterates through all profiles and removes all possible None options. Except
/// for those that the system does not support and need to be set to None.
async fn remove_all_none_options() {
    assert!(!communications::SYSTEM_INFO.is_none().await);
    assert!(!communications::PROFILES_INFO.is_none().await);

    let info = communications::SYSTEM_INFO.get().await.clone().unwrap();

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

        if initial != profile {
            daemon_control::reset_reduced_update().await;
            daemon_control::update_profile(idx as u32, profile).await;
        }
    }

    daemon_control::get_profiles_info().await;
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
    if settings.epp.is_none() && cpu_info.has_epp {
        settings.epp = String::from("default").into();
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
