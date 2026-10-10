//! Compare a closed typed value with its source without constructing a second
//! JSON tree. The strict/canonical scans and typed decode have already run.

use super::ValidationFailure;
use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
use serde::ser::{
    Error as _, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde::Serialize;
use serde_json::value::RawValue;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
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
    raw: &RawValue,
    typed: &T,
    at: JsonPointer,
    skipped_members: &'static [&'static str],
) -> Result<(), ValidationFailure> {
    typed
        .serialize(Compare {
            raw,
            at,
            skipped_members,
        })
        .map_err(|error| error.0)
}

struct Compare<'a> {
    raw: &'a RawValue,
    at: JsonPointer,
    // Applied to this object only, never inherited by nested objects.
    skipped_members: &'static [&'static str],
}

impl<'a> Compare<'a> {
    fn mismatch(&self) -> CompareError {
        CompareError(malformed(self.at.clone()))
    }

    fn scalar<T: serde::de::DeserializeOwned + PartialEq>(
        self,
        expected: T,
    ) -> Result<(), CompareError> {
        let original: T = serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        if original == expected {
            Ok(())
        } else {
            Err(self.mismatch())
        }
    }

    fn object(self) -> Result<Children<'a>, CompareError> {
        let mut members: BTreeMap<String, &'a RawValue> =
            serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        for key in self.skipped_members {
            members.remove(*key);
        }
        Ok(Children {
            at: self.at,
            kind: ChildrenKind::Object {
                members,
                skipped_members: self.skipped_members,
                pending_key: None,
                first: None,
            },
        })
    }

    fn sequence(self, expected_len: Option<usize>) -> Result<Children<'a>, CompareError> {
        let items: Vec<&'a RawValue> =
            serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        if expected_len.is_some_and(|length| length != items.len()) {
            return Err(self.mismatch());
        }
        Ok(Children {
            at: self.at,
            kind: ChildrenKind::Sequence { items, next: 0 },
        })
    }
}

impl<'a> Serializer for Compare<'a> {
    type Ok = ();
    type Error = CompareError;
    type SerializeSeq = Children<'a>;
    type SerializeTuple = Children<'a>;
    type SerializeTupleStruct = Children<'a>;
    type SerializeTupleVariant = Children<'a>;
    type SerializeMap = Children<'a>;
    type SerializeStruct = Children<'a>;
    type SerializeStructVariant = Children<'a>;

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
        let number =
            serde_json::Number::from_f64(f64::from(value)).ok_or_else(|| self.mismatch())?;
        self.scalar(number)
    }
    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        let number = serde_json::Number::from_f64(value).ok_or_else(|| self.mismatch())?;
        self.scalar(number)
    }
    fn serialize_char(self, value: char) -> Result<(), Self::Error> {
        self.serialize_str(&value.to_string())
    }
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        let original: Cow<'_, str> =
            serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        if original == value {
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
        children.field(variant, value)?;
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
        let members: BTreeMap<String, &RawValue> =
            serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        if members.len() != 1 {
            return Err(self.mismatch());
        }
        let raw = *members.get(variant).ok_or_else(|| self.mismatch())?;
        Compare {
            raw,
            at: self.at.key(variant),
            skipped_members: &[],
        }
        .sequence(Some(len))
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
        let members: BTreeMap<String, &RawValue> =
            serde_json::from_str(self.raw.get()).map_err(|_| self.mismatch())?;
        if members.len() != 1 {
            return Err(self.mismatch());
        }
        let raw = *members.get(variant).ok_or_else(|| self.mismatch())?;
        Compare {
            raw,
            at: self.at.key(variant),
            skipped_members: &[],
        }
        .object()
    }
}

struct Children<'a> {
    at: JsonPointer,
    kind: ChildrenKind<'a>,
}

enum ChildrenKind<'a> {
    Sequence {
        items: Vec<&'a RawValue>,
        next: usize,
    },
    Object {
        members: BTreeMap<String, &'a RawValue>,
        skipped_members: &'static [&'static str],
        pending_key: Option<String>,
        first: Option<(String, CompareError)>,
    },
}

impl Children<'_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CompareError> {
        let ChildrenKind::Sequence { items, next } = &mut self.kind else {
            unreachable!()
        };
        let raw = items
            .get(*next)
            .ok_or_else(|| CompareError(malformed(self.at.clone())))?;
        let at = self.at.clone().index(*next);
        *next += 1;
        value.serialize(Compare {
            raw,
            at,
            skipped_members: &[],
        })
    }

    fn field<T: ?Sized + Serialize>(&mut self, key: &str, value: &T) -> Result<(), CompareError> {
        let ChildrenKind::Object {
            members,
            skipped_members,
            first,
            ..
        } = &mut self.kind
        else {
            unreachable!()
        };
        if skipped_members.contains(&key) {
            return Ok(());
        }
        let result = match members.remove(key) {
            Some(raw) => value.serialize(Compare {
                raw,
                at: self.at.clone().key(key),
                skipped_members: &[],
            }),
            // `first_difference` reports the container if the typed form has
            // a member that the source did not have.
            None => Err(CompareError(malformed(self.at.clone()))),
        };
        if let Err(error) = result {
            if first.as_ref().is_none_or(|(prior, _)| key < prior.as_str()) {
                *first = Some((key.to_owned(), error));
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<(), CompareError> {
        match self.kind {
            ChildrenKind::Sequence { items, next } => {
                if next == items.len() {
                    Ok(())
                } else {
                    Err(CompareError(malformed(self.at)))
                }
            }
            ChildrenKind::Object { members, first, .. } => {
                let extra = members.into_iter().next().map(|(key, _)| key);
                match (first, extra) {
                    (Some((key, _error)), Some(extra)) if extra < key => {
                        Err(CompareError(malformed(self.at.key(&extra))))
                    }
                    (Some((_, error)), _) => Err(error),
                    (None, Some(extra)) => Err(CompareError(malformed(self.at.key(&extra)))),
                    (None, None) => Ok(()),
                }
            }
        }
    }
}

impl SerializeSeq for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTuple for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleStruct for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeTupleVariant for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeMap for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let ChildrenKind::Object { pending_key, .. } = &mut self.kind else {
            unreachable!()
        };
        let serialized = serde_json::to_string(key).map_err(|_| CompareError::custom("map key"))?;
        *pending_key =
            Some(serde_json::from_str(&serialized).map_err(|_| CompareError::custom("map key"))?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let ChildrenKind::Object { pending_key, .. } = &mut self.kind else {
            unreachable!()
        };
        let key = pending_key
            .take()
            .ok_or_else(|| CompareError::custom("map value without key"))?;
        self.field(&key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeStruct for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.field(key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
impl SerializeStructVariant for Children<'_> {
    type Ok = ();
    type Error = CompareError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.field(key, value)
    }
    fn end(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
