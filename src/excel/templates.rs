use super::arrays::ArrayHandler;
use super::header::{HtmlTable, HtmlTableItem, ObjectTable, ObjectTableItem};
use super::types::TypeRegistry;
use crate::error::{ExcelParseError, ExcelResult};
use crate::util::html::HtmlEscape;
use std::collections::HashMap;

/// Template types for save variable patterns
#[derive(Debug, Clone)]
pub enum SaveTemplate {
    /// all#Item.addAll($content)
    AllTemplate { target: String },
    /// single#Item($name, {displayName: $displayName}, $tags)
    SingleTemplate { target: String, pattern: String },
    /// Default window.xx format
    Default { target: String },
}

/// Template processor for handling different save template patterns
pub struct TemplateProcessor;

impl TemplateProcessor {
    /// Parse save variable template from string
    pub fn parse_save_template(save_var: &str) -> ExcelResult<SaveTemplate> {
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

    /// Generate table code with appropriate template
    pub fn generate_table_with_template(
        table: &ObjectTable,
        template: &SaveTemplate,
    ) -> ExcelResult<String> {
        match template {
            SaveTemplate::AllTemplate { target } => AllTemplateProcessor::generate(table, target),
            SaveTemplate::SingleTemplate { target, .. } => {
                SingleTemplateProcessor::generate(table, target)
            }
            SaveTemplate::Default { target } => DefaultTemplateProcessor::generate(table, target),
        }
    }
}

/// Processor for all# templates
pub struct AllTemplateProcessor;

impl AllTemplateProcessor {
    /// Generate all# template: all#Item.addAll($content)
    pub fn generate(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let items_json = Self::generate_items_json(table)?;
        let expanded = target.replace("$content", &items_json);
        Ok(format!("{expanded};\n\n"))
    }

    /// Generate items as JSON array for all# templates
    fn generate_items_json(table: &ObjectTable) -> ExcelResult<String> {
        let mut js_code = String::new();
        js_code.push_str("[\n");

        let type_registry = TypeRegistry::new(table.headers.clone(), table.type_defs.clone());

        for (i, item) in table.items.iter().enumerate() {
            if i > 0 {
                js_code.push_str(",\n");
            }
            js_code.push_str("    {\n");

            let field_parts =
                ArrayHandler::process_array_fields(item, &table.headers, &type_registry)?;

            js_code.push_str(&field_parts.join(",\n"));
            js_code.push_str("\n    }");
        }

        js_code.push_str("\n]");
        Ok(js_code)
    }
}

/// Processor for single# templates  
pub struct SingleTemplateProcessor;

impl SingleTemplateProcessor {
    /// Generate single# template: single#Item($name,{displayName:$displayName},$tags)
    pub fn generate(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let mut js_code = String::new();

        for item in &table.items {
            let expanded = Self::expand_template_params(target, item, table)?;
            js_code.push_str(&format!("{expanded};\n"));
        }

        js_code.push('\n');
        Ok(js_code)
    }

