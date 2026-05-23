use anyhow::{bail, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::github;
use crate::version::{GodotVersion, VersionQuery};

pub fn install(config: &Config, version: &str, mono: Option<bool>, silent: bool) -> Result<()> {
    let query = VersionQuery::from_input(version).context("Invalid version format")?;

    let mono_flag_provided = mono.is_some();
    let mono = match mono {
        Some(m) => m,
        None => ask_mono()?,
    };

    println!("{}", "Fetching releases...".dimmed());
    let releases = github::fetch_releases_cached(config)?;
    let release = github::find_matching_release(&releases, &query)?;
    let ver = GodotVersion::from_tag(&release.tag_name).context("Failed to parse release tag")?;
    let ver_mono = GodotVersion {
        mono,
        ..ver.clone()
    };

    let need_confirm = !silent || !mono_flag_provided;
    if need_confirm {
        println!("  Found version: {}", format!("{ver_mono}").green().bold());
        if !ask_yes_no("Install this version?")? {
            println!("{}", "Installation cancelled.".yellow());
            return Ok(());
        }
    }

    let assets = github::find_platform_assets(&release.assets, mono)?;

    let version_dir = config.engine_dir.join(ver_mono.folder_name());
    if version_dir.exists() {
        bail!(
            "Version {} is already installed at {}",
            ver_mono,
            version_dir.display()
        );
    }

    std::fs::create_dir_all(&config.temp_dir).context("Failed to create temp directory")?;
    std::fs::create_dir_all(&config.engine_dir).context("Failed to create engine directory")?;

    let main_zip = download_with_progress(
        &assets.main_asset.browser_download_url,
        &config.temp_dir.join(&assets.main_asset.name),
        assets.main_asset.size,
    )?;

    let console_zip = if let Some(console_asset) = assets.console_asset {
        Some(download_with_progress(
            &console_asset.browser_download_url,
            &config.temp_dir.join(&console_asset.name),
            console_asset.size,
        )?)
    } else {
        None
    };

    println!("{}", "Extracting...".dimmed());
    extract_zip_strip_prefix(&main_zip, &version_dir)?;

    if let Some(console_zip) = console_zip {
        extract_zip_merge(&console_zip, &version_dir)?;
    }

    rename_executables(&version_dir)?;

    cleanup_temp(&config.temp_dir, &assets.main_asset.name);
    if let Some(ca) = assets.console_asset {
        cleanup_temp(&config.temp_dir, &ca.name);
    }

    println!(
        "  {} {}",
        "✓".green(),
        format!("Installed {ver_mono}").green().bold()
    );

    let installed = get_installed_versions(config)?;
    if installed.len() > 1 {
        if ask_yes_no("Set this version as current?")? {
            update_current_symlink(config, &ver_mono.folder_name())?;
            println!(
                "  {} Current version set to {}",
                "✓".green(),
                ver_mono.to_string().green().bold()
            );
        }
    } else {
        update_current_symlink(config, &ver_mono.folder_name())?;
        println!(
            "  {} Current version set to {}",
            "✓".green(),
            ver_mono.to_string().green().bold()
        );
    }

    Ok(())
}

pub fn rm(config: &Config, version: &str, mono: Option<bool>, silent: bool) -> Result<()> {
    let query = VersionQuery::from_input(version).context("Invalid version format")?;

    let installed = get_installed_versions(config)?;
    if installed.is_empty() {
        bail!("No Godot versions installed");
    }

    let mono_flag_provided = mono.is_some();
    let matched: Vec<GodotVersion> = installed
        .iter()
        .filter(|v| query.matches_loose(v))
        .cloned()
        .collect();

    if matched.is_empty() {
        bail!("No matching installed version found for '{version}'");
    }

    let target = if matched.len() == 1 {
        &matched[0]
    } else {
        let latest = matched.iter().max().unwrap();
        latest
    };

    let mono = match mono {
        Some(m) => m,
        None => {
            let has_mono = matched.iter().any(|v| v.mono);
            let has_non_mono = matched.iter().any(|v| !v.mono);
            if has_mono && has_non_mono {
                ask_mono()?
            } else {
                target.mono
            }
        }
    };

    let target = GodotVersion {
        mono,
        ..target.clone()
    };

    let need_confirm = !silent || !mono_flag_provided;
    if need_confirm {
        println!("  Will remove: {}", format!("{target}").red().bold());
        if !ask_yes_no("Continue?")? {
            println!("{}", "Removal cancelled.".yellow());
            return Ok(());
        }
    }

    let version_dir = config.engine_dir.join(target.folder_name());
    if !version_dir.exists() {
        bail!("Version directory not found: {}", version_dir.display());
    }

    let current_target = read_current_link(config);
    let is_current = current_target.as_deref() == Some(target.folder_name().as_str());

    std::fs::remove_dir_all(&version_dir).context("Failed to remove version directory")?;

    println!(
        "  {} Removed {}",
        "✓".green(),
        target.to_string().green().bold()
    );

    if is_current {
        println!(
            "  {} Current version was pointing to the removed version.",
            "!".yellow()
        );
        let remaining = get_installed_versions(config)?;
        if let Some(latest) = remaining.iter().max() {
            update_current_symlink(config, &latest.folder_name())?;
            println!(
                "  {} Current version updated to {}",
                "✓".green(),
                latest.to_string().green().bold()
            );
        } else {
            let current_link = config.current_link_path();
            if current_link.exists() || current_link.symlink_metadata().is_ok() {
                remove_symlink(&current_link)?;
            }
            println!(
                "  {} No versions installed, removed current link",
                "!".yellow()
            );
        }
    }

    Ok(())
}

pub fn list(config: &Config, beta: bool) -> Result<()> {
    println!("{}", "Fetching releases...".dimmed());
    let releases = match github::fetch_releases_cached(config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} Failed to fetch releases: {e}", "!".yellow());
            Vec::new()
        }
    };

    let installed = get_installed_versions(config)?;
    let installed_folders: Vec<String> = installed.iter().map(|v| v.folder_name()).collect();

    let current_folder = read_current_link(config);

    if releases.is_empty() && installed.is_empty() {
        println!("No Godot versions available.");
        return Ok(());
    }

    let mut versions: Vec<GodotVersion> = releases
        .iter()
        .filter_map(|r| GodotVersion::from_tag(&r.tag_name))
        .collect();
    versions.sort_by(|a, b| a.cmp(b));
    versions.dedup();

    let filtered: Vec<GodotVersion> = versions
        .into_iter()
        .filter(|ver| beta || ver.is_stable() || installed_folders.contains(&ver.folder_name()))
        .collect();

    let mut groups: std::collections::BTreeMap<u32, Vec<GodotVersion>> =
        std::collections::BTreeMap::new();
    for ver in filtered {
        groups.entry(ver.major).or_default().push(ver);
    }

    // Reverse BTreeMap order so newest major comes first
    let groups: Vec<(u32, Vec<GodotVersion>)> = groups.into_iter().rev().collect();

    let col_gap = 2;
    // Compute column widths based on visible char width (excluding ANSI escapes)
    let col_widths: Vec<usize> = groups
        .iter()
        .map(|(major, vers)| {
            let header_len = format!("Godot {major}").len();
            let max_ver_len = vers
                .iter()
                .map(|v| {
                    let base = v.to_string().len(); // visible chars, no ANSI
                    let bullet_and_space = 2; // "● " or "○ "
                    let current_extra = if current_folder.as_deref()
                        == Some(v.folder_name().as_str())
                        && installed_folders.contains(&v.folder_name())
                    {
                        10 // " (current)"
                    } else {
                        0
                    };
                    bullet_and_space + base + current_extra
                })
                .max()
                .unwrap_or(0);
            std::cmp::max(header_len, max_ver_len) + col_gap
        })
        .collect();

    // Print rows row-by-row across all columns, bottom-aligned
    let max_rows = groups.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    for row in 0..max_rows {
        let mut cells = Vec::new();
        for ((_, vers), col_w) in groups.iter().zip(col_widths.iter()) {
            let offset = max_rows - vers.len();
            let data_row: Option<usize> = row.checked_sub(offset);

            if let Some(idx) = data_row {
                let ver = &vers[idx];
                let is_installed = installed_folders.contains(&ver.folder_name());
                let is_current = current_folder.as_deref() == Some(ver.folder_name().as_str());

                let visible_len;
                let text = if is_installed {
                    let current_marker = if is_current { " (current)" } else { "" };
                    visible_len = 2 + ver.to_string().len() + current_marker.len();
                    format!(
                        "{} {}{}",
                        "●".green(),
                        ver.to_string().green().bold(),
                        current_marker.green()
                    )
                } else {
                    visible_len = 2 + ver.to_string().len();
                    format!("{} {}", "○".dimmed(), ver.to_string().dimmed())
                };
                let padding = *col_w - visible_len;
                cells.push(format!("{text}{:padding$}", ""));
            } else {
                cells.push(format!("{:width$}", "", width = *col_w));
            }
        }
        println!("{}", cells.join(""));
    }

    // Print header at the bottom
    let sep_row = col_widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<String>>()
        .join("");
    println!("{sep_row}");

    let header_row = groups
        .iter()
        .zip(col_widths.iter())
        .map(|((major, _), w)| {
            let header = format!("Godot {major}");
            let padding = *w - header.len();
            format!("{}{:padding$}", header.bold(), "")
        })
        .collect::<Vec<String>>()
        .join("");
    println!("{header_row}");

    if !installed.is_empty() {
        println!(
            "\n  {} installed version(s)",
            installed.len().to_string().green().bold()
        );
    }

    Ok(())
}

