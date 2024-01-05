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
                    let green = Style::new().green().bold();
                    let dim = Style::new().dim().bold();
                    println!(
                        "{} Please confirm your installation: \n\t> {} {} {} <",
                        dim.apply_to("[1/3]"),
                        cyan.apply_to("Godot"),
                        cyan.apply_to(file),
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
                        println!(
                            "{} {}",
                            dim.apply_to("[2/3]"),
                            yellow.apply_to("Downloading...")
                        );
                        utils::download(&client, url).await;
                        println!("{}", green.apply_to("Download completed!"));
                        println!(
                            "{} {}",
                            dim.apply_to("[3/3]"),
                            yellow.apply_to("Unzipping...")
                        );
                    } else {
                        print!("{}", red.apply_to("Installation aborted"))
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
