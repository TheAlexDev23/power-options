#[cfg(feature = "communication")]
pub mod communication;
pub mod config;
pub mod profile;
pub mod profiles_generator;
pub mod systeminfo;

mod helpers;

use serde::{Deserialize, Serialize};

pub use config::*;
pub use helpers::{WhiteBlackList, WhiteBlackListType};
pub use profile::*;
pub use systeminfo::*;

use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use log::{debug, trace};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum ReducedUpdate {
    CPU,
    CPUCores,
    SingleCPUCore(u32),
    MultipleCPUCores(Vec<u32>),
    Screen,
    Radio,
    Network,
    ASPM,
    PCI,
    USB,
    SATA,
    Kernel,
}

pub struct Instance {
    profiles_path: String,
    config: Config,
    profiles_info: ProfilesInfo,
    reduced_update: Option<ReducedUpdate>,
    temporary_override: Option<String>,
}

impl Instance {
    pub fn new(config: Config, profiles_path: &str) -> Instance {
        let profiles = parse_profiles(&config, profiles_path);
        Instance {
            profiles_path: String::from(profiles_path),
            config,
            profiles_info: ProfilesInfo {
                profiles,
                ..Default::default()
            },
            reduced_update: None,
            temporary_override: None,
        }
    }

    pub fn set_profile_override(&mut self, name: String) {
        self.temporary_override = Some(name);
        self.update();
    }
    pub fn try_set_profile_override(&mut self, name: String) {
        if try_find_profile_index_by_name(&self.profiles_info.profiles, &name).is_none() {
            debug!("Not updating profile override because profile name does not match with any existing profiles");
        } else {
            self.set_profile_override(name);
        }
    }

    pub fn remove_profile_override(&mut self) {
        self.temporary_override = None;
        self.update();
    }

    pub fn set_reduced_update(&mut self, reduced_update: ReducedUpdate) {
        self.reduced_update = Some(reduced_update);
    }
    pub fn reset_reduced_update(&mut self) {
        self.reduced_update = None;
    }

    pub fn update(&mut self) {
        let profiles = &self.profiles_info.profiles;
        let active_profile = if let Some(ref temporary_override) = self.temporary_override {
            debug!("Picking temporary profile override");
            profile::find_profile_index_by_name(&profiles, &temporary_override)
        } else if let Some(ref profile_override) = self.config.profile_override {
            debug!("Picking settings profile override");
            profile::find_profile_index_by_name(&profiles, profile_override)
        } else if helpers::system_on_ac() {
            debug!("Picking AC profile");
            profile::find_profile_index_by_name(&profiles, &self.config.ac_profile)
        } else {
            debug!("Picking BAT profile");
            profile::find_profile_index_by_name(&profiles, &self.config.bat_profile)
        };

        self.profiles_info.active_profile = active_profile;

        if let Some(ref reduced_update) = self.reduced_update {
            self.profiles_info
                .get_active_profile()
                .apply_reduced(reduced_update);
        } else {
            self.profiles_info.get_active_profile().apply_all();
        }
    }

    pub fn update_config(&mut self, config: Config) {
        debug!("Updating config...");
        trace!("New config: {config:#?}");

        self.config = config;
        // We might have updated the profiles too in the config, so reloading them is a must
        self.profiles_info.profiles = parse_profiles(&self.config, &self.profiles_path);
        self.update();
    }

    pub fn update_profile(&mut self, idx: usize, profile: Profile) {
        debug!("Updating profile No {idx}");
        trace!("New profile: {profile:#?}");

        self.profiles_info.profiles[idx] = profile;
        // We actually need to update the underlying files
        serialize_profiles(&self.profiles_info.profiles, &self.profiles_path);
        self.update();
    }
}

pub fn parse_config(path: &str) -> Config {
    toml::from_str::<Config>(&fs::read_to_string(path).expect("Could not read config"))
        .expect("Could not parse config")
}

fn parse_profiles(config: &Config, path: &str) -> Vec<Profile> {
    let mut profiles = Vec::new();
    for profile_name in config.profiles.iter() {
        let path = PathBuf::from(format!("{path}/{profile_name}.toml"));
        let mut file = fs::File::open(&path).expect("Could not read file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read file");

        let mut profile: Profile = toml::from_str(&contents).expect("Could not parse profile");
        profile.profile_name = profile_name.clone();
        profiles.push(profile);
    }

    profiles
}

fn serialize_profiles(profiles: &Vec<Profile>, path: &str) {
    for profile in profiles.iter() {
        let path = PathBuf::from(format!("{path}/{}.toml", profile.profile_name));
        let mut file = fs::File::create(&path).expect("Could not read file");
        file.write(
            toml::to_string_pretty(profile)
                .expect("Could not serialize profile")
                .as_bytes(),
        )
        .expect("Could not write to profile file");
    }
}
