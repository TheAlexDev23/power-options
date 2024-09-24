use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

pub fn file_content_to_string<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content
}

// Will read file at path and return a list of elements with space as the separator
// Will panic with io errors
pub fn file_content_to_list<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content.split(" ").map(String::from).collect()
}

// Will read file at path and parse u32
// Will panic with io errors and parsing errors
pub fn file_content_to_u32<P: AsRef<Path>>(path: P) -> u32 {
    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content.parse().unwrap()
}

pub fn try_file_content_to_u32<P: AsRef<Path>>(path: P) -> Option<u32> {
    let mut file = File::open(path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content.parse().ok()
}

// Will read file at path and return true if content is 1 false otherwise
// Will return false if the file doesn't exist but will panic if some io issues appear
pub fn file_content_to_bool<P: AsRef<Path>>(path: P) -> bool {
    if fs::metadata(path.as_ref()).is_err() {
        return false;
    }

    let mut file = File::open(path).expect("Could not open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read file");

    content = content.strip_suffix("\n").unwrap_or(&content).to_string();
    content = content.strip_suffix(" ").unwrap_or(&content).to_string();

    content == "1" || content == "Y"
}
