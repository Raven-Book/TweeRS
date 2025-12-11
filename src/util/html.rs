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
}
