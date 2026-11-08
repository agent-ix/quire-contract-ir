// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Reading one selected model document under the caller's budget (FR-038
//! "Budget-charged strict read", model-document stage).
//!
//! The document is read once by `quire-canonical` through its reservation
//! entry, and its declarations are read from that reading by
//! [`super::view::View`]; no `serde_json::Value` is built.

use super::charged::arc_bytes;
use super::view::View;
use super::{
    units, Budget, ModelRefusal, SelectionFailure, SelectionRefusal, DOCUMENT_BYTES_PER_WORK,
};
use crate::checked_package::common::{pointer_from_steps, Step};
use crate::checked_package::shared::{
    CheckedPackageLimit, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use crate::checked_package::v2::budgeted_read::ReadStorageReservation;
use crate::checked_package::v2::ValidationFailure;
use quire_canonical::{Document, Items, Members, Node, ReadError};
use std::sync::Arc;

/// FR-322 step 1's admission checks over the document supplied under the
/// selected identity: it reads, holds no number without an exact RFC 8785
/// spelling, and carries the selected identity as its own `package.identity`.
/// Reading is charged to the work counter before it starts, one unit per
/// [`DOCUMENT_BYTES_PER_WORK`] bytes, so a document too large for the limit
/// is `incomplete` at its root and is never read; every reservation the
/// reader makes, and the retained `Arc`, are charged to read storage first.
pub(super) fn admit_document(
    identity: &str,
    supplied: Option<&[u8]>,
    budget: &mut Budget<'_, '_>,
) -> Result<Arc<Document>, SelectionFailure> {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    let Some(bytes) = supplied else {
        return Err(SelectionRefusal::at(
            Code::MissingImport,
            Cause::MissingSelection,
            Some("identity"),
        )
        .into());
    };
    budget.charge_form(
        units(bytes.len().div_ceil(DOCUMENT_BYTES_PER_WORK)),
        "model_document",
    )?;
    // Bytes past the reader's limit are the read's `incomplete` for `bytes`;
    // a number past the double range has no exact RFC 8785 spelling, so the
    // read names it by pointer and text and it is refused as any other
    // inexact number is.
    let limit = budget.bytes;
    let reading = match budget.reader_budget("model_document") {
        Some(storage) => quire_canonical::read_with_reservation(
            bytes,
            limit,
            &mut ReadStorageReservation(storage),
        ),
        None => quire_canonical::read(bytes, limit),
    };
    let reading = match reading {
        Ok(reading) => reading,
        Err(ReadError::Limit(limit)) => return Err(bytes_exceeded(limit).into()),
        Err(ReadError::ReservationDenied { .. }) => return Err(budget.reader_denial().into()),
        Err(ReadError::NumberOutOfRange {
            pointer, lexeme, ..
        }) => {
            // The reader's pointer is RFC 6901 by its contract, so it is
            // taken as the escaped text it is.
            let document_pointer = JsonPointer::from_escaped(pointer);
            let cause = if Spelling::of(&lexeme).is_whole_past_2_pow_53() {
                Cause::InexactInteger
            } else {
                Cause::InexactNumber
            };
            return Err(SelectionFailure::InexactNumber {
                document_pointer,
                cause,
            });
        }
        Err(_) => {
            return Err(SelectionRefusal::at(
                Code::MalformedWire,
                Cause::MalformedJson,
                Some("identity"),
            )
            .into());
        }
    };
    // Decided on the text of each number, before any rounding to a double.
    if let Some((document_pointer, cause)) = first_inexact_number(&reading, budget)? {
        return Err(SelectionFailure::InexactNumber {
            document_pointer,
            cause,
        });
    }
    if identity_of(View::root(&reading)) != Some(identity) {
        return Err(SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some("identity"),
        )
        .into());
    }
    budget.request(arc_bytes::<Document>(), "model_document")?;
    Ok(Arc::new(reading))
}

/// A Semantic IR 2.0.0 document's own `package.identity`. The document's
/// `package.version` is neither required nor read: a selection binds by
/// identity alone.
pub(super) fn identity_of(document: View<'_>) -> Option<&str> {
    document.get("package")?.get("identity")?.as_str()
}

