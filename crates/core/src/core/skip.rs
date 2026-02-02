//! JavaScript object parser for extracting fields from format.js files

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

type ParseResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Skip unknown fields and extract only known fields from window.storyFormat(...)
pub fn skip(content: &str, known_fields: &[&str]) -> ParseResult<String> {
    let start = content
        .find("window.storyFormat")
        .ok_or("Could not find window.storyFormat in format file")?;
    let obj_start = content[start..]
        .find('{')
        .map(|i| start + i)
        .ok_or("Could not find opening brace")?;

    let fields = parse_js_object(&content[obj_start..])?;

    let mut json_obj = String::from("{");
    let mut first = true;

    for key in known_fields {
        if let Some(value) = fields.get(*key) {
            if !first {
                json_obj.push(',');
            }
            first = false;
            json_obj.push_str(&format!("\"{}\":{}", key, value));
        }
    }
    json_obj.push('}');

    Ok(json_obj)
}

/// Parse a JS object and extract top-level key-value pairs
pub fn parse_js_object(s: &str) -> ParseResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    let mut chars = s.chars().peekable();

    if chars.next() != Some('{') {
        return Err("Expected opening brace".into());
    }

    skip_whitespace(&mut chars);

    while let Some(&c) = chars.peek() {
        if c == '}' {
            break;
        }

        let key = parse_key(&mut chars)?;
        skip_whitespace(&mut chars);

        if chars.next() != Some(':') {
            return Err("Expected colon after key".into());
        }
        skip_whitespace(&mut chars);

        let value = parse_value(&mut chars)?;
        result.insert(key, value);

        skip_whitespace(&mut chars);

        if chars.peek() == Some(&',') {
            chars.next();
            skip_whitespace(&mut chars);
        }
    }

    Ok(result)
}

fn skip_whitespace(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_key(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    let mut key = String::new();

    match chars.peek() {
        Some(&'"') | Some(&'\'') => {
            let quote = chars.next().unwrap();
            for c in chars.by_ref() {
                if c == quote {
                    break;
                }
                key.push(c);
            }
        }
        _ => {
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '$' {
                    key.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    Ok(key)
}

fn parse_value(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    match chars.peek() {
        Some(&'"') | Some(&'\'') => parse_string_value(chars),
        Some(&'t') | Some(&'f') => parse_bool_or_unknown(chars),
        Some(&c) if c.is_ascii_digit() || c == '-' => parse_number_value(chars),
        _ => skip_unknown_value(chars),
    }
}

fn parse_string_value(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    let quote = chars.next().unwrap();
    let mut result = String::from("\"");

    while let Some(c) = chars.next() {
        if c == '\\' {
            result.push(c);
            if let Some(next) = chars.next() {
                result.push(next);
            }
        } else if c == quote {
            break;
        } else if c == '"' && quote == '\'' {
            result.push_str("\\\"");
        } else {
            result.push(c);
        }
    }
    result.push('"');

    Ok(result)
}

/// Parse boolean (true/false) or skip unknown identifier (like function)
fn parse_bool_or_unknown(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() {
            value.push(c);
            chars.next();
        } else {
            break;
        }
    }

    // If it's true/false, return as-is; otherwise it's something like "function"
    if value == "true" || value == "false" {
        Ok(value)
    } else {
        // It's an unknown value (e.g., function), skip the rest
        skip_unknown_value(chars)
    }
}

fn parse_number_value(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
            value.push(c);
            chars.next();
        } else {
            break;
        }
    }
    Ok(value)
}

/// Skip unknown values (functions, objects, arrays, etc.)
fn skip_unknown_value(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '"';
    let mut template_depth = 0; // Track nested ${} in template strings

    while let Some(&c) = chars.peek() {
        if in_string {
            chars.next();
            if c == '\\' {
                chars.next();
            } else if string_char == '`' && c == '$' {
                // Check for ${ in template string
                if chars.peek() == Some(&'{') {
                    chars.next();
                    template_depth += 1;
                }
            } else if c == string_char && template_depth == 0 {
                in_string = false;
            } else if c == '}' && template_depth > 0 {
                template_depth -= 1;
            }
        } else {
            match c {
                '"' | '\'' | '`' => {
                    in_string = true;
                    string_char = c;
                    chars.next();
                }
                '{' | '[' | '(' => {
                    depth += 1;
                    chars.next();
                }
                '}' | ']' | ')' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    chars.next();
                }
                ',' if depth == 0 => break,
                _ => {
                    chars.next();
                }
            }
        }
    }

    Ok("null".to_string())
}
