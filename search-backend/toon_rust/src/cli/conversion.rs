use crate::cli::json_stream::json_stream_from_events;
use crate::cli::json_stringify::json_stringify_lines;
use crate::decode::decoders as decoder_impl;
use crate::decode::event_builder::{build_node_from_events, node_to_json};
use crate::decode::expand::expand_paths_safe;
use crate::error::{Result, ToonError};
use crate::options::{
    DecodeOptions, DecodeStreamOptions, EncodeOptions, ExpandPathsMode, resolve_decode_options,
};
use crate::{JsonValue, StringOrNumberOrBoolOrNull};

/// Encode JSON input to TOON lines.
///
/// # Errors
///
/// Returns an error if the JSON input is invalid.
pub fn encode_to_toon_lines(
    input_json: &str,
    options: Option<EncodeOptions>,
) -> Result<Vec<String>> {
    let value: serde_json::Value =
        serde_json::from_str(input_json).map_err(|err| ToonError::json_parse(&err))?;
    let converted = JsonValue::from(value);
    Ok(crate::encode::encode_lines(converted, options))
}

/// Decode TOON input into JSON output chunks.
///
/// # Errors
///
/// Returns an error if decoding fails or strict validation errors occur.
pub fn decode_to_json_chunks(input: &str, options: Option<DecodeOptions>) -> Result<Vec<String>> {
    let resolved = resolve_decode_options(options);

    if resolved.expand_paths == ExpandPathsMode::Safe {
        let value = decode_to_value(input, &resolved)?;
        return Ok(json_stringify_lines(&value, resolved.indent));
    }

    let events = decode_events(input, resolved.indent, resolved.strict)?;
    json_stream_from_events(events, resolved.indent)
}

fn decode_events(input: &str, indent: usize, strict: bool) -> Result<Vec<crate::JsonStreamEvent>> {
    let lines = input
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();

    decoder_impl::decode_stream_sync(
        lines,
        Some(DecodeStreamOptions {
            indent: Some(indent),
            strict: Some(strict),
        }),
    )
}

fn decode_to_value(
    input: &str,
    options: &crate::options::ResolvedDecodeOptions,
) -> Result<JsonValue> {
    let events = decode_events(input, options.indent, options.strict)?;
    let mut node = build_node_from_events(events)?;

    if options.expand_paths == ExpandPathsMode::Safe {
        node = expand_paths_safe(node, options.strict)?;
    }

    Ok(node_to_json(node))
}

#[must_use]
pub fn json_stringify_null(indent: usize) -> Vec<String> {
    json_stringify_lines(
        &JsonValue::Primitive(StringOrNumberOrBoolOrNull::Null),
        indent,
    )
}
