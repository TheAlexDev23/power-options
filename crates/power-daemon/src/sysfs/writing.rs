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


fn write_str_inner(path: impl AsRef<Path>, payload: &str) -> io::Result<()> {
    let path = path.as_ref();
    debug!("Writing payload \"{payload}\" to path {}", path.display());
    let mut fh = fs::File::options().write(true).truncate(true).open(path)?;
    fh.write_all(payload.as_bytes())?;
    fh.flush()?;
    Ok(())
}