//! The skeleton's serializer (`super::Skeleton`): the steps each of serde's data types adds. Split out of
//! `ruleset/structure.rs` whole (slice M5-X2, plan decision T28), which (M5-X2) also keeps each number's value for
//! `Ruleset::check`.

use std::fmt;

use serde::ser::{self, Serialize};

use super::{Skeleton, Token};

/// The skeleton's serialization never fails; serde needs an error type all the same.
#[derive(Debug)]
pub(in crate::ruleset) struct Never;

impl fmt::Display for Never {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a skeleton never fails")
    }
}

impl std::error::Error for Never {}

impl ser::Error for Never {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Never
    }
}

impl ser::Serializer for &mut Skeleton {
    type Ok = ();
    type Error = Never;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), Never> {
        self.push(Token::Bool(v));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Never> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Never> {
        self.number(v as u64, v as f64);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Never> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Never> {
        self.number(v, v as f64);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Never> {
        self.float(v.to_bits().into(), v.into());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Never> {
        self.float(v.to_bits(), v);
        Ok(())
    }

    fn serialize_char(self, _: char) -> Result<(), Never> {
        self.push(Token::Blank);
        Ok(())
    }

    fn serialize_str(self, _: &str) -> Result<(), Never> {
        self.push(Token::Blank);
        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<(), Never> {
        self.push(Token::Blank);
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Never> {
        self.push(Token::None);
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Never> {
        self.push(Token::Some);
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Never> {
        self.push(Token::Unit);
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Never> {
        self.push(Token::Unit);
        Ok(())
    }

    fn serialize_unit_variant(self, name: &'static str, _: u32, variant: &'static str) -> Result<(), Never> {
        // A table's key (a kind, a base, a tech, an ability) names the entry its value is.
        if self.in_key {
            self.entry = variant;
        }
        self.push(Token::Variant(name, variant));
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, value: &T) -> Result<(), Never> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Never> {
        self.push(Token::Variant(name, variant));
        self.within.push((name, self.entry));
        let result = value.serialize(&mut *self);
        self.within.pop();
        result
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self, Never> {
        self.open("");
        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self, Never> {
        self.open("");
        Ok(self)
    }

    fn serialize_tuple_struct(self, name: &'static str, _: usize) -> Result<Self, Never> {
        self.open(name);
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self, Never> {
        self.push(Token::Variant(name, variant));
        self.open(name);
        Ok(self)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self, Never> {
        self.open("");
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, _: usize) -> Result<Self, Never> {
        self.open(name);
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self, Never> {
        self.push(Token::Variant(name, variant));
        self.open(name);
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeMap for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Never> {
        self.in_key = true;
        let result = key.serialize(&mut **self);
        self.in_key = false;
        result
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Never> {
        self.field(key, value)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Skeleton {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Never> {
        self.field(key, value)
    }

    fn end(self) -> Result<(), Never> {
        self.close();
        Ok(())
    }
}
