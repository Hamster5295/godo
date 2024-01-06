mod procedure;
mod remote;
mod utils;
mod version;

use std::fs;

use clap::{Parser, Subcommand};
use console::Style;
use dialoguer::Confirm;
use remote::search_remote_version;
use reqwest::Client;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Install Godot with optional specific version.
    Install {
        /// The version to install.
        version: Option<String>,

        /// Whether to install the Mono version (with C# support).
        #[arg(short, long)]
        mono: bool,
    },
    /// List available Godot versions.
    Available {
        /// Whether to list prereleased versions
        #[arg(short, long)]
        prerelease: bool,
    },
    /// List installed Godot versions.
    List,
    /// Run Godot with specific version.
    Run {
        /// The version to run. Automaticly runs the latest stable version when not specified.
        version: Option<String>,

        /// Whether to run the Mono version.
        #[arg(short, long)]
        mono: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match &args.command {
        Command::Install { version, mono } => {
            handle_install(version, mono).await;
        }
        Command::Available { prerelease } => handle_available(prerelease).await,
        Command::List => handle_list(),
        Command::Run { version, mono } => handle_run(version, mono),
    }
}

async fn handle_available(prerelease: &bool) {
    let client = Client::new();
    remote::list_avail(&client, *prerelease).await;
}

fn handle_list() {
    let dim = Style::new().dim();
    let yellow = Style::new().yellow().bold();

    println!("{}", yellow.apply_to("Installed"));
    println!("{}", dim.apply_to("=".repeat(15)));

    let installed_dirs = utils::get_installed_dirs();
    for dir in installed_dirs {
        if let Some(ver) = version::parse(dir) {
            println!("{}", ver.short_name());
        }
    }
}

fn handle_run(version: &Option<String>, mono: &bool) {
    if let Some(ver) = utils::search_installed_version(version, *mono) {
        println!("{}", ver.version_name())
    }
}

async fn handle_install(version: &Option<String>, mono: &bool) {
    let cyan = Style::new().cyan().bold().bright();
    let yellow = Style::new().yellow().bold();
    let red = Style::new().red().bold();
    let green = Style::new().green().bold();

    let client = Client::new();
    match search_remote_version(&client, version, *mono).await {
        Some((ver, url)) => {
            let proc = &mut procedure::new(4);
            let version_name = ver.version_name();
            let file_name = ver.dir_name();

            // Confirm before download
            proc.next("Please confirm your installation:".to_string());
            println!("\t> {} <", cyan.apply_to(&version_name));
            if Confirm::new()
                .with_prompt("Do you want to proceed?")
                .default(true)
                .show_default(true)
                .wait_for_newline(true)
                .interact()
                .unwrap()
            {
                // Start download
                proc.next(format!("{}", yellow.apply_to("Downloading...")));
                let path = remote::download(&client, file_name, url).await;
                proc.finish("Download Completed!".to_string());

                // Unzip
                proc.next(format!("{}", yellow.apply_to("Unzipping...")));
                remote::unzip(&path);
                proc.finish("Unzipped!".to_string());

                // Remove the original zipped file
                proc.next(format!("{}", yellow.apply_to("Clearing cache...")));
                fs::remove_file(&path).unwrap();
                proc.finish("Cleared!".to_string());

                // Finished!
                println!(
                    "{} is now {}.",
                    cyan.apply_to(&version_name),
                    green.apply_to("READY")
                );
                println!(
                    "Use {} {} to start.",
                    yellow.apply_to("godo run"),
                    green.apply_to(ver.tag())
                );
            } else {
                println!("{}", red.apply_to("Installation aborted"))
            }
        }
        None => {
            println!(
                "{}\n> Use {} to find another version.",
                red.apply_to("No Suitable Version found."),
                yellow.apply_to("godo available")
            )
        }
    };
}
