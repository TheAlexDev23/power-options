use std::{sync::Mutex, time::Duration};

use lazy_static::lazy_static;
use log::{debug, trace};
use power_daemon::communication::client::SystemInfoClient;
use tokio::sync::mpsc;

use super::SYSTEM_INFO;

lazy_static! {
    static ref SYSINFO_SYNC: Mutex<Option<mpsc::UnboundedSender<(Duration, SystemInfoSyncType)>>> =
        None.into();
}

#[derive(PartialEq, Clone, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
enum SyncingState {
    NotSyncing,
    Syncing(Duration, SystemInfoSyncType),
}

pub fn start_system_info_sync_routine() {
    debug!("Initializing system info synchronization routine");
    let (sender, mut receiver) = mpsc::unbounded_channel();
    *SYSINFO_SYNC.lock().unwrap() = Some(sender);
    tokio::spawn(async move {
        let system_info_client = SystemInfoClient::new().await.unwrap();

        let mut syncing_state = SyncingState::NotSyncing;

        loop {
            if syncing_state == SyncingState::NotSyncing {
                trace!("Waiting on first system info sync request");
                let (duration, sync_type) = receiver.recv().await.unwrap();
                syncing_state = SyncingState::Syncing(duration, sync_type);
            } else if let SyncingState::Syncing(ref duration, ref sync_type) = syncing_state {
                if *sync_type != SystemInfoSyncType::Whole && SYSTEM_INFO.is_none().await {
                    trace!("Obtaining partial system info without obtaining full system info first. Obtaining full system info.");
                    SYSTEM_INFO
                        .set(system_info_client.get_system_info().await.unwrap())
                        .await;
                }

                debug!("Syncing system info: {sync_type:?}");

                match sync_type {
                    SystemInfoSyncType::None => {}
                    SystemInfoSyncType::Whole => {
                        SYSTEM_INFO
                            .set(system_info_client.get_system_info().await.unwrap())
                            .await
                    }
                    SystemInfoSyncType::CPU => {
                        let updated = system_info_client.get_cpu_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| v.as_mut().unwrap().cpu_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::PCI => {
                        let updated = system_info_client.get_pci_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| v.as_mut().unwrap().pci_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::USB => {
                        let updated = system_info_client.get_usb_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| v.as_mut().unwrap().usb_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::SATA => {
                        let updated = system_info_client.get_sata_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| v.as_mut().unwrap().sata_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::Firmware => {
                        let updated = system_info_client.get_firmware_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| v.as_mut().unwrap().firmware_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::Opt => {
                        let updated = system_info_client
                            .get_optional_features_info()
                            .await
                            .unwrap();
                        SYSTEM_INFO
                            .set_mut(move |v| {
                                v.as_mut().unwrap().opt_features_info = updated.clone()
                            })
                            .await
                    }
                }

                tokio::select! {
                    (duration, sync_type) = crate::helpers::recv_different(&mut receiver, (*duration, sync_type.clone())) => {
                        syncing_state = SyncingState::Syncing(duration, sync_type);
                        trace!("Updating sync state: {syncing_state:?}");
                    },
                    _ = tokio::time::sleep(*duration) => {},
                }
            }
        }
    });
}

pub async fn obtain_full_info_once() {
    debug!("Obtaining full system info once");
    let system_info_client = SystemInfoClient::new().await.unwrap();
    SYSTEM_INFO
        .set(system_info_client.get_system_info().await.unwrap())
        .await;
}

pub fn set_system_info_sync(duration: Duration, sync_type: SystemInfoSyncType) {
    debug!("Updating system info synchronization: {sync_type:?} every {duration:?}");
    if let Some(sender) = SYSINFO_SYNC.lock().unwrap().as_ref() {
        sender.send((duration, sync_type)).unwrap();
    }
}