    /// Expand single template parameters like "$name, {displayName: $displayName}, $tags"
    fn expand_template_params(
        params: &str,
        item: &ObjectTableItem,
        table: &ObjectTable,
    ) -> ExcelResult<String> {
        let type_registry = TypeRegistry::new(table.headers.clone(), table.type_defs.clone());
        let field_parts = ArrayHandler::process_array_fields(item, &table.headers, &type_registry)?;

        let mut processed_fields = HashMap::new();

        for field_part in field_parts {
            if let Some(colon_pos) = field_part.find(": ") {
                let field_name = field_part[8..colon_pos].trim();
                let field_value = &field_part[colon_pos + 2..];
                processed_fields.insert(field_name.to_string(), field_value.to_string());
            }
        }

        for (field_name, field_value) in &item.fields {
            if !field_name.contains('#') && !processed_fields.contains_key(field_name) {
                let formatted_value = if field_value.starts_with('{')
                    || field_value.starts_with('[')
                    || field_value.parse::<f64>().is_ok()
                {
                    field_value.clone()
                } else {
                    format!("\"{}\"", field_value.replace('"', "\\\""))
                };
                processed_fields.insert(field_name.clone(), formatted_value);
            }
        }

        let mut expanded = params.to_string();
        for (field_name, field_value) in processed_fields {
            let placeholder = format!("${field_name}");
            expanded = expanded.replace(&placeholder, &field_value);
        }

        Ok(expanded)
    }
}

/// Processor for default templates
pub struct DefaultTemplateProcessor;

impl DefaultTemplateProcessor {
    /// Generate default window.xx template
    pub fn generate(table: &ObjectTable, target: &str) -> ExcelResult<String> {
        let items_json = AllTemplateProcessor::generate_items_json(table)?;
        Ok(format!("{target} = {items_json};\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_save_template_parsing() {
        let template = TemplateProcessor::parse_save_template("all#Item.addAll($content)").unwrap();
        match template {
            SaveTemplate::AllTemplate { target } => {
                assert_eq!(target, "Item.addAll($content)");
            }
            _ => panic!("Expected AllTemplate"),
        }

        let template = TemplateProcessor::parse_save_template(
            "single#Item($name,{displayName:$displayName},$tags)",
        )
        .unwrap();
        match template {
            SaveTemplate::SingleTemplate { target, .. } => {
                assert_eq!(target, "Item($name,{displayName:$displayName},$tags)");
            }
            _ => panic!("Expected SingleTemplate"),
        }

        let template = TemplateProcessor::parse_save_template("window.items").unwrap();
        match template {
            SaveTemplate::Default { target } => {
                assert_eq!(target, "window.items");
            }
            _ => panic!("Expected Default template"),
        }

        let result = TemplateProcessor::parse_save_template("unknown#target");
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

        let result =
            AllTemplateProcessor::generate(&object_table, "Item.addAll($content)").unwrap();
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

        let result = SingleTemplateProcessor::generate(
            &object_table,
            "Item($name,{displayName:$displayName},$tags)",
        )
        .unwrap();
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

        let result = DefaultTemplateProcessor::generate(&object_table, "window.items").unwrap();
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

        let result = SingleTemplateProcessor::expand_template_params(
            "$name, $value, $config",
            &item,
            &table,
        )
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

        let result =
            AllTemplateProcessor::generate(&object_table, "Item.check($content, $content)")
                .unwrap();

        assert!(result.contains("Item.check([\n    {\n        id: 1,\n        name: \"test\"\n    }\n], [\n    {\n        id: 1,\n        name: \"test\"\n    }\n]);"));
    }
}

/// HTML template processor for HtmlTable
pub struct HtmlTemplateProcessor;

impl HtmlTemplateProcessor {
    /// Generate HTML from HtmlTable
    pub fn generate(table: &HtmlTable) -> ExcelResult<String> {
        let mut html = String::new();
        html.push_str("<tweers-exceldata>\n");
        html.push_str(&format!("\t<{}>\n", table.save_name));

        for (index, item) in table.items.iter().enumerate() {
            html.push_str(&Self::generate_item_html(
                item,
                &table.save_name,
                index + 1,
            )?);
        }

        html.push_str(&format!("\t</{}>\n", table.save_name));
        html.push_str("</tweers-exceldata>");
        Ok(html)
    }

    /// Generate HTML for a single item
    fn generate_item_html(
        item: &HtmlTableItem,
        save_name: &str,
        default_id: usize,
    ) -> ExcelResult<String> {
        // Always generate id in format {save_name}-{save_id}
        let id = format!("{}-{}", save_name, default_id);

        // Build attributes
        let mut attributes = Vec::new();
        attributes.push(format!("id=\"{}\"", HtmlEscape::escape_attribute(&id)));

        // Collect fields that should be child tags
        let mut child_tags = Vec::new();

        // Process each field
        for (key, value) in &item.fields {
            if key == "id" {
                // id is already handled above
                continue;
            }

            // name column always as data-name attribute
            if key == "name" {
                attributes.push(format!(
                    "data-name=\"{}\"",
                    HtmlEscape::escape_attribute(value)
                ));
            }
            // Other fields as child tags
            else {
                child_tags.push((key.clone(), value.clone()));
            }
        }

        let attrs_str = attributes.join(" ");

        // Generate the div tag with child tags if any
        if child_tags.is_empty() {
            Ok(format!("\t\t<div {}></div>\n", attrs_str))
        } else {
            let mut html = format!("\t\t<div {}>\n", attrs_str);
            for (tag_name, content) in child_tags {
                let escaped_content = HtmlEscape::escape_content(&content);
                html.push_str(&format!(
                    "\t\t\t<{}>{}</{}>\n",
                    tag_name, escaped_content, tag_name
                ));
            }
            html.push_str("\t\t</div>\n");
            Ok(html)
        }
    }
}
