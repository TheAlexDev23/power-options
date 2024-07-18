use std::{
    fs::{self, File},
    io::Write,
};

use clap::{Parser, Subcommand};
use colored::Colorize;
use log::{debug, error, trace, Level, Log, Metadata, Record};
use nix::unistd::Uid;

use power_daemon::profiles_generator::DefaultProfileType;

use power_daemon::{Config, Instance, SystemInfo};

use power_daemon::communication::server::CommunicationServer;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: OpMode,
}

#[derive(Debug, Clone, Subcommand)]
enum OpMode {
    Daemon,
    Refresh,
}

static LOGGER: StdoutLogger = StdoutLogger;

struct StdoutLogger;
impl Log for StdoutLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let msg = format!("[{}] {}", record.level(), record.args());
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
    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(log::LevelFilter::Trace);

    let args = Args::parse();
    match args.mode {
        OpMode::Daemon => daemon().await,
        OpMode::Refresh => todo!(),
    }
}

pub const CONFIG_FILE: &str = "/etc/power-daemon/config.toml";
pub const PROFILES_DIRECTORY: &str = "/etc/power-daemon/profiles";

async fn daemon() {
    // From now on, we are the daemon
    proctitle::set_title("power-daemon");

    if !Uid::effective().is_root() {
        error!("Root priviliges required");
        return;
    }

    if fs::metadata("/etc/power-daemon").is_err() {
        fs::create_dir_all("/etc/power-daemon").expect("Could not create config directory");
    }

    create_config_file_if_necessary();
    create_profile_files_if_necessary();
    let config = power_daemon::parse_config(CONFIG_FILE);

    let mut handle = Instance::new(config, PROFILES_DIRECTORY);
    handle.update();
    let _com_server = CommunicationServer::new(handle)
        .await
        .expect("Could not intialize communications server");

    loop {
        std::thread::park();
    }
}

fn create_config_file_if_necessary() {
    if fs::metadata(CONFIG_FILE).is_err() {
        debug!("Creating default config");

        let mut config = File::create(CONFIG_FILE).expect("Could not create config file");
        let content = &toml::to_string_pretty(&Config::create_default()).unwrap();

        trace!("{}", content);

        config
            .write(content.as_bytes())
            .expect("Could not write to config");
    }
}

fn create_profile_files_if_necessary() {
    if fs::metadata(PROFILES_DIRECTORY).is_err() {
        debug!("Creating default profiles");

        fs::create_dir_all(PROFILES_DIRECTORY).expect("Could not create profiles directory");

        let system_info: SystemInfo = SystemInfo::obtain();

        trace!("{:#?}", system_info);

        power_daemon::profiles_generator::create_profile_file(
            PROFILES_DIRECTORY,
            DefaultProfileType::Superpowersave,
            &system_info,
        );
        power_daemon::profiles_generator::create_profile_file(
            PROFILES_DIRECTORY,
            DefaultProfileType::Powersave,
            &system_info,
        );
        power_daemon::profiles_generator::create_profile_file(
            PROFILES_DIRECTORY,
            DefaultProfileType::Balanced,
            &system_info,
        );
        power_daemon::profiles_generator::create_profile_file(
            PROFILES_DIRECTORY,
            DefaultProfileType::Performance,
            &system_info,
        );
        power_daemon::profiles_generator::create_profile_file(
            PROFILES_DIRECTORY,
            DefaultProfileType::Ultraperformance,
            &system_info,
        );
    }
}
