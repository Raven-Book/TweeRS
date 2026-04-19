/// HTML escaping utilities
pub struct HtmlEscape;

impl HtmlEscape {
    /// Escape HTML content (for text content inside tags)
    /// Escapes: &, <, >, ", '
    pub fn escape_content(text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
    }

    /// Escape HTML attribute values
    /// Escapes: &, <, >, ", '
    /// Note: For attribute values, both " and ' should be escaped
    pub fn escape_attribute(text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
    }

    /// Unescape HTML entities commonly found in text content and attribute values.
    pub fn unescape(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            if ch != '&' {
                result.push(ch);
                continue;
            }

            let mut end = start + ch.len_utf8();
            let mut found_semicolon = false;

            while let Some((idx, next_ch)) = chars.peek().copied() {
                if next_ch == ';' {
                    end = idx;
                    chars.next();
                    found_semicolon = true;
                    break;
                }

                if next_ch.is_whitespace() || next_ch == '&' {
                    break;
                }

                end = idx + next_ch.len_utf8();
                chars.next();

                if end - start > 32 {
                    break;
                }
            }

            if !found_semicolon {
                result.push('&');
                if end > start + 1 {
                    result.push_str(&text[start + 1..end]);
                }
                continue;
            }

            let entity = &text[start + 1..end];
            if let Some(decoded) = Self::decode_entity(entity) {
                result.push_str(&decoded);
            } else {
                result.push('&');
                result.push_str(entity);
                result.push(';');
            }
        }

        result
    }

    fn decode_entity(entity: &str) -> Option<String> {
        match entity {
            "amp" => Some("&".to_string()),
            "lt" => Some("<".to_string()),
            "gt" => Some(">".to_string()),
            "quot" => Some("\"".to_string()),
            "apos" | "#39" => Some("'".to_string()),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                u32::from_str_radix(&entity[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
            }
            _ if entity.starts_with('#') => entity[1..]
                .parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlEscape;

    #[test]
    fn test_unescape_named_and_numeric_entities() {
        let text = "&lt;div&gt;Tom &amp; Jerry&#39;s &#x4F60;&#22909;&lt;/div&gt;";
        assert_eq!(HtmlEscape::unescape(text), "<div>Tom & Jerry's 你好</div>");
    }
}
