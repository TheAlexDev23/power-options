use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use log::{debug, error, warn};

/// Writes a bool value to a /sys path, mapping `true` to "1" and `false` to
/// "0".
///
/// Logs when encountering an error but does not crash.
pub fn write_bool(path: impl AsRef<Path>, value: bool) {
    let payload = if value { "1" } else { "0" };
    write_str(path, payload)
}

/// Writes a u32 value to a /sys path.
///
/// Logs when encountering an error but does not crash.
pub fn write_u32(path: impl AsRef<Path>, value: u32) {
    let payload = value.to_string();
    write_str(path, &payload)
}

/// Writes a string value to a /sys path.
///
/// Logs when encountering an error but does not crash.
pub fn write_str(path: impl AsRef<Path>, payload: &str) {
    let path = path.as_ref();
    match write_str_inner(path, payload) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!(
                "Attempted to write \"{payload}\" to {}, but path does not exist!",
                path.display()
            );
        }
        Err(e) => {
            error!(
                "Error writing \"{payload}\" to path {}: {e:?}",
                path.display()
            );
        }
    }
}

/// Writes a value to all CPU paths under /sys/devices/cpu/cpu* that are
/// actually single CPU core management directories (which end in numbers),
/// warning if any of the individual cores can't have that value set.
pub fn write_all_cores(path: impl AsRef<Path>, data: &str) {
    let path = path.as_ref();
    let raw = fs::read_dir("/sys/devices/system/cpu/").expect("Error reading CPU list");
    let relevant = raw.filter_map(Result::ok).filter(|ent| {
        let raw_filename = ent.file_name();
        let name = raw_filename.to_string_lossy();
        let Some((_, suffix)) = name.split_once("cpu") else {
            return false;
        };
        suffix.parse::<u32>().is_ok()
    });
    let to_write = relevant.map(|ent| ent.path().join(path));
    for target in to_write {
        if !target.exists() {
            warn!(
                "Attempted to write {data} to CPU sysfs path {}, but that path does not exist!",
                target.display()
            );
            continue;
        }
        write_str(target, data);
    }
}

fn write_str_inner(path: impl AsRef<Path>, payload: &str) -> io::Result<()> {
    let path = path.as_ref();
    debug!("Writing payload \"{payload}\" to path {}", path.display());
    let mut fh = fs::File::options().write(true).truncate(true).open(path)?;
    fh.write_all(payload.as_bytes())?;
    fh.flush()?;
    Ok(())
}
