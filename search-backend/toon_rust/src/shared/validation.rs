use crate::shared::constants::{DEFAULT_DELIMITER, LIST_ITEM_MARKER};
use crate::shared::literal_utils::{is_boolean_or_null_literal, is_numeric_like};

#[must_use]
pub fn is_valid_unquoted_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    for ch in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            continue;
        }
        return false;
    }

    true
}

#[must_use]
pub fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    for ch in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            continue;
        }
        return false;
    }

    true
}

#[must_use]
pub fn is_safe_unquoted(value: &str, delimiter: char) -> bool {
    if value.is_empty() {
        return false;
    }

    if value.trim() != value {
        return false;
    }

    if is_boolean_or_null_literal(value) || is_numeric_like(value) {
        return false;
    }

    if value.contains(':') {
        return false;
    }

    if value.contains('"') || value.contains('\\') {
        return false;
    }

    if value.contains('[') || value.contains(']') || value.contains('{') || value.contains('}') {
        return false;
    }

    if value.contains('\n') || value.contains('\r') || value.contains('\t') {
        return false;
    }

    if value.contains(delimiter) {
        return false;
    }

    if value.starts_with(LIST_ITEM_MARKER) {
        return false;
    }

    true
}

#[must_use]
pub const fn default_delimiter() -> char {
    DEFAULT_DELIMITER
}
