use std::{
    fs,
    path::{Path, PathBuf},
};

use log::error;
use regex::Regex;

use crate::sysfs::reading::{file_content_to_u32, try_file_content_to_u32};

use super::{reading::file_content_to_string, writing::write_u32};

pub struct IntelRaplInterface {
    pub interface_type: InterfaceType,

    pub long_term: Option<IntelRaplConstraint>,
    pub short_term: Option<IntelRaplConstraint>,
    pub peak_power: Option<IntelRaplConstraint>,
}

impl IntelRaplInterface {
    pub fn from_path(path: &Path) -> Option<IntelRaplInterface> {
        let name = file_content_to_string(path.join("name"));

        let mut ret = IntelRaplInterface {
            interface_type: if name == "package-0" {
                InterfaceType::Package
            } else if name == "core" {
                InterfaceType::Core
            } else if name == "uncore" {
                InterfaceType::Uncore
            } else {
                error!("Unexpected intel rapl interface. Expected package-0, core or uncore");
                return None;
            },
            long_term: None,
            peak_power: None,
            short_term: None,
        };

        for idx in 0..=2 {
            let name_path = path.join(format!("constraint_{}_name", idx));
            let time_path = path.join(format!("constraint_{}_time_window_us", idx));
            let power_path = path.join(format!("constraint_{}_power_limit_uw", idx));

            if fs::metadata(path.join(&name_path)).is_err() {
                continue;
            }

            let interface = IntelRaplConstraint {
                power_limit_uw: file_content_to_u32(&power_path).into(),
                time_window_us: try_file_content_to_u32(&time_path),
                path: path.to_path_buf(),
            };

            let name = file_content_to_string(&name_path);

            if name == "long_term" {
                ret.long_term = interface.into();
            } else if name == "short_term" {
                ret.short_term = interface.into();
            } else if name == "peak_power" {
                ret.peak_power = interface.into();
            } else {
                error!("Unexpected intel rapl constraint. Does not match long_term, short_term or peak_power");
            }
        }

        ret.into()
    }
}

pub enum InterfaceType {
    Package,
    Core,
    Uncore,
}

pub struct IntelRaplConstraint {
    pub power_limit_uw: u32,
    /// Some constraints lack a time window because they are the fallback
    /// constraints that are supposed to run infinetly
    pub time_window_us: Option<u32>,

    path: PathBuf,
}

impl IntelRaplConstraint {
    pub fn set_power_limit(&self, limit: u32) {
        write_u32(&self.path, limit);
    }
}

pub fn iterate_rapl_interfaces() -> Option<impl Iterator<Item = IntelRaplInterface>> {
    if fs::metadata("/sys/class/powercap").is_err() {
        None
    } else {
        let re = Regex::new(r"^intel-rapl(:\d+){1,2}").unwrap();
        fs::read_dir("/sys/class/powercap")
            .expect("Could not read powercap sysfs dir")
            .flatten()
            .filter(move |d| re.is_match(&d.file_name().into_string().unwrap()))
            .map(|d| IntelRaplInterface::from_path(&d.path()))
            .flatten()
            .into()
    }
}
