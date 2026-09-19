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
//! `transcript` is the single source of truth for every fact a `Witness`
//! exposes. [`Witness::harness_symbol`], [`Witness::check`],
//! [`Witness::check_text`], [`Witness::concrete_values`], and
//! [`Witness::decode`] all re-derive their answer from `transcript` on
//! demand rather than from a separately retained field, so none of them can
//! disagree with what `transcript` actually says — including for a `Witness`
//! built directly by `Deserialize`, which bypasses [`Witness::parse`]
//! entirely and could otherwise carry a stored field that lies about its own
//! transcript.

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
    /// reachability, never falsity. Neither [`Witness::parse`] nor
    /// [`Witness::check`] ever return this variant as `Ok`: a transcript
    /// whose only playback block is a cover check is refused outright
    /// (`kani_witness_cover_refused`) by the same re-derivation both use, so
    /// this variant can never appear as an accepted witness's check kind.
    /// The variant exists so refusal diagnostics and tests exercising
    /// [`super::replay`]'s structural validation can name the case being
    /// refused.
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
#[serde(deny_unknown_fields)]
pub struct WitnessBinding {
    /// The generator's identifier for this binding.
    pub identifier: String,
    /// The declared primitive type of the binding.
    pub value_type: WitnessValueType,
}

/// One decoded concrete value, typed by the schema that named it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WitnessValue {
    /// A decoded boolean.
    Boolean(bool),
    /// A decoded signed 64-bit integer.
    Integer(i64),
}

/// The evaluated witness Kani retained for one falsified check.
///
/// `transcript` is the only stored field. `harness_symbol`, `check`, and
/// `check_text` are methods, not fields: each re-derives its answer from
/// `transcript` on every call, so a `Witness` cannot hold a field that
/// disagrees with its own transcript. This matters most for a `Witness`
/// built directly by `Deserialize`, which bypasses [`Witness::parse`] and
/// its structural validation entirely — with no independent `check` field to
/// forge, a deserialized packet cannot claim `WitnessCheck::Assertion` while
/// `transcript` is verbatim a cover (or absent, or malformed) playback block.
///
/// `transcript` is private and `Witness` implements no public constructor
/// other than [`Witness::parse`]: a struct literal outside this module fails
/// to compile (there is no field to name), and [`Deserialize`] is
/// hand-written below to route every wire value through `parse` as well. A
/// transcript that is not a single selected Kani assertion playback block is
/// therefore unrepresentable, not merely unparsed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Witness {
    /// The exact retained text of the selected assertion playback block: the
    /// single source of truth for every fact this type exposes. There is no
    /// separately stored copy of the harness symbol, check kind, check text,
    /// or concrete bytes, so none of them can disagree with what this text
    /// says.
    transcript: String,
}

/// Routes every deserialized `Witness` through [`Witness::parse`], the same
/// admission path a caller-constructed one goes through, so a wire value
/// whose `transcript` is not a single selected Kani assertion playback block
/// is refused at deserialization rather than accepted and only later found
/// to be untrustworthy.
impl<'de> Deserialize<'de> for Witness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireWitness {
            transcript: String,
        }
        let wire = WireWitness::deserialize(deserializer)?;
        Witness::parse(WITNESS_SOURCE_ID, WITNESS_CONTEXT, &wire.transcript).map_err(|outcome| {
            serde::de::Error::custom(format!(
                "witness transcript refused ({}): {}",
                outcome.code, outcome.context
            ))
        })
    }
}

/// The harness symbol, check kind, and check text one playback block
/// encodes, alongside the exact (re-trimmed) transcript text they were
/// derived from.
///
/// This is an internal parsing product, never itself stored on [`Witness`]:
/// [`Witness::parse`] uses it only to populate `transcript`, and every
/// accessor that exposes one of these three facts re-derives it through
/// [`Witness::derived`] instead of caching it, so a `Witness` — including one
/// built directly by `Deserialize` — can never carry a stored value that
/// disagrees with its own `transcript`.
struct ParsedTranscript {
    harness_symbol: String,
    check: WitnessCheck,
    check_text: String,
    transcript: String,
}

