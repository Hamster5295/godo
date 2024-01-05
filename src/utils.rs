use console::style;
use std::{fs, io::Error};

pub const INSTALL_PATH: &str = "downloads";

#[macro_export]
macro_rules! err {
    ($title:tt, $content:expr) => {
        panic!("{}\n{}", style($title).red().bold(), style($content).dim())
    };
    ($title:tt) => {
        panic!("{}", style($title).red().bold())
    };
}

pub fn get_dir_name(tag: &String, mono: &bool) -> String {
    let mut result = format!("Godot_{}", tag);
    if *mono {
        result += "_mono";
    }
    result
}

pub fn get_version_name(tag: &String, mono: &bool) -> String {
    "Godot ".to_string() + &get_version_name_short(tag, mono)
}

pub fn get_version_name_short(tag: &String, mono: &bool) -> String {
    let mut result = format!("{}", tag);
    if *mono {
        result += " mono";
    }
    result
}

pub fn parse_version(name: &String) -> Result<(&str, bool), Error> {
    let results: Vec<&str> = name.split(&[' ', '_']).collect();
    if results.len() < 2 {
        Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Wrong format! Require Godot_[version]_[mono] or Godot [version] [mono], found {}",
                name
            ),
        ))
    } else {
        Ok((results[1], results.len() >= 3))
    }
}

pub fn create_install_path() {
    fs::create_dir_all(INSTALL_PATH)
        .unwrap_or_else(|err| err!("Unable to create install directory: ", err.to_string()));
}

pub fn get_installed_dirs() -> Vec<String> {
    create_install_path();

    let mut installs = vec![];
    for dir_result in fs::read_dir(INSTALL_PATH).unwrap() {
        let dir = dir_result.unwrap();
        let path = dir.path();
        if path.is_dir() {
            installs.push(dir.file_name().to_str().unwrap().to_owned());
        }
    }
    installs
}
