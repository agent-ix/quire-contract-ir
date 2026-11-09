//! Compare a closed typed value against its original document without a second
//! tree. Healthy comparisons borrow every path and map key; only the selected
//! final diagnostic or a transient key absent from the source owns storage.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path as BasePath, Refusal, Site};
use super::{TermNode, ValidationFailure, WorkMeter};
use crate::checked_package::common::{Step, Trail};
use crate::checked_package::shared::CheckedPackageRefusalCode;
use serde::ser::{
    Error as _, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde::Serialize;
use std::{cell::RefCell, cmp::Ordering, fmt, sync::Arc};

/// Stack slots that remember the leading steps of a position. A position
/// deeper than this keeps being compared and counted, but only its first
/// steps are remembered for a diagnostic pointer, so depth never changes
/// whether the comparison succeeds.
const PATH_SLOTS: usize = 128;
/// Slots for a full pointer: the base trail, a path and one extra key.
const FULL_SLOTS: usize = PATH_SLOTS + 12 + 1;

#[derive(Clone)]
enum Key<'a> {
    Borrowed(&'a str),
    Owned(Arc<Box<str>>),
}
impl Key<'_> {
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned(text) => text,
        }
    }
}
#[derive(Clone)]
enum PathSegment<'a> {
    Key(Key<'a>),
    Index(usize),
}
impl PathSegment<'_> {
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Key(a), Self::Key(b)) => a.as_str().cmp(b.as_str()),
            (Self::Index(a), Self::Index(b)) => a.cmp(b),
            (Self::Key(_), Self::Index(_)) => Ordering::Less,
            (Self::Index(_), Self::Key(_)) => Ordering::Greater,
        }
    }
    fn step(&self) -> Step<'_> {
        match self {
            Self::Key(key) => Step::Key(key.as_str()),
            Self::Index(index) => Step::Index(*index),
        }
    }
}
struct Snapshot<'a> {
    segments: [Option<PathSegment<'a>>; PATH_SLOTS],
    depth: usize,
    root: bool,
}
struct Path<'a, 'm, 'b> {
    base: BasePath<'static>,
    segments: [Option<PathSegment<'a>>; PATH_SLOTS],
    depth: usize,
    best: RefCell<Option<Snapshot<'a>>>,
    denied: RefCell<Option<ValidationFailure>>,
    meter: RefCell<&'m mut WorkMeter<'b>>,
}
impl<'a> Path<'a, '_, '_> {
    fn push(&mut self, segment: PathSegment<'a>) -> bool {
        if let Some(slot) = self.segments.get_mut(self.depth) {
            *slot = Some(segment);
        }
        self.depth += 1;
        true
    }
    fn pop(&mut self) -> Option<PathSegment<'a>> {
        self.depth = self.depth.checked_sub(1)?;
        self.segments.get_mut(self.depth)?.take()
    }
    /// The number of remembered leading steps.
    fn stored(&self) -> usize {
        self.depth.min(PATH_SLOTS)
    }
    fn record(&self, root: bool) -> CompareError {
        let replace = self.best.borrow().as_ref().is_none_or(|prior| {
            self.segments[..self.stored()]
                .iter()
                .flatten()
                .zip(
                    prior.segments[..prior.depth.min(PATH_SLOTS)]
                        .iter()
                        .flatten(),
                )
                .map(|(a, b)| a.compare(b))
                .find(|order| *order != Ordering::Equal)
                .unwrap_or_else(|| self.depth.cmp(&prior.depth))
                == Ordering::Less
        });
        if replace {
            *self.best.borrow_mut() = Some(Snapshot {
                segments: self.segments.clone(),
                depth: self.depth,
                root,
            });
        }
        CompareError::Mismatch
    }
    fn mismatch(&self) -> CompareError {
        self.record(false)
    }
    fn normalize(&self, error: CompareError) -> CompareError {
        if error == CompareError::CustomRoot {
            self.record(true)
        } else {
            error
        }
    }
    fn request(&self, bytes: usize, extra: Option<&str>) -> Result<(), CompareError> {
        let mut steps = [Step::Index(0); FULL_SLOTS];
        let mut length = 0;
        for step in self.base.steps() {
            steps[length] = *step;
            length += 1;
        }
        for segment in self.segments[..self.stored()].iter().flatten() {
            steps[length] = segment.step();
            length += 1;
        }
        if let Some(key) = extra {
            steps[length] = Step::Key(key);
            length += 1;
        }
        let index = steps[..length]
            .iter()
            .filter_map(|step| match step {
                Step::Index(index) => Some(*index),
                _ => None,
            })
            .next_back()
            .unwrap_or(0);
        let trail = Trail::Base(&steps[..length]);
        let result = stage_storage::reserve(
            &mut self.meter.borrow_mut(),
            &Site::new(PackageReadStage::WireDecode, index, "wire", trail),
            bytes,
        );
        result.map_err(|stop| {
            if self.denied.borrow().is_none() {
                *self.denied.borrow_mut() = Some(stop);
            }
            CompareError::Budget
        })
    }
    fn owned_key(&self, text: &str) -> Result<Key<'a>, CompareError> {
        self.request(text.len(), Some(text))?;
        let owned: Box<str> = text.into();
        self.request(super::budgeted_read::arc_bytes::<Box<str>>(), Some(&owned))?;
        Ok(Key::Owned(Arc::new(owned)))
    }
    fn finish(self, error: CompareError) -> ValidationFailure {
        if error == CompareError::CustomRoot {
            self.record(true);
        }
        if let Some(stop) = self.denied.into_inner() {
            return stop;
        }
        let selected = self
            .best
            .into_inner()
            .expect("a mismatch records its borrowed path");
        let mut steps = [Step::Index(0); FULL_SLOTS];
        let mut length = 0;
        if !selected.root {
            for step in self.base.steps() {
                steps[length] = *step;
                length += 1;
            }
            for segment in selected.segments[..selected.depth.min(PATH_SLOTS)]
                .iter()
                .flatten()
            {
                steps[length] = segment.step();
                length += 1;
            }
        }
        let index = steps[..length]
            .iter()
            .filter_map(|step| match step {
                Step::Index(index) => Some(*index),
                _ => None,
            })
            .next_back()
            .unwrap_or(0);
        let trail = Trail::Base(&steps[..length]);
        stage_storage::refuse(
            self.meter.into_inner(),
            &Site::new(PackageReadStage::WireDecode, index, "wire", trail),
            Refusal::new(CheckedPackageRefusalCode::MalformedWire, trail),
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompareError {
    Mismatch,
    CustomRoot,
    Budget,
}
impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "closed wire differs from typed decode")
    }
}
impl std::error::Error for CompareError {}
impl serde::ser::Error for CompareError {
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self::CustomRoot
    }
}

