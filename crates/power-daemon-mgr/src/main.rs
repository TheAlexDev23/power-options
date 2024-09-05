use std::path::Path;
use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use colored::Colorize;
use log::{debug, error, trace, Level, Log, Metadata, Record};
use nix::unistd::Uid;

use power_daemon::{
    communication::client::ControlClient, profiles_generator::DefaultProfileType, ReducedUpdate,
};

use power_daemon::{Config, Instance, SystemInfo};

use power_daemon::communication::server::CommunicationServer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
    #[command(subcommand)]
    mode: OpMode,
}

#[derive(Debug, Clone, Subcommand)]
enum OpMode {
    GenerateFiles {
        #[arg(long)]
        path: PathBuf,
        /// Path of the executable for this program
        #[arg(long)]
        program_path: PathBuf,
        /// Make sure the daemon starts with maximum verbosity
        #[arg(long, action=clap::ArgAction::SetTrue)]
        verbose_daemon: bool,
    },
    Daemon,
    RefreshFull,
    RefreshUSB,
    RefreshPCI,
    PrintSystemInfo,
}

static LOGGER: StdoutLogger = StdoutLogger;

struct StdoutLogger;
impl Log for StdoutLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let msg = format!(
            "{} ==> [{}]: {}",
            record.level(),
            record.target(),
            record.args()
        );
        let msg = match record.level() {
            Level::Error => msg.red(),
            Level::Warn => msg.yellow(),
            Level::Info | Level::Debug | Level::Trace => msg.white(),
        };

        if record.level() >= Level::Warn {
            eprintln!("{}", msg)
        } else {
            println!("{}", msg);
        }
    }

    fn flush(&self) {}
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(args.verbose.log_level_filter());

    match args.mode {
        OpMode::Daemon => daemon().await,
        OpMode::GenerateFiles {
            path,
            program_path,
            verbose_daemon,
        } => generate_files(path, program_path, verbose_daemon),
        OpMode::RefreshFull => refresh_full().await,
        OpMode::RefreshUSB => refresh_reduced(ReducedUpdate::USB).await,
        OpMode::RefreshPCI => {
            refresh_reduced(ReducedUpdate::PCI).await;
            refresh_reduced(ReducedUpdate::ASPM).await;
        }
        OpMode::PrintSystemInfo => {
            println!("{:#?}", SystemInfo::obtain());
        }
    }
}

pub const CONFIG_FILE: &str = "/etc/power-options/config.toml";
pub const PROFILES_DIRECTORY: &str = "/etc/power-options/profiles";

async fn daemon() {
    // From now on, we are the daemon
    proctitle::set_title("power-daemon");

    if !Uid::effective().is_root() {
        error!("Root priviliges required");
        return;
    }

    let config_path = Path::new(CONFIG_FILE);
    let profiles_path = Path::new(PROFILES_DIRECTORY);

    let config = power_daemon::parse_config(config_path);
    let mut handle = Instance::new(config, config_path, profiles_path);

    handle.update_full();

    let _com_server = CommunicationServer::new(handle)
        .await
        .expect("Could not initialize communications server");

    loop {
        std::thread::park();
    }
}

fn generate_files(path: PathBuf, program_path: PathBuf, verbose_daemon: bool) {
    generate_config(&path);
    generate_profiles(&path);
    generate_udev_file(&path, &program_path);
    generate_acpi_file(&path, &program_path);
    generate_dbus_file(&path);
    genereate_systemd_file(&path, &program_path, verbose_daemon);
}

async fn refresh_full() {
    let client = ControlClient::new()
        .await
        .expect("Could not intialize control client");
    client
        .update_full()
        .await
        .expect("Could not reset reducedu update");
}

async fn refresh_reduced(reduced_update: ReducedUpdate) {
    let client = ControlClient::new()
        .await
        .expect("Could not intialize control client");
    client
        .update_reduced(reduced_update)
        .await
        .expect("Could not reset reducedu update");
}

fn generate_config(path: &Path) {
    debug!("Creating default config");

    let dir = path.join("etc/power-options/");

    fs::create_dir_all(&dir).expect("Could not create directory");

    let content = &toml::to_string_pretty(&Config::create_default()).unwrap();

    fs::write(dir.join("config.toml"), content).expect("Could not write to file");
}

fn generate_profiles(path: &Path) {
    debug!("Creating default profiles");

    let dir = path.join("etc/power-options/profiles/");

    fs::create_dir_all(&dir).expect("Could not create directory");

    let system_info: SystemInfo = SystemInfo::obtain();

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

    let dir = path.join("lib/systemd/system/");
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
