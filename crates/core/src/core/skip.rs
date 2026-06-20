//! JavaScript object parser for extracting fields from format.js files

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

type ParseResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Parse a JS object and extract top-level key-value pairs
pub fn parse_js_object(s: &str) -> ParseResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    let mut chars = s.chars().peekable();

    if chars.next() != Some('{') {
        return Err("Expected opening brace".into());
    }

    skip(&mut chars)?;

    while let Some(&c) = chars.peek() {
        if c == '}' {
            break;
        }

        let key = parse_key(&mut chars)?;
        skip(&mut chars)?;

        if chars.next() != Some(':') {
            return Err("Expected colon after key".into());
        }
        skip(&mut chars)?;

        let value = parse_value(&mut chars)?;
        result.insert(key, value);

        skip(&mut chars)?;

        if chars.peek() == Some(&',') {
            chars.next();
            skip(&mut chars)?;
        }
    }

    Ok(result)
}

fn skip(chars: &mut Peekable<Chars>) -> ParseResult<()> {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '/' && peek_next(chars) == Some('/') {
            skip_line_comment(chars);
        } else if c == '/' && peek_next(chars) == Some('*') {
            skip_block_comment(chars)?;
        } else {
            break;
        }
    }
    Ok(())
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
                if is_identifier_part(c) {
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
    let mut value = String::new();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(escaped) = parse_escape_sequence(chars)? {
                value.push(escaped);
            }
        } else if c == quote {
            break;
        } else {
            value.push(c);
        }
    }

    Ok(serde_json::to_string(&value)?)
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
        // It's an unknown value (e.g., function), skip the rest.
        skip_unknown_value_after_identifier(chars)
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
    skip_unknown_value_inner(chars, true)
}

fn skip_unknown_value_after_identifier(chars: &mut Peekable<Chars>) -> ParseResult<String> {
    skip_unknown_value_inner(chars, false)
}

fn skip_unknown_value_inner(
    chars: &mut Peekable<Chars>,
    mut can_start_regex: bool,
) -> ParseResult<String> {
    let mut depth = 0;

    while let Some(&c) = chars.peek() {
        if depth == 0 && (c == ',' || c == '}') {
            break;
        }

        match c {
            '"' | '\'' => {
                skip_quoted_string(chars, c)?;
                can_start_regex = false;
            }
            '`' => {
                skip_template_literal(chars)?;
                can_start_regex = false;
            }
            '/' if peek_next(chars) == Some('/') => {
                skip_line_comment(chars);
            }
            '/' if peek_next(chars) == Some('*') => {
                skip_block_comment(chars)?;
            }
            '/' if can_start_regex => {
                skip_regex_literal(chars)?;
                can_start_regex = false;
            }
            '{' | '[' | '(' => {
                depth += 1;
                chars.next();
                can_start_regex = true;
            }
            '}' | ']' | ')' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                chars.next();
                can_start_regex = false;
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if is_identifier_start(c) => {
                let word = consume_identifier(chars);
                can_start_regex = matches!(
                    word.as_str(),
                    "return"
                        | "throw"
                        | "case"
                        | "delete"
                        | "void"
                        | "typeof"
                        | "yield"
                        | "await"
                        | "new"
                        | "in"
                        | "of"
                        | "instanceof"
                );
            }
            c if c.is_ascii_digit() => {
                consume_number_like(chars);
                can_start_regex = false;
            }
            _ => {
                chars.next();
                can_start_regex = matches!(
                    c,
                    '=' | ':'
                        | ','
                        | ';'
                        | '!'
                        | '?'
                        | '&'
                        | '|'
                        | '+'
                        | '-'
                        | '*'
                        | '%'
                        | '~'
                        | '^'
                        | '<'
                        | '>'
                );
            }
        }
    }

    Ok("null".to_string())
}

