use super::header::{
    HeaderRegistry, ObjectTable, ObjectTableItem, ParameterTable, RawHeaderData, TableResult,
};
use super::types::{DataType, TypeRegistry};
use crate::error::{ExcelParseError, ExcelResult};
use calamine::{Data, Reader, Xlsx, open_workbook};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum SaveTemplate {
    /// all#Item.addAll($content)
    AllTemplate { target: String },
    /// single#Item($name, {displayName: $displayName}, $tags)
    SingleTemplate { target: String, pattern: String },
    /// Default window.xx format
    Default { target: String },
}

pub struct ExcelParser {
    workbook: Xlsx<std::io::BufReader<std::fs::File>>,
    header_registry: HeaderRegistry,
}

impl ExcelParser {
    pub fn new<P: AsRef<Path>>(path: P) -> ExcelResult<Self> {
        let workbook = open_workbook(path)?;
        Ok(ExcelParser {
            workbook,
            header_registry: HeaderRegistry::default(),
        })
    }

    /// Extract raw header data from the first # row to the last # row
    fn extract_raw_header_data(&self, rows: &[&[Data]]) -> ExcelResult<RawHeaderData> {
        let mut start_row = None;
        let mut end_row = None;

        // Find first and last row starting with #
        for (i, row) in rows.iter().enumerate() {
            if !row.is_empty() && row[0].to_string().starts_with('#') {
                if start_row.is_none() {
                    start_row = Some(i);
                }
                end_row = Some(i);
            }
        }

        match (start_row, end_row) {
            (Some(start), Some(end)) => {
                let header_rows: Vec<Vec<String>> = rows[start..=end]
                    .iter()
                    .map(|row| row.iter().map(|cell| cell.to_string()).collect())
                    .collect();

                Ok(RawHeaderData {
                    rows: header_rows,
                    start_row: start,
                    end_row: end,
                })
            }
            _ => Err(ExcelParseError::invalid_format(
                "No header rows found (rows starting with #)",
            )),
        }
    }

    /// Parse a sheet using the header registry system
    pub fn parse_sheet(&mut self, sheet_name: &str) -> ExcelResult<Box<dyn TableResult>> {
        let range = self
            .workbook
            .worksheet_range(sheet_name)
            .map_err(|_| ExcelParseError::WorksheetNotFound(sheet_name.to_string()))?;

        let rows: Vec<&[Data]> = range.rows().collect();
        if rows.len() < 2 {
            return Err(ExcelParseError::invalid_format(
                "Need at least 2 rows (header + data)",
            ));
        }

        // Extract raw header data
        let raw_header_data = self.extract_raw_header_data(&rows)?;

        // Get data rows
        let data_start_row = raw_header_data.end_row + 1;
        let data_rows = if data_start_row < rows.len() {
            &rows[data_start_row..]
        } else {
            &[]
        };

        // Parse complete table using registry
        self.header_registry
            .parse_table(&raw_header_data, data_rows)
    }

    pub fn generate_javascript(
        object_tables: &[ObjectTable],
        parameter_tables: &[ParameterTable],
    ) -> ExcelResult<String> {
        let mut js_code = String::new();

        // Generate object tables with template support
        for table in object_tables {
            let save_template = Self::parse_save_template(&table.save_var)?;
            js_code.push_str(&Self::generate_table_with_template(table, &save_template)?);
        }

        // Generate parameter tables
        for table in parameter_tables {
            js_code.push_str(&format!("{} = {{\n", table.save_var));
            for (i, param) in table.parameters.iter().enumerate() {
                if i > 0 {
                    js_code.push_str(",\n");
                }

                let formatted_value = match param.var_type.as_str() {
                    "int" | "float" | "number" => param.value.clone(),
                    "bool" | "boolean" => param.value.clone(),
                    _ => format!("\"{}\"", param.value.replace('"', "\\\"")),
                };

                js_code.push_str(&format!("    {}: {}", param.name, formatted_value));
            }
            js_code.push_str("\n};\n\n");
        }

        Ok(js_code)
    }

