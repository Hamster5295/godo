mod procedure;
mod remote;
mod utils;

use std::fs;

use clap::{Parser, Subcommand};
use console::Style;
use dialoguer::Confirm;
use remote::get_download_info;
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

            match get_download_info(&client, version_str, *mono).await {
                Some((tag, url)) => {
                    let proc = &mut procedure::new(4);
                    let cur_version = utils::get_version_name(&tag, mono);

                    // Confirm before download
                    proc.next("Please confirm your installation:".to_string());
                    println!("\t> {} <", cyan.apply_to(&cur_version));
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
                        let path = remote::download(
                            &client,
                            utils::get_file_name(&tag, mono),
                            url,
                        )
                        .await;
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
                            cyan.apply_to(&cur_version),
                            green.apply_to("READY")
                        );
                        println!(
                            "Use {} {} to start.",
                            yellow.apply_to("godo run"),
                            green.apply_to(&tag)
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
            remote::list_avail(&client, *prerelease).await;
        }
    }
}
