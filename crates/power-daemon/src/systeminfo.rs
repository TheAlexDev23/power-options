pub struct SystemInfo {
    pub cpu_info: CPUAvailableSettings,
}

pub struct CPUAvailableSettings {
    pub governors: Vec<String>,
    pub energy_performance_preferences: Vec<String>,
    pub turbo: bool,
}