    /// Parse save variable template
    fn parse_save_template(save_var: &str) -> ExcelResult<SaveTemplate> {
        if let Some(hash_pos) = save_var.find('#') {
            let prefix = &save_var[..hash_pos];
            let target = &save_var[hash_pos + 1..];

            match prefix {
                "all" => Ok(SaveTemplate::AllTemplate {
                    target: target.to_string(),
                }),
                "single" => Ok(SaveTemplate::SingleTemplate {
                    target: target.to_string(),
                    pattern: String::new(),
                }),
                _ => Err(ExcelParseError::invalid_format(format!(
                    "Unknown template prefix: {prefix}"
                ))),
            }
        } else {
            Ok(SaveTemplate::Default {
                target: save_var.to_string(),
            })
        }
    }

    /// Generate table code with template
    fn generate_table_with_template(
        table: &ObjectTable,
        template: &SaveTemplate,
    ) -> ExcelResult<String> {
        match template {
            SaveTemplate::AllTemplate { target } => Self::generate_all_template(table, target),
            SaveTemplate::SingleTemplate { target, .. } => {
                Self::generate_single_template(table, target)
            }
            SaveTemplate::Default { target } => Self::generate_default_template(table, target),
        }
    }

    /// Generate all# template: all#Item.addAll($content)
    fn generate_all_template(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let items_json = Self::generate_items_json(table)?;

        if target.contains("$content") {
            let expanded = target.replace("$content", &items_json);
            Ok(format!("{expanded};\n\n"))
        } else {
            // Fallback for backward compatibility
            Ok(format!("{target}({items_json});\n\n"))
        }
    }

    /// Generate single# template: single#Item($name,{displayName:$displayName},$tags)
    fn generate_single_template(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let mut js_code = String::new();

        for item in &table.items {
            // Parse the template pattern
            if let Some(open_paren) = target.find('(') {
                let function_name = &target[..open_paren];
                let params_part = &target[open_paren + 1..];

                if let Some(close_paren) = params_part.rfind(')') {
                    let params = &params_part[..close_paren];
                    let expanded_params = Self::expand_single_template_params(params, item, table)?;
                    js_code.push_str(&format!("{function_name}({expanded_params});\n"));
                }
            } else {
                return Err(ExcelParseError::invalid_format(
                    "Single template must contain function call pattern",
                ));
            }
        }

        js_code.push('\n');
        Ok(js_code)
    }

    /// Generate default window.xx template
    fn generate_default_template(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let items_json = Self::generate_items_json(table)?;
        Ok(format!("{target} = {items_json};\n\n"))
    }

