#![allow(non_snake_case)]

mod communication_services;
mod helpers;
mod setting_groups;

use std::time::Duration;

use communication_services::{
    control_service, system_info_service, ControlAction, ControlRoutine, SystemInfoSyncType,
};
use setting_groups::{cpu::CPUGroup, network::NetworkGroup, radio::RadioGroup};

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use power_daemon::{ProfilesInfo, SystemInfo};
use tracing::Level;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_resizable(true)
                    .with_maximizable(false)
                    .with_min_inner_size(LogicalSize::new(800, 500))
                    .with_title("Power options"),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let system_info = use_signal(|| Option::None);
    let system_info_routine = use_coroutine(move |rx| system_info_service(rx, system_info));

    let config = use_signal(|| Option::None);
    let profiles_info = use_signal(|| Option::None);
    let control_routine = use_coroutine(move |rx| control_service(rx, config, profiles_info));
    control_routine.send((ControlAction::GetProfilesInfo, None));

    let current_settings_tab = use_signal(|| 0);

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        script { src: "helpers.js" }

        PowerProfilesNav { profiles_info, control_routine }
        SettingGroupsNav { current_tab: current_settings_tab }
        SettingGroup {
            current_tab: current_settings_tab,
            system_info,
            profiles_info,
            control_routine,
            system_info_routine
        }
    }
}

#[component]
fn PowerProfilesNav(
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
) -> Element {
    let waiting = use_signal(|| false);
    let mut waiting_future_idx = use_signal(|| 0);

    if profiles_info.read().is_some() {
        let mut buttons = Vec::new();
        for (idx, profile) in profiles_info
            .read()
            .as_ref()
            .unwrap()
            .profiles
            .iter()
            .enumerate()
        {
            let profile_name = profile.profile_name.clone();
            buttons.push((idx, profile_name.clone(), move |_| {
                waiting_future_idx.set(idx);
                control_routine.send((ControlAction::ResetReducedUpdate, Some(waiting)));
                control_routine.send((
                    ControlAction::SetProfileOverride(profile_name.clone()),
                    Some(waiting),
                ));
                control_routine.send((ControlAction::GetProfilesInfo, Some(waiting)));
            }))
        }

        rsx! {
            nav { class: "profiles",
                ul {
                    for button in buttons {
                        li {
                            if *waiting.read() && button.0 == *waiting_future_idx.read() {
                                div { class: "spinner" }
                            } else {
                                button {
                                    onclick: button.2,
                                    class: if button.0 == profiles_info.read().as_ref().unwrap().active_profile {
                                        "active"
                                    } else {
                                        ""
                                    },
                                    "{button.1}"
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn SettingGroupsNav(current_tab: Signal<u8>) -> Element {
    let setting_groups = vec![
        ("icons/navbar-cpu.svg", "CPU"),
        ("icons/navbar-screen.svg", "Screen"),
        ("icons/navbar-radio.svg", "Radio devices"),
        ("icons/navbar-network.svg", "Network"),
        ("icons/navbar-aspm.svg", "PCI"),
        ("icons/navbar-usb.svg", "USB"),
        ("icons/navbar-sata.svg", "SATA"),
    ];

    rsx! {
        nav { class: "setting-groups-selector",
            ul {
                for (group_id , group) in setting_groups.iter().enumerate() {
                    li {
                        onclick: move |_| {
                            current_tab.set(group_id as u8);
                        },
                        class: if current_tab() == group_id as u8 { "selected" } else { "" },
                        img { src: group.0 }
                        button { "{group.1}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingGroup(
    current_tab: Signal<u8>,
    system_info: Signal<Option<SystemInfo>>,
    profiles_info: Signal<Option<ProfilesInfo>>,
    control_routine: ControlRoutine,
    system_info_routine: Coroutine<(Duration, SystemInfoSyncType)>,
) -> Element {
    let current_tab_val = current_tab();
    rsx! {
        div { class: "settings-group",
            if current_tab_val == 0 {
                CPUGroup {
                    system_info,
                    profiles_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 2 {
                RadioGroup { profiles_info, control_routine }
            } else if current_tab_val == 3 {
                NetworkGroup { profiles_info, control_routine }
            } else {
                PlaceholderGroup { current_tab }
            }
        }
    }
}

#[component]
fn PlaceholderGroup(current_tab: Signal<u8>) -> Element {
    rsx! {
        div { "Placeholder group {current_tab}" }
    }
}
