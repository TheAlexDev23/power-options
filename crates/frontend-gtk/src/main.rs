#![allow(deprecated)]

pub mod communications;
pub mod components;
pub mod helpers;

use std::time::Duration;

use relm4::prelude::*;

use communications::system_info::SystemInfoSyncType;
use components::*;

#[tokio::main]
async fn main() {
    communications::daemon_control::setup_control_client().await;

    tokio::join!(
        communications::daemon_control::get_profiles_info(),
        communications::daemon_control::get_profile_override(),
        communications::daemon_control::get_config(),
    );

    communications::system_info::start_system_info_sync_routine();
    communications::system_info::set_system_info_sync(
        Duration::from_secs_f32(5.0),
        SystemInfoSyncType::None,
    );

    communications::system_info::obtain_full_info_once().await;

    RelmApp::new("io.github.thealexdev23.power-options.frontend").run_async::<App>(());
}