fn peek_next(chars: &Peekable<Chars>) -> Option<char> {
    let mut clone = chars.clone();
    clone.next();
    clone.next()
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_identifier_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn consume_identifier(chars: &mut Peekable<Chars>) -> String {
    let mut word = String::new();
    while let Some(&c) = chars.peek() {
        if is_identifier_part(c) {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    word
}

fn consume_number_like(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-') {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_escape_sequence(chars: &mut Peekable<Chars>) -> ParseResult<Option<char>> {
    let Some(c) = chars.next() else {
        return Ok(Some('\\'));
    };

    match c {
        '"' | '\'' | '\\' | '/' => Ok(Some(c)),
        '0' => Ok(Some('\0')),
        'b' => Ok(Some('\u{0008}')),
        'f' => Ok(Some('\u{000c}')),
        'n' => Ok(Some('\n')),
        'r' => Ok(Some('\r')),
        't' => Ok(Some('\t')),
        'v' => Ok(Some('\u{000b}')),
        'u' => parse_unicode_escape(chars),
        'x' => Ok(Some(parse_hex_escape(chars, 2)?)),
        '\r' => {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            Ok(None)
        }
        '\n' => Ok(None),
        _ => Ok(Some(c)),
    }
}

fn parse_unicode_escape(chars: &mut Peekable<Chars>) -> ParseResult<Option<char>> {
    let value = parse_hex_value(chars, 4)?;

    if (0xd800..=0xdbff).contains(&value) {
        let Some(low) = parse_low_surrogate(chars)? else {
            return Err("Missing low surrogate after high surrogate escape".into());
        };
        let code_point = 0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00);
        return Ok(Some(char::from_u32(code_point).ok_or_else(|| {
            format!("Invalid unicode escape: {code_point:x}")
        })?));
    }

    if (0xdc00..=0xdfff).contains(&value) {
        return Err("Unexpected low surrogate escape".into());
    }

    Ok(Some(char::from_u32(value).ok_or_else(|| {
        format!("Invalid unicode escape: {value:x}")
    })?))
}

fn parse_low_surrogate(chars: &mut Peekable<Chars>) -> ParseResult<Option<u32>> {
    if chars.peek() != Some(&'\\') || peek_next(chars) != Some('u') {
        return Ok(None);
    }

    chars.next();
    chars.next();

    let value = parse_hex_value(chars, 4)?;
    if !(0xdc00..=0xdfff).contains(&value) {
        return Err("Invalid low surrogate escape".into());
    }

    Ok(Some(value))
}

fn parse_hex_escape(chars: &mut Peekable<Chars>, digits: usize) -> ParseResult<char> {
    let value = parse_hex_value(chars, digits)?;
    char::from_u32(value).ok_or_else(|| format!("Invalid hex escape: {value:x}").into())
}

fn parse_hex_value(chars: &mut Peekable<Chars>, digits: usize) -> ParseResult<u32> {
    let mut value = 0;
    for _ in 0..digits {
        let Some(c) = chars.next() else {
            return Err("Unexpected end of unicode escape".into());
        };
        let Some(digit) = c.to_digit(16) else {
            return Err(format!("Invalid hex escape character: {c}").into());
        };
        value = value * 16 + digit;
    }

    Ok(value)
}

fn skip_quoted_string(chars: &mut Peekable<Chars>, quote: char) -> ParseResult<()> {
    chars.next();

    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c == quote {
            return Ok(());
        }
    }

    Err("Unterminated string literal".into())
}

fn skip_template_literal(chars: &mut Peekable<Chars>) -> ParseResult<()> {
    chars.next();

    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c == '`' {
            return Ok(());
        } else if c == '$' && chars.peek() == Some(&'{') {
            chars.next();
            skip_template_expression(chars)?;
        }
    }

    Err("Unterminated template literal".into())
}

fn skip_template_expression(chars: &mut Peekable<Chars>) -> ParseResult<()> {
    let mut depth = 1;
    let mut can_start_regex = true;

    while let Some(&c) = chars.peek() {
        match c {
            '"' | '\'' => {
                skip_quoted_string(chars, c)?;
                can_start_regex = false;
            }
            '`' => {
                skip_template_literal(chars)?;
                can_start_regex = false;
            }
            '/' if peek_next(chars) == Some('/') => {
                skip_line_comment(chars);
            }
            '/' if peek_next(chars) == Some('*') => {
                skip_block_comment(chars)?;
            }
            '/' if can_start_regex => {
                skip_regex_literal(chars)?;
                can_start_regex = false;
            }
            '{' | '[' | '(' => {
                depth += 1;
                chars.next();
                can_start_regex = true;
            }
            '}' | ']' | ')' => {
                depth -= 1;
                chars.next();
                if depth == 0 {
                    return Ok(());
                }
                can_start_regex = false;
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if is_identifier_start(c) => {
                let word = consume_identifier(chars);
                can_start_regex = matches!(
                    word.as_str(),
                    "return"
                        | "throw"
                        | "case"
                        | "delete"
                        | "void"
                        | "typeof"
                        | "yield"
                        | "await"
                        | "new"
                        | "in"
                        | "of"
                        | "instanceof"
                );
            }
            c if c.is_ascii_digit() => {
                consume_number_like(chars);
                can_start_regex = false;
            }
            _ => {
                chars.next();
                can_start_regex = matches!(
                    c,
                    '=' | ':'
                        | ','
                        | ';'
                        | '!'
                        | '?'
                        | '&'
                        | '|'
                        | '+'
                        | '-'
                        | '*'
                        | '%'
                        | '~'
                        | '^'
                        | '<'
                        | '>'
                );
            }
        }
    }

    Err("Unterminated template expression".into())
}

fn skip_regex_literal(chars: &mut Peekable<Chars>) -> ParseResult<()> {
    chars.next();
    let mut in_class = false;

    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c == '[' {
            in_class = true;
        } else if c == ']' {
            in_class = false;
        } else if c == '/' && !in_class {
            while let Some(&flag) = chars.peek() {
                if flag.is_alphabetic() {
                    chars.next();
                } else {
                    break;
                }
            }
            return Ok(());
        }
    }

    Err("Unterminated regex literal".into())
}

fn skip_line_comment(chars: &mut Peekable<Chars>) {
    chars.next();
    chars.next();

    for c in chars.by_ref() {
        if c == '\n' {
            break;
        }
    }
}

fn skip_block_comment(chars: &mut Peekable<Chars>) -> ParseResult<()> {
    chars.next();
    chars.next();

    let mut previous = '\0';
    for c in chars.by_ref() {
        if previous == '*' && c == '/' {
            return Ok(());
        }
        previous = c;
    }

    Err("Unterminated block comment".into())
}
