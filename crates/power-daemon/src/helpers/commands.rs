use std::{
    fs,
    process::{Command, Stdio},
};

use log::{debug, error, trace};

pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn run_command(command_name: &str) {
    debug!("running: {command_name}");

    let mut command = get_command_from_string(command_name);

    let output = command
        .spawn()
        .unwrap_or_else(|e| panic!("Could not run command {command_name}: {e}"))
        .wait_with_output()
        .unwrap_or_else(|e| panic!("Could not wait command {command_name}: {e}"));

    if !output.stdout.is_empty() {
        trace!(
            "Command output: {}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
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

    let mut process = get_command_from_string(command);

    let output = process
        .output()
        .unwrap_or_else(|e| panic!("Could not get command output: {command}: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    trace!("Output is: {stdout} {stderr}");

    (stdout, stderr)
}

pub fn run_graphical_command(command: &str) {
    debug!("running graphical command: {command}");

    let output = get_command_from_string(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", ":0")
        .env("XAUTHORITY", get_xauthority())
        .spawn()
        .unwrap_or_else(|e| panic!("Could not run graphical command: {command}: {e}"))
        .wait_with_output()
        .unwrap_or_else(|e| panic!("Could not wait for graphical command: {command}: {e}"));

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

pub fn run_graphical_command_in_background(command: &str) -> std::process::Child {
    debug!("running graphical command in background: {command}");

    get_command_from_string(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", ":0")
        .env("XAUTHORITY", get_xauthority())
        .spawn()
        .unwrap_or_else(|e| panic!("Could not run graphical command in background: {command}: {e}"))
}

fn get_command_from_string(command: &str) -> Command {
    let parts = shellwords::split(command)
        .unwrap_or_else(|e| panic!("Could not parse command parts: {command}: {e}"));
    let (cmd, args) = parts
        .split_first()
        .unwrap_or_else(|| panic!("Could not split first of arguments vector: {command}"));
    let mut command = Command::new(cmd);
    command.args(args);
    command
}

fn get_xauthority() -> String {
    // Point to the XAuthority of the first user, not the best way to do it but
    // will work for most users.

    format!(
        "/home/{}/.Xauthority",
        fs::read_dir("/home")
            .expect("Could not read home dir")
            .flatten()
            .next()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
    )
}
