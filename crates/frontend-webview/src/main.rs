#![allow(non_snake_case)]

mod communication_services;
mod helpers;
mod setting_groups;
mod settings;

use std::time::Duration;

use communication_services::{
    background_daemon_sync_routine, control_routine_send_multiple, control_service,
    system_info_service, ControlAction, ControlRoutine, SystemInfoSyncType,
};
use setting_groups::{
    cpu::CPUGroup, kernel::KernelGroup, network::NetworkGroup, pci::PCIAndASPMGroup,
    radio::RadioGroup, sata::SATAGroup, screen::ScreenGroup, usb::USBGroup,
};
use settings::SettingsMenu;

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use power_daemon::{ProfilesInfo, SystemInfo};
use tracing::Level;

fn main() {
    const PROPER_PATH: &str = "/usr/lib/power-options-webview";
    if std::env::current_dir().unwrap().display().to_string() != PROPER_PATH {
        println!("flags: {:?}", std::env::args().collect::<Vec<_>>());
        let no_change_dir = std::env::args().any(|p| p == "--no-change-dir");

        if !no_change_dir {
            println!("Program was not run in {PROPER_PATH}. Rerunning under proper directory...");
            println!("If you want to prevent this behaviour pass in the --no-change-dir flag");

            std::env::set_current_dir(PROPER_PATH).expect("Could not change path");
        }
    }

    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_resizable(true)
                    .with_maximizable(false)
                    .with_min_inner_size(LogicalSize::new(950, 560))
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
    let active_profile_override = use_signal(|| None);
    let control_routine = use_coroutine(move |rx| {
        control_service(rx, config, profiles_info, active_profile_override)
    });

    let active_profile_name = use_signal(|| None);
    let _ = use_coroutine(move |_: UnboundedReceiver<()>| {
        background_daemon_sync_routine(active_profile_name, profiles_info, control_routine)
    });

    control_routine_send_multiple(
        control_routine,
        &[
            ControlAction::GetProfilesInfo,
            ControlAction::GetProfileOverride,
        ],
        None,
    );

    let settings_opened = use_signal(|| false);

    let current_settings_tab = use_signal(|| 0);

    rsx! {
        link { rel: "stylesheet", href: "assets/main.css" }
        link { rel: "stylesheet", href: "assets/color.css" }
        link { rel: "stylesheet", href: "assets/custom-elements.css" }

        script { src: "helpers.js" }

        if settings_opened() {
            SettingsMenu {
                settings_opened,
                config,
                profiles_info,
                control_routine,
                system_info_routine
            }
        } else {
            div { class: "top-bar",
                PowerProfilesNav {
                    profiles_info,
                    control_routine,
                    active_profile_override
                }

                ManageProfilesButton { settings_opened }
            }

            div { display: "flex",
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
    }
}

#[component]
fn PowerProfilesNav(
    profiles_info: Signal<Option<ProfilesInfo>>,
    active_profile_override: ReadOnlySignal<Option<String>>,
    control_routine: ControlRoutine,
) -> Element {
    let waiting_override_set = use_signal(|| false);
    let mut future_override_idx = use_signal(|| 0);

    let waiting_override_remove = use_signal(|| false);

    if profiles_info().is_some() {
        let mut buttons = Vec::new();
        for (idx, profile) in profiles_info()
            .as_ref()
            .unwrap()
            .profiles
            .iter()
            .enumerate()
        {
            let profile_name = profile.profile_name.clone();
            buttons.push((idx, profile_name.clone(), move |_| {
                future_override_idx.set(idx);
                control_routine_send_multiple(
                    control_routine,
                    &[
                        ControlAction::SetProfileOverride(profile_name.clone()),
                        ControlAction::GetProfilesInfo,
                        ControlAction::GetProfileOverride,
                    ],
                    Some(waiting_override_set),
                );
            }))
        }

        rsx! {
            nav { class: "profiles-selector",
                ul {
                    for mut button in buttons {
                        li {
                            if waiting_override_set() && button.0 == future_override_idx() {
                                div { class: "spinner" }
                            } else {
                                div { display: "flex", align_items: "center",
                                    button {
                                        onclick: move |e| {
                                            if waiting_override_remove() || waiting_override_set() {
                                                return;
                                            }
                                            button.2(e);
                                        },
                                        class: if button.0 == profiles_info().as_ref().unwrap().active_profile {
                                            if active_profile_override().is_some()
                                                && active_profile_override().unwrap() == button.1
                                            {
                                                "temporary-override"
                                            } else {
                                                "active"
                                            }
                                        } else {
                                            ""
                                        },
                                        "{button.1}"
                                    }
                                    if active_profile_override().is_some()
                                        && active_profile_override().unwrap() == button.1
                                    {
                                        if waiting_override_remove() {
                                            div {
                                                display: "inline-block",
                                                class: "spinner"
                                            }
                                        } else {
                                            button {
                                                onclick: move |_| {
                                                    if waiting_override_remove() || waiting_override_set() {
                                                        return;
                                                    }
                                                    control_routine_send_multiple(
                                                        control_routine,
                                                        &[
                                                            ControlAction::RemoveProfileOverride,
                                                            ControlAction::GetProfilesInfo,
                                                            ControlAction::GetProfileOverride,
                                                        ],
                                                        Some(waiting_override_remove),
                                                    );
                                                },
                                                img { src: "assets/icons/cross.svg" }
                                            }
                                        }
                                    }
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
fn ManageProfilesButton(settings_opened: Signal<bool>) -> Element {
    rsx! {
        div {
            button {
                onclick: move |_| {
                    settings_opened.set(true);
                },
                class: "primary",
                font_size: "14px",
                "Manage Profiles"
            }
        }
    }
}

#[component]
fn SettingGroupsNav(current_tab: Signal<u8>) -> Element {
    let setting_groups = vec![
        ("assets/icons/navbar-cpu.svg", "CPU"),
        ("assets/icons/navbar-screen.svg", "Screen"),
        ("assets/icons/navbar-radio.svg", "Radio devices"),
        ("assets/icons/navbar-network.svg", "Network"),
        ("assets/icons/navbar-aspm.svg", "PCI"),
        ("assets/icons/navbar-usb.svg", "USB"),
        ("assets/icons/navbar-sata.svg", "SATA"),
        ("assets/icons/linux-tux.svg", "Kernel"),
    ];

    rsx! {
        nav { class: "side-bar",
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
            } else if current_tab_val == 1 {
                ScreenGroup {
                    profiles_info,
                    system_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 2 {
                RadioGroup { profiles_info, control_routine, system_info_routine }
            } else if current_tab_val == 3 {
                NetworkGroup {
                    profiles_info,
                    system_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 4 {
                PCIAndASPMGroup {
                    system_info,
                    profiles_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 5 {
                USBGroup {
                    system_info,
                    profiles_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 6 {
                SATAGroup {
                    system_info,
                    profiles_info,
                    control_routine,
                    system_info_routine
                }
            } else if current_tab_val == 7 {
                KernelGroup { profiles_info, control_routine, system_info_routine }
            } else {
                PlaceholderGroup { current_tab }
            }
        }

        br {}
        br {}
        br {}
    }
}

#[component]
fn PlaceholderGroup(current_tab: Signal<u8>) -> Element {
    rsx! {
        div { "Placeholder group {current_tab}" }
    }
}
