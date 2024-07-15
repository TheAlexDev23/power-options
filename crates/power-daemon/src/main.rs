use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use nix::unistd::Uid;

use colored::*;
use log::{debug, error, trace, Level, Log, Metadata, Record};

use config::Config;
use profile::{Profile, ProfilesInfo};
use profiles_generator::DefaultProfileType;
use systeminfo::SystemInfo;

mod config;
mod helpers;
mod profile;
mod profiles_generator;
mod systeminfo;

const CONFIG_FILE: &str = "/etc/power-daemon/config.toml";
const PROFILES_DIRECTORY: &str = "/etc/power-daemon/profiles";

static mut TEMPORARY_OVERRIDE: Option<String> = None;
static mut CONFIG: Option<Config> = None;
static mut PROFILES_INFO: Option<ProfilesInfo> = None;

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

fn main() {
    if !Uid::effective().is_root() {
        error!("Root priviliges required");
        return;
    }

    log::set_logger(&LOGGER).expect("Could not set logger");
    log::set_max_level(log::LevelFilter::Trace);

    if fs::metadata("/etc/power-daemon").is_err() {
        fs::create_dir_all("/etc/power-daemon").expect("Could not create config directory");
    }
    if fs::metadata(CONFIG_FILE).is_err() {
        debug!("Creating default config");

        let mut config = File::create(CONFIG_FILE).expect("Could not create config file");
        let content = &toml::to_string_pretty(&Config::create_default()).unwrap();

        trace!("{}", content);

        config
            .write(content.as_bytes())
            .expect("Could not write to config");
    }
    if fs::metadata(PROFILES_DIRECTORY).is_err() {
        debug!("Creating default profiles");

        fs::create_dir_all(PROFILES_DIRECTORY).expect("Could not create profiles directory");

        let system_info = SystemInfo::obtain();

        trace!("{:#?}", system_info);

        create_profile_file(DefaultProfileType::Superpowersave, &system_info);
        create_profile_file(DefaultProfileType::Powersave, &system_info);
        create_profile_file(DefaultProfileType::Balanced, &system_info);
        create_profile_file(DefaultProfileType::Performance, &system_info);
        create_profile_file(DefaultProfileType::Ultraperformance, &system_info);
    }

    parse_config();
    parse_profiles();

    let profile = unsafe { PROFILES_INFO.as_ref().unwrap().get_active_profile() };

    profile.cpu_settings.apply();
    profile.cpu_core_settings.apply();
    profile.screen_settings.apply();
    profile.radio_settings.apply();
    profile.network_settings.apply();
    profile.aspm_settings.apply();
    profile.pci_settings.apply();
    profile.usb_settings.apply();
    profile.sata_settings.apply();
    profile.kernel_settings.apply();
}

fn create_profile_file(profile_type: DefaultProfileType, system_info: &SystemInfo) {
    debug!("Creating profile of type {profile_type:?}");

    let name = match profile_type {
        DefaultProfileType::Superpowersave => "superpowersave",
        DefaultProfileType::Powersave => "powersave",
        DefaultProfileType::Balanced => "balanced",
        DefaultProfileType::Performance => "performance",
        DefaultProfileType::Ultraperformance => "ultraperformance",
    };

    let profile = profiles_generator::create_default(name, profile_type, system_info);

    let path = PathBuf::from_str(PROFILES_DIRECTORY)
        .unwrap()
        .join(format!("{name}.toml"));

    let mut file = File::create(path).expect("Could not create profile file");

    let content = toml::to_string_pretty(&profile).unwrap();

    trace!("{content}");

    file.write(content.as_bytes())
        .expect("Could not write to profile file");
}

fn parse_config() {
    unsafe {
        CONFIG = Some(
            toml::from_str::<Config>(
                &fs::read_to_string(CONFIG_FILE).expect("Could not read config file"),
            )
            .expect("Could not parse config file")
            .into(),
        );
    }
}

fn parse_profiles() {
    let mut profiles = Vec::new();
    for profile_name in unsafe { CONFIG.as_ref().unwrap().profiles.iter() } {
        let path = PathBuf::from(format!("{PROFILES_DIRECTORY}/{profile_name}.toml"));
        let mut file = fs::File::open(&path).expect("Could not read file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read file");

        let mut profile: Profile = toml::from_str(&contents).expect("Could not parse profile");
        profile.profile_name = profile_name.clone();
        profiles.push(profile);
    }

    unsafe {
        // Order of priority for profile picking:
        // Temporary override > Config override > whatever profile corresponds to the power state

        let active_profile = if let Some(ref temporary_override) = TEMPORARY_OVERRIDE {
            debug!("Picking temporary profile override");
            profile::find_profile_index_by_name(&profiles, &temporary_override)
        } else if let Some(ref profile_override) = CONFIG.as_ref().unwrap().profile_override {
            debug!("Picking settings profile override");
            profile::find_profile_index_by_name(&profiles, profile_override)
        } else if system_on_ac() {
            debug!("Picking AC profile");
            profile::find_profile_index_by_name(&profiles, &CONFIG.as_ref().unwrap().ac_profile)
        } else {
            debug!("Picking BAT profile");
            profile::find_profile_index_by_name(&profiles, &CONFIG.as_ref().unwrap().bat_profile)
        };

        PROFILES_INFO = Some(
            ProfilesInfo {
                profiles,
                active_profile,
            }
            .into(),
        );
    }
}

fn system_on_ac() -> bool {
    let mut ac_online = false;

    if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if let Ok(type_path) = fs::read_to_string(entry_path.join("type")) {
                    let supply_type = type_path.trim();
                    if supply_type == "Mains" {
                        if let Ok(ac_status) = fs::read_to_string(entry_path.join("online")) {
                            ac_online = ac_status.trim() == "1";
                        }
                    }
                }
            }
        }
    }

    ac_online
}
