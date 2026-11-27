//! A unit's walk into a Tunnel or a building that holds units — `systems/controls/commands.md > Enter` ("The unit walks
//! to the structure's entrance side and goes inside"; its TargetObject "the structure being entered", design change
//! 0319), `factions/syndicate/tunnels.md > Tunnel Network > Walking in` (design changes 0246 and 0247), slice M4-C
//! (`crate::network`), and the same walk into an Armory's entrance doorway (slice M5-F1, `crate::holding`; design
//! change 0347 point 1, `core/entities.md > Entrance and exit (production and research buildings)`: "The entrance is a
//! dedicated doorway at the middle of the entrance side (like the Exit Doorway): units walk to it and go in there").
//!
//! "Units enter through the entry path by walking into it at their normal speed, one behind another with normal
//! spacing, like walking through any one-unit-wide passage; there is no fixed hold time per unit." The walk is a plain
//! move to the entry path's spot (`network::EntryPath::stand`), by the movement every unit has: the units of a
//! line follow each other to the one spot, and nothing here orders them, looks for another spot or holds a unit
//! (`systems/controls/behaviors.md > Core principle: predictable, not optimal`, design change 0223). "No special
//! waiting rule: a unit heading for the entry path moves like any unit [...]; if the way is genuinely blocked, the
//! normal Failure rule applies".
//!
//! The behaviour only walks, and never completes the order itself: the step right after phase 5a does
//! (`Sim::step_entries`), by taking the unit off the map once it has walked in, or by ending the order of a unit
//! whose walk ended short of the path or whose target no longer suits it, with the events and the removal a
//! behaviour may not make.

use glam::Vec2;

use crate::ids::ObjectId;
use crate::math;
use crate::mover::{MoveGoal, MoveStatus};
use crate::shape::Shape;

use super::state::Enter;
use super::world::{Unit, World};

/// An Enter order on `target` starts (`orders::start`): `unit` heads for the entry path's spot, where its box stands
/// against the wall. With no such Tunnel or building it stands, and its walk is over at once. (M6-SV) Its box is the
/// one it has driving in, facing the wall: a vehicle's Silhouette turned with it (slice M5-V1, `Shape::silhouette`),
/// so that a Bike's walk ends 28 su from a north- or south-facing side, its length against it, where it can stand; a
/// square Silhouette's is its own.
pub(super) fn begin(target: ObjectId, unit: &mut Unit, world: &World) -> Enter {
    if let Some(path) = world.entry_path(target)
        && unit.kind.def().silhouette().is_some()
    {
        let facing = math::vector_angle(-path.outward);
        let half = Shape::silhouette(unit.kind, Vec2::ZERO, facing).bound_half();
        unit.mover.go_to(MoveGoal::Point(path.stand(half)), None, world.tick);
    }
    Enter::default()
}

/// One tick of the walk. It is over once the move ended, whatever its outcome (`MoveStatus::Arrived`, `Failed`: the
/// normal Failure rule, design change 0247). Never complete (module docs).
pub(super) fn run(state: &mut Enter, unit: &Unit) -> bool {
    if unit.mover.status != MoveStatus::Moving {
        state.walked = true;
    }
    false
}
