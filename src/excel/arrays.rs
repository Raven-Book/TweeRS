use super::header::ObjectTableItem;
use super::types::{DataType, TypeRegistry};
use crate::error::{ExcelParseError, ExcelResult};
use std::collections::HashMap;

/// Type alias for complex array field structure
type ArrayFieldMap = HashMap<String, Vec<(usize, String, DataType)>>;

/// Array field handler for processing array-like fields in Excel tables
pub struct ArrayHandler;

impl ArrayHandler {
    /// Process array fields in an item and generate formatted field parts
    pub fn process_array_fields(
        item: &ObjectTableItem,
        headers: &[String],
        type_registry: &TypeRegistry,
    ) -> ExcelResult<Vec<String>> {
        let mut field_parts = Vec::new();

        for (header_idx, header) in headers.iter().enumerate() {
            if let Some(value) = item.fields.get(header) {
                if header.contains('#') {
                    continue;
                }

                let formatted_value = Self::format_field_value(value, header_idx, type_registry)?;
                field_parts.push(format!("        {header}: {formatted_value}"));
            }
        }

        let array_fields = Self::collect_array_fields(item, headers, type_registry)?;
        let array_field_parts =
            Self::generate_array_field_parts(array_fields, item, type_registry)?;

        for (array_name, combined_values) in array_field_parts {
            field_parts.retain(|part| !part.contains(&format!("{array_name}#")));

            field_parts.retain(|part| !part.starts_with(&format!("        {array_name}: ")));

            field_parts.push(format!(
                "        {array_name}: [{}]",
                combined_values.join(", ")
            ));
        }

        Ok(field_parts)
    }

    /// Format a single field value based on its type
    fn format_field_value(
        value: &str,
        header_idx: usize,
        type_registry: &TypeRegistry,
    ) -> ExcelResult<String> {
        match type_registry.get_type_by_index(header_idx) {
            Some(DataType::Array(element_type)) => Self::format_array_value(value, element_type),
            Some(DataType::Object) => Self::format_object_value(value),
            Some(_) | None => Ok(type_registry.format_value_by_index(header_idx, value)),
        }
    }

    /// Format array value - handle different array formats
    fn format_array_value(value: &str, element_type: &DataType) -> ExcelResult<String> {
        if value.starts_with('[') && value.ends_with(']') {
            let inner = &value[1..value.len() - 1];
            if !inner.is_empty() {
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
                Ok(format!("[{}]", elements.join(",")))
            } else {
                Ok("[]".to_string())
            }
        } else if value.starts_with('{') && value.ends_with('}') {
            Ok(format!("[{value}]"))
        } else {
            Ok(format!("[{}]", element_type.format_value(value)))
        }
    }

    /// Format object value - ensure proper object format
    fn format_object_value(value: &str) -> ExcelResult<String> {
        if (value.starts_with('{') && value.ends_with('}'))
            || (value.starts_with('[') && value.ends_with(']'))
        {
            Ok(value.to_string())
        } else {
            Ok(format!("{{{value}}}"))
        }
    }

    /// Collect and organize array fields from item
    fn collect_array_fields(
        item: &ObjectTableItem,
        headers: &[String],
        type_registry: &TypeRegistry,
    ) -> ExcelResult<ArrayFieldMap> {
        let mut array_fields: ArrayFieldMap = HashMap::new();

        for (header_idx, header) in headers.iter().enumerate() {
            if let Some(hash_pos) = header.find('#') {
                let array_name = &header[..hash_pos];
                if let Ok(index) = header[hash_pos + 1..].parse::<usize>()
                    && let Some(field_value) = item.fields.get(header)
                    && !field_value.is_empty()
                {
                    let field_type = type_registry
                        .get_type_by_index(header_idx)
                        .unwrap_or(&DataType::String)
                        .clone();

                    array_fields
                        .entry(array_name.to_string())
                        .or_default()
                        .push((index, field_value.clone(), field_type));
                }
            }
        }

        Ok(array_fields)
    }

