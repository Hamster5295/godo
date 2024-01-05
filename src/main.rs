mod procedure;
mod utils;
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
            match get_download_info(&client, version_str, mono.to_owned()).await {
                Some((file, url)) => {
                    let cyan = Style::new().cyan().bold().bright();
                    let yellow = Style::new().yellow().bold();
                    let red = Style::new().red().bold();

                    let proc = &mut procedure::new(3);
                    proc.next("Please confirm your installation:".to_string());
                    println!(
                        "\t> {} {} {} <",
                        cyan.apply_to("Godot"),
                        cyan.apply_to(&file),
                        cyan.apply_to(if *mono { "mono" } else { "" })
                    );
                    if Confirm::new()
                        .with_prompt("Do you want to proceed?")
                        .default(true)
                        .show_default(true)
                        .wait_for_newline(true)
                        .interact()
                        .unwrap()
                    {
                        proc.next(format!("{}", yellow.apply_to("Downloading...")));
                        utils::download(
                            &client,
                            format!("Godot_{}{}", &file, if *mono { "_mono" } else { "" }),
                            url,
                        )
                        .await;
                        proc.finish("Download Completed!".to_string());
                        proc.next(format!("{}", yellow.apply_to("Unzipping...")));
                        proc.finish("Unzipped!".to_string());
                    } else {
                        println!("{}", red.apply_to("Installation aborted"))
                    }
                }
                None => {
                    println!("No suitable version found for your system.")
                }
            };
        }
        Command::Available { prerelease } => {
            let client = Client::new();
            utils::list_avail(&client, *prerelease).await;
        }
    }
}
