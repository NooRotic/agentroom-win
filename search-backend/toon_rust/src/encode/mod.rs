pub mod encoders;
pub mod folding;
pub mod normalize;
pub mod primitives;
pub mod replacer;

#[cfg(feature = "async-stream")]
pub mod async_encode;

#[cfg(feature = "async-stream")]
pub use async_encode::{
    AsyncEncodeEventStream, AsyncEncodeStream, encode_async, encode_events_async,
    encode_lines_async,
};

use crate::encode::normalize::normalize_json_value;
use crate::encode::replacer::apply_replacer;
use crate::options::{EncodeOptions, resolve_encode_options};
use crate::shared::validation::is_valid_unquoted_key;
use crate::{JsonStreamEvent, JsonValue};

pub fn encode(input: impl Into<JsonValue>, options: Option<EncodeOptions>) -> String {
    let lines = encode_lines(input, options);
    lines.join("\n")
}

pub fn encode_lines(input: impl Into<JsonValue>, options: Option<EncodeOptions>) -> Vec<String> {
    let resolved = resolve_encode_options(options);
    let normalized = normalize_json_value(input.into());
    let replaced = if let Some(replacer) = &resolved.replacer {
        apply_replacer(&normalized, replacer)
    } else {
        normalized
    };
    encoders::encode_json_value(&replaced, &resolved)
}

/// Encode a JSON value into a stream of events.
///
/// This produces the same event sequence that `decode_stream_sync` would emit
/// when decoding the TOON representation of this JSON value.
#[must_use]
pub fn encode_stream_events(
    input: impl Into<JsonValue>,
    options: Option<EncodeOptions>,
) -> Vec<JsonStreamEvent> {
    let resolved = resolve_encode_options(options);
    let normalized = normalize_json_value(input.into());
    let replaced = if let Some(replacer) = &resolved.replacer {
        apply_replacer(&normalized, replacer)
    } else {
        normalized
    };

    let mut events = Vec::new();
    emit_events(&replaced, &mut events);
    events
}

fn emit_events(value: &JsonValue, events: &mut Vec<JsonStreamEvent>) {
    match value {
        JsonValue::Primitive(p) => {
            events.push(JsonStreamEvent::Primitive { value: p.clone() });
        }
        JsonValue::Array(arr) => {
            events.push(JsonStreamEvent::StartArray { length: arr.len() });
            for item in arr {
                emit_events(item, events);
            }
            events.push(JsonStreamEvent::EndArray);
        }
        JsonValue::Object(obj) => {
            events.push(JsonStreamEvent::StartObject);
            for (key, val) in obj {
                events.push(JsonStreamEvent::Key {
                    key: key.clone(),
                    was_quoted: !is_valid_unquoted_key(key),
                });
                emit_events(val, events);
            }
            events.push(JsonStreamEvent::EndObject);
        }
    }
}
