//! Evaluated Kani concrete-playback witnesses.
//!
//! A `kani::concrete_playback_run` block is the only artifact Kani retains
//! that actually reproduces a falsified check: the untyped bytes it hands to
//! `kani::any()` in declaration order. This module parses that fenced block
//! into a typed [`Witness`] and joins its untyped bytes with the schema a
//! generator declared, refusing rather than guessing whenever the two
//! disagree.
//!
//! A transcript may retain more than one playback block (a reached cover
//! statement alongside a falsified contract is an ordinary run): [`Witness::parse`]
//! delimits the blocks and selects the single assertion block among them,
//! rather than scanning the whole text for the first `Check for` / `let
//! concrete_vals` occurrence.
//!
//! `transcript` is the single source of truth for the concrete bytes:
//! [`Witness::concrete_values`] and [`Witness::decode`] both parse them from
//! it on demand rather than from a separately retained copy, so they cannot
//! drift from what Kani actually emitted.

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
    /// reachability, never falsity. [`Witness::parse`] never returns this as
    /// the check of a parsed witness: a cover-only block is refused outright.
    /// The variant exists so callers that construct a `Witness` directly
    /// (tests exercising [`super::replay`]'s structural validation) can name
    /// the case they are refusing.
    Cover,
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Witness {
    /// The harness symbol the playback block names.
    pub harness_symbol: String,
    /// The check kind the playback block was generated for.
    pub check: WitnessCheck,
    /// The exact, possibly multi-line, text of the `Check for` clause.
    pub check_text: String,
    /// The exact retained text of the selected assertion playback block: the
    /// single source of truth for [`Witness::concrete_values`] and
    /// [`Witness::decode`]. There is no separately stored copy of the
    /// concrete bytes, so they cannot disagree with what this text says.
    pub transcript: String,
}

impl Witness {
    /// Parses the fenced playback block(s) Kani emits in a `counterexample`
    /// string.
    ///
    /// A transcript may contain more than one `Test generated for harness`
    /// block (for example a reached cover statement alongside a falsified
    /// contract, both ordinary parts of the same run). Blocks are delimited
    /// on that marker and classified by check kind:
    /// - exactly one `assertion` block is required to succeed: it is parsed
    ///   and returned;
    /// - zero `assertion` blocks refuses — as a cover refusal if any block
    ///   was a reached cover statement, otherwise naming the other check kind
    ///   Kani reported (`kani_witness_check_kind_refused`), since only an
    ///   assertion witnesses falsity;
    /// - more than one `assertion` block refuses rather than silently
    ///   selecting the first, since guessing which falsification is the
    ///   relevant one is not this module's call to make.
    pub fn parse(source_id: &str, context: &str, block: &str) -> Result<Self, KaniOutcome> {
        let refuse = |code: &'static str| {
            KaniOutcome::non_success(KaniOutcomeKind::InvalidInput, code, source_id, context)
        };

        let harness_marker = "/// Test generated for harness `";
        let starts = find_all(block, harness_marker);
        if starts.is_empty() {
            return Err(refuse("kani_witness_harness_missing"));
        }
        let sub_blocks: Vec<&str> = starts
            .iter()
            .enumerate()
            .map(|(i, &start)| {
                let end = starts.get(i + 1).copied().unwrap_or(block.len());
                &block[start..end]
            })
            .collect();

        let mut assertion_block: Option<&str> = None;
        let mut saw_cover = false;
        let mut other_kind: Option<&str> = None;
        for sub in sub_blocks {
            let (kind_text, _) = locate_check_kind(sub, source_id, context)?;
            match kind_text {
                "assertion" => {
                    if assertion_block.is_some() {
                        return Err(KaniOutcome::non_success(
                            KaniOutcomeKind::Refused,
                            "kani_witness_multiple_assertions_refused",
                            source_id,
                            context,
                        ));
                    }
                    assertion_block = Some(sub);
                }
                "cover" => saw_cover = true,
                other => {
                    other_kind.get_or_insert(other);
                }
            }
        }

