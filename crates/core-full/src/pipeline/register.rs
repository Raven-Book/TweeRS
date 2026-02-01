// Node registration for core-full pipeline nodes
use tweers_core::pipeline::NodeRegistry;

use super::nodes::basic::{
    DataAggregatorNode, FileChangeDetectorNode, FileCollectorNode, FileParserNode, FileWriterNode,
    HtmlGeneratorNode,
};

/// Register all core-full pipeline nodes
pub fn register_nodes(registry: &mut NodeRegistry) {
    // File I/O nodes
    registry.register("file_collector", || Box::new(FileCollectorNode));
    registry.register("file_change_detector", || Box::new(FileChangeDetectorNode));
    registry.register("file_parser", || Box::new(FileParserNode));
    registry.register("file_writer", || Box::new(FileWriterNode));

    // Data processing nodes
    registry.register("data_aggregator", || Box::new(DataAggregatorNode));
    registry.register("html_generator", || Box::new(HtmlGeneratorNode));
}
