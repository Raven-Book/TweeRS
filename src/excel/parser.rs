use super::types::{DataType, TypeRegistry};
use crate::error::{ExcelParseError, ExcelResult};
use calamine::{Data, Reader, Xlsx, open_workbook};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTableItem {
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTable {
    pub save_var: String,
    pub table_type: String, // "obj", "item", etc.
    pub headers: Vec<String>,
    pub type_defs: Vec<String>,
    pub items: Vec<ObjectTableItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterItem {
    pub name: String,
    pub var_type: String,
    pub value: String,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTable {
    pub save_var: String,
    pub parameters: Vec<ParameterItem>,
}

#[derive(Debug, Clone)]
struct TableHeader {
    save_var: String,
    table_type: String,
    headers: Vec<String>,
    type_defs: Vec<String>,
}

pub struct ExcelParser {
    workbook: Xlsx<std::io::BufReader<std::fs::File>>,
}

impl ExcelParser {
    pub fn new<P: AsRef<Path>>(path: P) -> ExcelResult<Self> {
        let workbook = open_workbook(path)?;
        Ok(ExcelParser { workbook })
    }

    pub fn parse_object_table(&mut self, sheet_name: &str) -> ExcelResult<ObjectTable> {
        let range = self
            .workbook
            .worksheet_range(sheet_name)
            .map_err(|_| ExcelParseError::WorksheetNotFound(sheet_name.to_string()))?;

        let rows: Vec<&[Data]> = range.rows().collect();
        if rows.len() < 3 {
            return Err(ExcelParseError::invalid_format(
                "Need at least 3 header rows",
            ));
        }

        // Parse header information from first 3 rows (can be in any order)
        let header_info = self.parse_object_headers(&rows[0..3])?;

        // Parse data rows
        let mut items = Vec::new();
        for row in &rows[3..] {
            if let Some(item) = self.parse_object_data_row(row, &header_info.headers)? {
                items.push(item);
            }
        }

        Ok(ObjectTable {
            save_var: header_info.save_var,
            table_type: header_info.table_type,
            headers: header_info.headers,
            type_defs: header_info.type_defs,
            items,
        })
    }

    pub fn parse_parameter_table(&mut self, sheet_name: &str) -> ExcelResult<ParameterTable> {
        let range = self
            .workbook
            .worksheet_range(sheet_name)
            .map_err(|_| ExcelParseError::WorksheetNotFound(sheet_name.to_string()))?;

        let rows: Vec<&[Data]> = range.rows().collect();
        if rows.len() < 2 {
            return Err(ExcelParseError::invalid_format(
                "Need at least 2 header rows",
            ));
        }

        // Parse header information from first 2 rows (can be in any order)
        let (save_var, headers) = self.parse_parameter_headers(&rows[0..2])?;

        // Parse data rows
        let mut parameters = Vec::new();
        for row in &rows[2..] {
            if let Some(param) = self.parse_parameter_data_row(row, &headers)? {
                parameters.push(param);
            }
        }

        Ok(ParameterTable {
            save_var,
            parameters,
        })
    }

    fn parse_object_headers(&self, header_rows: &[&[Data]]) -> ExcelResult<TableHeader> {
        let mut save_var = String::new();
        let mut table_type = String::new();
        let mut headers = Vec::new();
        let mut type_defs = Vec::new();

        for row in header_rows {
            if row.is_empty() {
                continue;
            }

            let first_cell = row[0].to_string();

            if first_cell.starts_with("#save") {
                if row.len() > 1 {
                    save_var = row[1].to_string();
                } else {
                    return Err(ExcelParseError::InvalidFormat(
                        "Missing save variable name".to_string(),
                    ));
                }
            } else if first_cell.starts_with("#")
                && !first_cell.starts_with("#type")
                && !first_cell.starts_with("#var")
            {
                // This is the table type row (e.g., #obj, #item, etc.) but not #var
                table_type = first_cell[1..].to_string(); // Remove the '#'
                headers = row.iter().skip(1).map(|cell| cell.to_string()).collect();
            } else if first_cell.starts_with("#type") {
                type_defs = row.iter().skip(1).map(|cell| cell.to_string()).collect();
            }
        }

        if save_var.is_empty() {
            return Err(ExcelParseError::missing_header("save"));
        }
        if table_type.is_empty() {
            return Err(ExcelParseError::missing_header("table type"));
        }
        if headers.is_empty() {
            return Err(ExcelParseError::missing_header("column headers"));
        }

        Ok(TableHeader {
            save_var,
            table_type,
            headers,
            type_defs,
        })
    }

    fn parse_parameter_headers(
        &self,
        header_rows: &[&[Data]],
    ) -> ExcelResult<(String, Vec<String>)> {
        let mut save_var = String::new();
        let mut headers = Vec::new();

        for row in header_rows {
            if row.is_empty() {
                continue;
            }

            let first_cell = row[0].to_string();

            if first_cell.starts_with("#save") {
                if row.len() > 1 {
                    save_var = row[1].to_string();
                } else {
                    return Err(ExcelParseError::InvalidFormat(
                        "Missing save variable name".to_string(),
                    ));
                }
            } else if first_cell.starts_with("#var") {
                headers = row.iter().skip(1).map(|cell| cell.to_string()).collect();
            }
        }

        if save_var.is_empty() {
            return Err(ExcelParseError::MissingHeader("save".to_string()));
        }
        if headers.is_empty() {
            return Err(ExcelParseError::MissingHeader("var headers".to_string()));
        }

        // Validate required headers for parameter table
        let required = ["name", "type", "value"];
        for req in &required {
            if !headers.contains(&req.to_string()) {
                return Err(ExcelParseError::MissingHeader(req.to_string()));
            }
        }

        Ok((save_var, headers))
    }

    fn parse_object_data_row(
        &self,
        row: &[Data],
        headers: &[String],
    ) -> ExcelResult<Option<ObjectTableItem>> {
        if row.is_empty() {
            return Ok(None);
        }

        let mut fields = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            if i + 1 < row.len() {
                let value = row[i + 1].to_string();
                if !value.is_empty() {
                    fields.insert(header.clone(), value);
                }
            }
        }

        if fields.is_empty() {
            return Ok(None);
        }

        Ok(Some(ObjectTableItem { fields }))
    }

    fn parse_parameter_data_row(
        &self,
        row: &[Data],
        headers: &[String],
    ) -> ExcelResult<Option<ParameterItem>> {
        if row.is_empty() {
            return Ok(None);
        }

        let mut data_map = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            if i + 1 < row.len() {
                data_map.insert(header.clone(), row[i + 1].to_string());
            }
        }

        let name = data_map
            .get("name")
            .ok_or_else(|| ExcelParseError::MissingHeader("name".to_string()))?
            .clone();

        let var_type = data_map
            .get("type")
            .ok_or_else(|| ExcelParseError::MissingHeader("type".to_string()))?
            .clone();

        let value = data_map
            .get("value")
            .ok_or_else(|| ExcelParseError::MissingHeader("value".to_string()))?
            .clone();

        let comment = data_map.get("comment").unwrap_or(&String::new()).clone();

        if name.is_empty() {
            return Ok(None);
        }

        Ok(Some(ParameterItem {
            name,
            var_type,
            value,
            comment,
        }))
    }

    pub fn generate_javascript(
        object_tables: &[ObjectTable],
        parameter_tables: &[ParameterTable],
    ) -> ExcelResult<String> {
        let mut js_code = String::new();

        // Generate object tables
        for table in object_tables {
            js_code.push_str(&format!("{} = [\n", table.save_var));

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
                        } else if value.starts_with('[') && value.ends_with(']') {
                            // Direct array format like [1,4,3] - need to reformat based on type
                            if let Some(DataType::Array(element_type)) =
                                type_registry.get_type_by_index(header_idx)
                            {
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
                            } else {
                                value.clone()
                            }
                        } else {
                            // Use type definition if available
                            type_registry.format_value_by_index(header_idx, value)
                        };

                        field_parts.push(format!("        {header}: {formatted_value}"));
                    }
                }

                // Special handling for array fields - collect all field#N fields into arrays
                let mut array_fields: HashMap<String, Vec<(usize, String)>> = HashMap::new();
                for (field_name, field_value) in &item.fields {
                    if let Some(hash_pos) = field_name.find('#') {
                        let array_name = &field_name[..hash_pos];
                        if let Ok(index) = field_name[hash_pos + 1..].parse::<usize>() {
                            if !field_value.is_empty() {
                                array_fields
                                    .entry(array_name.to_string())
                                    .or_default()
                                    .push((index, field_value.clone()));
                            }
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
                    indexed_values.sort_by_key(|(index, _)| *index);
                    for (index, _) in &indexed_values {
                        let required_index = *index;

                        // Check if we have enough base elements or previous indexed elements
                        let mut available_positions = base_array_length;
                        for (prev_index, _) in &indexed_values {
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
                    let max_index = indexed_values.iter().map(|(i, _)| *i).max().unwrap_or(0);
                    while combined_values.len() < max_index {
                        // Fill gaps with empty values or extend array
                        combined_values.push("null".to_string());
                    }

                    // Add indexed values
                    for (index, value) in indexed_values {
                        let array_index = index - 1; // Convert to 0-based index
                        if array_index < combined_values.len() {
                            // Replace existing value
                            combined_values[array_index] = array_element_type.format_value(&value);
                        } else {
                            // Extend array
                            combined_values.push(array_element_type.format_value(&value));
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
            js_code.push_str("\n];\n\n");
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

            // Parse each worksheet
            for sheet_name in &worksheet_names {
                // Try parsing as object table first
                match parser.parse_object_table(sheet_name) {
                    Ok(table) => {
                        object_tables.push(table);
                        continue;
                    }
                    Err(_) => {
                        // If object table parsing fails, try parameter table
                        match parser.parse_parameter_table(sheet_name) {
                            Ok(table) => {
                                parameter_tables.push(table);
                            }
                            Err(_) => {
                                // Skip sheets that don't match either format
                                continue;
                            }
                        }
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
            assert!(e.to_string().contains("Array index validation failed"));
        }
    }
}
