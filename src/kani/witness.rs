//! Evaluated Kani concrete-playback witnesses.
//!
//! A `kani::concrete_playback_run` block is the only artifact Kani retains
//! that actually reproduces a falsified check: the untyped bytes it hands to
//! `kani::any()` in declaration order. This module parses that fenced block
//! into a typed [`Witness`] and joins its untyped bytes with the schema a
//! generator declared, refusing rather than guessing whenever the two
//! disagree.

use serde::{Deserialize, Serialize};

use super::{KaniOutcome, KaniOutcomeKind};

/// The Kani check kind a playback block was generated for.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessCheck {
    /// `Check for \`assertion\`` — a falsified assertion. The only check kind
    /// that witnesses falsity.
    Assertion,
    /// `Check for \`cover\`` — a reached cover statement. Witnesses
    /// reachability, never falsity.
    Cover,
    /// Any other declared check kind Kani may emit; parsed but not further
    /// classified.
    Other,
}

/// The declared primitive type of one nondeterministic `kani::any()` binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessValueType {
    /// A one-byte boolean.
    Boolean,
    /// An eight-byte little-endian signed integer.
    I64,
}

impl WitnessValueType {
    /// The exact byte width Kani's concrete playback encodes for this type.
    const fn byte_width(self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::I64 => 8,
        }
    }
}

/// One declared `kani::any()` binding, in the harness's call order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WitnessBinding {
    /// The generator's identifier for this binding.
    pub identifier: String,
    /// The declared primitive type of the binding.
    pub value_type: WitnessValueType,
}

/// One decoded concrete value, typed by the schema that named it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WitnessValue {
    /// A decoded boolean.
    Boolean(bool),
    /// A decoded signed 64-bit integer.
    Integer(i64),
}

/// The evaluated witness Kani retained for one falsified check.
///
/// `concrete_values` is deliberately untyped: Kani's playback block carries
/// raw bytes with no schema of its own. [`Witness::decode`] is the only place
/// that assigns them meaning, and it does so against a schema the generator
/// supplies rather than by inference.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Witness {
    /// The harness symbol the playback block names.
    pub harness_symbol: String,
    /// The check kind the playback block was generated for.
    pub check: WitnessCheck,
    /// The exact, possibly multi-line, text of the `Check for` clause.
    pub check_text: String,
    /// Untyped concrete bytes, one entry per `kani::any()` call, in
    /// declaration order.
    pub concrete_values: Vec<Vec<u8>>,
    /// The exact retained playback block text, kept as the single source of
    /// truth for [`Witness::decode`]'s cross-check against Kani's own
    /// decoded-value comments.
    pub transcript: String,
}

impl Witness {
    /// Parses the fenced playback block Kani emits in a `counterexample`
    /// string. A cover playback is refused: it witnesses reachability, not
    /// falsity.
    pub fn parse(source_id: &str, context: &str, block: &str) -> Result<Self, KaniOutcome> {
        let refuse = |code: &'static str| {
            KaniOutcome::non_success(KaniOutcomeKind::InvalidInput, code, source_id, context)
        };

        let harness_symbol = extract_delimited(block, "Test generated for harness `", '`')
            .ok_or_else(|| refuse("kani_witness_harness_missing"))?
            .to_owned();

        let check_marker = "Check for `";
        let check_start = block
            .find(check_marker)
            .ok_or_else(|| refuse("kani_witness_check_missing"))?;
        let after_marker = &block[check_start + check_marker.len()..];
        let kind_end = after_marker
            .find('`')
            .ok_or_else(|| refuse("kani_witness_check_missing"))?;
        let kind_text = &after_marker[..kind_end];
        let check = match kind_text {
            "assertion" => WitnessCheck::Assertion,
            "cover" => {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_cover_refused",
                    source_id,
                    context,
                ));
            }
            _ => WitnessCheck::Other,
        };

        let after_kind = &after_marker[kind_end + 1..];
        let colon = after_kind
            .find(':')
            .ok_or_else(|| refuse("kani_witness_check_text_missing"))?;
        let after_colon = &after_kind[colon + 1..];
        let quote_start = after_colon
            .find('"')
            .ok_or_else(|| refuse("kani_witness_check_text_missing"))?;
        let after_quote = &after_colon[quote_start + 1..];
        let quote_end = find_unescaped_quote(after_quote)
            .ok_or_else(|| refuse("kani_witness_check_text_missing"))?;
        let check_text = after_quote[..quote_end].to_owned();

        let entries = extract_concrete_entries(block, source_id, context)?;
        if entries.is_empty() {
            return Err(refuse("kani_witness_concrete_vals_missing"));
        }
        let concrete_values = entries.into_iter().map(|(_, bytes)| bytes).collect();

        Ok(Self {
            harness_symbol,
            check,
            check_text,
            concrete_values,
            transcript: block.trim().to_owned(),
        })
    }

    /// Joins the untyped concrete bytes with the schema a generator declared
    /// for this harness, cross-checking every decoded value against Kani's
    /// own `//` decoded-value comment.
    ///
    /// Refuses rather than guesses on:
    /// - `kani_witness_arity_mismatch`: the schema and the concrete bytes
    ///   disagree on how many `kani::any()` calls were made.
    /// - `kani_witness_width_mismatch`: a byte vector's length does not match
    ///   the declared primitive's width.
    /// - `kani_witness_comment_mismatch`: a decoded value disagrees with
    ///   Kani's own decoded-value comment, or that comment cannot be
    ///   recovered from the retained transcript.
    pub fn decode(
        &self,
        schema: &[WitnessBinding],
    ) -> Result<Vec<(String, WitnessValue)>, KaniOutcome> {
        let source_id = self.harness_symbol.as_str();
        let context = self.check_text.as_str();

        if self.concrete_values.len() != schema.len() {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_witness_arity_mismatch",
                source_id,
                context,
            ));
        }

        let comments = extract_concrete_entries(&self.transcript, source_id, context)?
            .into_iter()
            .map(|(comment, _)| comment)
            .collect::<Vec<_>>();
        if comments.len() != self.concrete_values.len() {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_witness_comment_mismatch",
                source_id,
                context,
            ));
        }

        let mut decoded = Vec::with_capacity(schema.len());
        for ((binding, bytes), comment) in schema.iter().zip(&self.concrete_values).zip(&comments) {
            let width = binding.value_type.byte_width();
            if bytes.len() != width {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::InvalidInput,
                    "kani_witness_width_mismatch",
                    binding.identifier.as_str(),
                    source_id,
                ));
            }
            let value = match binding.value_type {
                WitnessValueType::Boolean => WitnessValue::Boolean(bytes[0] != 0),
                WitnessValueType::I64 => {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(bytes);
                    WitnessValue::Integer(i64::from_le_bytes(buf))
                }
            };
            if !comment_agrees(&value, comment) {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::InvalidInput,
                    "kani_witness_comment_mismatch",
                    binding.identifier.as_str(),
                    source_id,
                ));
            }
            decoded.push((binding.identifier.clone(), value));
        }
        Ok(decoded)
    }
}

