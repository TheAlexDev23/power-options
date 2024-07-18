use std::time::Duration;

use dioxus::prelude::*;

use power_daemon::communication::client::ControlClient;
use power_daemon::systeminfo::SystemInfo;
use power_daemon::Config;
use power_daemon::ProfilesInfo;
use power_daemon::{communication::client::SystemInfoClient, Profile};

use crate::helpers::{wait_for_diff_msg, wait_for_msg};

#[derive(PartialEq, Clone)]
pub enum SystemInfoSyncType {
    Whole,
    CPU,
    ASPM,
}

pub async fn system_info_service(
    mut rx: UnboundedReceiver<(Duration, SystemInfoSyncType)>,
    mut system_info: Signal<Option<SystemInfo>>,
) {
    // Have we started refreshing yet
    let mut refreshing = false;
    let mut refresh_duration = None;
    let mut sync_type = None;

    let client = SystemInfoClient::new()
        .await
        .expect("Could not start system info client");

    loop {
        if refreshing {
            if *sync_type.as_ref().unwrap() != SystemInfoSyncType::Whole
                && system_info.read().is_none()
            {
                system_info.set(Some(
                    client
                        .get_system_info()
                        .await
                        .expect("Could not get system info"),
                ));
            }

            match sync_type.as_ref().unwrap() {
                SystemInfoSyncType::Whole => system_info.set(Some(
                    client
                        .get_system_info()
                        .await
                        .expect("Could not get system info"),
                )),
                SystemInfoSyncType::CPU => {
                    system_info.as_mut().unwrap().cpu_info = client
                        .get_cpu_info()
                        .await
                        .expect("Could not get system info")
                }
                SystemInfoSyncType::ASPM => {
                    system_info.as_mut().unwrap().aspm_info = client
                        .get_aspm_info()
                        .await
                        .expect("Could not get system info")
                }
            }

            tokio::select! {
                msg = wait_for_diff_msg((refresh_duration.unwrap(), sync_type.as_ref().unwrap().clone()), &mut rx) => {
                    let msg = msg;
                    refreshing = true;
                    refresh_duration = Some(msg.0);
                    sync_type = Some(msg.1);
                },
                _ = tokio::time::sleep(refresh_duration.unwrap()) => { },
            }
        } else {
            let msg = wait_for_msg(&mut rx).await;
            refresh_duration = Some(msg.0);
            sync_type = Some(msg.1);
            refreshing = true;
        }
    }
}

#[derive(PartialEq)]
pub enum ControlAction {
    GetConfig,
    GetProfilesInfo,

    UpdateConfig(Config),
    UpdateProfile(u32, Profile),

    SetProfileOverride(String),
    RemoveProfileOverride,
}

pub async fn control_service(
    mut rx: UnboundedReceiver<ControlAction>,
    mut config: Signal<Option<Config>>,
    mut profiles_info: Signal<Option<ProfilesInfo>>,
) {
    let control_client = ControlClient::new()
        .await
        .expect("Could not intialize control client");

    loop {
        if let Ok(Some(msg)) = rx.try_next() {
            match msg {
                ControlAction::GetConfig => {
                    config.set(Some(
                        control_client
                            .get_config()
                            .await
                            .expect("Could not obtain config"),
                    ));
                }
                ControlAction::GetProfilesInfo => profiles_info.set(Some(
                    control_client
                        .get_profiles_info()
                        .await
                        .expect("Could not obtain profiles info."),
                )),
                ControlAction::UpdateConfig(config) => control_client
                    .update_config(config)
                    .await
                    .expect("Could not update config"),
                ControlAction::UpdateProfile(idx, updated) => control_client
                    .update_profile(idx, updated)
                    .await
                    .expect("Could not update profile"),
                ControlAction::SetProfileOverride(profile_name) => control_client
                    .set_profile_override(profile_name)
                    .await
                    .expect("Could not set profile override"),
                ControlAction::RemoveProfileOverride => control_client
                    .remove_profile_override()
                    .await
                    .expect("Could not remove profile override"),
            }
        }

        tokio::time::sleep(Duration::from_millis(20)).await
    }
}
