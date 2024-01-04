use std::{
    env::consts,
    fs,
    io::{self, Write},
    iter,
};

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

    if prerelease {
        writer
            .write(format!("{:15}{:8}\n", "Name", "Prelease").as_bytes())
            .unwrap();
        writer
            .write((iter::repeat("=").take(23).collect::<String>() + "\n").as_bytes())
            .unwrap();
        for ver in versions {
            writer
                .write(format!("{:15}{:7}\n", ver.tag_name, ver.prerelease).as_bytes())
                .unwrap();
        }
    } else {
        writer.write(format!("{:15}\n", "Name").as_bytes()).unwrap();
        writer
            .write((iter::repeat("=").take(15).collect::<String>() + "\n").as_bytes())
            .unwrap();
        for ver in versions {
            writer
                .write(format!("{:15}\n", ver.tag_name).as_bytes())
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

pub async fn download(client: &Client, url: String) {
    let mut result = client
        .get(url)
        .header("User-Agent", "Hamster5295")
        .header("Accept", "application/vnd.github.full+json")
        .send()
        .await
        .unwrap_or_else(|err| panic!("Unable to fetch from github: {:?}", err));

    let mut writer = io::BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open("godot.zip")
            .unwrap(),
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
