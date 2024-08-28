use lazy_static::lazy_static;
use log::debug;
use log::trace;
use power_daemon::{communication::client::ControlClient, Config, Profile, ReducedUpdate};
use tokio::sync::MappedMutexGuard;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

use super::{CONFIG, PROFILES_INFO, PROFILE_OVERRIDE};

lazy_static! {
    static ref CLIENT: Mutex<Option<ControlClient>> = None.into();
}

pub async fn setup_control_client() {
    debug!("Setting up D-Bus control client.");
    *CLIENT.lock().await = Some(ControlClient::new().await.unwrap());
}

pub async fn get_config() {
    debug!("Obtaining config");
    CONFIG
        .set(get_client().await.get_config().await.unwrap())
        .await
}
pub async fn get_profiles_info() {
    debug!("Obtaining profiles info");
    PROFILES_INFO
        .set(get_client().await.get_profiles_info().await.unwrap())
        .await
}
pub async fn update_config(config: Config) {
    debug!("Updating config");
    get_client().await.update_config(config).await.unwrap();
}
pub async fn reset_profile(idx: u32) {
    debug!("Resetting profile {idx}");
    get_client().await.reset_profile(idx).await.unwrap();
}
pub async fn remove_profile(idx: u32) {
    debug!("Removing profile {idx}");
    get_client().await.remove_profile(idx).await.unwrap();
}
pub async fn swap_profiles(idx: u32, new_idx: u32) {
    debug!("Swapping profile {idx} with {new_idx}");
    get_client()
        .await
        .swap_profiles(idx, new_idx)
        .await
        .unwrap();
}
pub async fn update_profile_name(idx: u32, new_name: String) {
    debug!("Updating profile {idx} name to {new_name}");
    get_client()
        .await
        .update_profile_name(idx, new_name)
        .await
        .unwrap();
}
pub async fn update_profile_full(idx: u32, updated: Profile) {
    debug!("Updating profile {idx} fully");
    trace!("Updated profile: {updated:#?}");

    get_client()
        .await
        .update_profile_full(idx, updated)
        .await
        .unwrap();
}
pub async fn update_profile_reduced(idx: u32, updated: Profile, reduced_update: ReducedUpdate) {
    debug!("Updating profile {idx} reduced: {reduced_update:?}");
    trace!("Updated profile: {updated:#?}");

    get_client()
        .await
        .update_profile_reduced(idx, updated, reduced_update)
        .await
        .unwrap();
}

pub async fn update_full() {
    debug!("Updating fully");
    get_client().await.update_full().await.unwrap();
}
pub async fn update(reduced_update: ReducedUpdate) {
    debug!("Updating reduced: {reduced_update:?}");
    get_client().await.update_full().await.unwrap();
}

pub async fn get_profile_override() {
    debug!("Obtaining profile override");
    PROFILE_OVERRIDE
        .set(get_client().await.get_profile_override().await.unwrap())
        .await;
}
pub async fn set_profile_override(profile_name: String) {
    debug!("Setting profile override");
    get_client()
        .await
        .set_profile_override(profile_name)
        .await
        .unwrap();
}
pub async fn remove_profile_override() {
    debug!("Removing profile override profile override");
    get_client().await.remove_profile_override().await.unwrap();
}

async fn get_client() -> MappedMutexGuard<'static, ControlClient> {
    trace!("Locking on control client");
    MutexGuard::map(CLIENT.lock().await, |v| v.as_mut().unwrap())
}
