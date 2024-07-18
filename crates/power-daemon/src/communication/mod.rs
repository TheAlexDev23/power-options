#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

const WELL_KNOWN_NAME: &'static str = "io.github.thealexdev23.power_daemon";
const SYSTEM_INFO_OBJECT_NAME: &'static str = "/io/github/thealexdev23/power_daemon/system_info";

const CONTROL_OBJECT_NAME: &'static str = "/io/github/thealexdev23/power_daemon/control";
