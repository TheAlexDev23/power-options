use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::profiles_generator::DefaultProfileType;

use itertools::Itertools;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub ac_profile: String,
    pub bat_profile: String,

    pub profile_override: Option<String>,

    pub profiles: Vec<String>,
}

impl Config {
    pub fn create_default() -> Config {
        Config {
            ac_profile: DefaultProfileType::Performance.get_name(),
            bat_profile: DefaultProfileType::Powersave.get_name(),

            profile_override: None,

            profiles: DefaultProfileType::get_name_of_all(),
        }
    }
    pub fn create_empty() -> Config {
        Config {
            ac_profile: "Default".to_string(),
            bat_profile: "Default".to_string(),

            profile_override: None,

            profiles: vec!["Default".to_string()],
        }
    }

    /// Will attempt to parse `contentent`, if it fails will merge `content`
    /// with the deafult config and attempt to merge that too
    pub fn parse_or_default(content: &str) -> Config {
        match toml::from_str::<Config>(content) {
            Ok(c) => c,
            Err(_) => {
                warn!("Failed to parse config, attempting to migrate to newer version");

                let default_content = toml::to_string(&Config::create_default()).unwrap();
                let merged = serde_toml_merge::merge(
                    default_content.parse::<toml::Value>().unwrap(),
                    content.parse::<toml::Value>().unwrap(),
                )
                .expect("Could not merge default config and user config");

                debug!("Merged config: {merged:?}");

                let mut config = Config::deserialize(merged).unwrap();

                config.profiles = config.profiles.into_iter().unique().collect();

                config
            }
        }
    }
}