/// The reader's `incomplete` for `bytes`: the document's length as consumed
/// and no pointer, as every byte-limit outcome is (FR-038-AC-26).
fn bytes_exceeded(limit: quire_canonical::LimitExceeded) -> ValidationFailure {
    ValidationFailure::incomplete(
        CheckedPackageLimit::Bytes,
        limit.bound,
        limit.required,
        None,
    )
}

/// The RFC 6901 pointer and cause of the first number, in document order,
/// that has no exact RFC 8785 spelling (see [`inexact_cause`]); `None` when
/// there is none. Walks from an explicit heap stack, so a document of any
/// depth is walked on any thread stack; the stack and the step path grow
/// under the budget.
fn first_inexact_number(
    document: &Document,
    budget: &mut Budget<'_, '_>,
) -> Result<Option<(JsonPointer, CheckedPackageRefusalCause)>, ValidationFailure> {
    enum Open<'d> {
        Array(Items<'d>, usize),
        Object(Members<'d>),
    }
    let mut path: Vec<Step<'_>> = Vec::new();
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut next = Some(document.root());
    loop {
        if let Some(value) = next.take() {
            match value.node() {
                Node::Number(number) => {
                    if let Some(cause) = inexact_cause(number.text(), number.value()) {
                        return Ok(Some((pointer_from_steps(path.iter().copied()), cause)));
                    }
                }
                Node::Array(items) => {
                    budget.push(&mut open, Open::Array(items, 0), "declarations")?
                }
                Node::Object(members) => {
                    budget.push(&mut open, Open::Object(members), "declarations")?
                }
                Node::Null | Node::Bool(_) | Node::String(_) => {}
            }
        }
        // The path holds one step for each open container but the innermost
        // child being entered, so it is cut back to the open containers
        // before the next child's step is pushed.
        let depth = open.len().saturating_sub(1);
        match open.last_mut() {
            None => return Ok(None),
            Some(Open::Array(items, index)) => match items.next() {
                Some(item) => {
                    path.truncate(depth);
                    let step = Step::Index(*index);
                    *index = index.saturating_add(1);
                    budget.push(&mut path, step, "declarations")?;
                    next = Some(item);
                }
                None => {
                    open.pop();
                }
            },
            Some(Open::Object(members)) => match members.next() {
                Some((name, member)) => {
                    path.truncate(depth);
                    budget.push(&mut path, Step::Key(name), "declarations")?;
                    next = Some(member);
                }
                None => {
                    open.pop();
                }
            },
        }
    }
}

/// The cause of a model-document number's refusal, or `None` when the number
/// is admitted. `text` is the
/// number as spelled in the document and `value` the double `quire-canonical`
/// read from it. The decision is made on the text, never on a double:
///
/// - `inexact-integer` when the text denotes a whole value past 2^53,
///   however spelled;
/// - `inexact-number` when the digits and scale of the text differ from those
///   of the text `quire-canonical` writes for `value`, so the exact value of
///   the number is not the one its RFC 8785 encoding carries.
fn inexact_cause(text: &str, value: f64) -> Option<CheckedPackageRefusalCause> {
    use CheckedPackageRefusalCause as Cause;
    let spelled = Spelling::of(text);
    if spelled.is_whole_past_2_pow_53() {
        return Some(Cause::InexactInteger);
    }
    // Writing one finite double cannot fail, and its text is ASCII; were
    // either to fail, the number has no RFC 8785 text to be exact against, so
    // it is refused. The text is written into a fixed buffer on the stack.
    let mut written = DoubleText {
        bytes: [0; DOUBLE_TEXT_BYTES as usize],
        len: 0,
    };
    let mut writer = quire_canonical::Writer::new(
        &mut written,
        quire_canonical::Limits::new(DOUBLE_TEXT_BYTES),
    );
    let wrote = writer.number(value).is_ok();
    let exact = wrote
        && std::str::from_utf8(written.text())
            .is_ok_and(|written| spelled == Spelling::of(written));
    (!exact).then_some(Cause::InexactNumber)
}

/// The bytes one double's RFC 8785 text can take (at most 25), with room to
/// spare: the room one number's text is written in, not a cap on a document.
const DOUBLE_TEXT_BYTES: u64 = 64;

/// A fixed buffer one double's text is written into.
struct DoubleText {
    bytes: [u8; DOUBLE_TEXT_BYTES as usize],
    len: usize,
}

impl DoubleText {
    fn text(&self) -> &[u8] {
        self.bytes.get(..self.len).unwrap_or_default()
    }
}

impl quire_canonical::Sink for DoubleText {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
        let end = self.len.saturating_add(bytes.len());
        match self.bytes.get_mut(self.len..end) {
            Some(room) => {
                room.copy_from_slice(bytes);
                self.len = end;
                Ok(())
            }
            None => Err(quire_canonical::Error::Internal {
                invariant: "a double's text fits its buffer",
            }),
        }
    }
}

