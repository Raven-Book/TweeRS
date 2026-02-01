// Node registration for asset pipeline nodes
use tweers_core::pipeline::NodeRegistry;

use crate::nodes::{ArchiveCreatorNode, AssetCompressorNode};

/// Register all asset pipeline nodes
pub fn register_nodes(registry: &mut NodeRegistry) {
    registry.register("asset_compressor", || Box::new(AssetCompressorNode));
    registry.register("archive_creator", || Box::new(ArchiveCreatorNode));
}
