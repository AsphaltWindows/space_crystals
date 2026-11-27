//! Buildings that hold units — plan decision T60, slice M5-F1: one capability for every Cults building units go into,
//! the Armory first (`factions/cults/structures/armory.md`), later the Altar, the Temple, the Cathedral, the Shrine,
//! the Garrison, the Chop Shop and the Airstrip (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
//! Destruction`, design change 0319), and the GDO's Bunker. A type has it by its def (`StructureDef::holds`,
//! [`HoldsDef`]): an entrance side, an exit, a capacity and the kinds it takes.
//!
//! - **Inside** (plan decision T30, `crate::network`): a unit in a building is held off the map, at
//!   `Place::Building`, as a unit in the Tunnel Network is: it is no object of the map, it keeps its id, its HP and its
//!   load (design change 0347 point 3: "Loads: kept inside and carried out; lost if the Recruit is trained",
//!   `Sim::train`), and its orders are cleared. Its building's **pool** is the units held in it that no order reserves
//!   and that aren't on their way out, longest inside first (`Sim::pool`, slice M5-F2): the Armory's "StoredRecruits"
//!   (`factions/cults/structures/armory.md > ArmoryInstanceState`).
//! - **Entering** (`systems/controls/commands.md > Enter`: "The unit walks to the structure's entrance side and goes
//!   inside"; "Cults — Armory, Cathedral: Recruits only (to be trained), up to the building's internal capacity"; its
//!   TargetObject "the structure being entered", design change 0319, answering Q0108 point 1): M4-C's Enter order and
//!   entry path, one rule with the Tunnel's (`network::EntryPath`, `network::admits`). The entry path is the doorway
//!   at the middle of the entrance side ([`entry_path`]; design change 0347 point 1, `core/entities.md > Entrance and
//!   exit (production and research buildings)`: "The entrance is a dedicated doorway at the middle of the entrance side
//!   (like the Exit Doorway): units walk to it and go in there"): a unit goes in once its box is within reach of that
//!   middle (`EntryPath::reached`), and nowhere else along the side. The building takes a unit of a kind it accepts
//!   ([`admits`]) once it is in operation (0347 point 4, `factions/cults/structures/README.md > Entering pool
//!   buildings`: "a building under construction **can't be entered**; a right-click on it Assists", slice M5-E's
//!   Assist). A unit that reaches the entry path goes in if the building has room (`Sim::room`: every unit inside
//!   counts, those its orders reserved included; 0347 point 5: "Capacity counts reserved units: the queue can never
//!   exceed the capacity"); with none, "a unit arriving when the building is full has its Enter **end** and idles
//!   beside the building (the normal Failure rule)" (0347 point 2).
//! - **Eject All** (`factions/cults/structures/armory.md > Eject All`: "ejects all stored Recruits from the exit side
//!   in a rapid stream, one after another"): the pool comes out through the building's exit doorway, one at a time
//!   (`core/entities.md > Structure Type > Exit Doorway`, M2's `production::Doorway`), each once the one before it is
//!   out of the doorway, and walks to the building's rally point, straight out of it without one (design change 0348:
//!   "ejected and released units walk **to the RallyPoint**"; `sim/holding.rs`).
//! - **Evacuation** (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Everyone
//!   inside is evacuated alive"; design change 0349: "evacuated units **all appear at the same time inside its former
//!   footprint**, unharmed, spread over it"): as the building leaves the game, every unit inside it comes out on the
//!   footprint's tiles, idle, the units that overlap parted by the movement (`sim/holding/evacuation.rs`).

use glam::Vec2;

use crate::defs::{HoldsDef, ObjectKind};
use crate::network::{EntryPath, NoTransit};
use crate::object::{Object, Placement};
use crate::ruleset::rules;
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::units::SU_PER_GU;

/// The holding of `o`'s type ([`HoldsDef`]): `None` for anything but a building that holds units.
pub fn holds(o: &Object) -> Option<&'static HoldsDef> {
    o.def().structure.as_ref()?.holds.as_ref()
}

/// Whether objects of `kind` train the units they hold (slice M5-F2, `sim/producers/pool.rs`): the type holds units
/// and produces, the Armory first (plan decisions T70 and T71: the Cathedral, the Chop Shop and the Airstrip as data).
/// Design change 0337 calls them "pool buildings" (`factions/cults/structures/README.md > Choosing Recruits, Release
/// and Eject All`). (M5-N3) Not a type that researches on its queue, the Temple, whose units staff it ([`staffed`]).
pub fn trains(kind: ObjectKind) -> bool {
    (kind.def().structure.as_ref()).is_some_and(|s| s.holds.is_some() && s.production.is_some() && s.research.is_none())
}

/// (M5-N3) Whether objects of `kind` are staffed by the units inside them: tech buildings
/// (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "**Tech buildings** (Temple,
/// Gunworks, Machine Shop, Shrine)"), the types that hold units and research (`StructureDef::research`) or (M5-N5) cast
/// (`StructureDef::abilities`). The Temple and the Shrine so far; the other two hold no one yet. A research building
/// researches only while Recruits are inside, as fast as their number (`core/tech_and_research.md > Research >
/// Faction Exceptions > Cults — research is staffed by Recruits`; `Sim::staff`), the Shrine's energy regenerates by
/// them (`crate::energy`), and the **A/S/D/F** of either let one unit of a kind out at once
/// (`HoldCommand::ReleaseOne`; "Research and abilities keep the top row"), where a pool building ([`trains`]) releases
/// by its queue.
pub fn staffed(kind: ObjectKind) -> bool {
    (kind.def().structure.as_ref())
        .is_some_and(|s| s.holds.is_some() && (s.research.is_some() || s.abilities.is_some()))
}

/// Whether `building` takes a unit of `kind` in (`systems/controls/commands.md > Enter`: "an own structure that can
/// hold units"): it is a building that holds units, placed, with an owner, in operation (`Object::is_operational`;
/// design change 0347 point 4, `factions/cults/structures/README.md > Entering pool buildings`: "a building under
/// construction **can't be entered**"), and its type accepts the kind (`HoldsDef::accepts`). Whose units may go in,
/// and whether its pool has room, are the sim's (`Sim::step_entries`).
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

/// The entry path of `building`, a placed building that holds units: the doorway at the middle of its entrance side
/// (`HoldsDef::entrance`), facing out of it (design change 0347 point 1, `core/entities.md > Entrance and exit
/// (production and research buildings)`: "The entrance is a dedicated doorway at the middle of the entrance side"),
/// walked to `Holding::entry_standoff_su` clear of the wall. `None` for anything else.
pub fn entry_path(building: &Object) -> Option<EntryPath> {
    let holds = holds(building)?;
    let placement = building.placement.as_ref()?;
    let outward = outward(building.kind, placement, holds.entrance)?;
    Some(EntryPath {
        outward,
        spot: side_middle(placement, outward),
        standoff: rules().tuning.holding.entry_standoff_su,
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
