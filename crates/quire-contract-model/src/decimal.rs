//! The decimal-string spelling of the eight integer members (FR-013-AC-5).
//!
//! `IntegerType.minimum`/`maximum`, `RationalType.numerator_minimum`/
//! `numerator_maximum`/`maximum_denominator`, `IntegerLiteral.value` and the
//! rational literal's `numerator`/`denominator` travel as JSON strings of
//! minimal base-ten digits, grammar `^(0|-?[1-9][0-9]*)$`, because a JSON
//! number past 2^53 is not one RFC 8785 can spell exactly and
//! `quire-canonical` refuses it. Every other integer member that stays a
//! number (`maximum_items`, `revision`, a byte offset) is bounded at 2^53.

use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serializer,
};

use crate::{Diagnostic, DiagnosticCode};

/// Serializes an integer member as its decimal string.
pub(crate) fn serialize_decimal<T: fmt::Display, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

/// One of the eight integer members as it arrives on the wire: a string that
/// satisfies the grammar and has not yet been given a range. The decoder
/// accepts a string and nothing else, so a JSON number, a boolean or a
/// string outside the grammar is refused by the member's own type with no
/// separate scan of the numeral; the range check belongs to the model type
/// that owns the member (`invalid_numeric_bounds`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntegerString(String);

impl IntegerString {
    /// The member as an `i128`, or `invalid_numeric_bounds` at `path` when the
    /// grammar-valid string is out of range.
    pub(crate) fn to_i128(&self, path: &'static str) -> Result<i128, Diagnostic> {
        self.0.parse().map_err(|_| out_of_range(path))
    }
}

fn out_of_range(path: &'static str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidNumericBounds,
        "integer member is outside its range",
        path,
    )
}

/// Whether `text` is `0` or minimal base-ten digits with an optional leading
/// minus and no leading zero.
fn in_grammar(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    match digits.as_bytes() {
        [b'0'] => text.len() == 1,
        [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

impl<'de> Deserialize<'de> for IntegerString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DecimalVisitor;
        impl Visitor<'_> for DecimalVisitor {
            type Value = IntegerString;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a decimal integer string of the form ^(0|-?[1-9][0-9]*)$")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<IntegerString, E> {
                if in_grammar(value) {
                    Ok(IntegerString(value.to_owned()))
                } else {
                    Err(E::invalid_value(de::Unexpected::Str(value), &self))
                }
            }
        }
        deserializer.deserialize_str(DecimalVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tracing: TC-016, FR-013-AC-5.
    #[ix_trace_rs::trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_the_grammar_is_minimal_base_ten() {
        for accepted in [
            "0",
            "1",
            "-1",
            "10",
            "9223372036854775807",
            "-9223372036854775808",
        ] {
            assert!(in_grammar(accepted), "{accepted}");
        }
        for refused in [
            "", "-", "+1", "01", "-0", "-01", "1.0", "1e3", " 1", "1 ", "0x1", "--1",
        ] {
            assert!(!in_grammar(refused), "{refused:?}");
        }
    }

    /// Tracing: TC-016, FR-013-AC-5.
    #[ix_trace_rs::trace("TC-016", "FR-013-AC-5")]
    #[test]
    fn tc_016_range_is_decided_after_the_grammar() {
        let in_range: IntegerString = serde_json::from_str("\"-9223372036854775808\"").unwrap();
        assert_eq!(in_range.to_i128("p").unwrap(), i64::MIN as i128);
        let above: IntegerString = serde_json::from_str("\"9223372036854775808\"").unwrap();
        assert_eq!(above.to_i128("p").unwrap(), 9_223_372_036_854_775_808);
        let outside: IntegerString =
            serde_json::from_str("\"170141183460469231731687303715884105728\"").unwrap();
        assert_eq!(
            outside.to_i128("p").unwrap_err().code,
            DiagnosticCode::InvalidNumericBounds
        );
        for number in [
            "0",
            "1",
            "1.0",
            "9223372036854775807",
            "100000000000000000001",
            "true",
        ] {
            assert!(
                serde_json::from_str::<IntegerString>(number).is_err(),
                "{number}"
            );
        }
    }
}
