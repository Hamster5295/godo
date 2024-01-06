use console::style;
use std::fs;

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
