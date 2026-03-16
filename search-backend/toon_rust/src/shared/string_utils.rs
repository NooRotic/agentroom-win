use crate::shared::constants::{BACKSLASH, CARRIAGE_RETURN, DOUBLE_QUOTE, NEWLINE, TAB};

#[must_use]
pub fn escape_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => {
                out.push(BACKSLASH);
                out.push(BACKSLASH);
            }
            '"' => {
                out.push(BACKSLASH);
                out.push(DOUBLE_QUOTE);
            }
            '\n' => {
                out.push(BACKSLASH);
                out.push('n');
            }
            '\r' => {
                out.push(BACKSLASH);
                out.push('r');
            }
            '\t' => {
                out.push(BACKSLASH);
                out.push('t');
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Unescape a string literal body.
///
/// # Errors
///
/// Returns an error when the input contains invalid escape sequences or ends
/// with a trailing backslash.
pub fn unescape_string(value: &str) -> Result<String, String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch == BACKSLASH {
            let next = chars
                .next()
                .ok_or_else(|| "Invalid escape sequence: backslash at end of string".to_string())?;
            match next {
                'n' => out.push(NEWLINE),
                't' => out.push(TAB),
                'r' => out.push(CARRIAGE_RETURN),
                '\\' => out.push(BACKSLASH),
                '"' => out.push(DOUBLE_QUOTE),
                other => {
                    return Err(format!("Invalid escape sequence: \\{other}"));
                }
            }
        } else {
            out.push(ch);
        }
    }

    Ok(out)
}

#[must_use]
pub fn find_closing_quote(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == BACKSLASH as u8 && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if bytes[i] == DOUBLE_QUOTE as u8 {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[must_use]
pub fn find_unquoted_char(content: &str, target: char, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut i = start;
    let mut in_quotes = false;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_quotes && ch == BACKSLASH && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if ch == DOUBLE_QUOTE {
            in_quotes = !in_quotes;
            i += 1;
            continue;
        }
        if ch == target && !in_quotes {
            return Some(i);
        }
        i += 1;
    }
    None
}
