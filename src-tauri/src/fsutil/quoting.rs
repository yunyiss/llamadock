//! 引号与转义：仅在需要时给取值加引号。

/// 值含空白或引号时加双引号。
pub fn quote(value: &str) -> String {
    if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
}