    /// Generate combined array field parts with validation
    fn generate_array_field_parts(
        array_fields: ArrayFieldMap,
        item: &ObjectTableItem,
        type_registry: &TypeRegistry,
    ) -> ExcelResult<HashMap<String, Vec<String>>> {
        let mut result = HashMap::new();

        for (array_name, mut indexed_values) in array_fields {
            let array_element_type = Self::get_array_element_type(&array_name, type_registry);

            let mut combined_values =
                Self::get_base_array_values(&array_name, item, &array_element_type)?;
            let base_array_length = combined_values.len();

            Self::validate_array_indices(&indexed_values, &array_name, base_array_length)?;

            indexed_values.sort_by_key(|(index, _, _)| *index);
            Self::merge_indexed_values(&mut combined_values, indexed_values)?;

            result.insert(array_name, combined_values);
        }

        Ok(result)
    }

    /// Get array element type for a given array name
    fn get_array_element_type(array_name: &str, type_registry: &TypeRegistry) -> DataType {
        if let Some(data_type) = type_registry.get_type_for_header(array_name) {
            data_type.get_array_element_type()
        } else {
            DataType::String
        }
    }

    /// Get base array values from existing field
    fn get_base_array_values(
        array_name: &str,
        item: &ObjectTableItem,
        element_type: &DataType,
    ) -> ExcelResult<Vec<String>> {
        let mut combined_values = Vec::new();

        if let Some(base_value) = item.fields.get(array_name) {
            if base_value.starts_with('[') && base_value.ends_with(']') {
                let inner = &base_value[1..base_value.len() - 1];
                if !inner.is_empty() {
                    for item in inner.split(',') {
                        let trimmed = item.trim();
                        combined_values.push(element_type.format_value(trimmed));
                    }
                }
            }
        }

        Ok(combined_values)
    }

    /// Validate array indices for basic sanity (just check they are > 0)
    fn validate_array_indices(
        indexed_values: &[(usize, String, DataType)],
        _array_name: &str,
        _base_array_length: usize,
    ) -> ExcelResult<()> {
        for (index, _, _) in indexed_values {
            if *index == 0 {
                return Err(ExcelParseError::ArrayIndexError(
                    "Array index must be >= 1, found index 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Merge indexed values into combined array
    fn merge_indexed_values(
        combined_values: &mut Vec<String>,
        indexed_values: Vec<(usize, String, DataType)>,
    ) -> ExcelResult<()> {
        let max_index = indexed_values.iter().map(|(i, _, _)| *i).max().unwrap_or(0);
        while combined_values.len() < max_index {
            combined_values.push("null".to_string());
        }

        for (index, value, field_type) in indexed_values {
            let array_index = index - 1;
            let formatted_value = field_type.format_value(&value);
            if array_index < combined_values.len() {
                combined_values[array_index] = formatted_value;
            } else {
                combined_values.push(formatted_value);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_array_field_processing() {
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("tags".to_string(), "[\"tag1\"]".to_string());
        item_fields.insert("tags#2".to_string(), "tag2".to_string());
        item_fields.insert("tags#3".to_string(), "tag3".to_string());

        let item = ObjectTableItem {
            fields: item_fields,
        };
        let headers = vec![
            "id".to_string(),
            "tags".to_string(),
            "tags#2".to_string(),
            "tags#3".to_string(),
        ];
        let type_defs = vec![
            "int".to_string(),
            "array<string>".to_string(),
            "string".to_string(),
            "string".to_string(),
        ];
        let type_registry = TypeRegistry::new(headers.clone(), type_defs);

        let result = ArrayHandler::process_array_fields(&item, &headers, &type_registry).unwrap();

        assert!(result.iter().any(|s| s.contains("id: 1")));
        assert!(result.iter().any(|s| s.contains("tags: [")
            && s.contains("tag1")
            && s.contains("tag2")
            && s.contains("tag3")));
    }

    #[test]
    fn test_array_index_validation_error() {
        let mut item_fields = HashMap::new();
        item_fields.insert("tags#0".to_string(), "invalid".to_string());

        let item = ObjectTableItem {
            fields: item_fields,
        };
        let headers = vec!["tags#0".to_string()];
        let type_defs = vec!["string".to_string()];
        let type_registry = TypeRegistry::new(headers.clone(), type_defs);

        let result = ArrayHandler::process_array_fields(&item, &headers, &type_registry);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("must be >= 1"));
        }
    }
}
