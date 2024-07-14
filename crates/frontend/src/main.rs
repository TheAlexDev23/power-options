#![allow(non_snake_case)]

mod helpers;
mod setting_groups;

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use tracing::Level;

use setting_groups::*;

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
    let current_settings_tab = use_signal(|| 0);
    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        script { src:"helpers.js" }

        PowerProfilesNav {}
        SettingGroupsNav {
            current_tab: current_settings_tab
        }
        SettingGroup {
            current_tab: current_settings_tab
        }
    }
}

#[component]
fn PowerProfilesNav() -> Element {
    rsx! {
        nav {
            class: "power-modes",
            ul {
                li {
                    button { "Powersave" }
                }
                li {
                    button { "Balanced" }
                }
                li {
                    button { "Peformance" }
                }
            }
        }

    }
}

#[component]
fn SettingGroupsNav(current_tab: Signal<u8>) -> Element {
    let setting_groups = vec![
        ("icons/navbar-cpu.svg", "CPU"),
        ("icons/navbar-cpu.svg", "CPU extra"),
        ("icons/navbar-screen.svg", "Screen"),
        ("icons/navbar-radio.svg", "Radio devices"),
        ("icons/navbar-network.svg", "Network"),
        ("icons/navbar-aspm.svg", "PCI and ASPM"),
        ("icons/navbar-usb.svg", "USB"),
        ("icons/navbar-sata.svg", "SATA"),
    ];

    rsx! {
        nav {
            class: "setting-groups-selector",
            ul {
                for (group_id, group) in setting_groups.iter().enumerate() {
                    li {
                        onclick: move |_| {
                            current_tab.set(group_id as u8);
                        },
                        class: if current_tab() == group_id as u8 {
                            "selected"
                        } else {
                            ""
                        },
                        img { src: group.0 }
                        button {
                            "{group.1}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingGroup(current_tab: Signal<u8>) -> Element {
    rsx! {
        div {
            class: "settings-group",
            match current_tab() {
                0 => cpu::CPUGroup(),
                1 => PlaceholderGroup(current_tab),
                2 => PlaceholderGroup(current_tab),
                3 => PlaceholderGroup(current_tab),
                4 => PlaceholderGroup(current_tab),
                5 => PlaceholderGroup(current_tab),
                6 => PlaceholderGroup(current_tab),
                7 => PlaceholderGroup(current_tab),
                _ => rsx! { "Unknown group" },
            }
        }
    }
}

fn PlaceholderGroup(current_tab: Signal<u8>) -> Element {
    rsx! {
        div {
            "Placeholder group {current_tab}"
        }
    }
}
