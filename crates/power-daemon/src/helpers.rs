use std::{
    io::Write,
    process::{Child, Command, Stdio},
};

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

static mut SHELL_INSTANCE: Option<Child> = None;

pub fn run_command(command: &str) {
    unsafe {
        if SHELL_INSTANCE.is_none() {
            SHELL_INSTANCE = Some(
                Command::new("sh")
                    .stdin(Stdio::piped())
                    .spawn()
                    .expect("Could not spawn shell process"),
            );
        }

        let stdin = SHELL_INSTANCE
            .as_mut()
            .unwrap()
            .stdin
            .as_mut()
            .expect("Could not open attached shell process stdin");

        writeln!(stdin, "{command}").expect("Could not write to attached shell stdin");
    }
}

// Runs command, returns (stdout, stdin), does not check for argument validity or program succesful completion.
// Wil panic if: can't parse arguments, can't create command, can't run command
pub fn run_command_with_output_unchecked(command: &str) -> (String, String) {
    let args = shell_words::split(command).unwrap();

    let mut args_iter = args.iter();

    let mut command_proc = Command::new(args_iter.next().unwrap());

    for arg in args_iter {
        command_proc.arg(arg);
    }

    let result = command_proc.output().unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();

    (stdout, stderr)
}