pub(super) fn compare<T: Serialize>(
    raw: TermNode<'_>,
    typed: &T,
    at: BasePath<'static>,
    skipped_members: &'static [&'static str],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut path = Path {
        base: at,
        segments: std::array::from_fn(|_| None),
        depth: 0,
        best: RefCell::new(None),
        denied: RefCell::new(None),
        meter: RefCell::new(meter),
    };
    let result = typed.serialize(Compare {
        raw,
        path: &mut path,
        skipped_members,
    });
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(path.finish(error)),
    }
}

/// Compare Display output without allocating a formatting buffer.
struct TextMatch<'a> {
    expected: &'a str,
    offset: usize,
    equal: bool,
}
impl fmt::Write for TextMatch<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.equal &= self
            .expected
            .as_bytes()
            .get(self.offset..self.offset.saturating_add(text.len()))
            == Some(text.as_bytes());
        self.offset = self.offset.saturating_add(text.len());
        Ok(())
    }
}
fn display_matches(value: &impl fmt::Display, expected: &str) -> bool {
    let mut out = TextMatch {
        expected,
        offset: 0,
        equal: true,
    };
    fmt::write(&mut out, format_args!("{value}")).is_ok()
        && out.equal
        && out.offset == expected.len()
}

struct Compare<'a, 'p, 'm, 'b> {
    raw: TermNode<'a>,
    path: &'p mut Path<'a, 'm, 'b>,
    // Applied to this object only, never inherited by nested objects.
    skipped_members: &'static [&'static str],
}

