mod cli;
mod logging;
mod update;

use clap::Parser;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::error;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{Cli, Commands};
use crate::update::update_command;
use tweers_asset::{ArchiveCreatorNode, AssetCompressorNode};
use tweers_core::config::constants;
use tweers_core_full::commands::{build_command_with_nodes, pack_command_with_nodes};
use tweers_js::manager::ScriptManager;
use tweers_js::nodes::{DataProcessorNode, HtmlProcessorNode};

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
            let script_manager = ScriptManager::default();
            let mut data_nodes: Vec<Box<dyn tweers_core_full::pipeline::PipeNode + Send + Sync>> =
                vec![];
            let mut html_nodes: Vec<Box<dyn tweers_core_full::pipeline::PipeNode + Send + Sync>> =
                vec![];

            if script_manager.has_data_scripts() {
                data_nodes.push(Box::new(
                    DataProcessorNode::new(script_manager.clone())
                        .expect("Failed to create DataProcessorNode"),
                ));
            }

            if script_manager.has_html_scripts() {
                html_nodes.push(Box::new(
                    HtmlProcessorNode::new(script_manager.clone())
                        .expect("Failed to create HtmlProcessorNode"),
                ));
            }

            build_command_with_nodes(
                sources,
                output_path,
                watch,
                is_debug,
                base64,
                start_passage,
                data_nodes,
                html_nodes,
            )
            .await?;
        }
        Commands::Html2Twee {
            input_path,
            output_path,
        } => {
            if input_path.is_dir() {
                return Err(format!(
                    "Input path is a directory, expected an HTML file: {}",
                    input_path.display()
                )
                .into());
            }

            let output_path = resolve_html2twee_output_path(&input_path, output_path)?;
            let html = std::fs::read_to_string(&input_path)?;
            let twee = tweers_core::api::html_to_twee(&html)?;

            if output_path.exists() && !confirm_overwrite(&output_path)? {
                println!("Canceled. Output was not overwritten.");
                return Ok(());
            }

            if let Some(parent) = output_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, twee)?;
            println!("Output written to: {}", output_path.display());
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

fn resolve_html2twee_output_path(
    input_path: &Path,
    output_path: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    match output_path {
        Some(path) => {
            if path.is_dir() {
                let file_name = default_html2twee_file_name(input_path)?;
                Ok(path.join(file_name))
            } else {
                Ok(path)
            }
        }
        None => Ok(default_html2twee_output_path(input_path)?),
    }
}

fn default_html2twee_output_path(
    input_path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parent.join(default_html2twee_file_name(input_path)?))
}

fn default_html2twee_file_name(
    input_path: &Path,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let stem = input_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            format!(
                "Failed to derive output file name from {}",
                input_path.display()
            )
        })?;

    Ok(format!("{stem}.twee"))
}

fn confirm_overwrite(path: &Path) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        print!(
            "Output already exists: {}. Overwrite? [Y/N]: ",
            path.display()
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        match response.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("Please enter Y or N.");
            }
        }
    }
}
