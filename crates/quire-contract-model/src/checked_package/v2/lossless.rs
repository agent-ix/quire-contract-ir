//! Compare a closed typed value with its source without constructing a second
//! JSON tree. The strict/canonical scans and typed decode have already run.

use super::{TermNode, ValidationFailure};
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use serde::ser::{
    Error as _, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

const MAX_PATH_DEPTH: usize = 128;

struct Path {
    base: JsonPointer,
    segments: [Option<PathSegment>; MAX_PATH_DEPTH],
    depth: usize,
}

enum PathSegment {
    Key(Cow<'static, str>),
    Index(usize),
}

impl Path {
    fn push(&mut self, segment: PathSegment) -> bool {
        let Some(slot) = self.segments.get_mut(self.depth) else {
            return false;
        };
        *slot = Some(segment);
        self.depth += 1;
        true
    }

    fn pop(&mut self) -> Option<PathSegment> {
        self.depth = self.depth.checked_sub(1)?;
        self.segments[self.depth].take()
    }

    fn pointer(&self) -> JsonPointer {
        self.segments.iter().take(self.depth).flatten().fold(
            self.base.clone(),
            |pointer, segment| match segment {
                PathSegment::Key(key) => pointer.key(key),
                PathSegment::Index(index) => pointer.index(*index),
            },
        )
    }

    fn pointer_with(&self, segment: &PathSegment) -> JsonPointer {
        match segment {
            PathSegment::Key(key) => self.pointer().key(key),
            PathSegment::Index(index) => self.pointer().index(*index),
        }
    }
}

#[derive(Debug)]
pub(super) struct CompareError(ValidationFailure);

impl fmt::Display for CompareError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "closed wire differs from typed decode")
    }
}

impl std::error::Error for CompareError {}

impl serde::ser::Error for CompareError {
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self(malformed(JsonPointer::root()))
    }
}

pub(super) fn compare<T: Serialize>(
    raw: TermNode<'_>,
    typed: &T,
    at: JsonPointer,
    skipped_members: &'static [&'static str],
) -> Result<(), ValidationFailure> {
    let mut path = Path {
        base: at,
        segments: std::array::from_fn(|_| None),
        depth: 0,
    };
    typed
        .serialize(Compare {
            raw,
            path: &mut path,
            skipped_members,
        })
        .map_err(|error| error.0)
}

struct Compare<'a, 'p> {
    raw: TermNode<'a>,
    path: &'p mut Path,
    // Applied to this object only, never inherited by nested objects.
    skipped_members: &'static [&'static str],
}

impl<'a, 'p> Compare<'a, 'p> {
    fn mismatch(&self) -> CompareError {
        CompareError(malformed(self.path.pointer()))
    }

    fn scalar<T: Serialize>(self, expected: T) -> Result<(), CompareError> {
        expected.serialize(ScalarCompare {
            raw: self.raw,
            path: self.path,
        })
    }

    fn object(self) -> Result<Children<'a, 'p>, CompareError> {
        if !self.raw.is_object() {
            return Err(self.mismatch());
        }
        Ok(Children {
            path: self.path,
            pop_on_finish: false,
            kind: ChildrenKind::Object {
                raw: self.raw,
                skipped_members: self.skipped_members,
                pending_key: None,
                seen: [None; 64],
                seen_len: 0,
                seen_count: 0,
                first: None,
            },
        })
    }

    fn sequence(self, expected_len: Option<usize>) -> Result<Children<'a, 'p>, CompareError> {
        let items = self.raw.items().ok_or_else(|| self.mismatch())?;
        if expected_len.is_some_and(|length| length != items.len()) {
            return Err(self.mismatch());
        }
        Ok(Children {
            path: self.path,
            pop_on_finish: false,
            kind: ChildrenKind::Sequence { items, next: 0 },
        })
    }
}

struct ScalarCompare<'a, 'p> {
    raw: TermNode<'a>,
    path: &'p mut Path,
}

