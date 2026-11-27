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
//! same structure when their skeletons are the same, and the first token where they part names the change. (M5-X2)
//! The skeleton keeps each number's value beside its blank, which `Ruleset::check` compares with the built-in rules'
//! ([`Skeleton::numbers_beside`]).

mod serializer;

use std::fmt;

use serde::Serialize;

use serializer::Never;

/// A place in a ruleset: the entry (the kind, the unit base, the tech or the ability, by name, or the part of the
/// ruleset: `factions`, `tuning`) and the field, the innermost one named. Where a ruleset's structure differs from
/// another's ([`super::Ruleset::structural_diff`]), or (M5-X1b) where a number isn't finite
/// ([`super::Ruleset::non_finite`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RuleField {
    pub entry: &'static str,
    pub field: &'static str,
}

impl fmt::Display for RuleField {
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

/// (M5-X2) The name of the first enum variant `value` serializes, `""` for none: a kind's (`ObjectKind::HandOfHeaven`'s
/// "HandOfHeaven"), as a [`RuleField`] names its entry.
pub(super) fn variant_name<T: Serialize + ?Sized>(value: &T) -> &'static str {
    let skeleton = Skeleton::of(value);
    let mut variants = skeleton.steps.iter().filter_map(|step| match step.token {
        Token::Variant(_, variant) => Some(variant),
        _ => None,
    });
    variants.next().unwrap_or("")
}

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

/// A token with where it was: the entry and the innermost field ([`RuleField`]).
#[derive(Copy, Clone, Debug)]
struct Step {
    token: Token,
    entry: &'static str,
    field: &'static str,
    /// (M5-X2) A number's value, `None` for any other token: what `Ruleset::check` compares
    /// ([`Skeleton::numbers_beside`]), which the structure doesn't.
    value: Option<f64>,
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
    /// (M5-X1b) Whether the value being serialized stays out of the skeleton: a field in [`FREE`], whose numbers are
    /// checked all the same ([`Skeleton::non_finite`]).
    muted: bool,
    /// (M5-X1b) The first number that isn't finite, if one isn't, and where.
    pub(super) non_finite: Option<RuleField>,
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
    pub(super) fn diff(&self, other: &Skeleton) -> Option<RuleField> {
        let at = |step: &Step| RuleField {
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

    /// (M5-X2) The numbers of `other`, a skeleton of the same structure, each beside this one's at the same place,
    /// with where they are, in order: up to the first step where the two part, if they do (`Ruleset::check`). The
    /// numbers in [`FREE`] fields aren't in skeletons.
    pub(super) fn numbers_beside<'a>(
        &'a self,
        other: &'a Skeleton,
    ) -> impl Iterator<Item = (f64, f64, RuleField)> + 'a {
        (self.steps.iter().zip(&other.steps))
            .take_while(|(a, b)| a.token == b.token)
            .filter_map(|(a, b)| {
                let at = RuleField {
                    entry: b.entry,
                    field: b.field,
                };
                Some((a.value?, b.value?, at))
            })
    }

    fn push(&mut self, token: Token) {
        self.push_value(token, None);
    }

    fn push_value(&mut self, token: Token, value: Option<f64>) {
        if self.muted {
            return;
        }
        self.steps.push(Step {
            token,
            entry: self.entry,
            field: self.field,
            value,
        });
    }

    /// A float, by its bits and its value: noted where it is if it isn't finite.
    fn float(&mut self, bits: u64, value: f64) {
        if !value.is_finite() && self.non_finite.is_none() {
            self.non_finite = Some(RuleField {
                entry: self.entry,
                field: self.field,
            });
        }
        self.number(bits, value);
    }

    /// A number, by its bits (a structure's, [`KEPT`]) and its value.
    fn number(&mut self, bits: u64, value: f64) {
        let token = if self.keep_number {
            Token::Number(bits)
        } else {
            Token::Blank
        };
        self.push_value(token, Some(value));
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
            let muted = std::mem::replace(&mut self.muted, true);
            value.serialize(&mut *self)?;
            self.muted = muted;
        } else {
            let keep = std::mem::replace(&mut self.keep_number, KEPT.contains(&(within, key)));
            value.serialize(&mut *self)?;
            self.keep_number = keep;
        }
        self.field = field;
        Ok(())
    }
}
