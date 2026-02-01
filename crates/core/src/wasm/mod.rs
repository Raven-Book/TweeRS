// WASM bindings module - only compiled when wasm feature is enabled

pub mod api;
pub mod types;

use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
