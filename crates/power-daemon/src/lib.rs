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
pub use profiles_generator::DefaultProfileType;
pub use systeminfo::*;
pub use zbus::Error as ZBusError;

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use log::{debug, error, trace};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum ReducedUpdate {
    None,
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
    profiles_path: PathBuf,
    config_path: PathBuf,
    config: Config,
    profiles_info: ProfilesInfo,
    temporary_override: Option<String>,
}

impl Instance {
    pub fn new(config: Config, config_path: &Path, profiles_path: &Path) -> Instance {
        let profiles = parse_profiles(&config, profiles_path);
        Instance {
            profiles_path: PathBuf::from(profiles_path),
            config_path: PathBuf::from(config_path),
            config,
            profiles_info: ProfilesInfo {
                profiles,
                ..Default::default()
            },
            temporary_override: None,
        }
    }

    pub fn set_profile_override(&mut self, name: String) {
        self.temporary_override = Some(name);
        self.update_full();
    }
    pub fn try_set_profile_override(&mut self, name: String) {
        if self
            .profiles_info
            .try_find_profile_index_by_name(&name)
            .is_none()
        {
            debug!("Not updating profile override because profile name does not match with any existing profiles");
        } else {
            self.set_profile_override(name);
        }
    }
    pub fn remove_profile_override(&mut self) {
        self.temporary_override = None;
        self.update_full();
    }

    pub fn update_full(&mut self) {
        self.profiles_info.active_profile = self.pick_profile();

        self.profiles_info.get_active_profile().apply_all();
    }
    pub fn update_reduced(&mut self, reduced_update: ReducedUpdate) {
        self.profiles_info.active_profile = self.pick_profile();
        self.profiles_info
            .get_active_profile()
            .apply_reduced(&reduced_update);
    }

    pub fn update_config(&mut self, config: Config) {
        debug!("Updating config...");
        trace!("New config: {config:#?}");

        self.config = config;
        serialize_config(&self.config, &self.config_path);

        // We might have updated the profiles too in the config, so reloading them is a must
        self.profiles_info.profiles = parse_profiles(&self.config, &self.profiles_path);

        self.update_full();
    }

    pub fn get_active_profile_name(&self) -> String {
        self.profiles_info.get_active_profile().profile_name.clone()
    }

    pub fn create_profile(&mut self, profile_type: DefaultProfileType) {
        debug!("Creating profile of type {profile_type:?}");

        let base_name = "New Profile";
        let mut profile_name = base_name.to_string();
        let mut count = 1;
        while self.config.profiles.contains(&profile_name) {
            profile_name = format!("{} #{}", base_name, count);
            count += 1;
        }

        profiles_generator::create_profile_file_with_name(
            profile_name.clone(),
            self.profiles_path.clone(),
            profile_type,
            &SystemInfo::obtain(),
        );

        self.config.profiles.push(profile_name.clone());
        serialize_config(&self.config, &self.config_path);
        // parse_profiles obtains profiles according to the order defined in the
        // config. If the config's order changed then re-callign parse_profiles
        // should give a list of profiles in the new order
        self.profiles_info.profiles = parse_profiles(&self.config, &self.profiles_path);
    }

    pub fn reset_profile(&mut self, idx: usize) {
        if self.verify_index_ranges(idx) {
            return;
        }

        debug!("Resetting profile No {idx}");
        let system_info = SystemInfo::obtain();

        self.profiles_info.profiles[idx] =
            self.profiles_info.profiles[idx].get_original_values(&system_info);
        serialize_profiles(&self.profiles_info.profiles, &self.profiles_path);

        self.update_full();
    }

    pub fn remove_profile(&mut self, idx: usize) {
        if self.profiles_info.profiles.len() <= 1 {
            error!(
                "There's only 1 or less available profiles. Cannot remove remaining. Ignoring..."
            );
            return;
        }

        if self.verify_index_ranges(idx) {
            return;
        }

        if self.profiles_info.active_profile == idx {
            error!("Cannot remove currently active profile, ignoring...");
            return;
        }

        if self.profiles_info.active_profile > idx {
            self.profiles_info.active_profile -= 1;
        }

        let profile_to_remove = &self.profiles_info.profiles[idx];
        let profile_to_remove_name = profile_to_remove.profile_name.clone();

        let mut should_update = false;

        if let Some(ref temporary_override) = self.temporary_override {
            if *temporary_override == profile_to_remove.profile_name {
                self.temporary_override = None;
                should_update = true;
            }
        }
        if let Some(ref persistent_override) = self.config.profile_override {
            if *persistent_override == profile_to_remove.profile_name {
                self.config.profile_override = None;
                should_update = true;
            }
        }

        self.config.profiles.remove(
            self.config
                .profiles
                .iter()
                .position(|p| *p == profile_to_remove.profile_name)
                .unwrap(),
        );
        self.profiles_info.profiles.remove(idx);

        // This needs to be done after removing the actual profile from the
        // list, so that the .first() and .last() values would not point to
        // profiles that may not exist after deletion
        if self.config.bat_profile == profile_to_remove_name {
            self.config.bat_profile = self
                .profiles_info
                .profiles
                .first()
                .unwrap()
                .profile_name
                .clone();
            should_update = true;
        }
        if self.config.ac_profile == profile_to_remove_name {
            self.config.ac_profile = self
                .profiles_info
                .profiles
                .last()
                .unwrap()
                .profile_name
                .clone();
            should_update = true;
        }

        fs::remove_file(
            self.profiles_path
                .join(&format!("{}.toml", &profile_to_remove_name)),
        )
        .expect("Could not remove profile file");

        serialize_config(&self.config, &self.config_path);

        if should_update {
            self.update_full();
        }
    }

