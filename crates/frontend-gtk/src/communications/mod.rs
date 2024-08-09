use lazy_static::lazy_static;

use power_daemon::{Config, ProfilesInfo, SystemInfo};

use crate::helpers::SyncedValue;

pub mod daemon_control;
pub mod system_info;

lazy_static! {
    pub static ref CONFIG: SyncedValue<Config> = SyncedValue::new();
    pub static ref PROFILES_INFO: SyncedValue<ProfilesInfo> = SyncedValue::new();
    pub static ref PROFILE_OVERRIDE: SyncedValue<Option<String>> = SyncedValue::new();
    pub static ref SYSTEM_INFO: SyncedValue<SystemInfo> = SyncedValue::new();
}
