use crate::error::{ExcelParseError, ExcelResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Header parsing system
#[derive(Debug, Clone)]
pub struct RawHeaderData {
    pub rows: Vec<Vec<String>>,
    pub start_row: usize,
    pub end_row: usize,
}

// Trait for header parsers
pub trait HeaderParser: Send + Sync {
    fn required_headers(&self) -> Vec<&'static str>;
    fn parse_complete_table(
        &self,
        raw_data: &RawHeaderData,
        data_rows: &[&[calamine::Data]],
    ) -> ExcelResult<Box<dyn TableResult>>;
    fn parser_name(&self) -> &'static str;
}

// Trait for parsed table results
pub trait TableResult: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn table_type(&self) -> &str;
}

// Registry for header parsers
pub struct HeaderRegistry {
    parsers: Vec<Box<dyn HeaderParser>>,
}

impl HeaderRegistry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register<T: HeaderParser + 'static>(&mut self, parser: T) {
        self.parsers.push(Box::new(parser));
    }

    pub fn parse_table(
        &self,
        raw_data: &RawHeaderData,
        data_rows: &[&[calamine::Data]],
    ) -> ExcelResult<Box<dyn TableResult>> {
        for parser in &self.parsers {
            if self.can_parse(parser.as_ref(), raw_data) {
                return parser.parse_complete_table(raw_data, data_rows);
            }
        }

        Err(ExcelParseError::invalid_format(
            "No suitable header parser found for this table format",
        ))
    }

    fn can_parse(&self, parser: &dyn HeaderParser, raw_data: &RawHeaderData) -> bool {
        let required_headers = parser.required_headers();
        let mut found_headers = std::collections::HashSet::new();

        for row in &raw_data.rows {
            if !row.is_empty() {
                let first_cell = &row[0];
                if first_cell.starts_with('#') {
                    found_headers.insert(first_cell.as_str());
                }
            }
        }

        required_headers
            .iter()
            .all(|&header| found_headers.contains(header))
    }
}

impl Default for HeaderRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(ObjectTableHeaderParser);
        registry.register(ParameterTableHeaderParser);
        registry
    }
}

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

impl TableResult for ObjectTable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn table_type(&self) -> &str {
        "object"
    }
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

impl TableResult for ParameterTable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn table_type(&self) -> &str {
        "parameter"
    }
}

// Object table header parser implementation
pub struct ObjectTableHeaderParser;

impl HeaderParser for ObjectTableHeaderParser {
    fn required_headers(&self) -> Vec<&'static str> {
        vec!["#save", "#obj", "#type"]
    }

    fn parse_complete_table(
        &self,
        raw_data: &RawHeaderData,
        data_rows: &[&[calamine::Data]],
    ) -> ExcelResult<Box<dyn TableResult>> {
        let mut save_var = String::new();
        let mut table_type = String::new();
        let mut headers = Vec::new();
        let mut type_defs = Vec::new();

        for row in &raw_data.rows {
            if row.is_empty() {
                continue;
            }

            let first_cell = &row[0];

            if first_cell.starts_with("#save") {
                if row.len() > 1 {
                    save_var = row[1].clone();
                } else {
                    return Err(ExcelParseError::InvalidFormat(
                        "Missing save variable name".to_string(),
                    ));
                }
            } else if first_cell.starts_with("#obj") {
                // This is the table type row
                table_type = first_cell[1..].to_string(); // Remove the '#'
                headers = row.iter().skip(1).cloned().collect();
            } else if first_cell.starts_with("#type") {
                type_defs = row.iter().skip(1).cloned().collect();
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

        // Parse data rows
        let mut items = Vec::new();
        for row in data_rows {
            if let Some(item) = Self::parse_object_data_row(row, &headers)? {
                items.push(item);
            }
        }

        let table = ObjectTable {
            save_var,
            table_type,
            headers,
            type_defs,
            items,
        };

        Ok(Box::new(table))
    }

    fn parser_name(&self) -> &'static str {
        "ObjectTableHeaderParser"
    }
}

impl ObjectTableHeaderParser {
    fn parse_object_data_row(
        row: &[calamine::Data],
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
}

// Parameter table header parser implementation
pub struct ParameterTableHeaderParser;

impl HeaderParser for ParameterTableHeaderParser {
    fn required_headers(&self) -> Vec<&'static str> {
        vec!["#save", "#var"]
    }

    fn parse_complete_table(
        &self,
        raw_data: &RawHeaderData,
        data_rows: &[&[calamine::Data]],
    ) -> ExcelResult<Box<dyn TableResult>> {
        let mut save_var = String::new();
        let mut headers = Vec::new();

        for row in &raw_data.rows {
            if row.is_empty() {
                continue;
            }

            let first_cell = &row[0];

            if first_cell.starts_with("#save") {
                if row.len() > 1 {
                    save_var = row[1].clone();
                } else {
                    return Err(ExcelParseError::InvalidFormat(
                        "Missing save variable name".to_string(),
                    ));
                }
            } else if first_cell.starts_with("#var") {
                headers = row.iter().skip(1).cloned().collect();
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

        // Parse data rows
        let mut parameters = Vec::new();
        for row in data_rows {
            if let Some(param) = Self::parse_parameter_data_row(row, &headers)? {
                parameters.push(param);
            }
        }

        let table = ParameterTable {
            save_var,
            parameters,
        };

        Ok(Box::new(table))
    }

    fn parser_name(&self) -> &'static str {
        "ParameterTableHeaderParser"
    }
}

impl ParameterTableHeaderParser {
    fn parse_parameter_data_row(
        row: &[calamine::Data],
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
}
