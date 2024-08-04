use serde::{Deserialize, Serialize};

use crate::profiles_generator::DefaultProfileType;

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
}