/// Returns whether Kani's own decoded-value comment agrees with `value`.
fn comment_agrees(value: &WitnessValue, comment: &str) -> bool {
    let comment = comment.trim();
    match value {
        WitnessValue::Integer(expected) => comment
            .parse::<i64>()
            .is_ok_and(|parsed| parsed == *expected),
        WitnessValue::Boolean(expected) => match comment.to_ascii_lowercase().as_str() {
            "true" => *expected,
            "false" => !*expected,
            "1" => *expected,
            "0" => !*expected,
            _ => false,
        },
    }
}

/// Returns the text between `start_marker` and the next `end` character.
fn extract_delimited<'a>(text: &'a str, start_marker: &str, end: char) -> Option<&'a str> {
    let start = text.find(start_marker)? + start_marker.len();
    let rest = &text[start..];
    let end_idx = rest.find(end)?;
    Some(&rest[..end_idx])
}

/// Returns the byte index of the next `"` not preceded by an odd number of
/// `\` escapes.
fn find_unescaped_quote(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte != b'"' {
            continue;
        }
        let mut backslashes = 0;
        let mut j = i;
        while j > 0 && bytes[j - 1] == b'\\' {
            backslashes += 1;
            j -= 1;
        }
        if backslashes % 2 == 0 {
            return Some(i);
        }
    }
    None
}

/// Returns the byte range of the bracketed body starting at `text[open_idx]`,
/// which must be `[`, and the index just past the matching `]`.
fn bracket_body(text: &str, open_idx: usize) -> Option<(&str, usize)> {
    let bytes = text.as_bytes();
    if bytes.get(open_idx) != Some(&b'[') {
        return None;
    }
    let mut depth = 0i32;
    for i in open_idx..bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&text[open_idx + 1..i], i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Parses the `let concrete_vals: Vec<Vec<u8>> = vec![ ... ];` section of a
/// playback block into `(decoded-value comment, untyped bytes)` pairs, in
/// declaration order.
///
/// This is the single parser for that section: [`Witness::parse`] uses it to
/// populate `concrete_values`, and [`Witness::decode`] uses it again, against
/// the retained `transcript`, to recover Kani's own decoded-value comments
/// for its cross-check — never trusting a cached copy of them.
fn extract_concrete_entries(
    text: &str,
    source_id: &str,
    context: &str,
) -> Result<Vec<(String, Vec<u8>)>, KaniOutcome> {
    let refuse = |code: &'static str| {
        KaniOutcome::non_success(KaniOutcomeKind::InvalidInput, code, source_id, context)
    };

    let marker = "let concrete_vals";
    let marker_pos = text
        .find(marker)
        .ok_or_else(|| refuse("kani_witness_concrete_vals_missing"))?;
    let after_marker = &text[marker_pos + marker.len()..];
    let vec_marker = "vec![";
    let vec_pos = after_marker
        .find(vec_marker)
        .ok_or_else(|| refuse("kani_witness_concrete_vals_missing"))?;
    let outer_open = vec_pos + vec_marker.len() - 1;
    let (body, _) = bracket_body(after_marker, outer_open)
        .ok_or_else(|| refuse("kani_witness_concrete_vals_malformed"))?;

    let mut entries = Vec::new();
    let mut rest = body;
    while let Some(comment_pos) = rest.find("//") {
        let after_comment_marker = &rest[comment_pos + "//".len()..];
        let line_end = after_comment_marker
            .find('\n')
            .unwrap_or(after_comment_marker.len());
        let comment_text = after_comment_marker[..line_end].trim().to_owned();
        let after_comment_line = &after_comment_marker[line_end..];

        let inner_vec_pos = after_comment_line
            .find(vec_marker)
            .ok_or_else(|| refuse("kani_witness_comment_missing"))?;
        let inner_open = inner_vec_pos + vec_marker.len() - 1;
        let (inner_body, inner_end) = bracket_body(after_comment_line, inner_open)
            .ok_or_else(|| refuse("kani_witness_concrete_vals_malformed"))?;

        let mut bytes = Vec::new();
        for token in inner_body.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            let byte: u8 = token
                .parse()
                .map_err(|_| refuse("kani_witness_byte_invalid"))?;
            bytes.push(byte);
        }
        entries.push((comment_text, bytes));
        rest = &after_comment_line[inner_end..];
    }
    Ok(entries)
}
