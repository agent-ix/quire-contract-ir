//! The Rust spellings of QSpec FR-450 ("Rust spellings") that an abstraction
//! relation body carries (FR-346): a Rust identifier, a path of them, a field
//! (an identifier or a tuple-field index) and a receiver (`self` or an
//! identifier).
//!
//! A Rust identifier is `IDENTIFIER` of the Rust Reference for the 2021
//! edition: a non-keyword identifier, or a raw identifier `r#` followed by an
//! identifier or keyword other than `crate`, `self`, `super`, `Self` and `_`.
//! Its content is in Unicode Normalization Form C, so two spellings of one
//! identifier are equal bytes. These are lexical keys: nothing here resolves a
//! name against Rust code.

use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_normalization::is_nfc;

/// The 2021 edition's strict and reserved keywords: an identifier may not be
/// one of them unless written raw.
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// The keywords a raw identifier may not spell (FR-450).
const NOT_RAW: [&str; 5] = ["crate", "self", "super", "Self", "_"];

/// `IDENTIFIER_OR_KEYWORD`: `XID_Start XID_Continue*`, or `_ XID_Continue+`.
fn is_identifier_or_keyword(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some('_') => text.len() > 1 && chars.all(is_xid_continue),
        Some(first) => is_xid_start(first) && chars.all(is_xid_continue),
        None => false,
    }
}

/// A Rust identifier (FR-450 `rust-name`, one segment of a `rust-path`).
pub(super) fn is_rust_identifier(text: &str) -> bool {
    if !is_nfc(text) {
        return false;
    }
    match text.strip_prefix("r#") {
        Some(raw) => is_identifier_or_keyword(raw) && !NOT_RAW.contains(&raw),
        None => is_identifier_or_keyword(text) && !KEYWORDS.contains(&text),
    }
}

/// A `rust-path`: one or more Rust identifiers.
pub(super) fn is_rust_path(segments: &[Box<str>]) -> bool {
    !segments.is_empty() && segments.iter().all(|segment| is_rust_identifier(segment))
}

/// A `rust-field`: a Rust identifier, or a tuple-field index (`0` or a
/// decimal integer with no leading zero).
pub(super) fn is_rust_field(text: &str) -> bool {
    is_rust_identifier(text) || is_tuple_index(text)
}

/// A `rust-receiver`: exactly `self`, or one Rust identifier.
pub(super) fn is_rust_receiver(text: &str) -> bool {
    text == "self" || is_rust_identifier(text)
}

fn is_tuple_index(text: &str) -> bool {
    text == "0"
        || (text.starts_with(|first: char| ('1'..='9').contains(&first))
            && text.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::{is_rust_field, is_rust_identifier, is_rust_path, is_rust_receiver};

    /// Tracing: TC-225, FR-346-AC-6
    #[test]
    fn tc_225_identifiers_are_non_keyword_or_raw_identifiers() {
        for text in ["version", "_x", "a1", "r#type", "r#match", "Straße", "r#a"] {
            assert!(is_rust_identifier(text), "{text}");
        }
        for text in [
            "", "_", "crate", "self", "Self", "super", "type", "fn", "yield", "9lives", "a b",
            "a-b", "r#crate", "r#self", "r#super", "r#Self", "r#_", "r#", "r#9", "e\u{301}",
        ] {
            assert!(!is_rust_identifier(text), "{text}");
        }
    }

    /// Tracing: TC-225, FR-346-AC-6
    #[test]
    fn tc_225_paths_fields_and_receivers_take_their_own_syntax() {
        let path = |segments: &[&str]| {
            segments
                .iter()
                .map(|segment| Box::from(*segment))
                .collect::<Vec<Box<str>>>()
        };
        assert!(is_rust_path(&path(&["config_store", "ConfigVersion"])));
        assert!(!is_rust_path(&path(&[])));
        assert!(!is_rust_path(&path(&["config_store", "crate"])));
        assert!(!is_rust_path(&path(&["9lives"])));
        for text in ["0", "7", "10", "version", "r#type"] {
            assert!(is_rust_field(text), "{text}");
        }
        for text in ["", "01", "-1", "1a", "not a name", "self"] {
            assert!(!is_rust_field(text), "{text}");
        }
        for text in ["self", "next", "r#type"] {
            assert!(is_rust_receiver(text), "{text}");
        }
        for text in ["", "Self", "crate", "a b"] {
            assert!(!is_rust_receiver(text), "{text}");
        }
    }
}
