use std::time::Duration;

use dioxus::prelude::*;

use power_daemon::communication::client::ControlClient;
use power_daemon::systeminfo::SystemInfo;
use power_daemon::Config;
use power_daemon::DefaultProfileType;
use power_daemon::ProfilesInfo;
use power_daemon::ReducedUpdate;
use power_daemon::{communication::client::SystemInfoClient, Profile};

use crate::helpers::coroutine_extensions::{wait_for_diff_msg, wait_for_msg};

#[derive(PartialEq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum SystemInfoSyncType {
    None,
    Whole,
    CPU,
    PCI,
    USB,
    SATA,
    Firmware,
    Opt,
}

pub type SystemInfoRoutine = Coroutine<(Duration, SystemInfoSyncType)>;

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
            if *sync_type.as_ref().unwrap() != SystemInfoSyncType::Whole && system_info().is_none()
            {
                system_info.set(Some(
                    client
                        .get_system_info()
                        .await
                        .expect("Could not get system info"),
                ));
            }

            match sync_type.as_ref().unwrap() {
                SystemInfoSyncType::None => {}
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
                SystemInfoSyncType::PCI => {
                    system_info.as_mut().unwrap().pci_info = client
                        .get_pci_info()
                        .await
                        .expect("Could not get system info")
                }
                SystemInfoSyncType::USB => {
                    system_info.as_mut().unwrap().usb_info = client
                        .get_usb_info()
                        .await
                        .expect("Could not get system info")
                }
                SystemInfoSyncType::SATA => {
                    system_info.as_mut().unwrap().sata_info = client
                        .get_sata_info()
                        .await
                        .expect("Could not get system info")
                }
                SystemInfoSyncType::Firmware => {
                    system_info.as_mut().unwrap().firmware_info = client
                        .get_firmware_info()
                        .await
                        .expect("Could not get system info")
                }
                SystemInfoSyncType::Opt => {
                    system_info.as_mut().unwrap().opt_features_info = client
                        .get_optional_features_info()
                        .await
                        .expect("Could not get system info")
                }
            }

            tokio::select! {
                msg = wait_for_diff_msg((refresh_duration.unwrap(), sync_type.as_ref().unwrap().clone()), &mut rx) => {
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

#[derive(Clone, Debug, PartialEq)]
pub enum ControlAction {
    GetConfig,
    GetProfilesInfo,

    UpdateConfig(Box<Config>),

    CreateProfile(DefaultProfileType),
    ResetProfile(u32),
    RenameProfile(u32, String),
    RemoveProfile(u32),
    SwapProfiles(u32, u32),

    UpdateProfileReduced(u32, Box<Profile>, ReducedUpdate),

    GetProfileOverride,
    SetProfileOverride(String),
    RemoveProfileOverride,
}

pub async fn background_daemon_sync_routine(
    mut active_profile_name: Signal<Option<String>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
) {
    let control_client = ControlClient::new()
        .await
        .expect("Could not initialize control client");

    active_profile_name.set(
        control_client
            .get_active_profile_name()
            .await
            .unwrap()
            .into(),
    );

    loop {
        let name = control_client.get_active_profile_name().await.unwrap();

        if active_profile_name().unwrap() != name {
            let local_info_up_to_date = if let Some(profiles_info) = profiles_info() {
                profiles_info.get_active_profile().profile_name == name
            } else {
                false
            };

            if !local_info_up_to_date {
                control_routine_send_multiple(
                    control_routine,
                    &[
                        ControlAction::GetProfilesInfo,
                        ControlAction::GetConfig,
                        ControlAction::GetProfileOverride,
                    ],
                    None,
                );
            }

            active_profile_name.set(name.into());
        }

        tokio::time::sleep(std::time::Duration::from_secs_f32(1.0)).await;
    }
}

pub type ControlRoutine = Coroutine<(ControlAction, Option<Signal<bool>>)>;

pub async fn control_service(
    mut rx: UnboundedReceiver<(ControlAction, Option<Signal<bool>>)>,
    mut config: Signal<Option<Config>>,
    mut profiles_info: Signal<Option<ProfilesInfo>>,
    mut active_profile_override: Signal<Option<String>>,
) {
    let control_client = ControlClient::new()
        .await
        .expect("Could not initialize control client");

    loop {
        if let Ok(Some(sent_msg)) = rx.try_next() {
            let msg = sent_msg.0;
            if let Some(mut signal) = sent_msg.1 {
                signal.set(true);
            }

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
                    .update_config(*config)
                    .await
                    .expect("Could not update config"),
                ControlAction::UpdateProfileReduced(idx, updated, reduced_update) => control_client
                    .update_profile_reduced(idx, *updated, reduced_update)
                    .await
                    .expect("Could not update profile"),
                ControlAction::CreateProfile(profile_type) => control_client
                    .create_profile(profile_type)
                    .await
                    .expect("Could not create profile"),
                ControlAction::SwapProfiles(idx, new_idx) => control_client
                    .swap_profiles(idx, new_idx)
                    .await
                    .expect("Could not create profile"),
                ControlAction::ResetProfile(idx) => control_client
                    .reset_profile(idx)
                    .await
                    .expect("Could not reset profile"),
                ControlAction::RenameProfile(idx, name) => control_client
                    .update_profile_name(idx, name)
                    .await
                    .expect("Could not reset profile"),
                ControlAction::RemoveProfile(idx) => control_client
                    .remove_profile(idx)
                    .await
                    .expect("Could not remove profile"),
                ControlAction::GetProfileOverride => active_profile_override.set(
                    control_client
                        .get_profile_override()
                        .await
                        .expect("Could not obtain profile override"),
                ),
                ControlAction::SetProfileOverride(profile_name) => control_client
                    .set_profile_override(profile_name)
                    .await
                    .expect("Could not set profile override"),
                ControlAction::RemoveProfileOverride => control_client
                    .remove_profile_override()
                    .await
                    .expect("Could not remove profile override"),
            }

            if let Some(mut signal) = sent_msg.1 {
                signal.set(false);
            }
        }

        tokio::time::sleep(Duration::from_millis(20)).await
    }
}

pub fn control_routine_send_multiple(
    control_routine: ControlRoutine,
    actions: &[ControlAction],
    signal: Option<Signal<bool>>,
) {
    if let Some(mut signal) = signal {
        signal.set(true);
    }
    assert!(actions.len() > 1);
    for action in actions.iter().take(actions.len() - 1) {
        control_routine.send((action.clone(), None));
    }
    control_routine.send((actions.last().unwrap().clone(), signal));
}
