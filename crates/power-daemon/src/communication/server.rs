use log::{debug, error, trace};

use tokio::sync::Mutex;
use zbus::{conn::Builder, interface, Connection, Error};

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
            .name("io.github.thealexdev23.power_daemon")?
            .serve_at(
                "/io/github/thealexdev23/power_daemon/control",
                ControlServer {
                    instance: instance.into(),
                },
            )?
            .serve_at(
                "/io/github/thealexdev23/power_daemon/system_info",
                SystemInfoServer,
            )?
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
        debug!(target: "D-BUS", "get_config");
        serde_json::to_string(&self.instance.lock().await.config).unwrap()
    }
    async fn get_profiles_info(&self) -> String {
        debug!(target: "D-BUS", "get_profiles_info");
        serde_json::to_string(&self.instance.lock().await.profiles_info).unwrap()
    }

    async fn update_full(&mut self) {
        debug!(target: "D-BUS", "update_full");
        self.instance.get_mut().update_full();
    }
    async fn update_reduced(&mut self, reduced_update: String) {
        debug!(target: "D-BUS", "update_reduced: {reduced_update}");
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
        debug!(target: "D-BUS", "update_config: {updated}");
        match serde_json::from_str(&updated) {
            Ok(conf) => {
                self.instance.get_mut().update_config(conf);
            }
            Err(error) => {
                error!("Could not parse new requested config: {error}")
            }
        }
    }

    async fn create_profile(&mut self, profile_type: String) {
        debug!(target: "D-BUS", "create_profile: {profile_type}");
        match serde_json::from_str(&profile_type) {
            Ok(profile_type) => self.instance.get_mut().create_profile(profile_type),
            Err(error) => {
                error!("Could not parse new requested profile type: {error}")
            }
        }
    }
    async fn reset_profile(&mut self, idx: u32) {
        debug!(target: "D-BUS", "reset_profile: {idx}");
        self.instance.get_mut().reset_profile(idx as usize);
    }
    async fn remove_profile(&mut self, idx: u32) {
        debug!(target: "D-BUS", "remove_profile: {idx}");
        self.instance.get_mut().remove_profile(idx as usize);
    }

    async fn swap_profiles(&mut self, idx: u32, new_idx: u32) {
        debug!(target: "D-BUS", "swap_profiles: {idx} with {new_idx}");
        self.instance
            .get_mut()
            .swap_profile_order(idx as usize, new_idx as usize);
    }
    async fn update_profile_name(&mut self, idx: u32, new_name: String) {
        debug!(target: "D-BUS", "update_profile_name: {idx} with {new_name}");
        self.instance
            .get_mut()
            .update_profile_name(idx as usize, new_name);
    }

    async fn update_profile_full(&mut self, idx: u32, updated: String) {
        debug!(target: "D-BUS", "update_profile_full: {idx}");
        trace!("New profile: {updated}");

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
        debug!(target: "D-BUS", "update_profile_redced: {idx} {reduced_update}");
        trace!("New profile: {updated}");

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
        debug!(target: "D-BUS", "get_profile_override");
        self.instance
            .get_mut()
            .temporary_override
            .clone()
            .unwrap_or_default()
    }
    async fn set_profile_override(&mut self, profile_name: String) {
        debug!(target: "D-BUS", "set_profile_override: {profile_name}");
        self.instance
            .get_mut()
            .try_set_profile_override(profile_name);
    }
    async fn remove_profile_override(&mut self) {
        debug!(target: "D-BUS", "remove_profile_override");
        self.instance.get_mut().remove_profile_override();
    }
}