        let Some(sub_block) = assertion_block else {
            if saw_cover {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_cover_refused",
                    source_id,
                    context,
                ));
            }
            if let Some(kind_text) = other_kind {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_check_kind_refused",
                    source_id,
                    kind_text,
                ));
            }
            return Err(refuse("kani_witness_check_missing"));
        };

        Self::parse_single(source_id, context, sub_block)
    }

    /// Parses exactly one already-delimited `Test generated for harness`
    /// block, which must be the selected assertion block.
    fn parse_single(source_id: &str, context: &str, sub_block: &str) -> Result<Self, KaniOutcome> {
        let refuse = |code: &'static str| {
            KaniOutcome::non_success(KaniOutcomeKind::InvalidInput, code, source_id, context)
        };

        let harness_symbol = extract_delimited(sub_block, "Test generated for harness `", '`')
            .ok_or_else(|| refuse("kani_witness_harness_missing"))?
            .to_owned();

        let (kind_text, after_kind) = locate_check_kind(sub_block, source_id, context)?;
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
            _ => {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_check_kind_refused",
                    source_id,
                    kind_text,
                ));
            }
        };

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

        // Validate the concrete-values section parses now, rather than
        // deferring the failure to the first `concrete_values`/`decode`
        // call. A zero-`kani::any()` harness legitimately has none (F9): an
        // empty result is not itself a refusal.
        extract_concrete_entries(sub_block, source_id, context)?;

        Ok(Self {
            harness_symbol,
            check,
            check_text,
            transcript: sub_block.trim().to_owned(),
        })
    }

    /// The untyped concrete bytes, one entry per `kani::any()` call, in
    /// declaration order, parsed from `transcript` on demand.
    pub fn concrete_values(&self) -> Result<Vec<Vec<u8>>, KaniOutcome> {
        let source_id = self.harness_symbol.as_str();
        let context = self.check_text.as_str();
        Ok(
            extract_concrete_entries(&self.transcript, source_id, context)?
                .into_iter()
                .map(|(_, bytes)| bytes)
                .collect(),
        )
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
    ///   Kani's own decoded-value comment.
    pub fn decode(
        &self,
        schema: &[WitnessBinding],
    ) -> Result<Vec<(String, WitnessValue)>, KaniOutcome> {
        let source_id = self.harness_symbol.as_str();
        let entries =
            extract_concrete_entries(&self.transcript, source_id, self.check_text.as_str())?;

        if entries.len() != schema.len() {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_witness_arity_mismatch",
                source_id,
                self.check_text.as_str(),
            ));
        }

        let mut decoded = Vec::with_capacity(schema.len());
        for (binding, (comment, bytes)) in schema.iter().zip(&entries) {
            let width = binding.value_type.byte_width();
            if bytes.len() != width {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::InvalidInput,
                    "kani_witness_width_mismatch",
                    source_id,
                    binding.identifier.as_str(),
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
                    source_id,
                    binding.identifier.as_str(),
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

/// Returns the byte offset of every non-overlapping occurrence of `needle` in
/// `haystack`, in order.
fn find_all(haystack: &str, needle: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut cursor = 0usize;
    while let Some(pos) = haystack[cursor..].find(needle) {
        offsets.push(cursor + pos);
        cursor += pos + needle.len();
    }
    offsets
}

/// Returns the byte offset, within `text`, of the start of the first line
/// whose content — trimmed of leading whitespace — begins with `"/// Check
/// for \`"`.
///
/// Anchoring to the doc-comment line that actually declares the check,
/// rather than searching the whole block for that substring, refuses to be
/// fooled by caller-controlled contract text earlier in the block (Kani
/// appends it to the harness doc line) that happens to contain the same
/// words.
fn find_check_line(text: &str) -> Option<usize> {
    let marker = "/// Check for `";
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with(marker) {
            return Some(offset + (line.len() - trimmed.len()));
        }
        offset += line.len();
    }
    None
}

/// Locates the `Check for` clause's check-kind text within `sub_block` (a
/// single, already-delimited `Test generated for harness` block), anchored
/// to the doc-comment line that declares it (see [`find_check_line`]).
///
/// Returns `(kind_text, after_kind)`, where `after_kind` is the remainder of
/// `sub_block` immediately following the kind's closing backtick — the
/// `: "..."` clause a full parse still needs to extract.
fn locate_check_kind<'a>(
    sub_block: &'a str,
    source_id: &str,
    context: &str,
) -> Result<(&'a str, &'a str), KaniOutcome> {
    let refuse = || {
        KaniOutcome::non_success(
            KaniOutcomeKind::InvalidInput,
            "kani_witness_check_missing",
            source_id,
            context,
        )
    };
    let check_marker = "/// Check for `";
    let check_start = find_check_line(sub_block).ok_or_else(refuse)?;
    let after_marker = &sub_block[check_start + check_marker.len()..];
    let kind_end = after_marker.find('`').ok_or_else(refuse)?;
    Ok((&after_marker[..kind_end], &after_marker[kind_end + 1..]))
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
/// This is the single parser for that section: [`Witness::parse_single`],
/// [`Witness::concrete_values`], and [`Witness::decode`] all call it against
/// the retained `transcript` rather than any cached copy.
///
/// The loop is driven off each `vec![` entry occurrence, not off `//`
/// comments: a value with no comment before it refuses
/// (`kani_witness_comment_missing`) instead of being silently skipped by a
/// scan that starts at the next comment it finds.
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
    let mut cursor = 0usize;
    while let Some(rel_pos) = body[cursor..].find(vec_marker) {
        let entry_pos = cursor + rel_pos;
        let preceding = &body[cursor..entry_pos];
        let comment_marker_pos = preceding
            .rfind("//")
            .ok_or_else(|| refuse("kani_witness_comment_missing"))?;
        let comment_scope = &preceding[comment_marker_pos + "//".len()..];
        let comment_line_end = comment_scope.find('\n').unwrap_or(comment_scope.len());
        let comment_text = comment_scope[..comment_line_end].trim().to_owned();

        let inner_open = entry_pos + vec_marker.len() - 1;
        let (inner_body, inner_end) = bracket_body(body, inner_open)
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
        cursor = inner_end;
    }
    Ok(entries)
}
