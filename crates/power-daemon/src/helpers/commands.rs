use std::{
    fs,
    process::{Command, Stdio},
};

use log::{debug, error, trace};

pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map_or(false, |output| output.status.success())
}

pub fn run_command(command: &str) {
    debug!("running: {command}");
    let output = Command::new("sh")
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

pub fn run_graphical_command(command: &str) {
    debug!("running graphical command: {command}");

    let output = Command::new("sh")
        .args(["-c", command])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", ":0")
        .env("XAUTHORITY", get_xauthority())
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

pub fn run_graphical_command_in_background(command: &str) -> std::process::Child {
    debug!("running graphical command in background: {command}");
    Command::new("sh")
        .args(["-c", command])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", ":0")
        .env("XAUTHORITY", get_xauthority())
        .spawn()
        .expect("Could not run command")
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