/// Placeholder `source_id`/`context` used only when [`Witness::derived`]
/// re-parses an already-retained `Witness`, where (unlike [`Witness::parse`],
/// which callers invoke with the real clause/profile identifiers) no
/// caller-supplied identifiers are available. The refusal `code` already
/// names the cause; these two exist only to satisfy [`KaniOutcome`]'s shape.
const WITNESS_SOURCE_ID: &str = "witness";
const WITNESS_CONTEXT: &str = "transcript";

impl Witness {
    /// Parses the fenced playback block(s) Kani emits in a `counterexample`
    /// string, selecting the single assertion block (see the private
    /// `derive` associated function for the delimiting/classification rules)
    /// and retaining only its `transcript`.
    pub fn parse(source_id: &str, context: &str, block: &str) -> Result<Self, KaniOutcome> {
        let parsed = Self::derive(source_id, context, block)?;
        Ok(Self {
            transcript: parsed.transcript,
        })
    }

    /// Re-parses `self.transcript` and returns the harness symbol, check
    /// kind, check text, and (re-trimmed) selected transcript it encodes.
    ///
    /// This is the single derivation point [`Witness::harness_symbol`],
    /// [`Witness::check`], [`Witness::check_text`], [`Witness::concrete_values`],
    /// and [`Witness::decode`] all share, and it runs the exact same
    /// delimiting/classification [`Witness::parse`] uses to construct a
    /// `Witness` in the first place. A `self.transcript` that is not a
    /// single, already-selected assertion block — for example a multi-block
    /// blob, or a verbatim cover block, arriving via `Deserialize` rather
    /// than `Witness::parse` — is re-classified and re-selected exactly as
    /// strictly here as it would be by `Witness::parse` itself, so no
    /// accessor can be fooled by a transcript that merely *contains* a valid
    /// block alongside something else.
    fn derived(&self) -> Result<ParsedTranscript, KaniOutcome> {
        Self::derive(WITNESS_SOURCE_ID, WITNESS_CONTEXT, &self.transcript)
    }

    /// The harness symbol the playback block names, re-derived from
    /// `transcript` on every call (see the module doc).
    pub fn harness_symbol(&self) -> Result<String, KaniOutcome> {
        self.derived().map(|parsed| parsed.harness_symbol)
    }

    /// The check kind the playback block was generated for, re-derived from
    /// `transcript` on every call. Never `Ok(WitnessCheck::Cover)` — see
    /// [`WitnessCheck::Cover`].
    pub fn check(&self) -> Result<WitnessCheck, KaniOutcome> {
        self.derived().map(|parsed| parsed.check)
    }

