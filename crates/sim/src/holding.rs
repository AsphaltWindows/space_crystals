//! Buildings that hold units — plan decision T60, slice M5-F1: one capability for every Cults building units go into,
//! the Armory first (`factions/cults/structures/armory.md`), later the Altar, the Temple, the Cathedral, the Shrine,
//! the Garrison, the Chop Shop and the Airstrip (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`, design change 0319), and the GDO's Bunker. A type has it by its def (`StructureDef::holds`,
//! [`HoldsDef`]): an entrance side, an exit, a capacity and the kinds it takes.
//!
//! - **Inside** (plan decision T30, `crate::network`): a unit in a building is held off the map, at
//!   `Place::Building`, as a unit in the Tunnel Network is: it is no object of the map, it keeps its id, its HP and its
//!   load (Placeholder (Q0104 point 3): "It keeps it inside and comes out with it"), and its orders are cleared. Its
//!   building's **pool** is the units held in it, longest inside first (`Sim::held_in`): the Armory's
//!   "StoredRecruits" (`factions/cults/structures/armory.md > ArmoryInstanceState`). The Recruits a training order
//!   reserves will leave it (slice M5-F2).
//! - **Entering** (`systems/controls/commands.md > Enter`: "The unit walks to the structure's entrance side and goes
//!   inside"; "Cults — Armory, Cathedral: Recruits only (to be trained), up to the building's internal capacity"; its
//!   TargetObject "the structure being entered", design change 0319, answering Q0108 point 1): M4-C's Enter order and
//!   entry path, one rule with the Tunnel's (`network::EntryPath`, `network::admits`). The entry path is the middle
//!   of the entrance side ([`entry_path`]; Placeholder (Q0104 point 1): "The middle of side B, as the Tunnel's Side
//!   A"). The building takes a unit of a kind it accepts ([`admits`]) once it is in operation (Placeholder (Q0104
//!   point 4): an Armory under construction can't be entered; "A right-click on one Assists", slice M5-E). A unit
//!   that reaches the entry path goes in if the pool has room (`Sim::room`; Placeholder (Q0104 point 5): "The pool
//!   only"); with none, "the Enter ends and it idles beside the Armory (the normal Failure rule)" (Placeholder (Q0104
//!   point 2)).
//! - **Eject All** (`factions/cults/structures/armory.md > Eject All`: "ejects all stored Recruits from the exit side
//!   in a rapid stream, one after another"): the pool comes out through the building's exit doorway, one at a time
//!   (`core/entities.md > Structure Type > Exit Doorway`, M2's `production::Doorway`), each once the one before it is
//!   out of the doorway, and walks straight out of it until slice M5-F2 gives the building its rally point
//!   (`sim/holding.rs`).
//! - **Evacuation** (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Everyone
//!   inside is evacuated alive"; "evacuated units appear around the building's footprint wherever there is open
//!   space, unharmed"): as the building leaves the game, every unit inside it comes out at the spots around its
//!   footprint (plan decision T62, `placement::spots`), idle (`sim/holding/evacuation.rs`).

use glam::Vec2;

use crate::defs::{HoldsDef, ObjectKind};
use crate::network::{EntryPath, NoTransit};
use crate::object::{Object, Placement};
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::units::SU_PER_GU;

/// The holding of `o`'s type ([`HoldsDef`]): `None` for anything but a building that holds units.
pub fn holds(o: &Object) -> Option<&'static HoldsDef> {
    o.def().structure.as_ref()?.holds.as_ref()
}

/// Whether `building` takes a unit of `kind` in (`systems/controls/commands.md > Enter`: "an own structure that can
/// hold units"): it is a building that holds units, placed, with an owner, in operation (`Object::is_operational`;
/// Placeholder (Q0104 point 4): "an Armory under construction can't be entered"), and its type accepts the kind
/// (`HoldsDef::accepts`). Whose units may go in, and whether its pool has room, are the sim's (`Sim::step_entries`).
pub fn admits(building: &Object, kind: ObjectKind) -> Result<(), NoTransit> {
    let holds = holds(building).ok_or(NoTransit::NotATunnel)?;
    if building.placement.is_none() || building.owner.is_none() || !building.is_operational() {
        return Err(NoTransit::NotATunnel);
    }
    kind.def().unit.as_ref().ok_or(NoTransit::NotAUnit)?;
    if holds.accepts.contains(&kind) {
        Ok(())
    } else {
        Err(NoTransit::NotAccepted)
    }
}

/// The entry path of `building`, a placed building that holds units: the middle of its entrance side
/// (`HoldsDef::entrance`), facing out of it (Placeholder (Q0104 point 1): "The middle of side B, as the Tunnel's Side
/// A"). `None` for anything else.
pub fn entry_path(building: &Object) -> Option<EntryPath> {
    let holds = holds(building)?;
    let placement = building.placement.as_ref()?;
    let outward = outward(building.kind, placement, holds.entrance)?;
    Some(EntryPath {
        outward,
        spot: side_middle(placement, outward),
    })
}

/// The unit vector out of the side lettered `letter` of a `kind` structure placed at `placement` (`crate::sides`):
/// of two or more so lettered, the first clockwise from the top.
fn outward(kind: ObjectKind, placement: &Placement, letter: SideLetter) -> Option<Vec2> {
    let letters = sides(kind, placement.orientation())?;
    let (x, y) = DIRECTIONS[letters.iter().position(|&l| l == letter)?];
    Some(Vec2::new(x as f32, y as f32))
}

/// The middle of the side of the footprint `placement` that faces along `outward`, on the side's line.
fn side_middle(placement: &Placement, outward: Vec2) -> Vec2 {
    let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
    placement.center() + outward * outward.abs().dot(half)
}

#[cfg(test)]
mod tests;
