use std::{sync::Mutex, time::Duration};

use lazy_static::lazy_static;
use power_daemon::communication::client::SystemInfoClient;
use tokio::sync::mpsc;

use super::SYSTEM_INFO;

lazy_static! {
    static ref SYSINFO_SYNC: Mutex<Option<mpsc::UnboundedSender<(Duration, SystemInfoSyncType)>>> =
        None.into();
}

#[derive(PartialEq, Clone)]
pub enum SystemInfoSyncType {
    None,
    Whole,
    CPU,
    PCI,
    USB,
    SATA,
}

#[derive(PartialEq, Clone)]
enum SyncingState {
    NotSyncing,
    Syncing(Duration, SystemInfoSyncType),
}

pub fn start_system_info_sync_routine() {
    let (sender, mut receiver) = mpsc::unbounded_channel();
    *SYSINFO_SYNC.lock().unwrap() = Some(sender);
    tokio::spawn(async move {
        let system_info_client = SystemInfoClient::new().await.unwrap();

        let mut syncing_state = SyncingState::NotSyncing;

        loop {
            if syncing_state == SyncingState::NotSyncing {
                let (duration, sync_type) = receiver.recv().await.unwrap();
                syncing_state = SyncingState::Syncing(duration, sync_type);
            } else if let SyncingState::Syncing(ref duration, ref sync_type) = syncing_state {
                if *sync_type != SystemInfoSyncType::Whole && SYSTEM_INFO.is_none_blocking() {
                    SYSTEM_INFO
                        .set(system_info_client.get_system_info().await.unwrap())
                        .await;
                }

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
                            .set_mut(move |mut v| v.as_mut().unwrap().cpu_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::PCI => {
                        let updated = system_info_client.get_pci_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |mut v| v.as_mut().unwrap().pci_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::USB => {
                        let updated = system_info_client.get_usb_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |mut v| v.as_mut().unwrap().usb_info = updated.clone())
                            .await
                    }
                    SystemInfoSyncType::SATA => {
                        let updated = system_info_client.get_sata_info().await.unwrap();
                        SYSTEM_INFO
                            .set_mut(move |mut v| v.as_mut().unwrap().sata_info = updated.clone())
                            .await
                    }
                }

                tokio::select! {
                    (duration, sync_type) = crate::helpers::recv_different(&mut receiver, (duration.clone(), sync_type.clone())) => {
                        syncing_state = SyncingState::Syncing(duration, sync_type);
                    },
                    _ = tokio::time::sleep(*duration) => {},
                }
            }
        }
    });
}

pub fn set_system_info_sync(duration: Duration, sync_type: SystemInfoSyncType) {
    if let Some(ref sender) = SYSINFO_SYNC.lock().unwrap().as_ref() {
        sender.send((duration, sync_type)).unwrap();
    }
}
