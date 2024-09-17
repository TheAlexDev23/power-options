#![allow(deprecated)]

pub mod communications;
pub mod components;
pub mod helpers;

use std::fs;

use clap::{command, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colored::Colorize;
use log::{Level, Log, Metadata, Record};

use relm4::prelude::*;

use components::*;

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

fn main() {
    initial_warning();

    let args = Args::parse();

    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(args.verbose.log_level_filter());

    set_panic_dialog();

    RelmApp::new("io.github.thealexdev23.power-options.frontend")
        .with_args(Vec::new())
        .run_async::<App>(());
}

fn initial_warning() {
    let warning_lock_dir = std::env::home_dir().unwrap().join(".local/share/power-options-gtk");
    fs::create_dir_all(&warning_lock_dir).expect("Could not create app directory");
    let warning_lock_path = warning_lock_dir.join("user-consent.lock");

    if fs::metadata(&warning_lock_path).is_err() {
        let agreed = std::process::Command::new("yad").args([
            "--selectable-labels",
            "--title",
            "Warning: the GTK frontend might change your settings",
            "--text",
            "While power-options supports the ability to disable some options, the GTK frontend doesn't.\nThe GTK frontend likely <b>will update and reaply</b> your profiles to make sure that the values for all options are set (unless those features are unsupported).\nDo you want to continue?"
        ]).spawn().expect("Could not spawn popup").wait().expect("Could not wait from popup").success();

        if !agreed {
            std::process::exit(-1);
        } else {
            fs::write(warning_lock_path, "").expect("Could not user agreement lock file");
        }
    }
}

fn set_panic_dialog() {
    std::panic::set_hook(Box::new(|info| {
        let secondary_message = info.to_string();

        let message = 
                format!("<b>Unexpected error occurred</b> 
                    \n- Please make sure that the power-options daemon is running.
                    \n- If this is your first time running the app since installing you might need to reboot.
                    \nFull panic message:\n{secondary_message}");

        log::error!("{message}");
        log::info!("Spawning panic dialog.");

        let _ = std::process::Command::new("yad")
            .args([
                "--selectable-labels",
                "--button",
                "yad-close",
                "--title",
                "Unexpected Panic",
                "--text",
                &message
            ])
            .spawn();
    }));
}
