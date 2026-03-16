use std::collections::HashSet;

use crate::error::{Result, ToonError};
use crate::{JsonPrimitive, JsonStreamEvent, JsonValue};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeValue {
    Primitive(JsonPrimitive),
    Array(Vec<Self>),
    Object(ObjectNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectNode {
    pub entries: Vec<(String, NodeValue)>,
    pub quoted_keys: HashSet<String>,
}

#[derive(Debug, Clone)]
enum BuildContext {
    Object {
        entries: Vec<(String, NodeValue)>,
        current_key: Option<String>,
        quoted_keys: HashSet<String>,
    },
    Array {
        items: Vec<NodeValue>,
    },
}

#[derive(Debug, Clone)]
struct BuildState {
    stack: Vec<BuildContext>,
    root: Option<NodeValue>,
}

/// Build a decoded node tree from a stream of events.
///
/// # Errors
///
/// Returns an error if the event stream is malformed (mismatched start/end
/// events, missing keys, or incomplete stacks).
pub fn build_node_from_events(
    events: impl IntoIterator<Item = JsonStreamEvent>,
) -> Result<NodeValue> {
    let mut state = BuildState {
        stack: Vec::new(),
        root: None,
    };

    for event in events {
        apply_event(&mut state, event)?;
    }

    finalize_state(state)
}

pub fn node_to_json(value: NodeValue) -> JsonValue {
    match value {
        NodeValue::Primitive(value) => JsonValue::Primitive(value),
        NodeValue::Array(items) => JsonValue::Array(items.into_iter().map(node_to_json).collect()),
        NodeValue::Object(obj) => JsonValue::Object(
            obj.entries
                .into_iter()
                .map(|(key, value)| (key, node_to_json(value)))
                .collect(),
        ),
    }
}

#[allow(clippy::too_many_lines)]
fn apply_event(state: &mut BuildState, event: JsonStreamEvent) -> Result<()> {
    match event {
        JsonStreamEvent::StartObject => {
            state.stack.push(BuildContext::Object {
                entries: Vec::new(),
                current_key: None,
                quoted_keys: HashSet::new(),
            });
        }
        JsonStreamEvent::EndObject => {
            let Some(context) = state.stack.pop() else {
                return Err(ToonError::unexpected_event("endObject", "with empty stack"));
            };
            let BuildContext::Object {
                entries,
                quoted_keys,
                ..
            } = context
            else {
                return Err(ToonError::mismatched_end("Object", "Array"));
            };
            let node = NodeValue::Object(ObjectNode {
                entries,
                quoted_keys,
            });
            if let Some(parent) = state.stack.last_mut() {
                match parent {
                    BuildContext::Object {
                        entries,
                        current_key,
                        ..
                    } => {
                        let Some(key) = current_key.take() else {
                            return Err(ToonError::message(
                                "Object endObject event without preceding key",
                            ));
                        };
                        entries.push((key, node));
                    }
                    BuildContext::Array { items } => {
                        items.push(node);
                    }
                }
            } else {
                state.root = Some(node);
            }
        }
        JsonStreamEvent::StartArray { .. } => {
            state.stack.push(BuildContext::Array { items: Vec::new() });
        }
        JsonStreamEvent::EndArray => {
            let Some(context) = state.stack.pop() else {
                return Err(ToonError::unexpected_event("endArray", "with empty stack"));
            };
            let BuildContext::Array { items } = context else {
                return Err(ToonError::mismatched_end("Array", "Object"));
            };
            let node = NodeValue::Array(items);
            if let Some(parent) = state.stack.last_mut() {
                match parent {
                    BuildContext::Object {
                        entries,
                        current_key,
                        ..
                    } => {
                        let Some(key) = current_key.take() else {
                            return Err(ToonError::message(
                                "Array endArray event without preceding key",
                            ));
                        };
                        entries.push((key, node));
                    }
                    BuildContext::Array { items } => {
                        items.push(node);
                    }
                }
            } else {
                state.root = Some(node);
            }
        }
        JsonStreamEvent::Key { key, was_quoted } => {
            let Some(BuildContext::Object {
                current_key,
                quoted_keys,
                ..
            }) = state.stack.last_mut()
            else {
                return Err(ToonError::unexpected_event(
                    "Key",
                    "outside of object context",
                ));
            };
            *current_key = Some(key.clone());
            if was_quoted {
                quoted_keys.insert(key);
            }
        }
        JsonStreamEvent::Primitive { value } => {
            if state.stack.is_empty() {
                state.root = Some(NodeValue::Primitive(value));
                return Ok(());
            }

            match state.stack.last_mut() {
                Some(BuildContext::Object {
                    entries,
                    current_key,
                    ..
                }) => {
                    let Some(key) = current_key.take() else {
                        return Err(ToonError::message(
                            "Primitive event without preceding key in object",
                        ));
                    };
                    entries.push((key, NodeValue::Primitive(value)));
                }
                Some(BuildContext::Array { items }) => {
                    items.push(NodeValue::Primitive(value));
                }
                None => {}
            }
        }
    }

    Ok(())
}

fn finalize_state(state: BuildState) -> Result<NodeValue> {
    if !state.stack.is_empty() {
        return Err(ToonError::event_stream(
            "Incomplete event stream: stack not empty at end",
        ));
    }

    state
        .root
        .ok_or_else(|| ToonError::event_stream("No root value built from events"))
}
