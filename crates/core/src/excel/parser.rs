use super::header::{
    HeaderRegistry, HtmlTable, ObjectTable, ParameterTable, RawHeaderData, TableResult,
};
use super::templates::{HtmlTemplateProcessor, TemplateProcessor};
use crate::error::{ExcelParseError, ExcelResult};
use calamine::{Data, Reader, Xlsx};
use std::io::Cursor;

/// Result of parsing an Excel file
#[derive(Debug, Clone)]
pub struct ExcelParseResult {
    pub javascript: String,
    pub html: String,
}

pub struct ExcelParser {
    workbook: Xlsx<Cursor<Vec<u8>>>,
    header_registry: HeaderRegistry,
}

impl ExcelParser {
    /// Create ExcelParser from bytes (WASM-compatible)
    pub fn from_bytes(bytes: Vec<u8>) -> ExcelResult<Self> {
        let cursor = Cursor::new(bytes);
        let workbook = Xlsx::new(cursor).map_err(|e| {
            ExcelParseError::config_error(format!("Failed to open workbook: {}", e))
        })?;
        Ok(ExcelParser {
            workbook,
            header_registry: HeaderRegistry::default(),
        })
    }

    /// Extract raw header data from the first # row to the last # row
    fn extract_raw_header_data(&self, rows: &[&[Data]]) -> ExcelResult<RawHeaderData> {
        let mut start_row = None;
        let mut end_row = None;

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

        let raw_header_data = self.extract_raw_header_data(&rows)?;

        let data_start_row = raw_header_data.end_row + 1;
        let data_rows = if data_start_row < rows.len() {
            &rows[data_start_row..]
        } else {
            &[]
        };

        self.header_registry
            .parse_table(&raw_header_data, data_rows)
    }

    pub fn generate_javascript(
        object_tables: &[ObjectTable],
        parameter_tables: &[ParameterTable],
    ) -> ExcelResult<String> {
        let mut js_code = String::new();

        for table in object_tables {
            let save_template = TemplateProcessor::parse_save_template(&table.save_var)?;
            js_code.push_str(&TemplateProcessor::generate_table_with_template(
                table,
                &save_template,
            )?);
        }

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

    pub fn generate_html(html_tables: &[HtmlTable]) -> ExcelResult<String> {
        let mut html_parts = Vec::new();

        for table in html_tables {
            html_parts.push(HtmlTemplateProcessor::generate(table)?);
        }

        Ok(html_parts.join("\n\n"))
    }

    /// Parse Excel from bytes and generate JavaScript code and HTML
    pub fn parse_from_bytes(bytes: Vec<u8>) -> ExcelResult<ExcelParseResult> {
        let mut parser = ExcelParser::from_bytes(bytes)?;
        let mut object_tables = Vec::new();
        let mut parameter_tables = Vec::new();
        let mut html_tables = Vec::new();

        let worksheet_names: Vec<String> = parser
            .workbook
            .worksheets()
            .into_iter()
            .map(|(name, _)| name)
            .collect();

        for sheet_name in &worksheet_names {
            match parser.parse_sheet(sheet_name) {
                Ok(table_result) => match table_result.table_type() {
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
                    "html" => {
                        if let Some(html_table) = table_result.as_any().downcast_ref::<HtmlTable>()
                        {
                            html_tables.push(html_table.clone());
                        }
                    }
                    _ => {
                        continue;
                    }
                },
                Err(_) => {
                    continue;
                }
            }
        }

        // Generate JavaScript for object and parameter tables
        let js_code = Self::generate_javascript(&object_tables, &parameter_tables)?;

        // Generate HTML for HTML tables
        let html_code = Self::generate_html(&html_tables)?;

        Ok(ExcelParseResult {
            javascript: js_code,
            html: html_code,
        })
    }
}
