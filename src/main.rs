use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tweers::config::constants;
use tweers::core::{parser::TweeParser, output::HtmlOutputHandler};

#[derive(Subcommand)]
#[command(version, about, long_about = None)]
enum Commands {
    /// Convert .twee/.tw files to HTML
    Build {
        /// Watch
        #[clap(short)]
        watch: bool,
        /// Output path
        #[clap(short, long, default_value = "index.html")]
        dist: PathBuf,
        /// Sources
        #[arg(trailing_var_arg = true, required = true)]
        sources: Vec<PathBuf>,
    },

    Zip {}
}


/// TweeRS Command
#[derive(Parser)]
#[command(about=None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands
}


fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    constants::init_constants();
    let args = Cli::parse();
    match args.cmd {
        Commands::Build { watch, dist, sources } => {
            build_command(sources, dist, watch)?;
        }
        Commands::Zip {} => {
            info!("Zip command not implemented yet");
        }
    }
    Ok(())
}


fn build_command(sources: Vec<PathBuf>, dist: PathBuf, watch: bool) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}