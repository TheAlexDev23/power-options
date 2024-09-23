mod setup;

use std::path::Path;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use colored::Colorize;
use log::{error, Level, Log, Metadata, Record};
use nix::unistd::Uid;

use power_daemon::{communication::client::ControlClient, ReducedUpdate};

use power_daemon::{Instance, SystemInfo};

use power_daemon::communication::server::CommunicationServer;
use setup::{generate_base_files, setup};

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
    Setup,
    GenerateBaseFiles {
        #[arg(long)]
        path: PathBuf,
        /// Path of the executable for this program
        #[arg(long)]
        program_path: PathBuf,
        /// Make sure the daemon starts with maximum verbosity
        #[arg(long, action=clap::ArgAction::SetTrue)]
        verbose_daemon: bool,
    },
    /// Lists the profile names
    ListProfiles,
    /// Creates a temporary override for a certain profile
    SetProfileOverride {
        profile_name: String,
    },
    ResetProfileOverride,
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

pub const CONFIG_FILE: &str = "/etc/power-options/config.toml";
pub const PROFILES_DIRECTORY: &str = "/etc/power-options/profiles";

#[tokio::main]
async fn main() {
    let args = Args::parse();

    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(args.verbose.log_level_filter());

    match args.mode {
        OpMode::Setup => setup(),
        OpMode::Daemon => daemon().await,
        OpMode::GenerateBaseFiles {
            path,
            program_path,
            verbose_daemon,
        } => generate_base_files(path, program_path, verbose_daemon),
        OpMode::ListProfiles => {
            println!(
                "{:?}",
                ControlClient::new()
                    .await
                    .expect("Could not create control client")
                    .get_config()
                    .await
                    .expect("Could not obtain config")
                    .profiles
            );
        }
        OpMode::SetProfileOverride { profile_name } => ControlClient::new()
            .await
            .expect("Could not create control client")
            .set_profile_override(profile_name)
            .await
            .expect("Could not set profile override"),
        OpMode::ResetProfileOverride => ControlClient::new()
            .await
            .expect("Could not create control client")
            .remove_profile_override()
            .await
            .expect("Could not reset profile override"),
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