    pub fn update_profile_name(&mut self, idx: usize, new_name: String) {
        if self.verify_index_ranges(idx) {
            return;
        }
        for profile in &self.config.profiles {
            if new_name == *profile {
                error!(
                    "Requested to update profile name to an already occupied name. Ignorring..."
                );
                return;
            }
        }

        let old_name = self.config.profiles[idx].clone();

        self.config.profiles[idx] = new_name.clone();
        self.profiles_info.profiles[idx].profile_name = new_name.clone();
        if self.config.ac_profile == old_name {
            self.config.ac_profile = new_name.clone();
        }
        if self.config.bat_profile == old_name {
            self.config.bat_profile = new_name.clone();
        }
        if let Some(ref profile_override) = self.config.profile_override {
            if *profile_override == old_name {
                self.config.profile_override = Some(new_name.clone());
            }
        }
        if let Some(ref profile_override) = self.temporary_override {
            if *profile_override == old_name {
                self.temporary_override = Some(new_name.clone());
            }
        }

        serialize_config(&self.config, &self.config_path);
        // Renaming a profile could cause a previous file with the same name
        // left behind. Therefore we need to clear the directory first and then serialize
        serialize_profiles_clean(&self.profiles_info.profiles, &self.profiles_path);
    }

    pub fn swap_profile_order(&mut self, idx: usize, new_idx: usize) {
        if self.verify_index_ranges(idx) || self.verify_index_ranges(idx) {
            return;
        }

        if self.profiles_info.active_profile == idx {
            self.profiles_info.active_profile = new_idx;
        } else if self.profiles_info.active_profile == new_idx {
            self.profiles_info.active_profile = idx;
        }

        let tmp = self.config.profiles[idx].clone();
        self.config.profiles[idx] = self.config.profiles[new_idx].clone();
        self.config.profiles[new_idx] = tmp;
        serialize_config(&self.config, &self.config_path);
        self.profiles_info.profiles = parse_profiles(&self.config, &self.profiles_path);
    }

    pub fn update_profile_full(&mut self, idx: usize, profile: Profile) {
        self.update_profile(idx, profile);

        if idx == self.profiles_info.active_profile {
            self.update_full();
        }
    }
    pub fn update_profile_reduced(
        &mut self,
        idx: usize,
        profile: Profile,
        reduced_update: ReducedUpdate,
    ) {
        self.update_profile(idx, profile);

        if idx == self.profiles_info.active_profile {
            self.update_reduced(reduced_update);
        }
    }

    /// Returns the index of the profile that should be selcted at the moment
    /// according to all settings and overrides
    fn pick_profile(&self) -> usize {
        if let Some(ref temporary_override) = self.temporary_override {
            debug!("Picking temporary profile override");
            self.profiles_info
                .find_profile_index_by_name(temporary_override)
        } else if let Some(ref profile_override) = self.config.profile_override {
            debug!("Picking settings profile override");
            self.profiles_info
                .find_profile_index_by_name(profile_override)
        } else if helpers::system_on_ac() {
            debug!("Picking AC profile");
            self.profiles_info
                .find_profile_index_by_name(&self.config.ac_profile)
        } else {
            debug!("Picking BAT profile");
            self.profiles_info
                .find_profile_index_by_name(&self.config.bat_profile)
        }
    }

    fn update_profile(&mut self, idx: usize, profile: Profile) {
        if self.verify_index_ranges(idx) {
            return;
        }

        debug!("Updating profile No {idx}");
        trace!("New profile: {profile:#?}");

        self.profiles_info.profiles[idx] = profile;
        // We actually need to update the underlying files
        serialize_profiles(&self.profiles_info.profiles, &self.profiles_path);
    }

    fn verify_index_ranges(&self, idx: usize) -> bool {
        if idx >= self.config.profiles.len() || idx >= self.profiles_info.profiles.len() {
            error!("Profile with requested index is outside of bounds, ignoring...");
            true
        } else {
            false
        }
    }
}

pub fn parse_config(path: &Path) -> Config {
    toml::from_str::<Config>(&fs::read_to_string(path).expect("Could not read config"))
        .expect("Could not parse config")
}

fn parse_profiles(config: &Config, path: &Path) -> Vec<Profile> {
    let mut profiles = Vec::new();
    for profile_name in config.profiles.iter() {
        let path = path.join(format!("{profile_name}.toml"));
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

pub fn serialize_config(config: &Config, path: &Path) {
    fs::write(
        path,
        toml::to_string_pretty(config).expect("Could not serialize config"),
    )
    .expect("Could not write to config");
}

fn serialize_profiles_clean(profiles: &[Profile], path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
    }

    serialize_profiles(profiles, path);
}

fn serialize_profiles(profiles: &[Profile], path: &Path) {
    for profile in profiles.iter() {
        let path = path.join(format!("{}.toml", profile.profile_name));
        let mut file = fs::File::create(&path).expect("Could not read file");
        file.write_all(
            toml::to_string_pretty(profile)
                .expect("Could not serialize profile")
                .as_bytes(),
        )
        .expect("Could not write to profile file");
    }
}
