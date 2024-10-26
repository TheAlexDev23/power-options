use std::{fs, path::Path};

use log::{debug, trace};
use power_daemon::{profiles_generator, Config, DefaultProfileType, SystemInfo};

use crate::helpers::yn_prompt;

pub fn setup(root: &Path) {
    println!("\nWarning: do you want power-options to generate profiles?");
    println!("By default, power-options will generate profiles based on the features of your system, and it might apply them. \nPlease refer to the wiki (https://github.com/TheAlexDev23/power-options/wiki/Default-generated-settings) to be aware of potential issues that might arise.");

    let agreed = yn_prompt("Are you sure you want to continue? If you answer no an empty default profile would be genereated.");

    if agreed {
        generate_config_files(root);
    } else {
        generate_empty_config_files(root);
    }
}

pub fn generate_base_files(path: &Path, program_path: &Path, verbose_daemon: bool) {
    generate_udev_file(path, program_path);
    generate_acpi_file(path, program_path);
    generate_dbus_file(path);
    genereate_systemd_file(path, program_path, verbose_daemon);
}

fn generate_config_files(path: &Path) {
    create_config(path, &Config::create_default());
    generate_profiles(path);
}

fn generate_empty_config_files(path: &Path) {
    create_config(path, &Config::create_empty());
    generate_empty_profile(path);
}

fn create_config(path: &Path, config: &Config) {
    debug!("Creating config");

    let dir = path.join("etc/power-options/");

    fs::create_dir_all(&dir).expect("Could not create directory");

    let content = &toml::to_string_pretty(config).unwrap();

    fs::write(dir.join("config.toml"), content).expect("Could not write to file");
}

fn generate_profiles(path: &Path) {
    debug!("Creating default profiles");

    let dir = path.join("etc/power-options/profiles/");

    fs::create_dir_all(&dir).expect("Could not create directory");

    let system_info = SystemInfo::obtain();

    power_daemon::profiles_generator::create_profile_file(
        &dir,
        DefaultProfileType::Superpowersave,
        &system_info,
    );
    power_daemon::profiles_generator::create_profile_file(
        &dir,
        DefaultProfileType::Powersave,
        &system_info,
    );
    power_daemon::profiles_generator::create_profile_file(
        &dir,
        DefaultProfileType::Balanced,
        &system_info,
    );
    power_daemon::profiles_generator::create_profile_file(
        &dir,
        DefaultProfileType::Performance,
        &system_info,
    );
    power_daemon::profiles_generator::create_profile_file(
        &dir,
        DefaultProfileType::Ultraperformance,
        &system_info,
    );
}

fn generate_empty_profile(path: &Path) {
    debug!("Creating empty profile");

    let dir = path.join("etc/power-options/profiles/");

    fs::create_dir_all(&dir).expect("Could not create directory");

    profiles_generator::create_empty_profile_file_with_name(dir, "Default");
}

fn generate_udev_file(path: &Path, program_path: &Path) {
    debug!("Generating udev file");

    let dir = path.join("usr/lib/udev/rules.d/");
    fs::create_dir_all(&dir).expect("Could not create directory");

    let program_path = program_path.display();

    let content = format!(
        r#"
# power-daemon - udev rules

ACTION=="add", SUBSYSTEM=="usb", DRIVER=="usb", ENV{{DEVTYPE}}=="usb_device", RUN+="{program_path} refresh-usb"

ACTION=="add", SUBSYSTEM=="pci", ENV{{DEVTYPE}}=="pci_device", RUN+="{program_path} refresh-pci"
"#
    );

    fs::write(dir.join("85-power-daemon.rules"), &content).expect("Could not write to file");
}

fn generate_acpi_file(path: &Path, program_path: &Path) {
    debug!("Generating ACPI file");

    let dir = path.join("etc/acpi/events/");
    fs::create_dir_all(&dir).expect("Could not create directory");

    let program_path = program_path.display();

    let content = format!(
        r#"
event=ac_adapter
action={program_path} refresh-full
"#
    );

    fs::write(dir.join("power-options"), content).expect("COuld not write to file");
}

fn generate_dbus_file(path: &Path) {
    debug!("Generating DBUS file");

    let dir = path.join("usr/share/dbus-1/system.d/");
    fs::create_dir_all(&dir).expect("Could not create directory");

    let content = r#"
<!-- This configuration file specifies the required security policies for the power-options daemon's communication d-bus channel to work. -->

<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <policy user="root">
    <allow own="io.github.thealexdev23.power_daemon"/>
    <allow send_destination="io.github.thealexdev23.power_daemon"/>
    <allow send_interface="io.github.thealexdev23.power_daemon.system_info"/>
  </policy>

  <policy context="default">
    <allow send_destination="io.github.thealexdev23.power_daemon"/>
  </policy>
</busconfig>
"#;

    trace!("{content}");

    fs::write(dir.join("power-daemon.conf"), content).expect("Could not write to file");
}

fn genereate_systemd_file(path: &Path, program_path: &Path, verbose_daemon: bool) {
    debug!("Generating systemd file");

    let dir = path.join("usr/lib/systemd/system/");
    fs::create_dir_all(&dir).expect("Could not create directory");

    let program_path = program_path.display();

    let content = format!(
        r#"
# power-options - systemd service

[Unit]
Description=power-options daemon
After=multi-user.target NetworkManager.service
Before=shutdown.target

[Service]
ExecStart={program_path} daemon {}

[Install]
WantedBy=multi-user.target"#,
        if verbose_daemon { "-vvv" } else { "" }
    );

    trace!("{content}");

    fs::write(dir.join("power-options.service"), content).expect("Could not write to file");
}
