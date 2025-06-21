use tweers::cli::{build_command, Cli, Commands};
use tracing::{error, info};
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tweers::config::constants;
use std::fs::OpenOptions;

#[tokio::main]
async fn main() {
    
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(constants::LOG_FILE)
        .expect("Failed to create log file");

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| EnvFilter::new("info"))
                )
        )
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_filter(EnvFilter::new("debug"))
        )
        .init();

    if let Err(e) = run().await {
        error!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    constants::init_constants();
    let args = Cli::parse();
    match args.cmd {
        Commands::Build { watch, dist, sources } => {
            build_command(sources, dist, watch).await?;
        }
        Commands::Zip {} => {
            info!("Zip command not implemented yet");
        }
    }
    Ok(())
}