    /// The exact, possibly multi-line, text of the `Check for` clause,
    /// re-derived from `transcript` on every call.
    pub fn check_text(&self) -> Result<String, KaniOutcome> {
        self.derived().map(|parsed| parsed.check_text)
    }

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
    ///   was a reached cover statement, otherwise as a check-kind refusal
    ///   (`kani_witness_check_kind_refused`), since only an assertion
    ///   witnesses falsity;
    /// - more than one `assertion` block refuses rather than silently
    ///   selecting the first, since guessing which falsification is the
    ///   relevant one is not this module's call to make.
    fn derive(
        source_id: &str,
        context: &str,
        block: &str,
    ) -> Result<ParsedTranscript, KaniOutcome> {
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
                _ => {
                    other_kind.get_or_insert(kind_text);
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
            if other_kind.is_some() {
                // F7: `context` already carries the caller's identifying
                // context (profile revision / check text), matching every
                // other refusal in this function; the cause is named by the
                // `code` alone, not by substituting the other check's kind
                // text into a field that means something else everywhere
                // else in the module.
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_check_kind_refused",
                    source_id,
                    context,
                ));
            }
            return Err(refuse("kani_witness_check_missing"));
        };

        Self::parse_single(source_id, context, sub_block)
    }

    /// Parses exactly one already-delimited `Test generated for harness`
    /// block, which must be the selected assertion block.
    fn parse_single(
        source_id: &str,
        context: &str,
        sub_block: &str,
    ) -> Result<ParsedTranscript, KaniOutcome> {
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
                // F7: see the matching comment in `derive` — `context`, not
                // `kind_text`, is what this function passes as context
                // everywhere else.
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_witness_check_kind_refused",
                    source_id,
                    context,
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

        Ok(ParsedTranscript {
            harness_symbol,
            check,
            check_text,
            transcript: sub_block.trim().to_owned(),
        })
    }

    /// The untyped concrete bytes, one entry per `kani::any()` call, in
    /// declaration order, parsed from `transcript` on demand.
    pub fn concrete_values(&self) -> Result<Vec<Vec<u8>>, KaniOutcome> {
        Ok(self
            .concrete_entries()?
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect())
    }

    /// The `(decoded-value comment, untyped bytes)` pairs `transcript`
    /// encodes, re-derived on every call. Shared by [`Witness::concrete_values`]
    /// (which discards the comment) and [`Witness::validate_concrete_entries`]
    /// (which does not).
    fn concrete_entries(&self) -> Result<Vec<(String, Vec<u8>)>, KaniOutcome> {
        let parsed = self.derived()?;
        extract_concrete_entries(
            &parsed.transcript,
            &parsed.harness_symbol,
            &parsed.check_text,
        )
    }

    /// Validates every concrete entry `transcript` encodes with no schema in
    /// hand: each byte vector must be non-empty, and wherever its length
    /// unambiguously determines a value kind under this module's closed
    /// vocabulary (1 byte: boolean; 8 bytes: i64 — see
    /// [`WitnessValueType::byte_width`]), it must also agree with Kani's own
    /// `//` decoded-value comment (`kani_witness_comment_mismatch`) — the
    /// same cross-check [`Witness::decode`] performs once a schema is
    /// available. A byte vector of any other length carries no meaning in
    /// this closed vocabulary and is left entirely to `decode`'s
    /// schema-driven arity/width refusal.
    ///
    /// Used by [`super::replay`]'s packet validation: without this, a packet
    /// whose `transcript` records a concrete value contradicting its own
    /// comment would replay as `witness_backed = true` despite never having
    /// reproduced what Kani actually recorded.
    pub(crate) fn validate_concrete_entries(&self) -> Result<(), KaniOutcome> {
        let parsed = self.derived()?;
        let entries = self.concrete_entries()?;
        let refuse = |code: &'static str| {
            KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                code,
                parsed.harness_symbol.as_str(),
                parsed.check_text.as_str(),
            )
        };
        for (comment, bytes) in &entries {
            if bytes.is_empty() {
                return Err(refuse("kani_witness_concrete_value_empty"));
            }
            let inferred = match bytes.len() {
                1 => Some(WitnessValue::Boolean(bytes[0] != 0)),
                8 => {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(bytes);
                    Some(WitnessValue::Integer(i64::from_le_bytes(buf)))
                }
                _ => None,
            };
            if let Some(value) = inferred {
                if !comment_agrees(&value, comment) {
                    return Err(refuse("kani_witness_comment_mismatch"));
                }
            }
        }
        Ok(())
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
    /// - `kani_witness_boolean_byte_invalid`: a boolean's byte is neither `0`
    ///   nor `1` — never inferred from "any nonzero byte" (AC-4).
    /// - `kani_witness_comment_mismatch`: a decoded value disagrees with
    ///   Kani's own decoded-value comment.
    pub fn decode(
        &self,
        schema: &[WitnessBinding],
    ) -> Result<Vec<(String, WitnessValue)>, KaniOutcome> {
        let parsed = self.derived()?;
        let source_id = parsed.harness_symbol.as_str();
        let entries =
            extract_concrete_entries(&parsed.transcript, source_id, parsed.check_text.as_str())?;

        if entries.len() != schema.len() {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_witness_arity_mismatch",
                source_id,
                parsed.check_text.as_str(),
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
                WitnessValueType::Boolean => {
                    // AC-4: never an inferred value. Kani's concrete-playback
                    // encoding for `bool` is exactly one byte, `0` or `1`;
                    // any other byte is not a boolean Kani could have
                    // produced, so refuse rather than infer `true` from any
                    // nonzero byte.
                    if bytes[0] > 1 {
                        return Err(KaniOutcome::non_success(
                            KaniOutcomeKind::InvalidInput,
                            "kani_witness_boolean_byte_invalid",
                            source_id,
                            binding.identifier.as_str(),
                        ));
                    }
                    WitnessValue::Boolean(bytes[0] != 0)
                }
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
