use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::helpers::{run_command, run_command_with_output};

use super::reading::file_content_to_u32;

pub struct IntelGpu {
    pub min_frequency: u32,
    pub max_frequency: u32,
    pub boost_frequency: u32,

    path: PathBuf,
}

impl IntelGpu {
    pub fn from_dir(entry: DirEntry) -> IntelGpu {
        IntelGpu {
            path: entry.path(),
            min_frequency: file_content_to_u32(entry.path().join("gt_min_freq_mhz")),
            max_frequency: file_content_to_u32(entry.path().join("gt_max_freq_mhz")),
            boost_frequency: file_content_to_u32(entry.path().join("gt_boost_freq_mhz")),
        }
    }

    pub fn set_min(&self, min: u32) {
        run_command(&format!(
            "echo {min} > {}",
            self.path.join("gt_min_freq_mhz").display()
        ));
    }
    pub fn set_max(&self, max: u32) {
        run_command(&format!(
            "echo {max} > {}",
            self.path.join("gt_max_freq_mhz").display()
        ));
    }
    pub fn set_boost(&self, boost: u32) {
        run_command(&format!(
            "echo {boost} > {}",
            self.path.join("gt_boost_freq_mhz").display()
        ));
    }
}

pub fn iterate_intel_gpus() -> impl IntoIterator<Item = IntelGpu> {
    fs::read_dir("/sys/class/drm")
        .expect("Could not read sysfs drm directory")
        .flatten()
        .filter(|entry| {
            if !entry.file_name().into_string().unwrap().starts_with("card") {
                false
            } else if !run_command_with_output(&format!(
                "readlink {}",
                entry.path().join("device/driver").display()
            ))
            .0
            .contains("i915")
            {
                false
            } else {
                true
            }
        })
        .map(IntelGpu::from_dir)
}
