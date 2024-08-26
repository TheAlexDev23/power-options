use log::{debug, error};

use tokio::sync::Mutex;
use zbus::{conn::Builder, interface, Connection, Error};

use super::{CONTROL_OBJECT_NAME, SYSTEM_INFO_OBJECT_NAME, WELL_KNOWN_NAME};
use crate::{
    systeminfo::{CPUInfo, SystemInfo},
    Instance, PCIInfo, SATAInfo, USBInfo,
};

pub struct CommunicationServer {
    _con: Connection,
}

impl CommunicationServer {
    pub async fn new(instance: Instance) -> Result<CommunicationServer, Error> {
        debug!("Initializing communications server");
        let con = Builder::system()?
            .name(WELL_KNOWN_NAME)?
            .serve_at(
                CONTROL_OBJECT_NAME,
                ControlServer {
                    instance: instance.into(),
                },
            )?
            .serve_at(SYSTEM_INFO_OBJECT_NAME, SystemInfoServer)?
            .build()
            .await?;
        debug!("Finished setting up communications server connection");
        Ok(CommunicationServer { _con: con })
    }
}

struct SystemInfoServer;

#[interface(name = "io.github.thealexdev23.power_daemon.system_info")]
impl SystemInfoServer {
    async fn get_system_info(&self) -> String {
        serde_json::to_string(&SystemInfo::obtain()).unwrap()
    }

    async fn get_cpu_info(&self) -> String {
        serde_json::to_string(&CPUInfo::obtain()).unwrap()
    }

    async fn get_pci_info(&self) -> String {
        serde_json::to_string(&PCIInfo::obtain()).unwrap()
    }

    async fn get_usb_info(&self) -> String {
        serde_json::to_string(&USBInfo::obtain()).unwrap()
    }

    async fn get_sata_info(&self) -> String {
        serde_json::to_string(&SATAInfo::obtain()).unwrap()
    }
}

struct ControlServer {
    instance: Mutex<Instance>,
}

#[interface(name = "io.github.thealexdev23.power_daemon.control")]
impl ControlServer {
    async fn get_config(&self) -> String {
        serde_json::to_string(&self.instance.lock().await.config).unwrap()
    }
    async fn get_profiles_info(&self) -> String {
        serde_json::to_string(&self.instance.lock().await.profiles_info).unwrap()
    }

    async fn update_full(&mut self) {
        self.instance.get_mut().update_full();
    }
    async fn update_reduced(&mut self, reduced_update: String) {
        let reduced_update = match serde_json::from_str(&reduced_update) {
            Ok(reduced_update) => reduced_update,
            Err(error) => {
                error!("Could not parse reduced update: {error}");
                return;
            }
        };
        self.instance.get_mut().update_reduced(reduced_update);
    }

    async fn update_config(&mut self, updated: String) {
        match serde_json::from_str(&updated) {
            Ok(conf) => {
                self.instance.get_mut().update_config(conf);
            }
            Err(error) => {
                error!("Could not parse new requested config: {error}")
            }
        }
    }

    async fn reset_profile(&mut self, idx: u32) {
        self.instance.get_mut().reset_profile(idx as usize);
    }
    async fn remove_profile(&mut self, idx: u32) {
        self.instance.get_mut().remove_profile(idx as usize);
    }

    async fn update_profile_full(&mut self, idx: u32, updated: String) {
        match serde_json::from_str(&updated) {
            Ok(profile) => {
                self.instance
                    .get_mut()
                    .update_profile_full(idx as usize, profile);
            }
            Err(error) => {
                error!("Could not parse updated profile: {error}")
            }
        }
    }
    async fn update_profile_reduced(&mut self, idx: u32, updated: String, reduced_update: String) {
        let reduced_update = match serde_json::from_str(&reduced_update) {
            Ok(reduced_update) => reduced_update,
            Err(error) => {
                error!("Could not parse reduced update: {error}");
                return;
            }
        };

        match serde_json::from_str(&updated) {
            Ok(profile) => {
                self.instance.get_mut().update_profile_reduced(
                    idx as usize,
                    profile,
                    reduced_update,
                );
            }
            Err(error) => {
                error!("Could not parse updated profile: {error}")
            }
        }
    }

    async fn get_profile_override(&mut self) -> String {
        self.instance
            .get_mut()
            .temporary_override
            .clone()
            .unwrap_or_default()
    }
    async fn set_profile_override(&mut self, profile_name: String) {
        self.instance
            .get_mut()
            .try_set_profile_override(profile_name);
    }
    async fn remove_profile_override(&mut self) {
        self.instance.get_mut().remove_profile_override();
    }
}
