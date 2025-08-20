/// Excel data type handling module
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Int,
    Float,
    Number,
    String,
    Bool,
    Boolean,
    Object,
    Array(Box<DataType>),
    Unknown(String),
}

impl DataType {
    /// Parse type definition string into DataType enum
    pub fn parse(type_str: &str) -> Self {
        let trimmed = type_str.trim().to_lowercase();

        match trimmed.as_str() {
            "int" | "integer" => DataType::Int,
            "float" | "double" => DataType::Float,
            "number" => DataType::Number,
            "string" | "str" => DataType::String,
            "bool" | "boolean" => DataType::Bool,
            "object" | "obj" => DataType::Object,
            _ => {
                if trimmed.starts_with("array<") && trimmed.ends_with(">") {
                    let inner_type = &trimmed[6..trimmed.len() - 1];
                    let element_type = DataType::parse(inner_type);
                    DataType::Array(Box::new(element_type))
                } else {
                    DataType::Unknown(type_str.to_string())
                }
            }
        }
    }

    /// Get the element type for array types
    pub fn get_array_element_type(&self) -> DataType {
        match self {
            DataType::Array(element_type) => *element_type.clone(),
            _ => DataType::String,
        }
    }

    /// Format a value according to its type for JavaScript output
    pub fn format_value(&self, value: &str) -> String {
        if value.trim().is_empty() || value.trim().to_lowercase() == "null" {
            return "null".to_string();
        }

        match self {
            DataType::Int | DataType::Float | DataType::Number => {
                if value.parse::<f64>().is_ok() {
                    value.to_string()
                } else {
                    format!("\"{}\"", value.replace('"', "\\\""))
                }
            }
            DataType::Bool | DataType::Boolean => match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => "true".to_string(),
                "false" | "0" | "no" | "off" => "false".to_string(),
                _ => format!("\"{}\"", value.replace('"', "\\\"")),
            },
            DataType::Object => {
                if (value.starts_with('{') && value.ends_with('}'))
                    || (value.starts_with('[') && value.ends_with(']'))
                {
                    value.to_string()
                } else {
                    format!("\"{}\"", value.replace('"', "\\\""))
                }
            }
            DataType::Array(element_type) => {
                if value.starts_with('[') && value.ends_with(']') {
                    let inner = &value[1..value.len() - 1];
                    if inner.is_empty() {
                        return "[]".to_string();
                    }

                    let elements: Vec<String> = inner
                        .split(',')
                        .map(|item| {
                            let trimmed = item.trim();
                            // Remove surrounding quotes if present before formatting
                            let unquoted = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                                &trimmed[1..trimmed.len() - 1]
                            } else {
                                trimmed
                            };
                            element_type.format_value(unquoted)
                        })
                        .collect();
                    format!("[{}]", elements.join(","))
                } else {
                    format!("[{}]", element_type.format_value(value))
                }
            }
            DataType::String => {
                format!("\"{}\"", value.replace('"', "\\\""))
            }
            DataType::Unknown(_) => {
                format!("\"{}\"", value.replace('"', "\\\""))
            }
        }
    }

    /// Check if this type represents a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, DataType::Int | DataType::Float | DataType::Number)
    }

    /// Check if this type represents a boolean type
    pub fn is_boolean(&self) -> bool {
        matches!(self, DataType::Bool | DataType::Boolean)
    }

    /// Check if this type represents an object type
    pub fn is_object(&self) -> bool {
        matches!(self, DataType::Object)
    }

    /// Check if this type represents an array type
    pub fn is_array(&self) -> bool {
        matches!(self, DataType::Array(_))
    }
}

/// Type registry for managing type definitions in a table
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    pub headers: Vec<String>,
    pub type_defs: Vec<DataType>,
}

impl TypeRegistry {
    pub fn new(headers: Vec<String>, type_strings: Vec<String>) -> Self {
        let type_defs = type_strings.iter().map(|s| DataType::parse(s)).collect();

        Self { headers, type_defs }
    }

    /// Get the data type for a specific header
    pub fn get_type_for_header(&self, header: &str) -> Option<&DataType> {
        self.headers
            .iter()
            .position(|h| h == header)
            .and_then(|index| self.type_defs.get(index))
    }

    /// Get the data type by index
    pub fn get_type_by_index(&self, index: usize) -> Option<&DataType> {
        self.type_defs.get(index)
    }

    /// Format a value according to the type definition for a specific header
    pub fn format_value_for_header(&self, header: &str, value: &str) -> String {
        if let Some(data_type) = self.get_type_for_header(header) {
            data_type.format_value(value)
        } else {
            format!("\"{}\"", value.replace('"', "\\\""))
        }
    }

    /// Format a value according to the type definition by index
    pub fn format_value_by_index(&self, index: usize, value: &str) -> String {
        if let Some(data_type) = self.get_type_by_index(index) {
            data_type.format_value(value)
        } else {
            format!("\"{}\"", value.replace('"', "\\\""))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_parsing() {
        assert_eq!(DataType::parse("int"), DataType::Int);
        assert_eq!(DataType::parse("string"), DataType::String);
        assert_eq!(DataType::parse("bool"), DataType::Bool);
        assert_eq!(DataType::parse("object"), DataType::Object);

        if let DataType::Array(element_type) = DataType::parse("array<int>") {
            assert_eq!(*element_type, DataType::Int);
        } else {
            panic!("Expected array type");
        }
    }

    #[test]
    fn test_value_formatting() {
        let int_type = DataType::Int;
        assert_eq!(int_type.format_value("123"), "123");
        assert_eq!(int_type.format_value("abc"), "\"abc\"");

        let bool_type = DataType::Bool;
        assert_eq!(bool_type.format_value("true"), "true");
        assert_eq!(bool_type.format_value("false"), "false");
        assert_eq!(bool_type.format_value("1"), "true");
        assert_eq!(bool_type.format_value("0"), "false");

        let object_type = DataType::Object;
        assert_eq!(object_type.format_value("{key:1}"), "{key:1}");
        assert_eq!(object_type.format_value("not_object"), "\"not_object\"");

        let string_type = DataType::String;
        assert_eq!(string_type.format_value("hello"), "\"hello\"");
    }

    #[test]
    fn test_type_registry() {
        let headers = vec!["id".to_string(), "name".to_string(), "config".to_string()];
        let types = vec![
            "int".to_string(),
            "string".to_string(),
            "object".to_string(),
        ];
        let registry = TypeRegistry::new(headers, types);

        assert_eq!(registry.format_value_for_header("id", "123"), "123");
        assert_eq!(registry.format_value_for_header("name", "test"), "\"test\"");
        assert_eq!(
            registry.format_value_for_header("config", "{key:1}"),
            "{key:1}"
        );
    }
}
