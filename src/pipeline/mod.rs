use std::any::Any;
use std::collections::HashMap;

pub struct PipeMap {
    data: HashMap<String, Box<dyn Send + Sync + Any>>,
}

impl PipeMap {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + Any>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: Send + Sync + Any>(&self, key: &str) -> Option<&T> {
        self.data.get(key)?.downcast_ref::<T>()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

impl Default for PipeMap {
    fn default() -> Self {
        Self::new()
    }
}

pub mod core;
pub mod error;
pub mod nodes;

// Re-export commonly used types
pub use core::{PipeNode, Pipeline};
pub use error::PipelineError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_data() {
        let mut data = PipeMap::new();

        data.insert("count", 42i32);
        data.insert("name", String::from("测试"));
        data.insert("files", vec!["a.txt", "b.txt"]);

        assert_eq!(data.get::<i32>("count"), Some(&42));
        assert_eq!(data.get::<String>("name"), Some(&String::from("测试")));
        assert_eq!(
            data.get::<Vec<&str>>("files"),
            Some(&vec!["a.txt", "b.txt"])
        );

        assert_eq!(data.get::<f64>("count"), None);
        assert_eq!(data.get::<i32>("nonexistent"), None);
    }
}
