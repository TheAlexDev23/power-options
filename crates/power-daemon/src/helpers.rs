use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    process::{Command, Stdio},
};

use log::{debug, error, trace};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct WhiteBlackList<T: PartialEq> {
    pub items: Vec<T>,
    pub list_type: WhiteBlackListType,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub enum WhiteBlackListType {
    #[default]
    Whitelist,
    Blacklist,
}

impl<T: PartialEq> WhiteBlackList<T> {
    /// If enable = true and no list provided, will return true for all items
    /// If enable = true, will return true for items in whitelist or only for items outside of blacklist
    /// If enable = false, will return false for all items
    pub fn should_enable_item(whiteblacklist: &Option<Self>, item: &T, enable: bool) -> bool {
        if !enable {
            // Always disable
            false
        } else if let Some(ref whiteblacklist) = whiteblacklist {
            match whiteblacklist.list_type {
                // If on whitelist always enable, otherwise always disable
                WhiteBlackListType::Whitelist => {
                    if whiteblacklist.items.iter().any(|i| i == item) {
                        true
                    } else {
                        false
                    }
                }
                // If on blacklist always disable, otherwise always enable
                WhiteBlackListType::Blacklist => {
                    if whiteblacklist.items.iter().any(|i| i == item) {
                        false
                    } else {
                        true
                    }
                }
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

pub fn run_command(command: &str) {
    debug!("running: {command}");
    let output = Command::new("zsh")
        .args(["-c", command])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Could not run command")
        .wait_with_output()
        .expect("Could not wait command");

    trace!(
        "Command output: {}",
        String::from_utf8(output.stdout).unwrap()
    );
    if !output.stderr.is_empty() {
        error!(
            "Command returned with stderr: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }
}

// Runs command, returns (stdout, stdin), does not check for argument validity or program succesful completion.
// Wil panic if: can't parse arguments, can't create command, can't run command
pub fn run_command_with_output(command: &str) -> (String, String) {
    trace!("getting output of: {command}");

    let mut command_proc = Command::new("sh");
    command_proc.args(["-c", command]);

    let result = command_proc.output().expect("Could not run command");

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();

    trace!("Output is: {stdout} {stderr}");

    (stdout, stderr)
}

pub fn file_content_to_string<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content
}

// Will read file at path and return a list of elements with space as the separator
// Will panic with io errors
pub fn file_content_to_list<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content.split(" ").map(String::from).collect()
}

// Will read file at path and parse u32
// Will panic with io errors and parsing errors
pub fn file_content_to_u32<P: AsRef<Path>>(path: P) -> u32 {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content.parse().unwrap()
}

// Will read file at path and return true if content is 1 false otherwise
// Will return false if the file doesn't exist but will panic if some io issues appear
pub fn file_content_to_bool<P: AsRef<Path>>(path: P) -> bool {
    if fs::metadata(path.as_ref()).is_err() {
        return false;
    }

    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    if content == "1" {
        true
    } else {
        false
    }
}

pub fn system_on_ac() -> bool {
    let mut ac_online = false;

    if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
        for entry in entries {
            if let Ok(entry) = entry {
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
    }

    ac_online
}