pub fn current(config: &Config, version: &str, mono: Option<bool>, silent: bool) -> Result<()> {
    let query = VersionQuery::from_input(version).context("Invalid version format")?;

    let installed = get_installed_versions(config)?;
    if installed.is_empty() {
        bail!("No Godot versions installed");
    }

    let matched: Vec<GodotVersion> = installed
        .iter()
        .filter(|v| query.matches_loose(v))
        .cloned()
        .collect();

    if matched.is_empty() {
        bail!("No matching installed version found for '{version}'");
    }

    let target = matched.iter().max().unwrap();

    let mono_flag_provided = mono.is_some();
    let mono = match mono {
        Some(m) => m,
        None => {
            let has_mono = matched.iter().any(|v| v.mono);
            let has_non_mono = matched.iter().any(|v| !v.mono);
            if has_mono && has_non_mono {
                ask_mono()?
            } else {
                target.mono
            }
        }
    };

    let target = GodotVersion {
        mono,
        ..target.clone()
    };

    let need_confirm = !silent || !mono_flag_provided;
    if need_confirm {
        println!(
            "  Set current version to: {}",
            format!("{target}").green().bold()
        );
        if !ask_yes_no("Continue?")? {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    update_current_symlink(config, &target.folder_name())?;
    println!(
        "  {} Current version set to {}",
        "✓".green(),
        target.to_string().green().bold()
    );

    Ok(())
}

pub fn run(config: &Config, version: Option<&str>, mono: Option<bool>) -> Result<()> {
    let target = if let Some(ver) = version {
        let query = VersionQuery::from_input(ver).context("Invalid version format")?;
        let installed = get_installed_versions(config)?;
        if installed.is_empty() {
            bail!("No Godot versions installed");
        }

        let matched: Vec<GodotVersion> = installed
            .iter()
            .filter(|v| query.matches_loose(v))
            .cloned()
            .collect();

        if matched.is_empty() {
            bail!("No matching installed version found for '{ver}'");
        }

        let target = matched.iter().max().unwrap().clone();

        let mono = match mono {
            Some(m) => m,
            None => {
                let has_mono = matched.iter().any(|v| v.mono);
                let has_non_mono = matched.iter().any(|v| !v.mono);
                if has_mono && has_non_mono {
                    ask_mono()?
                } else {
                    target.mono
                }
            }
        };

        GodotVersion { mono, ..target }
    } else {
        let current_folder = read_current_link(config);
        let folder = current_folder
            .context("No current version set. Run 'godo current <version>' first.")?;
        GodotVersion::from_folder(&folder).context("Failed to parse current version")?
    };

    let version_dir = config.engine_dir.join(target.folder_name());
    if !version_dir.exists() {
        bail!("Version {} is not installed", target);
    }

    let executable = find_godot_executable(&version_dir)?;

    println!(
        "{} {}",
        "Launching".dimmed(),
        target.to_string().green().bold()
    );

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        std::process::Command::new(&executable)
            .process_group(0)
            .current_dir(&version_dir)
            .spawn()
            .context("Failed to launch Godot")?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;
        std::process::Command::new(&executable)
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .current_dir(&version_dir)
            .spawn()
            .context("Failed to launch Godot")?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        std::process::Command::new(&executable)
            .current_dir(&version_dir)
            .spawn()
            .context("Failed to launch Godot")?;
    }

    Ok(())
}

fn ask_yes_no(prompt: &str) -> Result<bool> {
    print!("  {prompt} [Y/n] ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    Ok(answer.len() == 0 || answer == "y" || answer == "yes")
}

fn ask_mono() -> Result<bool> {
    ask_yes_no("Install mono version?")
}

pub fn get_installed_versions(config: &Config) -> Result<Vec<GodotVersion>> {
    let mut versions = Vec::new();
    if !config.engine_dir.exists() {
        return Ok(versions);
    }

    for entry in std::fs::read_dir(&config.engine_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "current" {
            continue;
        }
        if let Some(ver) = GodotVersion::from_folder(&name) {
            if entry.path().is_dir() {
                versions.push(ver);
            }
        }
    }

    versions.sort_by(|a, b| b.cmp(a));
    Ok(versions)
}

pub fn read_current_link(config: &Config) -> Option<String> {
    let link_path = config.current_link_path();
    let folder_name = link_path
        .symlink_metadata()
        .ok()
        .and_then(|meta| {
            if meta.file_type().is_symlink() {
                std::fs::read_link(&link_path).ok()
            } else {
                None
            }
        })
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .or_else(|| {
            if link_path.exists() {
                std::fs::read_to_string(&link_path).ok()
            } else {
                None
            }
        });
    folder_name
}

fn update_current_symlink(config: &Config, folder_name: &str) -> Result<()> {
    let link_path = config.current_link_path();
    let target_path = config.engine_dir.join(folder_name);

    if link_path.symlink_metadata().is_ok() {
        remove_symlink(&link_path)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target_path, &link_path)
            .context("Failed to create current symlink")?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(&target_path, &link_path).or_else(|_| {
            std::fs::write(&link_path, &folder_name)
                .context("Failed to create current link (symlink and fallback both failed)")
        })?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        std::fs::write(&link_path, &folder_name).context("Failed to create current link")?;
    }

    Ok(())
}

fn remove_symlink(path: &Path) -> Result<()> {
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() || meta.is_dir() {
        #[cfg(unix)]
        {
            std::fs::remove_file(path)?;
        }
        #[cfg(windows)]
        {
            if meta.file_type().is_symlink() {
                std::fs::remove_dir(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            std::fs::remove_file(path)?;
        }
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn download_with_progress(url: &str, dest: &Path, expected_size: u64) -> Result<PathBuf> {
    println!(
        "  {} {}",
        "↓".dimmed(),
        url.split('/').last().unwrap_or("file")
    );

    let response = ureq::AgentBuilder::new()
        .user_agent("godo - Godot Version Manager")
        .build()
        .get(url)
        .call()
        .context("Failed to download file")?;

    let total_size = if expected_size > 0 {
        expected_size
    } else {
        response
            .header("Content-Length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    };

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(dest).context("Failed to create temp file")?;
    let mut buf = [0u8; 8192];
    let mut downloaded: u64 = 0;

    loop {
        let n = reader
            .read(&mut buf)
            .context("Failed to read download stream")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .context("Failed to write to temp file")?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();

    Ok(dest.to_path_buf())
}

fn extract_zip_strip_prefix(zip_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path).context("Failed to open zip file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    std::fs::create_dir_all(dest).context("Failed to create destination directory")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };

        let stripped: std::path::PathBuf = entry_path.iter().skip(1).collect();
        if stripped.as_os_str().is_empty() {
            continue;
        }

        let outpath = dest.join(&stripped);

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

fn extract_zip_merge(zip_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path).context("Failed to open zip file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };

        let stripped: std::path::PathBuf = entry_path.iter().skip(1).collect();
        if stripped.as_os_str().is_empty() {
            continue;
        }

        let outpath = dest.join(&stripped);

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

fn rename_executables(dir: &Path) -> Result<()> {
    let exe_ext = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    let godot_name = format!("godot{exe_ext}");
    let console_name = format!("godot-console{exe_ext}");

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        if name.contains("console") && !name.starts_with("godot") {
            let new_path = dir.join(&console_name);
            std::fs::rename(&path, &new_path)?;
        } else if !name.contains("console")
            && !name.starts_with("godot")
            && !name.contains("godotsharp")
            && !name.ends_with(".dll")
            && !name.ends_with(".so")
            && !name.ends_with(".dylib")
        {
            let new_path = dir.join(&godot_name);
            std::fs::rename(&path, &new_path)?;
        }
    }

    Ok(())
}

fn find_godot_executable(version_dir: &Path) -> Result<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let exe_name = "godot.exe";
    #[cfg(not(target_os = "windows"))]
    let exe_name = "godot";

    let direct = version_dir.join(exe_name);
    if direct.exists() {
        return Ok(direct);
    }

    let app_path = version_dir
        .join("Godot.app")
        .join("Contents")
        .join("MacOS")
        .join("Godot");
    if app_path.exists() {
        return Ok(app_path);
    }

    for entry in std::fs::read_dir(version_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        if lower.starts_with("godot") && !lower.contains("console") && entry.path().is_file() {
            return Ok(entry.path());
        }
    }

    bail!(
        "Could not find Godot executable in {}",
        version_dir.display()
    );
}

fn cleanup_temp(temp_dir: &Path, filename: &str) {
    let path = temp_dir.join(filename);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn update(config: &Config) -> Result<()> {
    println!("{}", "Updating manifest from GitHub...".dimmed());
    let releases = github::fetch_releases_remote(config.github_token.as_deref())?;
    println!("  {} Fetched {} releases", "✓".green(), releases.len());
    Ok(())
}