/// The decimal digits of `2^53`, 9007199254740992.
const MAXIMUM_INTEGER_DIGITS: &str = "9007199254740992";

/// A JSON number's value as `digits` x 10^`scale`, with neither a leading nor
/// a trailing zero in `digits`, so two texts spell one value exactly when
/// their spellings are equal. Zero has no digits and no sign. The digits are
/// the borrowed `head` followed by the borrowed `tail`: the integer and
/// fraction parts of the text, trimmed.
#[derive(Debug)]
struct Spelling<'t> {
    negative: bool,
    head: &'t str,
    tail: &'t str,
    scale: i64,
}

impl<'t> Spelling<'t> {
    /// The spelling of `text`, a number `quire-canonical` read or wrote, so
    /// `-? int frac? exp?`.
    fn of(text: &'t str) -> Self {
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(unsigned) => (true, unsigned),
            None => (false, text),
        };
        let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        // The digits are `whole` then `fraction`, without leading zeros.
        let (head, tail) = match whole.trim_start_matches('0') {
            "" => ("", fraction.trim_start_matches('0')),
            head => (head, fraction),
        };
        let before = head.len().saturating_add(tail.len());
        // ... and without trailing zeros.
        let (head, tail) = match tail.trim_end_matches('0') {
            "" => (head.trim_end_matches('0'), ""),
            tail => (head, tail),
        };
        let significant = head.len().saturating_add(tail.len());
        if significant == 0 {
            return Self {
                negative: false,
                head: "",
                tail: "",
                scale: 0,
            };
        }
        let trailing = before.saturating_sub(significant);
        let scale = parse_exponent(exponent)
            .saturating_sub(i64::try_from(fraction.len()).unwrap_or(i64::MAX))
            .saturating_add(i64::try_from(trailing).unwrap_or(i64::MAX));
        Self {
            negative,
            head,
            tail,
            scale,
        }
    }

    fn digits(&self) -> impl Iterator<Item = u8> + '_ {
        self.head.bytes().chain(self.tail.bytes())
    }

    fn length(&self) -> usize {
        self.head.len().saturating_add(self.tail.len())
    }

    /// Whether the value is a whole number of magnitude greater than 2^53.
    /// `digits` has no trailing zero, so a negative `scale` leaves a nonzero
    /// fraction: the value is not whole.
    fn is_whole_past_2_pow_53(&self) -> bool {
        if self.length() == 0 || self.scale < 0 {
            return false;
        }
        let length = i64::try_from(self.length()).unwrap_or(i64::MAX);
        let maximum = i64::try_from(MAXIMUM_INTEGER_DIGITS.len()).unwrap_or(i64::MAX);
        match length.saturating_add(self.scale).cmp(&maximum) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            // Sixteen integer digits: `digits` padded with `scale` zeros.
            std::cmp::Ordering::Equal => {
                let padding = usize::try_from(self.scale).unwrap_or(0);
                self.digits()
                    .chain(std::iter::repeat_n(b'0', padding))
                    .cmp(MAXIMUM_INTEGER_DIGITS.bytes())
                    .is_gt()
            }
        }
    }
}

impl PartialEq for Spelling<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.negative == other.negative
            && self.scale == other.scale
            && self.length() == other.length()
            && self.digits().eq(other.digits())
    }
}

/// A JSON exponent, `[+-]?digits`, saturated to an `i64`.
fn parse_exponent(text: &str) -> i64 {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let magnitude = digits.bytes().fold(0_i64, |total, byte| {
        total
            .saturating_mul(10)
            .saturating_add(i64::from(byte.saturating_sub(b'0')))
    });
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// An unused refusal constructor kept private to this module's callers.
#[allow(dead_code)]
fn malformed() -> ModelRefusal {
    ModelRefusal::malformed()
}