    /// Generate items as JSON array (reusing existing complex logic)
    fn generate_items_json(table: &ObjectTable) -> ExcelResult<String> {
        let mut js_code = String::new();
        js_code.push_str("[\n");

        // Create type registry for this table
        let type_registry = TypeRegistry::new(table.headers.clone(), table.type_defs.clone());

        for (i, item) in table.items.iter().enumerate() {
            if i > 0 {
                js_code.push_str(",\n");
            }
            js_code.push_str("    {\n");

            let mut field_parts = Vec::new();
            for (header_idx, header) in table.headers.iter().enumerate() {
                if let Some(value) = item.fields.get(header) {
                    // Handle special cases for different field types
                    let formatted_value = if header.contains('#') {
                        // For array fields (field#N), skip individual rendering as they will be combined later
                        continue;
                    } else {
                        match type_registry.get_type_by_index(header_idx) {
                            Some(DataType::Array(element_type)) => {
                                // Handle array type - parse value regardless of format
                                if value.starts_with('[') && value.ends_with(']') {
                                    let inner = &value[1..value.len() - 1];
                                    if !inner.is_empty() {
                                        let elements: Vec<String> = inner
                                            .split(',')
                                            .map(|item| {
                                                let trimmed = item.trim();
                                                element_type.format_value(trimmed)
                                            })
                                            .collect();
                                        format!("[{}]", elements.join(", "))
                                    } else {
                                        "[]".to_string()
                                    }
                                } else if value.starts_with('{') && value.ends_with('}') {
                                    // Handle object format as single-item array
                                    format!("[{value}]")
                                } else {
                                    // Treat as single-item array
                                    format!("[{}]", element_type.format_value(value))
                                }
                            }
                            Some(DataType::Object) => {
                                // Handle object type - should be in {key:value} format
                                if value.starts_with('{') && value.ends_with('}') {
                                    value.clone()
                                } else {
                                    // Wrap non-object format in braces
                                    format!("{{{value}}}")
                                }
                            }
                            Some(_) | None => {
                                // Use type definition for formatting other types
                                type_registry.format_value_by_index(header_idx, value)
                            }
                        }
                    };

                    field_parts.push(format!("        {header}: {formatted_value}"));
                }
            }

            // Special handling for array fields - collect all field#N fields into arrays
            // Store (index, value, field_type) for each array field
            let mut array_fields: HashMap<String, Vec<(usize, String, DataType)>> = HashMap::new();
            for (header_idx, header) in table.headers.iter().enumerate() {
                if let Some(hash_pos) = header.find('#') {
                    let array_name = &header[..hash_pos];
                    if let Ok(index) = header[hash_pos + 1..].parse::<usize>()
                        && let Some(field_value) = item.fields.get(header)
                        && !field_value.is_empty()
                    {
                        // Get the type for this specific indexed field
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

            // Remove individual array#N fields and add combined arrays
            for (array_name, mut indexed_values) in array_fields {
                field_parts.retain(|part| !part.contains(&format!("{array_name}#")));

                // Find the type definition for this array
                let array_element_type = if let Some(base_header_idx) =
                    table.headers.iter().position(|h| h == &array_name)
                {
                    if let Some(data_type) = type_registry.get_type_by_index(base_header_idx) {
                        data_type.get_array_element_type()
                    } else {
                        DataType::String
                    }
                } else {
                    DataType::String
                };

                // Check if there's already a base array field (like "tags": [1,4,3])
                let mut combined_values = Vec::new();
                let base_array_length = if let Some(base_value) = item.fields.get(&array_name) {
                    if base_value.starts_with('[') && base_value.ends_with(']') {
                        // Parse existing array format [1,4,3] or ["a","b","c"]
                        let inner = &base_value[1..base_value.len() - 1];
                        if !inner.is_empty() {
                            for item in inner.split(',') {
                                let trimmed = item.trim();
                                combined_values.push(array_element_type.format_value(trimmed));
                            }
                        }
                    }
                    // Remove the base field since we'll replace it with combined array
                    field_parts
                        .retain(|part| !part.starts_with(&format!("        {array_name}: ")));
                    combined_values.len()
                } else {
                    0
                };

                // Validate array indices
                indexed_values.sort_by_key(|(index, _, _)| *index);
                for (index, _, _) in &indexed_values {
                    let required_index = *index;

                    // Check if we have enough base elements or previous indexed elements
                    let mut available_positions = base_array_length;
                    for (prev_index, _, _) in &indexed_values {
                        if *prev_index < required_index {
                            available_positions = available_positions.max(*prev_index);
                        }
                    }

                    // Validate index continuity
                    if required_index > available_positions + 1 {
                        return Err(ExcelParseError::ArrayIndexError(format!(
                            "{array_name}: found {array_name}#{required_index} but missing previous indices. Base array has {base_array_length} elements."
                        )));
                    }
                }

                // Extend array to accommodate new indices
                let max_index = indexed_values.iter().map(|(i, _, _)| *i).max().unwrap_or(0);
                while combined_values.len() < max_index {
                    // Fill gaps with empty values or extend array
                    combined_values.push("null".to_string());
                }

                // Add indexed values using each field's own type
                for (index, value, field_type) in indexed_values {
                    let array_index = index - 1; // Convert to 0-based index
                    let formatted_value = field_type.format_value(&value);
                    if array_index < combined_values.len() {
                        // Replace existing value
                        combined_values[array_index] = formatted_value;
                    } else {
                        // Extend array
                        combined_values.push(formatted_value);
                    }
                }

                field_parts.push(format!(
                    "        {array_name}: [{}]",
                    combined_values.join(", ")
                ));
            }

            js_code.push_str(&field_parts.join(",\n"));
            js_code.push_str("\n    }");
        }
        js_code.push_str("\n]");
        Ok(js_code)
    }

    /// Expand single template parameters like "$name, {displayName: $displayName}, $tags"
    fn expand_single_template_params(
        params: &str,
        item: &ObjectTableItem,
        _table: &ObjectTable,
    ) -> ExcelResult<String> {
        let mut expanded = params.to_string();

        // Replace $field_name with actual values from the item
        for (field_name, field_value) in &item.fields {
            let placeholder = format!("${field_name}");

            let formatted_value = if field_value.starts_with('{')
                || field_value.starts_with('[')
                || field_value.parse::<f64>().is_ok()
            {
                field_value.clone()
            } else {
                format!("\"{}\"", field_value.replace('"', "\\\""))
            };

            expanded = expanded.replace(&placeholder, &formatted_value);
        }

        Ok(expanded)
    }

    /// Parse an Excel file and generate JavaScript code
    pub async fn parse_file<P: AsRef<Path>>(path: P) -> ExcelResult<String> {
        let path = path.as_ref().to_path_buf();

        // Execute the entire parsing in a blocking task since calamine is not async
        tokio::task::spawn_blocking(move || -> ExcelResult<String> {
            let mut parser = ExcelParser::new(&path)?;
            let mut object_tables = Vec::new();
            let mut parameter_tables = Vec::new();

            // Get all worksheet names
            let worksheet_names: Vec<String> = parser
                .workbook
                .worksheets()
                .into_iter()
                .map(|(name, _)| name)
                .collect();

            // Parse each worksheet using the new registry system
            for sheet_name in &worksheet_names {
                match parser.parse_sheet(sheet_name) {
                    Ok(table_result) => {
                        match table_result.table_type() {
                            "object" => {
                                if let Some(object_table) =
                                    table_result.as_any().downcast_ref::<ObjectTable>()
                                {
                                    object_tables.push(object_table.clone());
                                }
                            }
                            "parameter" => {
                                if let Some(parameter_table) =
                                    table_result.as_any().downcast_ref::<ParameterTable>()
                                {
                                    parameter_tables.push(parameter_table.clone());
                                }
                            }
                            _ => {
                                // Skip unknown table types
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        // Skip sheets that don't match any known format
                        continue;
                    }
                }
            }

            // Generate JavaScript code
            Self::generate_javascript(&object_tables, &parameter_tables)
        })
        .await
        .map_err(|e| ExcelParseError::config_error(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::excel::header::ObjectTableItem;

    #[test]
    fn test_array_index_validation() {
        // Test case: tags#5 without tags#1-4 should fail
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("name".to_string(), "test".to_string());
        item_fields.insert("tags#5".to_string(), "invalid".to_string());

        let object_table = ObjectTable {
            save_var: "window.items".to_string(),
            table_type: "obj".to_string(),
            headers: vec!["id".to_string(), "name".to_string(), "tags#5".to_string()],
            type_defs: vec![
                "int".to_string(),
                "string".to_string(),
                "array<string>".to_string(),
            ],
            items: vec![ObjectTableItem {
                fields: item_fields,
            }],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("missing previous indices"));
        }
    }

    #[test]
    fn test_save_template_parsing() {
        // Test all# template parsing
        let template = ExcelParser::parse_save_template("all#Item.addAll($content)").unwrap();
        match template {
            SaveTemplate::AllTemplate { target } => {
                assert_eq!(target, "Item.addAll($content)");
            }
            _ => panic!("Expected AllTemplate"),
        }

        // Test single# template parsing
        let template =
            ExcelParser::parse_save_template("single#Item($name,{displayName:$displayName},$tags)")
                .unwrap();
        match template {
            SaveTemplate::SingleTemplate { target, .. } => {
                assert_eq!(target, "Item($name,{displayName:$displayName},$tags)");
            }
            _ => panic!("Expected SingleTemplate"),
        }

        // Test default template parsing
        let template = ExcelParser::parse_save_template("window.items").unwrap();
        match template {
            SaveTemplate::Default { target } => {
                assert_eq!(target, "window.items");
            }
            _ => panic!("Expected Default template"),
        }

        // Test unknown prefix
        let result = ExcelParser::parse_save_template("unknown#target");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_template_generation() {
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("name".to_string(), "test".to_string());

        let object_table = ObjectTable {
            save_var: "all#Item.addAll($content)".to_string(),
            table_type: "obj".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            type_defs: vec!["int".to_string(), "string".to_string()],
            items: vec![ObjectTableItem {
                fields: item_fields,
            }],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]).unwrap();
        assert!(result.contains("Item.addAll([\n"));
        assert!(result.contains("    {\n"));
        assert!(result.contains("        id: 1"));
        assert!(result.contains("        name: \"test\""));
        assert!(result.contains("    }\n]);"));
    }

    #[test]
    fn test_single_template_generation() {
        let mut item_fields1 = HashMap::new();
        item_fields1.insert("name".to_string(), "item1".to_string());
        item_fields1.insert("displayName".to_string(), "Item 1".to_string());
        item_fields1.insert("tags".to_string(), "[\"tag1\",\"tag2\"]".to_string());

        let mut item_fields2 = HashMap::new();
        item_fields2.insert("name".to_string(), "item2".to_string());
        item_fields2.insert("displayName".to_string(), "Item 2".to_string());
        item_fields2.insert("tags".to_string(), "[\"tag3\"]".to_string());

        let object_table = ObjectTable {
            save_var: "single#Item($name,{displayName:$displayName},$tags)".to_string(),
            table_type: "obj".to_string(),
            headers: vec![
                "name".to_string(),
                "displayName".to_string(),
                "tags".to_string(),
            ],
            type_defs: vec![
                "string".to_string(),
                "string".to_string(),
                "array<string>".to_string(),
            ],
            items: vec![
                ObjectTableItem {
                    fields: item_fields1,
                },
                ObjectTableItem {
                    fields: item_fields2,
                },
            ],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]).unwrap();
        assert!(result.contains("Item(\"item1\",{displayName:\"Item 1\"},[\"tag1\",\"tag2\"]);"));
        assert!(result.contains("Item(\"item2\",{displayName:\"Item 2\"},[\"tag3\"]);"));
    }

    #[test]
    fn test_default_template_generation() {
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("name".to_string(), "test".to_string());

        let object_table = ObjectTable {
            save_var: "window.items".to_string(),
            table_type: "obj".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            type_defs: vec!["int".to_string(), "string".to_string()],
            items: vec![ObjectTableItem {
                fields: item_fields,
            }],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]).unwrap();
        assert!(result.contains("window.items = [\n"));
        assert!(result.contains("    {\n"));
        assert!(result.contains("        id: 1"));
        assert!(result.contains("        name: \"test\""));
        assert!(result.contains("    }\n];\n"));
    }

    #[test]
    fn test_single_template_params_expansion() {
        let mut item_fields = HashMap::new();
        item_fields.insert("name".to_string(), "testItem".to_string());
        item_fields.insert("value".to_string(), "42".to_string());
        item_fields.insert("config".to_string(), "{enabled:true}".to_string());

        let item = ObjectTableItem {
            fields: item_fields,
        };
        let table = ObjectTable {
            save_var: "test".to_string(),
            table_type: "obj".to_string(),
            headers: vec![
                "name".to_string(),
                "value".to_string(),
                "config".to_string(),
            ],
            type_defs: vec![
                "string".to_string(),
                "int".to_string(),
                "object".to_string(),
            ],
            items: vec![],
        };

        let result =
            ExcelParser::expand_single_template_params("$name, $value, $config", &item, &table)
                .unwrap();

        assert_eq!(result, "\"testItem\", 42, {enabled:true}");
    }

    #[test]
    fn test_all_template_with_multiple_content_placeholders() {
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("name".to_string(), "test".to_string());

        let object_table = ObjectTable {
            save_var: "all#Item.check($content, $content)".to_string(),
            table_type: "obj".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            type_defs: vec!["int".to_string(), "string".to_string()],
            items: vec![ObjectTableItem {
                fields: item_fields,
            }],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]).unwrap();
        // Should replace both $content placeholders with the same array
        assert!(result.contains("Item.check([\n    {\n        id: 1,\n        name: \"test\"\n    }\n], [\n    {\n        id: 1,\n        name: \"test\"\n    }\n]);"));
    }

    #[test]
    fn test_all_template_backward_compatibility() {
        // Test that templates without $content still work (backward compatibility)
        let mut item_fields = HashMap::new();
        item_fields.insert("id".to_string(), "1".to_string());
        item_fields.insert("name".to_string(), "test".to_string());

        let object_table = ObjectTable {
            save_var: "all#Item.addAll".to_string(),
            table_type: "obj".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            type_defs: vec!["int".to_string(), "string".to_string()],
            items: vec![ObjectTableItem {
                fields: item_fields,
            }],
        };

        let result = ExcelParser::generate_javascript(&[object_table], &[]).unwrap();
        // Should fallback to function call format
        assert!(result.contains("Item.addAll([\n"));
        assert!(result.contains("    {\n"));
        assert!(result.contains("        id: 1"));
        assert!(result.contains("        name: \"test\""));
        assert!(result.contains("    }\n]);"));
    }
}
