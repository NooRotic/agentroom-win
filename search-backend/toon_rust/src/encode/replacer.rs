use crate::encode::normalize::normalize_json_value;
use crate::options::{EncodeReplacer, PathSegment};
use crate::{JsonArray, JsonObject, JsonValue};

pub fn apply_replacer(root: &JsonValue, replacer: &EncodeReplacer) -> JsonValue {
    let replaced_root = replacer("", root, &[]);
    if let Some(value) = replaced_root {
        let normalized = normalize_json_value(value);
        return transform_children(normalized, replacer, &[]);
    }

    transform_children(root.clone(), replacer, &[])
}

fn transform_children(
    value: JsonValue,
    replacer: &EncodeReplacer,
    path: &[PathSegment],
) -> JsonValue {
    match value {
        JsonValue::Object(entries) => JsonValue::Object(transform_object(entries, replacer, path)),
        JsonValue::Array(values) => JsonValue::Array(transform_array(values, replacer, path)),
        JsonValue::Primitive(value) => JsonValue::Primitive(value),
    }
}

fn transform_object(
    entries: JsonObject,
    replacer: &EncodeReplacer,
    path: &[PathSegment],
) -> JsonObject {
    let mut result = Vec::new();

    for (key, value) in entries {
        let mut next_path = path.to_vec();
        next_path.push(PathSegment::Key(key.clone()));

        let replacement = replacer(&key, &value, &next_path);
        if let Some(next_value) = replacement {
            let normalized = normalize_json_value(next_value);
            let transformed = transform_children(normalized, replacer, &next_path);
            result.push((key, transformed));
        }
    }

    result
}

fn transform_array(
    values: JsonArray,
    replacer: &EncodeReplacer,
    path: &[PathSegment],
) -> JsonArray {
    let mut result = Vec::new();

    for (idx, value) in values.into_iter().enumerate() {
        let mut next_path = path.to_vec();
        next_path.push(PathSegment::Index(idx));

        let key = idx.to_string();
        let replacement = replacer(&key, &value, &next_path);
        if let Some(next_value) = replacement {
            let normalized = normalize_json_value(next_value);
            let transformed = transform_children(normalized, replacer, &next_path);
            result.push(transformed);
        }
    }

    result
}