impl<'a, 'p, 'm, 'b> Compare<'a, 'p, 'm, 'b> {
    fn mismatch(&self) -> CompareError {
        self.path.mismatch()
    }

    fn scalar<T: Serialize>(self, expected: T) -> Result<(), CompareError> {
        expected.serialize(ScalarCompare {
            raw: self.raw,
            path: self.path,
        })
    }

    fn object(self) -> Result<Children<'a, 'p, 'm, 'b>, CompareError> {
        if !self.raw.is_object() {
            return Err(self.mismatch());
        }
        Ok(Children {
            path: self.path,
            pop_on_finish: false,
            seen: [None; 64],
            kind: ChildrenKind::Object {
                raw: self.raw,
                skipped_members: self.skipped_members,
                pending_key: None,
                seen_len: 0,
                seen_count: 0,
                first: None,
            },
        })
    }

    fn sequence(
        self,
        expected_len: Option<usize>,
    ) -> Result<Children<'a, 'p, 'm, 'b>, CompareError> {
        let items = self.raw.items().ok_or_else(|| self.mismatch())?;
        if expected_len.is_some_and(|length| length != items.len()) {
            return Err(self.mismatch());
        }
        Ok(Children {
            path: self.path,
            pop_on_finish: false,
            seen: [None; 64],
            kind: ChildrenKind::Sequence { items, next: 0 },
        })
    }
}

struct ScalarCompare<'a, 'p, 'm, 'b> {
    raw: TermNode<'a>,
    path: &'p mut Path<'a, 'm, 'b>,
}

