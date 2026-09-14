/// Maximum decoded semantic nodes accepted by one operation.
pub const MAX_SEMANTIC_NODES: u32 = 25_000;
/// Maximum recursive semantic depth accepted by one operation.
pub const MAX_SEMANTIC_DEPTH: u32 = 256;
/// Maximum entries accepted in one semantic collection.
pub const MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10_000;
/// Maximum syntactic JSON nesting accepted before deserialization.
pub const MAX_WIRE_JSON_DEPTH: u32 = MAX_SEMANTIC_DEPTH * 2 + 64;

pub(crate) fn json_nesting_depth(bytes: &[u8]) -> u32 {
    let mut depth = 0_u32;
    let mut maximum = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                maximum = maximum.max(depth);
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    maximum
}

pub(crate) fn json_nesting_exceeds(bytes: &[u8], maximum: u32) -> bool {
    json_nesting_depth(bytes) > maximum
}
