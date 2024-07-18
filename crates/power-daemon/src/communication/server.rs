use log::{debug, error};

use tokio::sync::Mutex;
use zbus::{conn::Builder, interface, Connection, Error};

use super::{CONTROL_OBJECT_NAME, SYSTEM_INFO_OBJECT_NAME, WELL_KNOWN_NAME};
use crate::{
    systeminfo::{ASPMInfo, CPUInfo, SystemInfo},
    Instance,
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

    async fn get_aspm_info(&self) -> String {
        serde_json::to_string(&ASPMInfo::obtain()).unwrap()
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

    async fn update_profile(&mut self, idx: u32, updated: String) {
        match serde_json::from_str(&updated) {
            Ok(profile) => {
                self.instance
                    .get_mut()
                    .update_profile(idx as usize, profile);
            }
            Err(error) => {
                error!("Could not parse new requested config: {error}")
            }
        }
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
