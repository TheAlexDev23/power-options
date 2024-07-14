use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub ac_profile: String,
    pub bat_profile: String,

    pub profile_override: Option<String>,

    pub profiles: Vec<String>,
}
