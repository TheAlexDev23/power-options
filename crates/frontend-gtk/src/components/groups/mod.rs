pub mod cpu;
pub mod cpu_cores;
pub mod kernel;
pub mod network;
pub mod pci;
pub mod radio;
pub mod sata;
pub mod usb;

pub use cpu::*;
pub use cpu_cores::*;
pub use radio::*;

lazy_static::lazy_static! {
    pub static ref CPU_MODES: Vec<&'static str> = vec!["active", "passive"];
    pub static ref CPU_EPPS: Vec<&'static str> = vec![
        "performance",
        "balance_performance",
        "default",
        "balance_power",
        "power",
    ];
    pub static ref CPU_GOVERNORS_ACTIVE: Vec<&'static str> = vec!["performance", "powersave"];
    pub static ref CPU_GOVERNORS_PASSIVE: Vec<&'static str> = vec![
        "conservative",
        "ondemand",
        "userspace",
        "powersave",
        "performance",
        "schedutil",
    ];
}
