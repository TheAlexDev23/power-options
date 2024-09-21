use std::fs;

use serde::{Deserialize, Serialize};

pub mod commands;

pub use commands::*;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct WhiteBlackList {
    pub items: Vec<String>,
    pub list_type: WhiteBlackListType,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub enum WhiteBlackListType {
    #[default]
    Whitelist,
    Blacklist,
}

impl WhiteBlackList {
    /// If enable = true and no list provided, will return true for all items
    /// If enable = true, will return true for items in whitelist or only for items outside of blacklist
    /// If enable = false, will return false for all items
    pub fn should_enable_item(whiteblacklist: &Option<Self>, item: &str, enable: bool) -> bool {
        if !enable {
            // Always disable
            false
        } else if let Some(ref whiteblacklist) = whiteblacklist {
            match whiteblacklist.list_type {
                // If on whitelist always enable, otherwise always disable
                WhiteBlackListType::Whitelist => whiteblacklist.items.iter().any(|i| i == item),
                // If on blacklist always disable, otherwise always enable
                WhiteBlackListType::Blacklist => !whiteblacklist.items.iter().any(|i| i == item),
            }
        } else {
            // No list, always enable
            true
        }
    }
}

impl WhiteBlackListType {
    pub fn to_display_string(&self) -> String {
        match self {
            WhiteBlackListType::Whitelist => String::from("Whitelist"),
            WhiteBlackListType::Blacklist => String::from("Blacklist"),
        }
    }

    pub fn from_display_string(display_string: &str) -> Option<WhiteBlackListType> {
        match display_string {
            "Whitelist" => Some(WhiteBlackListType::Whitelist),
            "Blacklist" => Some(WhiteBlackListType::Blacklist),
            _ => None,
        }
    }
}

pub fn system_on_ac() -> bool {
    let mut ac_online = false;

    if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(type_path) = fs::read_to_string(entry_path.join("type")) {
                let supply_type = type_path.trim();
                if supply_type == "Mains" {
                    if let Ok(ac_status) = fs::read_to_string(entry_path.join("online")) {
                        ac_online = ac_status.trim() == "1";
                    }
                }
            }
        }
    }

    ac_online
}
