mod commands;
mod config;
mod github;
mod version;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "godo")]
#[command(about = "A version manager for Godot Engine", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install Godot Engine with specific version
    Install {
        /// The version to install. Fuzzy matching is supported.
        version: String,

        /// Whether to install Mono version
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        mono: Option<bool>,

        /// Install directly without interactive interface, works only when --mono is specified
        #[arg(long)]
        silent: bool,
    },

    /// Remove a locally installed Godot version
    Rm {
        /// The version to remove. Fuzzy matching is supported.
        version: String,

        /// Whether to remove Mono version
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        mono: Option<bool>,

        /// Remove directly without interactive interface, works only when --mono is specified
        #[arg(long)]
        silent: bool,
    },

    /// List all available Godot versions
    List {
        /// Show pre-release versions (beta, rc, dev, alpha)
        #[arg(long)]
        beta: bool,
    },

    /// Set the current active Godot version
    Current {
        /// The version to set as current. Fuzzy matching is supported.
        version: String,

        /// Whether to select Mono version
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        mono: Option<bool>,

        /// Select directly without interactive interface, works only when --mono is specified
        #[arg(long)]
        silent: bool,
    },

    /// Launch a Godot Engine instance
    Run {
        /// The version to launch. Defaults to current if omitted. Fuzzy matching is supported.
        version: Option<String>,

        /// Whether to launch Mono version
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        mono: Option<bool>,
    },

    /// Update the Godot Engine release manifest manually
    Update,
}

fn main() {
    let cli = Cli::parse();

    let config = match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e}");
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Install {
            version,
            mono,
            silent,
        } => commands::install(&config, &version, mono, silent),
        Commands::Rm {
            version,
            mono,
            silent,
        } => commands::rm(&config, &version, mono, silent),
        Commands::List { beta } => commands::list(&config, beta),
        Commands::Current {
            version,
            mono,
            silent,
        } => commands::current(&config, &version, mono, silent),
        Commands::Run { version, mono } => commands::run(&config, version.as_deref(), mono),
        Commands::Update => commands::update(&config),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "!".red().bold(), e);
        std::process::exit(1);
    }
}
