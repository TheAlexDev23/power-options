#![allow(deprecated)]

pub mod communications;
pub mod components;
pub mod helpers;

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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(args.verbose.log_level_filter());

    RelmApp::new("io.github.thealexdev23.power-options.frontend")
        .with_args(Vec::new())
        .run_async::<App>(());
}
