// Node registration for JS pipeline nodes
use tweers_core::pipeline::NodeRegistry;

use crate::manager::ScriptManager;
use crate::nodes::DataProcessorNode;

/// Register all JS pipeline nodes
pub fn register_nodes(registry: &mut NodeRegistry, script_manager: ScriptManager) {
    registry.register("data_processor", move || {
        // Clone the script_manager for each node instance
        let sm = script_manager.clone();
        Box::new(
            DataProcessorNode::new(sm)
                .expect("Failed to create DataProcessorNode")
        )
    });
}
