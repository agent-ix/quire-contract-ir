//! Record a rejected closed-struct member during Serde's failed-decode path.
//!
//! The derive supplies each struct's field list to `deserialize_struct`; this
//! adapter observes that list without maintaining another wire schema. It
//! preserves Serde's own error and leaves pointer tracking to
//! `serde_path_to_error`.

use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::cell::Cell;
use std::fmt;

/// Whether the failed decode encountered a member outside a derived closed
/// struct's field list. Only the first failure is classified by the caller.
pub(super) struct ClosedMemberProvenance {
    unknown: Cell<bool>,
}

impl ClosedMemberProvenance {
    pub(super) fn new() -> Self {
        Self {
            unknown: Cell::new(false),
        }
    }

    pub(super) fn unknown_member(&self) -> bool {
        self.unknown.get()
    }

    pub(super) fn trace<D>(&self, deserializer: D) -> Traced<'_, D> {
        Traced {
            deserializer,
            provenance: self,
        }
    }
}

pub(super) struct Traced<'a, D> {
    deserializer: D,
    provenance: &'a ClosedMemberProvenance,
}

struct TracedSeed<'a, S> {
    seed: S,
    provenance: &'a ClosedMemberProvenance,
}

impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for TracedSeed<'_, S> {
    type Value = S::Value;

    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        self.seed.deserialize(self.provenance.trace(deserializer))
    }
}

struct TracedVisitor<'a, V> {
    visitor: V,
    fields: Option<&'static [&'static str]>,
    provenance: &'a ClosedMemberProvenance,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for TracedVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(formatter)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        self.visitor.visit_bool(value)
    }

    fn visit_i8<E: de::Error>(self, value: i8) -> Result<Self::Value, E> {
        self.visitor.visit_i8(value)
    }

    fn visit_i16<E: de::Error>(self, value: i16) -> Result<Self::Value, E> {
        self.visitor.visit_i16(value)
    }

    fn visit_i32<E: de::Error>(self, value: i32) -> Result<Self::Value, E> {
        self.visitor.visit_i32(value)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        self.visitor.visit_i64(value)
    }

    fn visit_i128<E: de::Error>(self, value: i128) -> Result<Self::Value, E> {
        self.visitor.visit_i128(value)
    }

    fn visit_u8<E: de::Error>(self, value: u8) -> Result<Self::Value, E> {
        self.visitor.visit_u8(value)
    }

    fn visit_u16<E: de::Error>(self, value: u16) -> Result<Self::Value, E> {
        self.visitor.visit_u16(value)
    }

    fn visit_u32<E: de::Error>(self, value: u32) -> Result<Self::Value, E> {
        self.visitor.visit_u32(value)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        self.visitor.visit_u64(value)
    }

    fn visit_u128<E: de::Error>(self, value: u128) -> Result<Self::Value, E> {
        self.visitor.visit_u128(value)
    }

    fn visit_f32<E: de::Error>(self, value: f32) -> Result<Self::Value, E> {
        self.visitor.visit_f32(value)
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        self.visitor.visit_f64(value)
    }

    fn visit_char<E: de::Error>(self, value: char) -> Result<Self::Value, E> {
        self.visitor.visit_char(value)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visitor.visit_str(value)
    }

    fn visit_borrowed_str<E: de::Error>(self, value: &'de str) -> Result<Self::Value, E> {
        self.visitor.visit_borrowed_str(value)
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        self.visitor.visit_string(value)
    }

    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        self.visitor.visit_bytes(value)
    }

    fn visit_borrowed_bytes<E: de::Error>(self, value: &'de [u8]) -> Result<Self::Value, E> {
        self.visitor.visit_borrowed_bytes(value)
    }

    fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        self.visitor.visit_byte_buf(value)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        self.visitor.visit_none()
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        self.visitor.visit_unit()
    }

    fn visit_some<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        self.visitor.visit_some(self.provenance.trace(deserializer))
    }

    fn visit_newtype_struct<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        self.visitor
            .visit_newtype_struct(self.provenance.trace(deserializer))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.visitor.visit_seq(TracedSeq {
            access,
            provenance: self.provenance,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.visitor.visit_map(TracedMap {
            access,
            fields: self.fields,
            provenance: self.provenance,
        })
    }

    fn visit_enum<A: EnumAccess<'de>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.visitor.visit_enum(TracedEnum {
            access,
            provenance: self.provenance,
        })
    }
}

