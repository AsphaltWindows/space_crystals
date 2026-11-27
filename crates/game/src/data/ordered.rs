//! A JSON value that keeps its objects' members in order ([`Ordered`]), so that an exported file lists a struct's
//! fields in the order the struct declares them, as it serializes (`super::export`): `serde_json::Value` sorts them,
//! the workspace's `serde_json` being without its `preserve_order` feature (and a new dependency is not to be had).

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::{Number, Value};

/// A JSON value whose objects keep their members' order.
#[derive(Clone, Debug, PartialEq)]
pub enum Ordered {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Ordered>),
    Object(Vec<(String, Ordered)>),
}

impl Ordered {
    /// `value` as JSON, in its serialization's order.
    pub fn of<T: Serialize + ?Sized>(value: &T) -> Ordered {
        let text = serde_json::to_string(value).expect("the rules and the client's tuning serialize");
        serde_json::from_str(&text).expect("JSON reads back")
    }

    /// The member `key` of an object.
    pub fn get(&self, key: &str) -> Option<&Ordered> {
        match self {
            Ordered::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The same value as a `serde_json::Value`, its members sorted.
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("JSON converts")
    }
}

impl Serialize for Ordered {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Ordered::Null => serializer.serialize_unit(),
            Ordered::Bool(b) => serializer.serialize_bool(*b),
            Ordered::Number(n) => n.serialize(serializer),
            Ordered::String(s) => serializer.serialize_str(s),
            Ordered::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Ordered::Object(members) => {
                let mut map = serializer.serialize_map(Some(members.len()))?;
                for (key, value) in members {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Ordered, D::Error> {
        deserializer.deserialize_any(OrderedVisitor)
    }
}

struct OrderedVisitor;

impl<'de> Visitor<'de> for OrderedVisitor {
    type Value = Ordered;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Ordered, E> {
        Ok(Ordered::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Ordered, E> {
        Ok(Ordered::Null)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Ordered, D::Error> {
        Ordered::deserialize(deserializer)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Ordered, E> {
        Ok(Ordered::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Ordered, E> {
        Ok(Ordered::Number(v.into()))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Ordered, E> {
        Ok(Ordered::Number(v.into()))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Ordered, E> {
        Number::from_f64(v)
            .map(Ordered::Number)
            .ok_or_else(|| E::custom("a number that isn't finite"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Ordered, E> {
        Ok(Ordered::String(v.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Ordered, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Ordered::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
        let mut members = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Ordered>()? {
            members.push((key, value));
        }
        Ok(Ordered::Object(members))
    }
}
