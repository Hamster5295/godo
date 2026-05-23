use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::version::{GodotVersion, VersionQuery};

const GITHUB_API_GODOT: &str = "https://api.github.com/repos/godotengine/godot/releases";
const GITHUB_API_BUILDS: &str = "https://api.github.com/repos/godotengine/godot-builds/releases";

#[derive(Debug, Deserialize, Serialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub fn fetch_releases_cached(config: &crate::config::Config) -> Result<Vec<GithubRelease>> {
    let manifest_path = crate::config::Config::manifest_path();

    if manifest_path.exists() {
        let modified = std::fs::metadata(&manifest_path)?
            .modified()
            .context("Failed to get manifest modification time")?;
        let elapsed = modified.elapsed().unwrap_or(std::time::Duration::MAX);
        if elapsed.as_secs() < config.invalidate_time {
            let content =
                std::fs::read_to_string(&manifest_path).context("Failed to read manifest cache")?;
            let releases: Vec<GithubRelease> =
                serde_json::from_str(&content).context("Failed to parse manifest cache")?;
            return Ok(releases);
        }
    }

    let releases = fetch_releases_remote(config.github_token.as_deref())?;
    save_manifest(&releases)?;
    Ok(releases)
}

pub fn fetch_releases_remote(token: Option<&str>) -> Result<Vec<GithubRelease>> {
    let godot_releases = fetch_releases_from(GITHUB_API_GODOT, token)?;
    let builds_releases = fetch_releases_from(GITHUB_API_BUILDS, token)?;

    let mut seen = std::collections::HashSet::new();
    let mut all_releases = Vec::new();

    for r in godot_releases.into_iter().chain(builds_releases) {
        if seen.insert(r.tag_name.clone()) {
            all_releases.push(r);
        }
    }

    Ok(all_releases)
}

fn save_manifest(releases: &[GithubRelease]) -> Result<()> {
    let manifest_path = crate::config::Config::manifest_path();
    let content = serde_json::to_string(releases).context("Failed to serialize manifest")?;
    std::fs::write(&manifest_path, content).context("Failed to write manifest cache")?;
    Ok(())
}

fn fetch_releases_from(base_url: &str, token: Option<&str>) -> Result<Vec<GithubRelease>> {
    let mut all_releases = Vec::new();
    let mut page = 1;

    let agent = ureq::agent();

    loop {
        let url = format!("{base_url}?per_page=100&page={page}");
        let mut request = agent.get(&url);

        if let Some(t) = token {
            request = request.header("Authorization", &format!("Bearer {t}"));
        }

        let mut response = request
            .call()
            .context("Failed to fetch releases from GitHub")?;

        let body = response.body_mut().read_to_string()
            .context("Failed to read response body")?;
        let releases: Vec<GithubRelease> =
            serde_json::from_str(&body).context("Failed to parse GitHub response")?;

        if releases.is_empty() {
            break;
        }

        let count = releases.len();
        all_releases.extend(releases);

        if count < 100 {
            break;
        }
        page += 1;
    }

    Ok(all_releases)
}

pub fn find_matching_release<'a>(
    releases: &'a [GithubRelease],
    query: &VersionQuery,
) -> Result<&'a GithubRelease> {
    let mut matched: Vec<&GithubRelease> = Vec::new();

    for release in releases {
        if let Some(ver) = GodotVersion::from_tag(&release.tag_name) {
            if query.matches_loose(&ver) {
                matched.push(release);
            }
        }
    }

    if matched.is_empty() {
        bail!("No matching release found for version query");
    }

    matched.sort_by(|a, b| {
        let va = GodotVersion::from_tag(&a.tag_name).unwrap();
        let vb = GodotVersion::from_tag(&b.tag_name).unwrap();
        vb.cmp(&va)
    });

    if query.pre.is_none() {
        let stable = matched.iter().find(|r| {
            GodotVersion::from_tag(&r.tag_name)
                .map(|v| v.is_stable())
                .unwrap_or(false)
        });
        if let Some(release) = stable {
            return Ok(release);
        }
    }

    Ok(matched[0])
}

#[derive(Debug)]
pub struct PlatformAssets<'a> {
    pub main_asset: &'a GithubAsset,
    pub console_asset: Option<&'a GithubAsset>,
}

pub fn find_platform_assets<'a>(
    assets: &'a [GithubAsset],
    mono: bool,
) -> Result<PlatformAssets<'a>> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let main_asset = assets
        .iter()
        .find(|a| is_main_platform_asset(&a.name, mono, os, arch))
        .context(format!(
            "No matching asset found for platform {os}-{arch} (mono={mono})"
        ))?;

    let console_asset = if os == "windows" {
        assets
            .iter()
            .find(|a| is_console_platform_asset(&a.name, mono, os, arch))
    } else {
        None
    };

    Ok(PlatformAssets {
        main_asset,
        console_asset,
    })
}

fn is_main_platform_asset(name: &str, mono: bool, os: &str, arch: &str) -> bool {
    let lower = name.to_lowercase();
    let is_mono = lower.contains("mono");
    if is_mono != mono {
        return false;
    }
    if lower.contains("console") {
        return false;
    }
    if lower.contains("export") || lower.contains("templates") || lower.contains("source") {
        return false;
    }
    if lower.contains("debug_symbols") || lower.contains("sha256") || lower.contains("md5") {
        return false;
    }

    match os {
        "linux" => {
            let arch_match = match arch {
                "x86_64" => {
                    lower.contains("x86_64") || lower.contains("x64") || lower.contains(".64")
                }
                "aarch64" => lower.contains("arm64") || lower.contains("aarch64"),
                _ => false,
            };
            lower.contains("linux") && arch_match
        }
        "macos" => {
            let os_match = lower.contains("macos") || lower.contains("osx");
            let arch_match = lower.contains("universal")
                || match arch {
                    "x86_64" => {
                        lower.contains("x86_64") || lower.contains("x64") || lower.contains(".64")
                    }
                    "aarch64" => lower.contains("arm64") || lower.contains("aarch64"),
                    _ => false,
                };
            os_match && arch_match
        }
        "windows" => {
            let arch_match = match arch {
                "x86_64" => {
                    (lower.contains("win64") || lower.contains("win64")) && !lower.contains("arm64")
                }
                "aarch64" => lower.contains("arm64"),
                _ => false,
            };
            lower.contains("win") && arch_match
        }
        _ => false,
    }
}

fn is_console_platform_asset(name: &str, mono: bool, os: &str, arch: &str) -> bool {
    let lower = name.to_lowercase();
    let is_mono = lower.contains("mono");
    if is_mono != mono {
        return false;
    }
    if !lower.contains("console") {
        return false;
    }
    if lower.contains("export") || lower.contains("templates") || lower.contains("source") {
        return false;
    }

    match os {
        "windows" => {
            let arch_match = match arch {
                "x86_64" => lower.contains("win64") && !lower.contains("arm64"),
                "aarch64" => lower.contains("arm64"),
                _ => false,
            };
            lower.contains("win") && arch_match
        }
        _ => false,
    }
}
