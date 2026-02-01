use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
#[command(version, about, long_about = None)]
pub enum Commands {
    /// Convert .twee/.tw files to HTML
    Build {
        /// Sources
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        /// Watch
        #[clap(short, long)]
        watch: bool,
        /// Output path
        #[clap(short = 'o', long, default_value = "index.html")]
        output_path: PathBuf,
        /// Debug mode
        #[clap(short = 't', long)]
        is_debug: bool,
        /// Convert images to Base64 fragments
        #[clap(short, long)]
        base64: bool,
        /// Start passage name
        #[clap(short = 's', long)]
        start_passage: Option<String>,
    },

    /// Build and pack with compressed assets
    Pack {
        /// Sources
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        /// Assets directories to compress
        #[clap(short = 'a', long = "assets")]
        assets_dirs: Vec<PathBuf>,
        /// Output archive path
        #[clap(short = 'o', long, default_value = "package.zip")]
        output_path: PathBuf,
        /// Enable fast compression (lower quality, faster speed)
        #[clap(short = 'f', long)]
        fast_compression: bool,
        /// Debug mode
        #[clap(short = 't', long)]
        is_debug: bool,
    },

    /// Update TweeRS to the latest release
    Update {
        /// Force update even if already latest version
        #[clap(short = 'f', long)]
        force: bool,
    },
}

/// TweeRS Command
#[derive(Parser)]
#[command(about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}