impl ScalarCompare<'_, '_> {
    fn mismatch(&self) -> CompareError {
        CompareError(malformed(self.path.pointer()))
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

impl Serializer for ScalarCompare<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = serde::ser::Impossible<(), CompareError>;
    type SerializeTuple = serde::ser::Impossible<(), CompareError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), CompareError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), CompareError>;
    type SerializeMap = serde::ser::Impossible<(), CompareError>;
    type SerializeStruct = serde::ser::Impossible<(), CompareError>;
    type SerializeStructVariant = serde::ser::Impossible<(), CompareError>;

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

impl<'a, 'p> Serializer for Compare<'a, 'p> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = Children<'a, 'p>;
    type SerializeTuple = Children<'a, 'p>;
    type SerializeTupleStruct = Children<'a, 'p>;
    type SerializeTupleVariant = Children<'a, 'p>;
    type SerializeMap = Children<'a, 'p>;
    type SerializeStruct = Children<'a, 'p>;
    type SerializeStructVariant = Children<'a, 'p>;

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
        if !self.path.push(PathSegment::Key(Cow::Borrowed(variant))) {
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
        if !self.path.push(PathSegment::Key(Cow::Borrowed(variant))) {
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

struct Children<'a, 'p> {
    path: &'p mut Path,
    pop_on_finish: bool,
    kind: ChildrenKind<'a>,
}

enum ChildrenKind<'a> {
    Sequence {
        items: super::TermItems<'a>,
        next: usize,
    },
    Object {
        raw: TermNode<'a>,
        skipped_members: &'static [&'static str],
        pending_key: Option<String>,
        seen: [Option<&'static str>; 64],
        seen_len: usize,
        seen_count: usize,
        first: Option<(String, CompareError)>,
    },
}

impl Children<'_, '_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        let ChildrenKind::Sequence { items, next } = &mut self.kind else {
            unreachable!()
        };
        let raw = items
            .next()
            .ok_or_else(|| CompareError(malformed(self.path.pointer())))?;
        if !self.path.push(PathSegment::Index(*next)) {
            return Err(CompareError(malformed(self.path.pointer())));
        }
        *next += 1;
        let result = value.serialize(Compare {
            raw,
            path: self.path,
            skipped_members: &[],
        });
        self.path.pop();
        result
    }

    fn field<T: ?Sized + Serialize>(&mut self, key: String, value: &T) -> Result<(), CompareError> {
        self.field_inner(Cow::Owned(key), None, value)
    }

    fn field_static<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CompareError> {
        self.field_inner(Cow::Borrowed(key), Some(key), value)
    }

    fn field_inner<T: ?Sized + Serialize>(
        &mut self,
        key: Cow<'static, str>,
        static_key: Option<&'static str>,
        value: &T,
    ) -> Result<(), CompareError> {
        let ChildrenKind::Object {
            raw,
            skipped_members,
            seen,
            seen_len,
            seen_count,
            first,
            ..
        } = &mut self.kind
        else {
            unreachable!()
        };
        if skipped_members.contains(&key.as_ref()) {
            return Ok(());
        }
        if !self.path.push(PathSegment::Key(key)) {
            return Err(CompareError(malformed(self.path.pointer())));
        }
        let key = match self.path.segments[self.path.depth - 1].as_ref() {
            Some(PathSegment::Key(key)) => key.as_ref(),
            _ => unreachable!(),
        };
        let result = match raw.get(key) {
            Some(child) => {
                *seen_count += 1;
                if let Some(key) = static_key {
                    if *seen_len < seen.len() {
                        seen[*seen_len] = Some(key);
                        *seen_len += 1;
                    }
                }
                value.serialize(Compare {
                    raw: child,
                    path: self.path,
                    skipped_members: &[],
                })
            }
            // `first_difference` reports the container if the typed form has
            // a member that the source did not have.
            None => Err(CompareError(malformed(self.path.pointer()))),
        };
        let key = match self.path.pop() {
            Some(PathSegment::Key(key)) => key,
            _ => unreachable!(),
        };
        if let Err(error) = result {
            if first
                .as_ref()
                .is_none_or(|(prior, _)| key.as_ref() < prior.as_str())
            {
                *first = Some((key.into_owned(), error));
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<(), CompareError> {
        let pop_on_finish = self.pop_on_finish;
        let path = self.path;
        let result = match self.kind {
            ChildrenKind::Sequence { items, .. } => {
                if items.len() == 0 {
                    Ok(())
                } else {
                    Err(CompareError(malformed(path.pointer())))
                }
            }
            ChildrenKind::Object {
                raw,
                skipped_members,
                seen,
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
                        .map(|(key, _)| key.to_owned())
                        .min();
                    match (first, extra) {
                        (Some((key, _error)), Some(extra)) if extra < key => Err(CompareError(
                            malformed(path.pointer_with(&PathSegment::Key(Cow::Owned(extra)))),
                        )),
                        (Some((_, error)), _) => Err(error),
                        (None, Some(extra)) => Err(CompareError(malformed(
                            path.pointer_with(&PathSegment::Key(Cow::Owned(extra))),
                        ))),
                        (None, None) => Ok(()),
                    }
                }
            }
        };
        if pop_on_finish {
            path.pop();
        }
        result
    }
}

impl SerializeSeq for Children<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTuple for Children<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleStruct for Children<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleVariant for Children<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

struct MapKey;

impl Serializer for MapKey {
    type Ok = String;
    type Error = CompareError;
    type SerializeSeq = serde::ser::Impossible<String, CompareError>;
    type SerializeTuple = serde::ser::Impossible<String, CompareError>;
    type SerializeTupleStruct = serde::ser::Impossible<String, CompareError>;
    type SerializeTupleVariant = serde::ser::Impossible<String, CompareError>;
    type SerializeMap = serde::ser::Impossible<String, CompareError>;
    type SerializeStruct = serde::ser::Impossible<String, CompareError>;
    type SerializeStructVariant = serde::ser::Impossible<String, CompareError>;

    fn serialize_bool(self, value: bool) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_i8(self, value: i8) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_i16(self, value: i16) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_i32(self, value: i32) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_i64(self, value: i64) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_i128(self, value: i128) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_u8(self, value: u8) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_u16(self, value: u16) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_u32(self, value: u32) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_u64(self, value: u64) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_u128(self, value: u128) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_f32(self, value: f32) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_f64(self, value: f64) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_char(self, value: char) -> Result<String, CompareError> {
        Ok(value.to_string())
    }
    fn serialize_str(self, value: &str) -> Result<String, CompareError> {
        Ok(value.to_owned())
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<String, CompareError> {
        Err(CompareError::custom("byte map key"))
    }
    fn serialize_none(self) -> Result<String, CompareError> {
        Err(CompareError::custom("null map key"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<String, CompareError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<String, CompareError> {
        Err(CompareError::custom("unit map key"))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, CompareError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<String, CompareError> {
        Ok(variant.to_owned())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, CompareError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, CompareError> {
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

impl SerializeMap for Children<'_, '_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let ChildrenKind::Object { pending_key, .. } = &mut self.kind else {
            unreachable!()
        };
        *pending_key = Some(key.serialize(MapKey)?);
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
impl SerializeStruct for Children<'_, '_> {
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
impl SerializeStructVariant for Children<'_, '_> {
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
    use super::{compare, TermNode};
    use crate::checked_package::common::ValidationFailure;
    use crate::checked_package::shared::{CheckedCapability, JsonPointer};
    use crate::checked_package::v2::CheckedCapabilityDisposition;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Pair {
        a: u64,
        b: u64,
    }

    #[test]
    fn document_comparison_keeps_the_first_difference_pointer() {
        let document =
            quire_canonical::read(br#"{"b":2,"a":9}"#, u64::MAX).expect("source document");
        let failure = compare(
            TermNode::from_ref(document.root()),
            &Pair { a: 1, b: 2 },
            JsonPointer::root().key("row"),
            &[],
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
        let failure = compare(
            TermNode::from_ref(document.root()),
            &Pair { a: 1, b: 2 },
            JsonPointer::root().key("row"),
            &[],
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
        compare(
            TermNode::from_ref(document.root()),
            &typed,
            JsonPointer::root().key("capability_report"),
            &[],
        )
        .expect("typed capability report matches source");
    }
}
