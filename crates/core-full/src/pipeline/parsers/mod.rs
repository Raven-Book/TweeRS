pub mod excel;
pub mod media;
pub mod registry;
pub mod text;
/// File parser module
pub mod r#trait;
pub mod twee;

pub use r#trait::FileParser;
pub use registry::FileParserRegistry;
