use std::fmt::Write;

use crate::JsonPrimitive;
use crate::StringOrNumberOrBoolOrNull;
use crate::shared::constants::{DEFAULT_DELIMITER, DOUBLE_QUOTE};
use crate::shared::string_utils::escape_string;
use crate::shared::validation::{is_safe_unquoted, is_valid_unquoted_key};

#[must_use]
pub fn encode_primitive(value: &JsonPrimitive, delimiter: char) -> String {
    match value {
        StringOrNumberOrBoolOrNull::Null => "null".to_string(),
        StringOrNumberOrBoolOrNull::Bool(value) => value.to_string(),
        StringOrNumberOrBoolOrNull::Number(value) => format_number(*value),
        StringOrNumberOrBoolOrNull::String(value) => encode_string_literal(value, delimiter),
    }
}

#[must_use]
pub fn encode_string_literal(value: &str, delimiter: char) -> String {
    if is_safe_unquoted(value, delimiter) {
        return value.to_string();
    }
    format!("{DOUBLE_QUOTE}{}{DOUBLE_QUOTE}", escape_string(value))
}

#[must_use]
pub fn encode_key(key: &str) -> String {
    if is_valid_unquoted_key(key) {
        return key.to_string();
    }
    format!("{DOUBLE_QUOTE}{}{DOUBLE_QUOTE}", escape_string(key))
}

#[must_use]
pub fn encode_and_join_primitives(values: &[JsonPrimitive], delimiter: char) -> String {
    if values.is_empty() {
        return String::new();
    }
    // Estimate: average 10 chars per primitive + delimiter
    let mut out = String::with_capacity(values.len() * 11);
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(delimiter);
        }
        out.push_str(&encode_primitive(value, delimiter));
    }
    out
}

#[must_use]
pub fn format_header(
    length: usize,
    key: Option<&str>,
    fields: Option<&[String]>,
    delimiter: char,
) -> String {
    let mut header = String::new();

    if let Some(key) = key {
        header.push_str(&encode_key(key));
    }

    if delimiter == DEFAULT_DELIMITER {
        let _ = write!(header, "[{length}]");
    } else {
        let _ = write!(header, "[{length}{delimiter}]");
    }

    if let Some(fields) = fields {
        header.push('{');
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                header.push(delimiter);
            }
            header.push_str(&encode_key(field));
        }
        header.push('}');
    }

    header.push(':');
    header
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value.is_nan() || !value.is_finite() {
        return "null".to_string();
    }
    value.to_string()
}
