//! What of a ruleset is its structure (plan decision T66, `Ruleset::structural_diff`): what a running game's state is
//! built on, so that `DebugCommand::SetRules` can't change it live (it needs a new game). The rule is one for every
//! field of every def, base, faction rule and tunable, those of later slices included:
//!
//! - **Numbers aren't structure:** every integer and float, whatever it counts (a cost, a time, a range, a capacity, a
//!   queue's length), applies at once. Names aren't either: the strings, which only label.
//! - **Everything else is:** an enum's variant (the kinds, the sides, the symmetry, the domains, the movement models,
//!   an attack's type), a flag, whether an optional part is there (`energy`, `abilities`, a `holds`, a turret, a
//!   repairer's `repairs`, a Shroud spell), and what a list holds and how long it is (the recipes' kinds, a building's
//!   `accepts`, the techs of a research building, a Tunnel's tiers). A footprint's and a Silhouette's sizes are
//!   numbers, but they shape the nav grids and the space every object takes, so they are structure too ([`KEPT`]).
//! - **Except** the fields in [`FREE`], which the game reads only when it starts something and keeps nothing derived
//!   from: they apply to what starts after the change.
//!
//! A value's structure is its serialization with every number and name blanked ([`Skeleton`]): two rulesets have the
//! same structure when their skeletons are the same, and the first token where they part names the change.

use std::fmt;

use serde::ser::{self, Serialize};

/// Where a ruleset's structure differs from another's ([`super::Ruleset::structural_diff`]): the entry (the kind, the
/// unit base, the tech or the ability, by name, or the part of the ruleset: `factions`, `tuning`) and the field, the
/// innermost one named.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StructuralChange {
    pub entry: &'static str,
    pub field: &'static str,
}

impl fmt::Display for StructuralChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}'s {}", self.entry, self.field)
    }
}

/// The non-number fields that aren't structure, by struct (or enum) and field, each read only as something starts:
/// - `Recipe::requires`, `StructureDef::requires` and `TechDef::requires_structures`, the TechPrerequisites: checked
///   when the work is queued or the building placed (`core/tech_and_research.md > Tech Prerequisites`: "checked only
///   when it is queued"), never of what already runs;
/// - `ProductionDef::default_rally`: whether a building has a default rally point is read once, as the building comes
///   into the game (`Sim::new_object`); a building keeps the rally it has;
/// - `TargetType::AoE`'s `blast`, the BlastDomain: fixed as each shot is fired (`combat::area::blast_domain`), so a
///   change takes the shots fired after it.
const FREE: &[(&str, &str)] = &[
    ("Recipe", "requires"),
    ("StructureDef", "requires"),
    ("TechDef", "requires_structures"),
    ("ProductionDef", "default_rally"),
    ("TargetType", "blast"),
];

/// The number fields that are structure, by struct (or enum) and field: the sizes of a footprint and of a Silhouette,
/// which the nav grids, the placement of every building and the space of every unit are built on (`Footprint`).
const KEPT: &[(&str, &str)] = &[("Footprint", "w"), ("Footprint", "h")];

/// One step of a value's serialization, numbers and names blanked.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Token {
    /// A number or a name, or a field in [`FREE`].
    Blank,
    /// A number that is structure ([`KEPT`]), by its bits.
    Number(u64),
    Bool(bool),
    None,
    Some,
    Unit,
    /// An enum's variant, by the enum's name and the variant's.
    Variant(&'static str, &'static str),
    /// A struct's field, by name.
    Field(&'static str),
    /// The start of a list, a map or a struct; [`Token::End`] closes it.
    Open,
    End,
}

/// A token with where it was: the entry and the innermost field ([`StructuralChange`]).
#[derive(Copy, Clone, Debug)]
struct Step {
    token: Token,
    entry: &'static str,
    field: &'static str,
}

/// A value's structure: its serialization's tokens (module docs).
#[derive(Default)]
pub(super) struct Skeleton {
    steps: Vec<Step>,
    entry: &'static str,
    field: &'static str,
    /// The structs, enums, lists and maps being serialized, innermost last, each by name (empty for a list or a map)
    /// with the entry outside it, which its end restores.
    within: Vec<(&'static str, &'static str)>,
    /// Whether a map key is being serialized: a variant there (a kind, a base, a tech, an ability) names the entry
    /// its value is, until the map ends.
    in_key: bool,
    /// Whether the number being serialized is structure ([`KEPT`]).
    keep_number: bool,
}

impl Skeleton {
    /// The structure of `value`.
    pub(super) fn of<T: Serialize + ?Sized>(value: &T) -> Skeleton {
        let mut skeleton = Skeleton {
            entry: "ruleset",
            field: "",
            ..Skeleton::default()
        };
        value
            .serialize(&mut skeleton)
            .expect("a ruleset serializes into a skeleton");
        skeleton
    }

    /// Where `other`'s structure first differs from this one's, if it does.
    pub(super) fn diff(&self, other: &Skeleton) -> Option<StructuralChange> {
        let at = |step: &Step| StructuralChange {
            entry: step.entry,
            field: step.field,
        };
        for (a, b) in self.steps.iter().zip(&other.steps) {
            if a.token != b.token {
                return Some(at(b));
            }
        }
        match self.steps.len().cmp(&other.steps.len()) {
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Less => Some(at(&other.steps[self.steps.len()])),
            std::cmp::Ordering::Greater => Some(at(&self.steps[other.steps.len()])),
        }
    }

    fn push(&mut self, token: Token) {
        self.steps.push(Step {
            token,
            entry: self.entry,
            field: self.field,
        });
    }

    fn number(&mut self, bits: u64) {
        let token = if self.keep_number {
            Token::Number(bits)
        } else {
            Token::Blank
        };
        self.push(token);
    }

    fn open(&mut self, name: &'static str) {
        self.within.push((name, self.entry));
        self.push(Token::Open);
    }

    fn close(&mut self) {
        if let Some((_, entry)) = self.within.pop() {
            self.entry = entry;
        }
        self.push(Token::End);
    }

    /// A struct's field `key` with its value, blanked if it is in [`FREE`].
    fn field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Never> {
        let within = self.within.last().map_or("", |&(name, _)| name);
        // The ruleset's own fields are its parts: `factions` and `tuning` (the tables' entries are named by key).
        if self.within.len() == 1 {
            self.entry = key;
        }
        let field = std::mem::replace(&mut self.field, key);
        self.push(Token::Field(key));
        if FREE.contains(&(within, key)) {
            self.push(Token::Blank);
        } else {
            let keep = std::mem::replace(&mut self.keep_number, KEPT.contains(&(within, key)));
            value.serialize(&mut *self)?;
            self.keep_number = keep;
        }
        self.field = field;
        Ok(())
    }
}

/// The skeleton's serialization never fails; serde needs an error type all the same.
#[derive(Debug)]
pub(super) struct Never;

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
        self.number(v as u64);
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
        self.number(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Never> {
        self.number(v.to_bits().into());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Never> {
        self.number(v.to_bits());
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