struct TracedSeq<'a, A> {
    access: A,
    provenance: &'a ClosedMemberProvenance,
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for TracedSeq<'_, A> {
    type Error = A::Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.access.next_element_seed(TracedSeed {
            seed,
            provenance: self.provenance,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.access.size_hint()
    }
}

struct TracedMap<'a, A> {
    access: A,
    fields: Option<&'static [&'static str]>,
    provenance: &'a ClosedMemberProvenance,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for TracedMap<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let Some(fields) = self.fields else {
            return self.access.next_key_seed(seed);
        };
        let Some(key) = self.access.next_key::<String>()? else {
            return Ok(None);
        };
        use de::IntoDeserializer;
        let unknown = !fields.contains(&key.as_str());
        let result = seed.deserialize(key.into_deserializer());
        if unknown && result.is_err() {
            self.provenance.unknown.set(true);
        }
        result.map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.access.next_value_seed(TracedSeed {
            seed,
            provenance: self.provenance,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.access.size_hint()
    }
}

struct TracedEnum<'a, A> {
    access: A,
    provenance: &'a ClosedMemberProvenance,
}

impl<'a, 'de, A: EnumAccess<'de>> EnumAccess<'de> for TracedEnum<'a, A> {
    type Error = A::Error;
    type Variant = TracedVariant<'a, A::Variant>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let (value, variant) = self.access.variant_seed(seed)?;
        Ok((
            value,
            TracedVariant {
                variant,
                provenance: self.provenance,
            },
        ))
    }
}

struct TracedVariant<'a, A> {
    variant: A,
    provenance: &'a ClosedMemberProvenance,
}

impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for TracedVariant<'_, A> {
    type Error = A::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.variant.unit_variant()
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        self.variant.newtype_variant_seed(TracedSeed {
            seed,
            provenance: self.provenance,
        })
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.variant.tuple_variant(
            len,
            TracedVisitor {
                visitor,
                fields: None,
                provenance: self.provenance,
            },
        )
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.variant.struct_variant(
            fields,
            TracedVisitor {
                visitor,
                fields: Some(fields),
                provenance: self.provenance,
            },
        )
    }
}

macro_rules! forward {
    ($($method:ident($($argument:ident: $kind:ty),*)),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(
                self,
                $($argument: $kind,)*
                visitor: V,
            ) -> Result<V::Value, Self::Error> {
                self.deserializer.$method(
                    $($argument,)*
                    TracedVisitor {
                        visitor,
                        fields: None,
                        provenance: self.provenance,
                    },
                )
            }
        )*
    };
}

impl<'de, D: de::Deserializer<'de>> de::Deserializer<'de> for Traced<'_, D> {
    type Error = D::Error;

    forward! {
        deserialize_any(), deserialize_bool(), deserialize_i8(), deserialize_i16(),
        deserialize_i32(), deserialize_i64(), deserialize_i128(), deserialize_u8(),
        deserialize_u16(), deserialize_u32(), deserialize_u64(), deserialize_u128(),
        deserialize_f32(), deserialize_f64(), deserialize_char(), deserialize_str(),
        deserialize_string(), deserialize_bytes(), deserialize_byte_buf(),
        deserialize_option(), deserialize_unit(), deserialize_seq(), deserialize_map(),
        deserialize_identifier(), deserialize_ignored_any(),
        deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str),
        deserialize_tuple(len: usize),
        deserialize_tuple_struct(name: &'static str, len: usize),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer.deserialize_struct(
            name,
            fields,
            TracedVisitor {
                visitor,
                fields: Some(fields),
                provenance: self.provenance,
            },
        )
    }

    fn is_human_readable(&self) -> bool {
        self.deserializer.is_human_readable()
    }
}
