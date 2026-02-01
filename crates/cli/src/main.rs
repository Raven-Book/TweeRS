mod cli;
mod logging;
mod update;

use clap::Parser;
use tracing::error;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{Cli, Commands};
use crate::update::update_command;
use tweers_asset::{ArchiveCreatorNode, AssetCompressorNode};
use tweers_core::config::constants;
use tweers_core_full::commands::{build_command, pack_command_with_nodes};

#[tokio::main]
async fn main() {
    constants::init_constants();

    let log_file = logging::create_log_file().expect("Failed to create log file");

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

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();
    match args.cmd {
        Commands::Build {
            watch,
            output_path,
            sources,
            is_debug,
            base64,
            start_passage,
        } => {
            build_command(sources, output_path, watch, is_debug, base64, start_passage).await?;
        }
        Commands::Pack {
            sources,
            assets_dirs,
            output_path,
            fast_compression,
            is_debug,
        } => {
            let pack_nodes: Vec<Box<dyn tweers_core_full::pipeline::PipeNode + Send + Sync>> =
                vec![Box::new(AssetCompressorNode), Box::new(ArchiveCreatorNode)];

            pack_command_with_nodes(
                sources,
                assets_dirs,
                output_path,
                fast_compression,
                is_debug,
                pack_nodes,
            )
            .await?;
        }
        Commands::Update { force } => {
            update_command(
                "https://api.github.com/repos/Raven-Book/TweeRS/releases/latest".to_string(),
                force,
            )
            .await?;
        }
    }
    Ok(())
}
