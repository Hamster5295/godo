use std::{
    borrow::Borrow,
    env::consts,
    fs,
    io::{self, Write},
    iter, vec,
};

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

async fn get_all_releases(client: &Client, prerelease: bool) -> Vec<Release> {
    let req = client
        .get(if prerelease {
            "https://api.github.com/repos/godotengine/godot-builds/releases"
        } else {
            "https://api.github.com/repos/godotengine/godot/releases"
        })
        .header("User-Agent", "Hamster5295");
    let response = req
        .send()
        .await
        .unwrap_or_else(|err| panic!("Failed to connect to github: {:?}", err));

    let mut results = response
        .json::<Vec<Release>>()
        .await
        .unwrap_or_else(|err| panic!("Failed to parse the result from github: {:?}", err));

    if !prerelease {
        let mut idx = results.len();
        while idx > 0 {
            idx -= 1;
            if results[idx].prerelease {
                results.remove(idx);
            }
        }
    }

    results
}

pub async fn list_avail(client: &Client, prerelease: bool) {
    let versions = get_all_releases(client, prerelease).await;

    let mut writer = io::BufWriter::new(io::stdout());
    let yellow = Style::new().yellow().bold();
    let green = Style::new().green().bold();
    let white = Style::new().white();
    let dim = Style::new().dim().bold();

    if prerelease {
        writer
            .write(
                format!(
                    "{:15}{:15}{:15}{:15}\n",
                    yellow.apply_to("Godot4"),
                    yellow.apply_to("Godot4-Pre"),
                    yellow.apply_to("Godot3"),
                    yellow.apply_to("Godot3-Pre"),
                )
                .as_bytes(),
            )
            .unwrap();
        writer
            .write(
                format!(
                    "{}",
                    dim.apply_to(iter::repeat("=").take(60).collect::<String>() + "\n")
                )
                .as_bytes(),
            )
            .unwrap();
        let mut ver4: Vec<Release> = vec![];
        let mut ver4_pre: Vec<Release> = vec![];
        let mut ver3: Vec<Release> = vec![];
        let mut ver3_pre: Vec<Release> = vec![];
        for ver in versions {
            if ver.tag_name.starts_with("4") {
                if ver.tag_name.ends_with("stable") {
                    ver4.push(ver);
                } else {
                    ver4_pre.push(ver)
                }
            } else {
                if ver.tag_name.ends_with("stable") {
                    ver3.push(ver);
                } else {
                    ver3_pre.push(ver)
                }
            }
        }

        let lenth = ver3
            .len()
            .max(ver3_pre.len().max(ver4.len().max(ver4_pre.len())));

        for i in 0..lenth {
            writer
                .write(
                    format!(
                        "{:15}{:15}{:15}{:15}\n",
                        if ver4.len() > i {
                            if i == 0 { &green } else { &white }.apply_to(ver4[i].tag_name.borrow())
                        } else {
                            white.apply_to("")
                        },
                        if ver4_pre.len() > i {
                            &ver4_pre[i].tag_name
                        } else {
                            ""
                        },
                        if ver3.len() > i {
                            if i == 0 { &green } else { &white }.apply_to(ver3[i].tag_name.borrow())
                        } else {
                            white.apply_to("")
                        },
                        if ver3_pre.len() > i {
                            &ver3_pre[i].tag_name
                        } else {
                            ""
                        },
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    } else {
        writer
            .write(
                format!(
                    "{:15}{:15}\n",
                    yellow.apply_to("Godot4"),
                    yellow.apply_to("Godot3"),
                )
                .as_bytes(),
            )
            .unwrap();
        writer
            .write(
                format!(
                    "{}",
                    dim.apply_to(iter::repeat("=").take(30).collect::<String>() + "\n")
                )
                .as_bytes(),
            )
            .unwrap();
        let mut ver4: Vec<Release> = vec![];
        let mut ver3: Vec<Release> = vec![];
        for ver in versions {
            if ver.tag_name.starts_with("4") {
                ver4.push(ver);
            } else {
                ver3.push(ver);
            }
        }

        let lenth = ver3.len().max(ver4.len());

        for i in 0..lenth {
            writer
                .write(
                    format!(
                        "{:15}{:15}\n",
                        if ver4.len() > i {
                            if i == 0 { &green } else { &white }.apply_to(ver4[i].tag_name.borrow())
                        } else {
                            white.apply_to("")
                        },
                        if ver3.len() > i {
                            if i == 0 { &green } else { &white }.apply_to(ver3[i].tag_name.borrow())
                        } else {
                            white.apply_to("")
                        },
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    }

    writer.flush().unwrap();
}

pub async fn get_download_info(
    client: &Client,
    version: String,
    mono: bool,
) -> Option<(String, String)> {
    let releases = get_all_releases(
        client,
        version.contains("-") && !version.ends_with("stable"),
    )
    .await;
    let result = {
        let mut idx = 0;
        let mut flag = false;
        for item in &releases {
            if item.tag_name.starts_with(version.as_str()) {
                flag = true;
                break;
            }
            idx += 1;
        }
        if flag {
            idx
        } else {
            return None;
        }
    };
    for item in &releases[result].assets {
        if item.name.contains(match consts::OS {
            "windows" => "win",
            "macos" => "macos",
            "linux" => "linux",
            _ => panic!("Unsupported operating system!"),
        }) && item.name.contains(match consts::ARCH {
            "x86_64" => {
                if consts::OS == "windows" {
                    "win64"
                } else {
                    "x86_64"
                }
            }
            "x86" => {
                if consts::OS == "windows" {
                    "win32"
                } else {
                    "x86_32"
                }
            }
            _ => {
                if consts::OS == "macos" {
                    "universal"
                } else if consts::ARCH.ends_with("64") {
                    "arm64"
                } else {
                    "arm32"
                }
            }
        }) && item.name.contains("mono") == mono
        {
            return Some((
                releases[result].tag_name.clone(),
                item.browser_download_url.clone(),
            ));
        }
    }
    None
}

pub async fn download(client: &Client, file_name: String, url: String) {
    let mut result = client
        .get(url)
        .header("User-Agent", "Hamster5295")
        .header("Accept", "application/vnd.github.full+json")
        .send()
        .await
        .unwrap_or_else(|err| {
            if err.is_timeout() {
                panic!("Connection Timeout: {:?}", err)
            } else if err.is_request() {
                panic!("Error with request: {:?}", err)
            } else if err.is_status() {
                panic!("Error when connecting: {:?}", err)
            } else {
                panic!("Error: {:?}", err)
            }
        });

    let path = format!("{}.zip", file_name);
    let mut writer = io::BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .unwrap_or_else(|err| match err.kind() {
                io::ErrorKind::NotFound => {
                    panic!("Path not found: {}", &path);
                }
                io::ErrorKind::PermissionDenied => {
                    panic!("Permission denied.\nPlease guarantee Write permission to the directory: {}",&path)
                },
                _=>{
                    panic!("Error: {:?}",err);
                }
            }),
    );

    let length = result.content_length().unwrap();
    let bar = ProgressBar::new(length).with_style(
        ProgressStyle::with_template(
            "[{percent}%] {bar:40.cyan/blue} {bytes:>7} / {total_bytes:7} [{bytes_per_sec}]",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    while let Some(content) = result
        .chunk()
        .await
        .unwrap_or_else(|err| panic!("Error when downloading: {:?}", err))
    {
        writer.write(&content).unwrap();
        bar.inc(content.len() as u64);
    }

    writer.flush().unwrap();
}
