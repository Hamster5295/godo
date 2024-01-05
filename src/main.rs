mod procedure;
mod utils;

use std::fs;

use clap::{Parser, Subcommand};
use console::Style;
use dialoguer::Confirm;
use reqwest::Client;
use utils::get_download_info;

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
        /// The version to install
        version: Option<String>,

        /// Whether to install the Mono version (with C# support)
        #[arg(short, long)]
        mono: bool,
    },
    /// List all the available released versions
    Available {
        /// Whether to list prereleased versions
        #[arg(short, long)]
        prerelease: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match &args.command {
        Command::Install { version, mono } => {
            let version_str: String;
            match version {
                Some(ver) => {
                    version_str = ver.clone();
                }
                None => version_str = "4".to_string(),
            }

            let client = Client::new();

            let cyan = Style::new().cyan().bold().bright();
            let yellow = Style::new().yellow().bold();
            let red = Style::new().red().bold();
            let green = Style::new().green().bold();

            match get_download_info(&client, version_str, mono.to_owned()).await {
                Some((file, url)) => {
                    let proc = &mut procedure::new(4);
                    proc.next("Please confirm your installation:".to_string());
                    let version_name = file.trim();
                    let cur_version = format!(
                        "{} {} {}",
                        "Godot",
                        &version_name,
                        if *mono { "mono" } else { "" }
                    );
                    println!("\t> {} <", cyan.apply_to(&cur_version));
                    if Confirm::new()
                        .with_prompt("Do you want to proceed?")
                        .default(true)
                        .show_default(true)
                        .wait_for_newline(true)
                        .interact()
                        .unwrap()
                    {
                        proc.next(format!("{}", yellow.apply_to("Downloading...")));
                        let path = utils::download(
                            &client,
                            format!("Godot_{}{}", &file, if *mono { "_mono" } else { "" }),
                            url,
                        )
                        .await;
                        proc.finish("Download Completed!".to_string());
                        proc.next(format!("{}", yellow.apply_to("Unzipping...")));
                        utils::unzip(&path);
                        proc.finish("Unzipped!".to_string());
                        proc.next(format!("{}", yellow.apply_to("Clearing cache...")));
                        fs::remove_file(&path).unwrap();
                        proc.finish("Cleared!".to_string());
                        println!(
                            "{} is now {}.",
                            cyan.apply_to(&cur_version),
                            green.apply_to("READY")
                        );
                        println!(
                            "Use {} {} to begin.",
                            yellow.apply_to("godo run"),
                            green.apply_to(&file)
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
        Command::Available { prerelease } => {
            let client = Client::new();
            utils::list_avail(&client, *prerelease).await;
        }
    }
}
