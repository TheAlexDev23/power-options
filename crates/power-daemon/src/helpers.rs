use std::{
    fs::{self, File},
    io::Read,
    process::{Command, Stdio},
};

use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum WhiteBlackList<T: PartialEq> {
    Whitelist(Vec<T>),
    Blacklist(Vec<T>),
}

impl<T: PartialEq> WhiteBlackList<T> {
    // If enable = true and no list provided, will return true for all items
    // If enable = true, will return true for items in whitelist or only for items outside of blacklist
    // If enable = false, will return false for all items
    pub fn should_enable_item(whiteblacklist: &Option<Self>, item: &T, enable: bool) -> bool {
        if !enable {
            // Always disable
            false
        } else if let Some(ref whiteblacklist) = whiteblacklist {
            match whiteblacklist {
                // If on whitelist always enable, otherwise always disable
                WhiteBlackList::Whitelist(ref list) => {
                    if list.iter().any(|i| i == item) {
                        true
                    } else {
                        false
                    }
                }
                // If on blacklist always disable, otherwise always enable
                WhiteBlackList::Blacklist(ref list) => {
                    if list.iter().any(|i| i == item) {
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

pub fn run_command(command: &str) {
    trace!("running: {command}");
    Command::new("zsh")
        .args(["-c", command])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Could not run command")
        .wait()
        .expect("Could not wait command");
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

    (stdout, stderr)
}

// Will read file at path and return a list of elements with space as the separator
// Will panic with io errors
pub fn file_content_to_list(path: &str) -> Vec<String> {
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
pub fn file_content_to_u32(path: &str) -> u32 {
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
pub fn file_content_to_bool(path: &str) -> bool {
    if fs::metadata(path).is_err() {
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
