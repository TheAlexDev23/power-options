use std::{
    fs,
    process::{Command, Stdio},
};

use log::{debug, error, trace, warn};

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

    let (display, xauth_path) = get_x_session_info();

    let output = get_command_from_string(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", display)
        .env("XAUTHORITY", xauth_path)
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

    let (display, xauth_path) = get_x_session_info();

    get_command_from_string(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("DISPLAY", display)
        .env("XAUTHORITY", xauth_path)
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

/// Returns the DISPLAY and XAUTHORITY values for the active X session
/// Uses modern system standards to detect active sessions and proper authentication
fn get_x_session_info() -> (String, String) {
    // Try to get session info from loginctl first (systemd-logind)
    if let Some((display, xauth)) = get_session_from_loginctl() {
        debug!(
            "Found X session via loginctl: DISPLAY={}, XAUTHORITY={}",
            display, xauth
        );
        return (display, xauth);
    }

    // Fallback to checking running X processes
    if let Some((display, xauth)) = get_session_from_processes() {
        debug!(
            "Found X session via process detection: DISPLAY={}, XAUTHORITY={}",
            display, xauth
        );
        return (display, xauth);
    }

    // Final fallback to display manager specific locations
    if let Some((display, xauth)) = get_session_from_display_managers() {
        debug!(
            "Found X session via display manager: DISPLAY={}, XAUTHORITY={}",
            display, xauth
        );
        return (display, xauth);
    }

    // Last resort: use the old method but with warning
    warn!("Could not detect X session properly, falling back to legacy method");
    let legacy_xauth = get_legacy_xauthority();
    (":0".to_string(), legacy_xauth)
}

/// Try to get X session info using loginctl (systemd-logind)
fn get_session_from_loginctl() -> Option<(String, String)> {
    // Get active sessions
    let output = Command::new("loginctl")
        .args(["list-sessions", "--no-header"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sessions_output = String::from_utf8(output.stdout).ok()?;

    // Parse each session to find one with a display
    for line in sessions_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let session_id = parts[0];

            // Get session details
            if let Some((display, user)) = get_session_display_info(session_id) {
                // Get the user's XAUTHORITY
                if let Some(xauth_path) = get_user_xauthority(&user, &display) {
                    return Some((display, xauth_path));
                }
            }
        }
    }

    None
}

/// Get display and user info for a specific session
fn get_session_display_info(session_id: &str) -> Option<(String, String)> {
    let output = Command::new("loginctl")
        .args([
            "show-session",
            session_id,
            "-p",
            "Type",
            "-p",
            "User",
            "-p",
            "Display",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let session_info = String::from_utf8(output.stdout).ok()?;
    let mut session_type = None;
    let mut user = None;
    let mut display = None;

    for line in session_info.lines() {
        if let Some(value) = line.strip_prefix("Type=") {
            session_type = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("User=") {
            user = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Display=") {
            if !value.is_empty() {
                display = Some(format!(":{}", value));
            }
        }
    }

    // Only return info for X11 sessions with a display
    if session_type == Some("x11".to_string()) {
        if let (Some(display_val), Some(user_val)) = (display, user) {
            Some((display_val, user_val))
        } else {
            None
        }
    } else {
        None
    }
}

/// Try to detect X session by examining running processes
fn get_session_from_processes() -> Option<(String, String)> {
    // Look for X server processes
    let output = Command::new("pgrep").args(["-a", "X"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let processes = String::from_utf8(output.stdout).ok()?;

    for line in processes.lines() {
        // Parse X server command line to extract display number
        if let Some(display_num) = extract_display_from_x_process(line) {
            let display = format!(":{}", display_num);

            // Try to find the user running this X session
            if let Some(user) = get_x_session_user(&display) {
                if let Some(xauth_path) = get_user_xauthority(&user, &display) {
                    return Some((display, xauth_path));
                }
            }
        }
    }

    None
}

/// Extract display number from X server process command line
fn extract_display_from_x_process(process_line: &str) -> Option<String> {
    // Look for patterns like ":0", ":1", etc. in the X server command line
    let parts: Vec<&str> = process_line.split_whitespace().collect();
    for part in parts {
        if part.starts_with(':') && part.len() > 1 && part[1..].parse::<u32>().is_ok() {
            return Some(part[1..].to_string());
        }
    }
    None
}

/// Try to find the user running the X session on the given display
fn get_x_session_user(display: &str) -> Option<String> {
    // Look for processes with the DISPLAY environment variable set
    let output = Command::new("ps").args(["axe"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let processes = String::from_utf8(output.stdout).ok()?;

    for line in processes.lines() {
        if line.contains(&format!("DISPLAY={}", display)) {
            // Extract username from ps output
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                // Try to get user info for this PID
                if let Ok(pid) = parts[0].parse::<u32>() {
                    if let Some(user) = get_process_user(pid) {
                        // Skip root and system users
                        if user != "root" && !user.starts_with("_") && user != "gdm" {
                            return Some(user);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get the user that owns a process
fn get_process_user(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(["-o", "user=", "-p", &pid.to_string()])
        .output()
        .ok()?;

    if output.status.success() {
        let user = String::from_utf8(output.stdout).ok()?;
        Some(user.trim().to_string())
    } else {
        None
    }
}

/// Try to get X session info from display manager specific locations
fn get_session_from_display_managers() -> Option<(String, String)> {
    // Try GDM first
    if let Some(xauth) = try_gdm_xauthority() {
        return Some((":0".to_string(), xauth));
    }

    // Try LightDM
    if let Some(xauth) = try_lightdm_xauthority() {
        return Some((":0".to_string(), xauth));
    }

    // Try SDDM
    if let Some(xauth) = try_sddm_xauthority() {
        return Some((":0".to_string(), xauth));
    }

    None
}

/// Try to find GDM X authority file
fn try_gdm_xauthority() -> Option<String> {
    let paths = [
        "/var/lib/gdm/:0.Xauth",
        "/run/user/120/gdm/Xauthority",
        "/var/lib/gdm3/:0.Xauth",
    ];

    for path in &paths {
        if fs::metadata(path).is_ok() {
            return Some(path.to_string());
        }
    }

    None
}

/// Try to find LightDM X authority file
fn try_lightdm_xauthority() -> Option<String> {
    let paths = ["/var/run/lightdm/root/:0", "/run/lightdm/root/:0"];

    for path in &paths {
        if fs::metadata(path).is_ok() {
            return Some(path.to_string());
        }
    }

    None
}

/// Try to find SDDM X authority file
fn try_sddm_xauthority() -> Option<String> {
    // SDDM uses random UUIDs, so we need to search
    let base_path = "/var/run/sddm";
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}

/// Get the XAUTHORITY file path for a specific user and display
fn get_user_xauthority(user: &str, display: &str) -> Option<String> {
    // First try the standard location
    let user_xauth = format!("/home/{}/.Xauthority", user);
    if fs::metadata(&user_xauth).is_ok() {
        // Setup xauth merge for root access
        setup_xauth_access(&user_xauth, display);
        return Some(user_xauth);
    }

    // Try XDG runtime directory
    if let Some(uid) = get_user_uid(user) {
        let _xdg_xauth = format!("/run/user/{}/xauth_*", uid);
        // This would need glob matching in a real implementation
        // For now, return the standard path
    }

    Some(user_xauth)
}

/// Get UID for a username
fn get_user_uid(username: &str) -> Option<u32> {
    let output = Command::new("id").args(["-u", username]).output().ok()?;

    if output.status.success() {
        let uid_str = String::from_utf8(output.stdout).ok()?;
        uid_str.trim().parse().ok()
    } else {
        None
    }
}

/// Setup xauth access for root user
fn setup_xauth_access(user_xauth_path: &str, display: &str) {
    // Extract the user's xauth entry and merge it for root
    let extract_output = Command::new("xauth")
        .args(["-f", user_xauth_path, "extract", "-", display])
        .output();

    if let Ok(output) = extract_output {
        if output.status.success() && !output.stdout.is_empty() {
            // Merge the extracted auth into root's xauth
            let merge_cmd = Command::new("xauth")
                .args(["merge", "-"])
                .stdin(Stdio::piped())
                .spawn();

            if let Ok(mut child) = merge_cmd {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(&output.stdout);
                }
                let _ = child.wait();
            }
        }
    }
}

/// Legacy method for backwards compatibility
fn get_legacy_xauthority() -> String {
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
