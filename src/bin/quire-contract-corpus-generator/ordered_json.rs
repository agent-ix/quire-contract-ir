//! JSON objects with declaration order preserved for byte-exact corpus regeneration.

use std::ops::{Index, IndexMut};

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum J {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
}

impl J {
    pub(super) fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub(super) fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
        match self {
            Self::Object(entries) => entries
                .iter_mut()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub(super) fn set(&mut self, key: &str, value: impl Into<Self>) {
        let Self::Object(entries) = self else {
            panic!("fixture builder expected an object");
        };
        if let Some((_, slot)) = entries.iter_mut().find(|(name, _)| name == key) {
            *slot = value.into();
        } else {
            entries.push((key.to_owned(), value.into()));
        }
    }

    pub(super) fn remove(&mut self, key: &str) -> Option<Self> {
        let Self::Object(entries) = self else {
            return None;
        };
        entries
            .iter()
            .position(|(name, _)| name == key)
            .map(|index| entries.remove(index).1)
    }

    pub(super) fn as_str(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub(super) fn as_array(&self) -> Option<&[Self]> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub(super) fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub(super) fn push(&mut self, value: impl Into<Self>) {
        self.as_array_mut()
            .expect("fixture builder expected an array")
            .push(value.into());
    }
}

impl Index<&str> for J {
    type Output = Self;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("fixture builder missing key {key}"))
    }
}

impl IndexMut<&str> for J {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        let Self::Object(entries) = self else {
            panic!("fixture builder expected an object")
        };
        &mut entries
            .iter_mut()
            .find(|(name, _)| name == key)
            .unwrap_or_else(|| panic!("fixture builder missing key {key}"))
            .1
    }
}

impl Index<usize> for J {
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_array().expect("fixture builder expected an array")[index]
    }
}

impl IndexMut<usize> for J {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self
            .as_array_mut()
            .expect("fixture builder expected an array")[index]
    }
}

impl From<&str> for J {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
impl From<String> for J {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<bool> for J {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<Vec<J>> for J {
    fn from(value: Vec<J>) -> Self {
        Self::Array(value)
    }
}
impl From<i32> for J {
    fn from(value: i32) -> Self {
        Self::Number(value.into())
    }
}
impl From<i64> for J {
    fn from(value: i64) -> Self {
        Self::Number(value.into())
    }
}
impl From<u32> for J {
    fn from(value: u32) -> Self {
        Self::Number(value.into())
    }
}
impl From<u64> for J {
    fn from(value: u64) -> Self {
        Self::Number(value.into())
    }
}
impl From<usize> for J {
    fn from(value: usize) -> Self {
        Self::Number(u64::try_from(value).expect("fixture index fits u64").into())
    }
}

impl Serialize for J {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Number(value) => value.serialize(serializer),
            Self::String(value) => serializer.serialize_str(value),
            Self::Array(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Self::Object(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for J {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedVisitor;
        impl<'de> Visitor<'de> for OrderedVisitor {
            type Value = J;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<J, E> {
                Ok(J::Null)
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<J, E> {
                Ok(J::Null)
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<J, E> {
                Ok(J::Bool(value))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<J, E> {
                Ok(J::Number(value.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<J, E> {
                Ok(J::Number(value.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<J, E> {
                serde_json::Number::from_f64(value)
                    .map(J::Number)
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<J, E> {
                Ok(J::String(value.to_owned()))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<J, E> {
                Ok(J::String(value))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<J, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(J::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<J, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(J::Object(entries))
            }
        }
        deserializer.deserialize_any(OrderedVisitor)
    }
}
