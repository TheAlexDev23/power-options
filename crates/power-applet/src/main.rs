use std::{
    io::Write,
    process::{Child, Command, Stdio},
    time::Duration,
};

use power_daemon::{communication::client::ControlClient, ProfilesInfo};

#[tokio::main]
async fn main() {
    let client = ControlClient::new()
        .await
        .expect("Could not initialize Control Client");

    let mut profiles_info = client
        .get_profiles_info()
        .await
        .expect("Could not obtain profiles_info");
    let mut profile_override = client
        .get_profile_override()
        .await
        .expect("Could not get profile override");

    #[allow(clippy::zombie_processes)]
    let mut process = tray_process(&profiles_info, profile_override.is_some());

    loop {
        let new_profiles_info = client
            .get_profiles_info()
            .await
            .expect("Could not obtain profiles_info");

        if profiles_info != new_profiles_info {
            profiles_info = new_profiles_info;
            profile_override = client
                .get_profile_override()
                .await
                .expect("Could not get profile override");

            process
                .stdin
                .unwrap()
                .write_all("quit".as_bytes())
                .expect("Could not kill yad process");

            process = tray_process(&profiles_info, profile_override.is_some());
        }

        std::thread::sleep(Duration::from_secs_f32(3.5));
    }
}

fn tray_process(profiles_info: &ProfilesInfo, has_profile_override: bool) -> Child {
    let mut menu = String::new();

    if has_profile_override {
        menu.push_str("Reset profile override ! power-daemon-mgr reset-profile-override");
    }

    for profile in &profiles_info.profiles {
        let name = &profile.profile_name;
        if profiles_info.get_active_profile() == profile {
            menu.push_str(&format!(
                "| ▶ {name} ! power-daemon-mgr set-profile-override \"{name}\"",
            ));
        } else {
            menu.push_str(&format!(
                "| {name} ! power-daemon-mgr set-profile-override \"{name}\""
            ));
        }
    }

    Command::new("yad")
        .args([
            "--notification",
            "--listen",
            "--image=power-options-tray",
            "--text=Manage Power Options",
            "--command=menu",
            &format!("--menu={menu}"),
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Could not spawn yad notification dialogue")
}
