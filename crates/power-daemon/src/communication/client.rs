use crate::{
    systeminfo::{CPUInfo, SystemInfo},
    Config, PCIInfo, Profile, ProfilesInfo, ReducedUpdate,
};
use zbus::proxy;

#[proxy(
    default_service = "io.github.thealexdev23.power_daemon",
    interface = "io.github.thealexdev23.power_daemon.system_info",
    default_path = "/io/github/thealexdev23/power_daemon/system_info"
)]
trait SystemInfoDBus {
    /// Returns a JSON encoded `SystemInfo`
    fn get_system_info(&self) -> zbus::Result<String>;

    /// Returns a JSON encoded `CPUInfo`
    fn get_cpu_info(&self) -> zbus::Result<String>;

    /// Returns a JSON encoded `PCIInfo`
    fn get_pci_info(&self) -> zbus::Result<String>;
}

#[derive(Clone)]
pub struct SystemInfoClient {
    dbus_con: zbus::Connection,
}

impl SystemInfoClient {
    pub async fn new() -> zbus::Result<Self> {
        let con = zbus::Connection::system().await?;
        Ok(Self { dbus_con: con })
    }

    pub async fn get_system_info(&self) -> zbus::Result<SystemInfo> {
        Ok(serde_json::from_str(&self.get_proxy().await?.get_system_info().await?).unwrap())
    }
    pub async fn get_cpu_info(&self) -> zbus::Result<CPUInfo> {
        Ok(serde_json::from_str(&self.get_proxy().await?.get_cpu_info().await?).unwrap())
    }
    pub async fn get_pci_info(&self) -> zbus::Result<PCIInfo> {
        Ok(serde_json::from_str(&self.get_proxy().await?.get_pci_info().await?).unwrap())
    }

    async fn get_proxy(&self) -> zbus::Result<SystemInfoDBusProxy> {
        Ok(SystemInfoDBusProxy::new(&self.dbus_con).await?)
    }
}

#[proxy(
    default_service = "io.github.thealexdev23.power_daemon",
    interface = "io.github.thealexdev23.power_daemon.control",
    default_path = "/io/github/thealexdev23/power_daemon/control"
)]
trait ControlDBus {
    async fn get_config(&self) -> zbus::Result<String>;
    async fn get_profiles_info(&self) -> zbus::Result<String>;

    async fn update(&self) -> zbus::Result<()>;

    async fn update_config(&self, updated: String) -> zbus::Result<()>;
    async fn update_profile(&self, idx: u32, updated: String) -> zbus::Result<()>;

    async fn set_reduced_update(&self, reduced_update: String) -> zbus::Result<()>;
    async fn reset_reduced_update(&self) -> zbus::Result<()>;

    async fn set_profile_override(&self, profile_name: String) -> zbus::Result<()>;
    async fn remove_profile_override(&self) -> zbus::Result<()>;
}

#[derive(Clone)]
pub struct ControlClient {
    dbus_con: zbus::Connection,
}

impl ControlClient {
    pub async fn new() -> zbus::Result<Self> {
        let con = zbus::Connection::system().await?;
        Ok(Self { dbus_con: con })
    }

    pub async fn get_config(&self) -> zbus::Result<Config> {
        Ok(serde_json::from_str(&self.get_proxy().await?.get_config().await?).unwrap())
    }
    pub async fn get_profiles_info(&self) -> zbus::Result<ProfilesInfo> {
        Ok(serde_json::from_str(&self.get_proxy().await?.get_profiles_info().await?).unwrap())
    }

    pub async fn update(&self) -> zbus::Result<()> {
        self.get_proxy().await?.update().await
    }

    pub async fn update_config(&self, config: Config) -> zbus::Result<()> {
        self.get_proxy()
            .await?
            .update_config(serde_json::to_string(&config).expect("Could not serialize config"))
            .await
    }
    pub async fn update_profile(&self, idx: u32, updated: Profile) -> zbus::Result<()> {
        self.get_proxy()
            .await?
            .update_profile(
                idx,
                serde_json::to_string(&updated).expect("Could not serialize config"),
            )
            .await
    }

    pub async fn set_reduced_update(&self, reduced_update: ReducedUpdate) -> zbus::Result<()> {
        self.get_proxy()
            .await?
            .set_reduced_update(
                serde_json::to_string(&reduced_update).expect("Could not serialize reduced update"),
            )
            .await
    }
    pub async fn reset_reduced_update(&self) -> zbus::Result<()> {
        self.get_proxy().await?.reset_reduced_update().await
    }

    pub async fn set_profile_override(&self, profile_name: String) -> zbus::Result<()> {
        self.get_proxy()
            .await?
            .set_profile_override(profile_name)
            .await
    }
    pub async fn remove_profile_override(&self) -> zbus::Result<()> {
        self.get_proxy().await?.remove_profile_override().await
    }

    async fn get_proxy(&self) -> zbus::Result<ControlDBusProxy> {
        Ok(ControlDBusProxy::new(&self.dbus_con).await?)
    }
}
