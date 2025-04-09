use console::style;
use std::{cmp::Ordering, fs, path::PathBuf};

use crate::version::{self, Version};

pub const INSTALL_PATH: &str = "app";

#[macro_export]
macro_rules! err {
    ($title:tt, $content:expr) => {
        panic!("{}\n{}", style($title).red().bold(), style($content).dim())
    };
    ($title:tt) => {
        panic!("{}", style($title).red().bold())
    };
}

pub fn get_install_path() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .with_file_name(INSTALL_PATH)
}

pub fn create_install_path() {
    fs::create_dir_all(get_install_path())
        .unwrap_or_else(|err| err!("Unable to create install directory: ", err.to_string()));
}

pub fn get_installed_dirs() -> Vec<String> {
    create_install_path();

    let mut installs = vec![];
    for dir_result in fs::read_dir(get_install_path()).unwrap() {
        let dir = dir_result.unwrap();
        let path = dir.path();
        if path.is_dir() {
            installs.push(dir.file_name().to_str().unwrap().to_owned());
        }
    }
    installs
}

pub fn get_installed_versions() -> Vec<Version> {
    let mut vers = vec![];
    for dir in get_installed_dirs() {
        if let Some(ver) = version::parse(dir) {
            vers.push(ver);
        }
    }
    vers
}

pub fn get_executables(dir: String) -> Vec<String> {
    let mut files = vec![];

    for dir_result in fs::read_dir(get_install_path().join(dir)).unwrap() {
        let dir = dir_result.unwrap();
        let path = dir.path();
        if path.is_file() {
            files.push(path.to_str().unwrap().to_owned());
        }
    }

    files
}

pub fn get_executable(dir: String, console: bool) -> Option<String> {
    let files = get_executables(dir);
    let mut result: Option<String> = None;
    for file in files {
        if !file.ends_with(".exe") {
            continue;
        }

        if let Some(ref res) = result {
            if console {
                if res.contains("console") && !file.contains("console") {
                    result = Some(file);
                }
            } else if !res.contains("console") && file.contains("console") {
                result = Some(file);
            }
        } else {
            result = Some(file);
        }
    }
    result
}

pub fn search_installed_version(keyword: &Option<String>, mono: Option<bool>) -> Option<Version> {
    let dirs: Vec<String> = get_installed_dirs();
    match keyword {
        Some(version) => {
            // Search based on the keyword
            search_installed_version_with_dirs(mono, dirs, |ver| ver.tag().starts_with(version))
        }
        None => {
            // No keyword, find the latest stable version
            search_installed_version_with_dirs(mono, dirs, |_ver| true)
        }
    }
}

fn search_installed_version_with_dirs<F>(
    mono: Option<bool>,
    dirs: Vec<String>,
    condition: F,
) -> Option<Version>
where
    F: Fn(&Version) -> bool,
{
    let mut result: Option<Version> = None;
    for dir in dirs {
        if let Some(ver) = version::parse(dir) {
            // Fit the keyword
            if condition(&ver) {
                if let Some(mono_flag) = mono {
                    if mono_flag != ver.mono() {
                        continue;
                    }
                }

                if let Some(ref cur_ver) = result {
                    if cur_ver.tag().ends_with("stable") && !ver.tag().ends_with("stable") {
                        continue;
                    }

                    match version::compare(ver.tag(), cur_ver.tag()) {
                        Ordering::Equal => {
                            if !cur_ver.mono() && ver.mono() {
                                result = Some(ver);
                            }
                        }
                        Ordering::Greater => {
                            result = Some(ver);
                        }
                        Ordering::Less => {
                            continue;
                        }
                    }
                } else {
                    result = Some(ver);
                }
            }
        }
    }
    result
}

pub fn uninstall_version(version: Version) {
    fs::remove_dir_all(get_install_path().join(version.dir_name()))
        .unwrap_or_else(|err| err!("Error when uninstalling:", err.to_string()));
}
