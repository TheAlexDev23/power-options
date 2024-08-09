use lazy_static::lazy_static;
use power_daemon::{communication::client::ControlClient, Config, Profile, ReducedUpdate};
use tokio::sync::MappedMutexGuard;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

use super::{CONFIG, PROFILES_INFO, PROFILE_OVERRIDE};

lazy_static! {
    static ref CLIENT: Mutex<Option<ControlClient>> = None.into();
}

pub async fn setup_control_client() {
    *CLIENT.lock().await = Some(ControlClient::new().await.unwrap());
}

pub async fn get_config() {
    CONFIG
        .set(get_client().await.get_config().await.unwrap())
        .await
}
pub async fn get_profiles_info() {
    PROFILES_INFO
        .set(get_client().await.get_profiles_info().await.unwrap())
        .await
}
pub async fn update_config(config: Config) {
    get_client().await.update_config(config).await.unwrap();
}
pub async fn reset_profile(idx: u32) {
    get_client().await.reset_profile(idx).await.unwrap();
}
pub async fn remove_profile(idx: u32) {
    get_client().await.remove_profile(idx).await.unwrap();
}
pub async fn update_profile(idx: u32, updated: Profile) {
    get_client()
        .await
        .update_profile(idx, updated)
        .await
        .unwrap();
}
pub async fn update() {
    get_client().await.update().await.unwrap();
}
pub async fn set_reduced_update(reduced_update: ReducedUpdate) {
    get_client()
        .await
        .set_reduced_update(reduced_update)
        .await
        .unwrap();
}
pub async fn reset_reduced_update() {
    get_client().await.reset_reduced_update().await.unwrap();
}
pub async fn get_profile_override() {
    PROFILE_OVERRIDE
        .set(get_client().await.get_profile_override().await.unwrap())
        .await;
}
pub async fn set_profile_override(profile_name: String) {
    get_client()
        .await
        .set_profile_override(profile_name)
        .await
        .unwrap();
}
pub async fn remove_profile_override() {
    get_client().await.remove_profile_override().await.unwrap();
}

async fn get_client() -> MappedMutexGuard<'static, ControlClient> {
    MutexGuard::map(CLIENT.lock().await, |v| v.as_mut().unwrap())
}
