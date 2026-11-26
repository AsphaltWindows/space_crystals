//! (M5-N2) Structures whose occupants fire from inside (`StructureDef::occupants_fire`, plan decision T72): the
//! Garrison's (`factions/cults/structures/garrison.md > Garrisoning`). How they fire is `combat::occupants`'.

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;

/// How the units a structure holds fire from it (`StructureDef::occupants_fire`): which kinds fire another kind's
/// attack, and which get a Range bonus. Every unit inside fires (`combat::occupants`), with the attack
/// [`OccupantFire::weapon_kind`] names and [`OccupantFire::range_bonus_gu`] added to its Range.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OccupantFire {
    /// The kinds inside that fire another kind's attack, each with that kind (`factions/cults/structures/garrison.md >
    /// Garrisoning`: "a Recruit or Disciple inside attacks with the Militant's AttackAttributes (see Militant)"). It is
    /// that kind's attack as its def has it: "**Augmentations don't change the borrowed attack** [...]: a Recruit or
    /// Disciple inside fires the Militant's attack unchanged".
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub stand_in: &'static [(ObjectKind, ObjectKind)],
    /// The Range bonus, grid units: "**Militants and Gunners** inside get **+1 grid unit of Range** on their attack".
    pub range_bonus_gu: f32,
    /// The kinds inside that get it: "Recruits and Disciples (firing with the Militant's attack) do not get the bonus".
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub bonus_for: &'static [ObjectKind],
}

impl OccupantFire {
    /// The kind whose attack a unit of `kind` inside fires: its stand-in's ([`OccupantFire::stand_in`]), else its own.
    pub fn weapon_kind(&self, kind: ObjectKind) -> ObjectKind {
        (self.stand_in.iter())
            .find(|&&(unit, _)| unit == kind)
            .map_or(kind, |&(_, stand_in)| stand_in)
    }

    /// The Range bonus a unit of `kind` inside gets, grid units: [`OccupantFire::range_bonus_gu`] for the kinds of
    /// [`OccupantFire::bonus_for`], none for the others.
    pub fn range_bonus(&self, kind: ObjectKind) -> f32 {
        if self.bonus_for.contains(&kind) {
            self.range_bonus_gu
        } else {
            0.0
        }
    }

    /// Whether a unit of `kind` inside fires its own attack ([`OccupantFire::weapon_kind`] is its kind): the one its
    /// augmentation may change, while a stand-in's is the stand-in type's own (`combat::occupants`).
    pub fn fires_own(&self, kind: ObjectKind) -> bool {
        self.weapon_kind(kind) == kind
    }
}
