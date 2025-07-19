use clap::Parser;
use std::fs::OpenOptions;
use tracing::error;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tweers::cli::{Cli, Commands, build_command, pack_command};
use tweers::config::constants;

#[tokio::main]
async fn main() {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(constants::LOG_FILE)
        .expect("Failed to create log file");

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_filter(
                    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
                ),
        )
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_filter(EnvFilter::new("debug")),
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
        Commands::Build {
            watch,
            output_path,
            sources,
            is_debug,
            base64
        } => {
            build_command(sources, output_path, watch, is_debug, base64).await?;
        }
        Commands::Pack {
            sources,
            assets_dirs,
            output_path,
            fast_compression,
            is_debug,
        } => {
            pack_command(
                sources,
                assets_dirs,
                output_path,
                fast_compression,
                is_debug,
            )
            .await?;
        }
    }
    Ok(())
}
