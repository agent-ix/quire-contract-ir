//! Canonical JSON and hostile-byte preflight for bridge-owned documents.

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use super::{BridgeError, BridgeErrorCode, BridgeLimits};

/// Serializes one bridge-owned value using sorted object keys and integer-only JSON.
pub fn encode<T: Serialize>(value: &T, limits: BridgeLimits) -> Result<Vec<u8>, BridgeError> {
    let limits = limits.effective();
    let value =
        serde_json::to_value(value).map_err(|_| invalid("value is not JSON", "document"))?;
    validate_scalar_domain(&value)?;
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| resource("canonical JSON allocation failed", "document"))?;
    preflight(&bytes, limits)?;
    if bytes.len() > limits.allocation_bytes {
        return Err(resource(
            "canonical JSON exceeds the selected allocation reservation",
            "document",
        ));
    }
    Ok(bytes)
}

/// Strictly decodes one exact canonical bridge-owned document.
pub fn decode<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    limits: BridgeLimits,
) -> Result<T, BridgeError> {
    let limits = limits.effective();
    preflight(bytes, limits)?;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    decoder.disable_recursion_limit();
    let value = T::deserialize(serde_stacker::Deserializer::new(&mut decoder)).map_err(|_| {
        invalid(
            "document does not have the required closed shape",
            "document",
        )
    })?;
    decoder
        .end()
        .map_err(|_| invalid("trailing data follows the document", "document"))?;
    let canonical = encode(&value, limits)?;
    if canonical != bytes {
        return Err(invalid("document bytes are not canonical", "document"));
    }
    Ok(value)
}

/// Checks byte, depth, string, and visited-work ceilings before JSON allocation.
pub fn preflight(bytes: &[u8], limits: BridgeLimits) -> Result<(), BridgeError> {
    let limits = limits.effective();
    if bytes.len() > limits.document_bytes {
        return Err(resource("document byte ceiling exceeded", "document"));
    }
    if core::str::from_utf8(bytes).is_err() {
        return Err(invalid("document is not UTF-8", "document"));
    }
    let mut depth = 0usize;
    let mut maximum_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut string_bytes = 0usize;
    let mut maximum_string = 0usize;
    let mut work = 0usize;
    for byte in bytes.iter().copied() {
        work = work.saturating_add(1);
        if in_string {
            if escaped {
                escaped = false;
                string_bytes = string_bytes.saturating_add(1);
            } else if byte == b'\\' {
                escaped = true;
                string_bytes = string_bytes.saturating_add(1);
            } else if byte == b'"' {
                in_string = false;
                maximum_string = maximum_string.max(string_bytes);
            } else {
                string_bytes = string_bytes.saturating_add(1);
            }
            continue;
        }
        match byte {
            b'"' => {
                in_string = true;
                string_bytes = 0;
            }
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                maximum_depth = maximum_depth.max(depth);
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    if maximum_depth > limits.json_depth {
        return Err(resource("JSON depth ceiling exceeded", "document"));
    }
    if maximum_string > limits.string_bytes {
        return Err(resource("JSON string ceiling exceeded", "document"));
    }
    if work > limits.visited_work {
        return Err(resource("visited-work ceiling exceeded", "document"));
    }
    Ok(())
}

fn validate_scalar_domain(root: &Value) -> Result<(), BridgeError> {
    let mut stack = vec![root];
    while let Some(value) = stack.pop() {
        match value {
            Value::Null => return Err(invalid("null is outside the bridge profile", "document")),
            Value::Number(number) if !(number.is_i64() || number.is_u64()) => {
                return Err(invalid("floating-point JSON is forbidden", "document"));
            }
            Value::Array(values) => stack.extend(values),
            Value::Object(values) => stack.extend(values.values()),
            Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    Ok(())
}

fn invalid(message: &'static str, path: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::InvalidNativePredicateProjection,
        message,
        path,
    )
}

fn resource(message: &'static str, path: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::PredicateProjectionResourceExhausted,
        message,
        path,
    )
}
