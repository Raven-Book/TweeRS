use serde::{Deserialize, Serialize};
use serde_json::Value;
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

    // Serialize specific types
    pub fn serialize_as<T: Serialize + Send + Sync + Any>(&self, key: &str) -> Option<Value> {
        let value = self.get::<T>(key)?;
        serde_json::to_value(value).ok()
    }

    // Deserialize from JSON Value
    pub fn deserialize_from<T: for<'de> Deserialize<'de> + Send + Sync + Any>(
        &mut self,
        key: &str,
        json: Value,
    ) -> Result<(), serde_json::Error> {
        let value: T = serde_json::from_value(json)?;
        self.insert(key, value);
        Ok(())
    }

    // Deserialize from JSON string
    pub fn deserialize_from_str<T: for<'de> Deserialize<'de> + Send + Sync + Any>(
        &mut self,
        key: &str,
        json_str: &str,
    ) -> Result<(), serde_json::Error> {
        let value: T = serde_json::from_str(json_str)?;
        self.insert(key, value);
        Ok(())
    }

    // Insert serializable value
    pub fn insert_serializable<T: Send + Sync + Any + Serialize>(&mut self, key: &str, value: T) {
        self.insert(key, value);
    }

    // Get all keys for debugging
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
}

impl Default for PipeMap {
    fn default() -> Self {
        Self::new()
    }
}

pub mod core;

pub mod nodes;

pub use crate::error::PipelineError;
pub use core::{PipeNode, Pipeline};

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

    #[test]
    fn test_serialization() {
        let mut data = PipeMap::new();

        data.insert("count", 42i32);
        data.insert("name", String::from("测试"));

        let count_json = data.serialize_as::<i32>("count");
        let name_json = data.serialize_as::<String>("name");

        assert!(count_json.is_some());
        assert!(name_json.is_some());

        let wrong_type = data.serialize_as::<f64>("count");
        assert!(wrong_type.is_none());
    }

    #[test]
    fn test_deserialization() {
        let mut data = PipeMap::new();

        data.deserialize_from_str::<i32>("count", "42").unwrap();
        data.deserialize_from_str::<String>("name", "\"测试\"")
            .unwrap();

        assert_eq!(data.get::<i32>("count"), Some(&42));
        assert_eq!(data.get::<String>("name"), Some(&String::from("测试")));

        use serde_json::json;
        let obj_value = json!({"key": "value", "number": 123});
        data.deserialize_from::<Value>("object", obj_value).unwrap();

        let retrieved = data.get::<Value>("object").unwrap();
        assert_eq!(retrieved["key"], "value");
        assert_eq!(retrieved["number"], 123);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut data1 = PipeMap::new();
        data1.insert("count", 42i32);
        data1.insert("name", String::from("测试"));

        // Serialize
        let count_json = data1.serialize_as::<i32>("count").unwrap();
        let name_json = data1.serialize_as::<String>("name").unwrap();

        // Create new PipeMap and deserialize
        let mut data2 = PipeMap::new();
        data2.deserialize_from::<i32>("count", count_json).unwrap();
        data2.deserialize_from::<String>("name", name_json).unwrap();

        // Verify data matches
        assert_eq!(data2.get::<i32>("count"), Some(&42));
        assert_eq!(data2.get::<String>("name"), Some(&String::from("测试")));
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        name: String,
        age: u32,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    }

    #[test]
    fn test_struct_serialization() {
        let mut data = PipeMap::new();

        let test_struct = TestStruct {
            name: "张三".to_string(),
            age: 25,
            tags: vec!["rust".to_string(), "programming".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("role".to_string(), "developer".to_string());
                map.insert("level".to_string(), "senior".to_string());
                map
            },
        };

        data.insert("user", test_struct.clone());

        let json_value = data.serialize_as::<TestStruct>("user").unwrap();

        let mut data2 = PipeMap::new();
        data2
            .deserialize_from::<TestStruct>("user", json_value)
            .unwrap();

        let retrieved = data2.get::<TestStruct>("user").unwrap();
        assert_eq!(retrieved.name, "张三");
        assert_eq!(retrieved.age, 25);
        assert_eq!(retrieved.tags, vec!["rust", "programming"]);
        assert_eq!(
            retrieved.metadata.get("role"),
            Some(&"developer".to_string())
        );
        assert_eq!(retrieved.metadata.get("level"), Some(&"senior".to_string()));

        assert_eq!(retrieved, &test_struct);
    }

    #[test]
    fn test_struct_from_json_string() {
        let mut data = PipeMap::new();

        let json_str = r#"{
            "name": "李四",
            "age": 30,
            "tags": ["backend", "database"],
            "metadata": {
                "department": "engineering",
                "location": "北京"
            }
        }"#;

        data.deserialize_from_str::<TestStruct>("user", json_str)
            .unwrap();

        let user = data.get::<TestStruct>("user").unwrap();
        assert_eq!(user.name, "李四");
        assert_eq!(user.age, 30);
        assert_eq!(user.tags, vec!["backend", "database"]);
        assert_eq!(
            user.metadata.get("department"),
            Some(&"engineering".to_string())
        );
        assert_eq!(user.metadata.get("location"), Some(&"北京".to_string()));
    }
}
