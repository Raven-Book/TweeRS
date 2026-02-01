// Node registration for JS pipeline nodes
use tweers_core::pipeline::NodeRegistry;

use crate::manager::ScriptManager;
use crate::nodes::{DataProcessorNode, HtmlProcessorNode};

/// Register all JS pipeline nodes
pub fn register_nodes(registry: &mut NodeRegistry, script_manager: ScriptManager) {
    let sm_clone = script_manager.clone();
    registry.register("data_processor", move || {
        let sm = sm_clone.clone();
        Box::new(
            DataProcessorNode::new(sm)
                .expect("Failed to create DataProcessorNode")
        )
    });

    registry.register("html_processor", move || {
        let sm = script_manager.clone();
        Box::new(
            HtmlProcessorNode::new(sm)
                .expect("Failed to create HtmlProcessorNode")
        )
    });
}