impl ScalarCompare<'_, '_, '_, '_> {
    fn mismatch(&self) -> CompareError {
        self.path.mismatch()
    }
    fn number(&self) -> Option<quire_canonical::Number<'_>> {
        match self.raw.node_ref().node() {
            quire_canonical::Node::Number(number) => Some(number),
            _ => None,
        }
    }
    fn integer(&self, expected: i128) -> Result<(), CompareError> {
        const MAXIMUM: i128 = 1 << 53;
        if expected.unsigned_abs() <= MAXIMUM as u128
            && self
                .number()
                .is_some_and(|number| number.text().parse::<i128>().ok() == Some(expected))
        {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn unsigned(&self, expected: u128) -> Result<(), CompareError> {
        const MAXIMUM: u128 = 1 << 53;
        if expected <= MAXIMUM
            && self
                .number()
                .is_some_and(|number| number.text().parse::<u128>().ok() == Some(expected))
        {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn float(&self, expected: f64) -> Result<(), CompareError> {
        if expected.is_finite()
            && self
                .number()
                .is_some_and(|number| number.value() == expected)
        {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
}

impl Serializer for ScalarCompare<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = serde::ser::Impossible<(), CompareError>;
    type SerializeTuple = serde::ser::Impossible<(), CompareError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), CompareError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), CompareError>;
    type SerializeMap = serde::ser::Impossible<(), CompareError>;
    type SerializeStruct = serde::ser::Impossible<(), CompareError>;
    type SerializeStructVariant = serde::ser::Impossible<(), CompareError>;

    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<(), Self::Error> {
        if self
            .raw
            .as_str()
            .is_some_and(|raw| display_matches(&value, raw))
        {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_bool(self, expected: bool) -> Result<(), CompareError> {
        if self.raw.as_bool() == Some(expected) {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_i8(self, value: i8) -> Result<(), CompareError> {
        self.integer(i128::from(value))
    }
    fn serialize_i16(self, value: i16) -> Result<(), CompareError> {
        self.integer(i128::from(value))
    }
    fn serialize_i32(self, value: i32) -> Result<(), CompareError> {
        self.integer(i128::from(value))
    }
    fn serialize_i64(self, value: i64) -> Result<(), CompareError> {
        self.integer(i128::from(value))
    }
    fn serialize_i128(self, value: i128) -> Result<(), CompareError> {
        self.integer(value)
    }
    fn serialize_u8(self, value: u8) -> Result<(), CompareError> {
        self.unsigned(u128::from(value))
    }
    fn serialize_u16(self, value: u16) -> Result<(), CompareError> {
        self.unsigned(u128::from(value))
    }
    fn serialize_u32(self, value: u32) -> Result<(), CompareError> {
        self.unsigned(u128::from(value))
    }
    fn serialize_u64(self, value: u64) -> Result<(), CompareError> {
        self.unsigned(u128::from(value))
    }
    fn serialize_u128(self, value: u128) -> Result<(), CompareError> {
        self.unsigned(value)
    }
    fn serialize_f32(self, value: f32) -> Result<(), CompareError> {
        self.float(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> Result<(), CompareError> {
        self.float(value)
    }
    fn serialize_char(self, value: char) -> Result<(), CompareError> {
        let mut chars = self.raw.as_str().unwrap_or_default().chars();
        if chars.next() == Some(value) && chars.next().is_none() {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_str(self, value: &str) -> Result<(), CompareError> {
        if self.raw.as_str() == Some(value) {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<(), CompareError> {
        Err(self.mismatch())
    }
    fn serialize_none(self) -> Result<(), CompareError> {
        self.serialize_unit()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), CompareError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), CompareError> {
        if self.raw.is_null() {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), CompareError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), CompareError> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), CompareError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), CompareError> {
        Err(self.mismatch())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, CompareError> {
        Err(self.mismatch())
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, CompareError> {
        Err(self.mismatch())
    }
}

impl<'a, 'p, 'm, 'b> Serializer for Compare<'a, 'p, 'm, 'b> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = Children<'a, 'p, 'm, 'b>;
    type SerializeTuple = Children<'a, 'p, 'm, 'b>;
    type SerializeTupleStruct = Children<'a, 'p, 'm, 'b>;
    type SerializeTupleVariant = Children<'a, 'p, 'm, 'b>;
    type SerializeMap = Children<'a, 'p, 'm, 'b>;
    type SerializeStruct = Children<'a, 'p, 'm, 'b>;
    type SerializeStructVariant = Children<'a, 'p, 'm, 'b>;

    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<(), Self::Error> {
        if self
            .raw
            .as_str()
            .is_some_and(|raw| display_matches(&value, raw))
        {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_bool(self, value: bool) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_i8(self, value: i8) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_i16(self, value: i16) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_i32(self, value: i32) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_i64(self, value: i64) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_i128(self, value: i128) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_u8(self, value: u8) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_u16(self, value: u16) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_u32(self, value: u32) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_u64(self, value: u64) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_u128(self, value: u128) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_f32(self, value: f32) -> Result<(), Self::Error> {
        self.scalar(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_char(self, value: char) -> Result<(), Self::Error> {
        self.scalar(value)
    }
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        if self.raw.as_str() == Some(value) {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<(), Self::Error> {
        let mut children = self.sequence(Some(value.len()))?;
        for byte in value {
            children.element(byte)?;
        }
        children.finish()
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.scalar(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let mut children = self.object()?;
        children.field_static(variant, value)?;
        children.finish()
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.sequence(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.sequence(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.sequence(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let members = self.raw.members().ok_or_else(|| self.mismatch())?;
        if members.len() != 1 {
            return Err(self.mismatch());
        }
        let raw = self.raw.get(variant).ok_or_else(|| self.mismatch())?;
        if !self.path.push(PathSegment::Key(Key::Borrowed(variant))) {
            return Err(self.mismatch());
        }
        Compare {
            raw,
            path: self.path,
            skipped_members: &[],
        }
        .sequence(Some(len))
        .map(|mut children| {
            children.pop_on_finish = true;
            children
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.object()
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.object()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let members = self.raw.members().ok_or_else(|| self.mismatch())?;
        if members.len() != 1 {
            return Err(self.mismatch());
        }
        let raw = self.raw.get(variant).ok_or_else(|| self.mismatch())?;
        if !self.path.push(PathSegment::Key(Key::Borrowed(variant))) {
            return Err(self.mismatch());
        }
        Compare {
            raw,
            path: self.path,
            skipped_members: &[],
        }
        .object()
        .map(|mut children| {
            children.pop_on_finish = true;
            children
        })
    }
}

struct Children<'a, 'p, 'm, 'b> {
    path: &'p mut Path<'a, 'm, 'b>,
    pop_on_finish: bool,
    kind: ChildrenKind<'a>,
    // Fixed scratch storage stays outside the role enum; neither role boxes it.
    seen: [Option<&'static str>; 64],
}
enum ChildrenKind<'a> {
    Sequence {
        items: super::TermItems<'a>,
        next: usize,
    },
    Object {
        raw: TermNode<'a>,
        skipped_members: &'static [&'static str],
        pending_key: Option<Key<'a>>,
        seen_len: usize,
        seen_count: usize,
        first: Option<Key<'a>>,
    },
}
impl<'a, 'p, 'm, 'b> Children<'a, 'p, 'm, 'b> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        let ChildrenKind::Sequence { items, next } = &mut self.kind else {
            unreachable!()
        };
        let raw = items.next().ok_or_else(|| self.path.mismatch())?;
        if !self.path.push(PathSegment::Index(*next)) {
            return Err(self.path.mismatch());
        }
        *next += 1;
        let result = value
            .serialize(Compare {
                raw,
                path: self.path,
                skipped_members: &[],
            })
            .map_err(|error| self.path.normalize(error));
        self.path.pop();
        result
    }
    fn field<T: ?Sized + Serialize>(
        &mut self,
        key: Key<'a>,
        value: &T,
    ) -> Result<(), CompareError> {
        self.field_inner(key, None, value)
    }
    fn field_static<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CompareError> {
        self.field_inner(Key::Borrowed(key), Some(key), value)
    }
    fn field_inner<T: ?Sized + Serialize>(
        &mut self,
        key: Key<'a>,
        static_key: Option<&'static str>,
        value: &T,
    ) -> Result<(), CompareError> {
        let ChildrenKind::Object {
            raw,
            skipped_members,
            seen_len,
            seen_count,
            first,
            ..
        } = &mut self.kind
        else {
            unreachable!()
        };
        if skipped_members.contains(&key.as_str()) {
            return Ok(());
        }
        let seen = &mut self.seen;
        let child = raw.get(key.as_str());
        if !self.path.push(PathSegment::Key(key.clone())) {
            return Err(self.path.mismatch());
        }
        let result = match child {
            Some(raw) => {
                *seen_count += 1;
                if let Some(key) = static_key {
                    if *seen_len < seen.len() {
                        seen[*seen_len] = Some(key);
                        *seen_len += 1;
                    }
                }
                value
                    .serialize(Compare {
                        raw,
                        path: self.path,
                        skipped_members: &[],
                    })
                    .map_err(|error| self.path.normalize(error))
            }
            None => Err(self.path.mismatch()),
        };
        self.path.pop();
        if let Err(error) = result {
            if error == CompareError::Budget {
                return Err(error);
            }
            if first
                .as_ref()
                .is_none_or(|prior| key.as_str() < prior.as_str())
            {
                *first = Some(key);
            }
        }
        Ok(())
    }
    fn finish(self) -> Result<(), CompareError> {
        let path = self.path;
        let seen = self.seen;
        let result = match self.kind {
            ChildrenKind::Sequence { items, .. } => {
                if items.len() == 0 {
                    Ok(())
                } else {
                    Err(path.mismatch())
                }
            }
            ChildrenKind::Object {
                raw,
                skipped_members,
                seen_len,
                seen_count,
                first,
                ..
            } => {
                let expected = raw
                    .members()
                    .into_iter()
                    .flatten()
                    .filter(|(key, _)| !skipped_members.contains(key))
                    .count();
                if seen_count == expected && first.is_none() {
                    Ok(())
                } else {
                    let extra = raw
                        .members()
                        .into_iter()
                        .flatten()
                        .filter(|(key, _)| !skipped_members.contains(key))
                        .filter(|(key, _)| {
                            !seen[..seen_len].iter().flatten().any(|seen| *seen == *key)
                        })
                        .map(|(key, _)| key)
                        .min();
                    match (first, extra) {
                        (Some(key), Some(extra)) if extra < key.as_str() => {
                            if !path.push(PathSegment::Key(Key::Borrowed(extra))) {
                                Err(path.mismatch())
                            } else {
                                let error = path.mismatch();
                                path.pop();
                                Err(error)
                            }
                        }
                        (Some(_), _) => Err(CompareError::Mismatch),
                        (None, Some(extra)) => {
                            if !path.push(PathSegment::Key(Key::Borrowed(extra))) {
                                Err(path.mismatch())
                            } else {
                                let error = path.mismatch();
                                path.pop();
                                Err(error)
                            }
                        }
                        (None, None) => Ok(()),
                    }
                }
            }
        };
        if self.pop_on_finish {
            path.pop();
        }
        result
    }
}

impl SerializeSeq for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTuple for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleStruct for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleVariant for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

struct MapKey<'a, 'p, 'm, 'b> {
    raw: TermNode<'a>,
    path: &'p mut Path<'a, 'm, 'b>,
}
struct ReservedText {
    text: String,
    capacity: usize,
}
impl fmt::Write for ReservedText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if self
            .text
            .len()
            .checked_add(value.len())
            .is_none_or(|length| length > self.capacity)
        {
            return Err(fmt::Error);
        }
        self.text.push_str(value);
        Ok(())
    }
}
struct TextLength(usize);
impl fmt::Write for TextLength {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}
impl<'a> MapKey<'a, '_, '_, '_> {
    fn text(self, text: &str) -> Result<Key<'a>, CompareError> {
        if let Some((name, _)) = self
            .raw
            .members()
            .into_iter()
            .flatten()
            .find(|(name, _)| *name == text)
        {
            return Ok(Key::Borrowed(name));
        }
        self.path.owned_key(text)
    }
    fn formatted(self, value: &impl fmt::Display) -> Result<Key<'a>, CompareError> {
        if let Some((name, _)) = self
            .raw
            .members()
            .into_iter()
            .flatten()
            .find(|(name, _)| display_matches(value, name))
        {
            return Ok(Key::Borrowed(name));
        }
        let mut count = TextLength(0);
        fmt::write(&mut count, format_args!("{value}")).map_err(|_| CompareError::CustomRoot)?;
        self.path.request(count.0, None)?;
        let mut out = ReservedText {
            text: String::with_capacity(count.0),
            capacity: count.0,
        };
        fmt::write(&mut out, format_args!("{value}")).map_err(|_| CompareError::CustomRoot)?;
        let text = out.text;
        // Formatting a concrete primitive is deterministic. A user Display
        // implementation cannot make this internal writer grow unnoticed.
        if text.len() != count.0 {
            return Err(CompareError::CustomRoot);
        }
        self.path
            .request(super::budgeted_read::arc_bytes::<Box<str>>(), Some(&text))?;
        Ok(Key::Owned(Arc::new(text.into_boxed_str())))
    }
}

impl<'a, 'p, 'm, 'b> Serializer for MapKey<'a, 'p, 'm, 'b> {
    type Ok = Key<'a>;
    type Error = CompareError;
    type SerializeSeq = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeTuple = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeTupleStruct = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeTupleVariant = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeMap = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeStruct = serde::ser::Impossible<Key<'a>, CompareError>;
    type SerializeStructVariant = serde::ser::Impossible<Key<'a>, CompareError>;

    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.formatted(&value)
    }
    fn serialize_bool(self, value: bool) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_i8(self, value: i8) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_i16(self, value: i16) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_i32(self, value: i32) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_i64(self, value: i64) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_i128(self, value: i128) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_u8(self, value: u8) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_u16(self, value: u16) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_u32(self, value: u32) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_u64(self, value: u64) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_u128(self, value: u128) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_f32(self, value: f32) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_f64(self, value: f64) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_char(self, value: char) -> Result<Key<'a>, CompareError> {
        self.formatted(&value)
    }
    fn serialize_str(self, value: &str) -> Result<Key<'a>, CompareError> {
        self.text(value)
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<Key<'a>, CompareError> {
        Err(CompareError::custom("byte map key"))
    }
    fn serialize_none(self) -> Result<Key<'a>, CompareError> {
        Err(CompareError::custom("null map key"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Key<'a>, CompareError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Key<'a>, CompareError> {
        Err(CompareError::custom("unit map key"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Key<'a>, CompareError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Key<'a>, CompareError> {
        Ok(Key::Borrowed(variant))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Key<'a>, CompareError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Key<'a>, CompareError> {
        Err(CompareError::custom("newtype map key"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, CompareError> {
        Err(CompareError::custom("sequence map key"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, CompareError> {
        Err(CompareError::custom("tuple map key"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, CompareError> {
        Err(CompareError::custom("tuple struct map key"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, CompareError> {
        Err(CompareError::custom("tuple variant map key"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, CompareError> {
        Err(CompareError::custom("map map key"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, CompareError> {
        Err(CompareError::custom("struct map key"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, CompareError> {
        Err(CompareError::custom("struct variant map key"))
    }
}

impl<'a, 'p, 'm, 'b> SerializeMap for Children<'a, 'p, 'm, 'b> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let ChildrenKind::Object {
            raw, pending_key, ..
        } = &mut self.kind
        else {
            unreachable!()
        };
        *pending_key = Some(key.serialize(MapKey {
            raw: *raw,
            path: self.path,
        })?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let ChildrenKind::Object { pending_key, .. } = &mut self.kind else {
            unreachable!()
        };
        let key = pending_key
            .take()
            .ok_or_else(|| CompareError::custom("map value without key"))?;
        self.field(key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeStruct for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.field_static(key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeStructVariant for Children<'_, '_, '_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.field_static(key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::BasePath;
    use super::{compare, TermNode};
    use crate::checked_package::common::ValidationFailure;
    use crate::checked_package::shared::CheckedCapability;
    use crate::checked_package::v2::CheckedCapabilityDisposition;
    use crate::checked_package::v2::{NativeJoinBudget, WorkMeter};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Pair {
        a: u64,
        b: u64,
    }

    #[derive(Serialize)]
    struct Link {
        next: Option<Box<Link>>,
        end: u64,
    }

    fn chain(depth: usize, end: u64) -> Link {
        let mut link = Link { next: None, end };
        for _ in 1..depth {
            link = Link {
                next: Some(Box::new(link)),
                end,
            };
        }
        link
    }

    fn chain_text(depth: usize, end: u64) -> String {
        let mut text = format!(r#"{{"next":null,"end":{end}}}"#);
        for _ in 1..depth {
            text = format!(r#"{{"next":{text},"end":{end}}}"#);
        }
        text
    }

    #[test]
    fn a_chain_past_the_remembered_steps_compares_like_a_shallow_one() {
        std::thread::Builder::new()
            .stack_size(64 << 20)
            .spawn(chain_past_the_remembered_steps)
            .expect("spawn")
            .join()
            .expect("chain comparison");
    }

    fn chain_past_the_remembered_steps() {
        for depth in [10, 200] {
            let document = quire_canonical::read(chain_text(depth, 1).as_bytes(), u64::MAX)
                .expect("source document");
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let mut meter = WorkMeter::over(&mut budget);
            compare(
                TermNode::from_ref(document.root()),
                &chain(depth, 1),
                BasePath::root().key("row"),
                &[],
                &mut meter,
            )
            .unwrap_or_else(|_| panic!("equal chain of {depth} compares equal"));
            let mut meter = WorkMeter::over(&mut budget);
            assert!(
                compare(
                    TermNode::from_ref(document.root()),
                    &chain(depth, 2),
                    BasePath::root().key("row"),
                    &[],
                    &mut meter,
                )
                .is_err(),
                "a differing chain of {depth} is refused"
            );
        }
    }

    #[test]
    fn document_comparison_keeps_the_first_difference_pointer() {
        let document =
            quire_canonical::read(br#"{"b":2,"a":9}"#, u64::MAX).expect("source document");
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut meter = WorkMeter::over(&mut budget);
        let failure = compare(
            TermNode::from_ref(document.root()),
            &Pair { a: 1, b: 2 },
            BasePath::root().key("row"),
            &[],
            &mut meter,
        )
        .expect_err("different first field");

        let ValidationFailure::Refused(failure) = failure else {
            panic!("comparison refusal")
        };
        assert_eq!(failure.primary_path().unwrap().to_string(), "/row/a");
    }

    #[test]
    fn document_comparison_locates_an_unmodeled_member() {
        let document =
            quire_canonical::read(br#"{"a":1,"b":2,"c":3}"#, u64::MAX).expect("source document");
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut meter = WorkMeter::over(&mut budget);
        let failure = compare(
            TermNode::from_ref(document.root()),
            &Pair { a: 1, b: 2 },
            BasePath::root().key("row"),
            &[],
            &mut meter,
        )
        .expect_err("extra field");

        let ValidationFailure::Refused(failure) = failure else {
            panic!("comparison refusal")
        };
        assert_eq!(failure.primary_path().unwrap().to_string(), "/row/c");
    }

    #[test]
    fn capability_report_compares_as_a_closed_typed_sequence() {
        let document = quire_canonical::read(
            br#"[{"feature":"quire.value.complete/v1","disposition":"available"}]"#,
            u64::MAX,
        )
        .expect("source document");
        let typed = [CheckedCapability {
            feature: "quire.value.complete/v1".into(),
            disposition: CheckedCapabilityDisposition::Available,
        }];
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut meter = WorkMeter::over(&mut budget);
        compare(
            TermNode::from_ref(document.root()),
            &typed,
            BasePath::root().key("capability_report"),
            &[],
            &mut meter,
        )
        .expect("typed capability report matches source");
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn lossless_dynamic_keys_borrow_or_pay_exactly_before_escaping() {
        use std::collections::BTreeMap;
        let document = quire_canonical::read(r#"{"é/~":7,"12":9}"#.as_bytes(), u64::MAX).unwrap();
        let typed = BTreeMap::from([("é/~", 7_u64), ("12", 9)]);
        let run = |source: &quire_canonical::Document, limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(compare(
                    TermNode::from_ref(source.root()),
                    &typed,
                    BasePath::root().key("row"),
                    &[],
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.unwrap(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (healthy, allocated, charged) = run(&document, 0);
        healthy.unwrap();
        assert_eq!((allocated, charged), (0, 0));
        let missing = quire_canonical::read(br#"{"12":9}"#, u64::MAX).unwrap();
        let (failure, allocated, required) = run(&missing, u64::MAX);
        let ValidationFailure::Refused(refusal) = failure.unwrap_err() else {
            panic!("shape refusal")
        };
        assert_eq!(refusal.primary_path().unwrap().as_str(), "/row/12");
        assert_eq!(allocated, required);
        let (exact, allocated, charged) = run(&missing, required);
        assert!(matches!(exact, Err(ValidationFailure::Refused(_))));
        assert_eq!((allocated, charged), (required, required));
        for limit in [0, required - 1] {
            let (short, allocated, charged) = run(&missing, limit);
            assert!(matches!(short, Err(ValidationFailure::Budget(_))));
            assert_eq!(allocated, charged);
            assert!(charged <= limit);
        }
    }

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-513")]
    #[test]
    fn lossless_nested_fault_order_is_lexical_and_array_indices_are_numeric() {
        #[derive(Serialize)]
        struct Reversed {
            z: u64,
            a: Vec<Pair>,
        }
        let document = quire_canonical::read(
            r#"{"z":9,"a":[{"a":0,"b":2}],"0/~é":1}"#.as_bytes(),
            u64::MAX,
        )
        .unwrap();
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let failure = compare(
            TermNode::from_ref(document.root()),
            &Reversed {
                z: 0,
                a: vec![Pair { a: 1, b: 2 }],
            },
            BasePath::root().key("row"),
            &[],
            &mut WorkMeter::over(&mut budget),
        )
        .unwrap_err();
        let ValidationFailure::Refused(refusal) = failure else {
            panic!("extra field precedes nested and reversed typed faults")
        };
        assert_eq!(refusal.primary_path().unwrap().as_str(), "/row/0~1~0é");
        let document = quire_canonical::read(br#"[0,0,0,0,0,0,0,0,0,0,0]"#, u64::MAX).unwrap();
        let mut typed = [0_u64; 11];
        typed[2] = 1;
        typed[10] = 1;
        let failure = compare(
            TermNode::from_ref(document.root()),
            &typed,
            BasePath::root().key("row"),
            &[],
            &mut WorkMeter::over(&mut budget),
        )
        .unwrap_err();
        let ValidationFailure::Refused(refusal) = failure else {
            panic!("array mismatch")
        };
        assert_eq!(refusal.primary_path().unwrap().as_str(), "/row/2");
    }
}
