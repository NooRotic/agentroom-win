use std::sync::Arc;

use crate::JsonValue;
use crate::shared::constants::DEFAULT_DELIMITER;

pub type EncodeReplacer =
    Arc<dyn Fn(&str, &JsonValue, &[PathSegment]) -> Option<JsonValue> + Send + Sync>;

#[derive(Clone)]
pub struct EncodeOptions {
    pub indent: Option<usize>,
    pub delimiter: Option<char>,
    pub key_folding: Option<KeyFoldingMode>,
    pub flatten_depth: Option<usize>,
    pub replacer: Option<EncodeReplacer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFoldingMode {
    Off,
    Safe,
}

#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub indent: Option<usize>,
    pub strict: Option<bool>,
    pub expand_paths: Option<ExpandPathsMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandPathsMode {
    Off,
    Safe,
}

#[derive(Debug, Clone, Default)]
pub struct DecodeStreamOptions {
    pub indent: Option<usize>,
    pub strict: Option<bool>,
}

#[derive(Clone)]
pub struct ResolvedEncodeOptions {
    pub indent: usize,
    pub delimiter: char,
    pub key_folding: KeyFoldingMode,
    pub flatten_depth: usize,
    pub replacer: Option<EncodeReplacer>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDecodeOptions {
    pub indent: usize,
    pub strict: bool,
    pub expand_paths: ExpandPathsMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[must_use]
pub fn resolve_encode_options(options: Option<EncodeOptions>) -> ResolvedEncodeOptions {
    let options = options.unwrap_or(EncodeOptions {
        indent: None,
        delimiter: None,
        key_folding: None,
        flatten_depth: None,
        replacer: None,
    });

    ResolvedEncodeOptions {
        indent: options.indent.unwrap_or(2),
        delimiter: options.delimiter.unwrap_or(DEFAULT_DELIMITER),
        key_folding: options.key_folding.unwrap_or(KeyFoldingMode::Off),
        flatten_depth: options.flatten_depth.unwrap_or(usize::MAX),
        replacer: options.replacer,
    }
}

#[must_use]
pub fn resolve_decode_options(options: Option<DecodeOptions>) -> ResolvedDecodeOptions {
    let options = options.unwrap_or(DecodeOptions {
        indent: None,
        strict: None,
        expand_paths: None,
    });

    ResolvedDecodeOptions {
        indent: options.indent.unwrap_or(2),
        strict: options.strict.unwrap_or(true),
        expand_paths: options.expand_paths.unwrap_or(ExpandPathsMode::Off),
    }
}
