use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub ac_profile: String,
    pub bat_profile: String,

    pub profile_override: Option<String>,

    pub profiles: Vec<String>,
}

impl Config {
    pub fn create_default() -> Config {
        Config {
            ac_profile: String::from("performance"),
            bat_profile: String::from("powersave"),

            profile_override: None,

            profiles: vec![
                "superpowersave",
                "powersave",
                "balanced",
                "performance",
                "ultraperformance",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }
}
