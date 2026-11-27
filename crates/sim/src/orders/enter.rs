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
//! (M6-SD) **An air unit** flies to the centre of the Tunnel's air entry door instead (`network::EntryPath::air_door`,
//! plan decision T96: "An air unit's Enter flies it to the entry door"; `factions/syndicate/tunnels.md > Tunnel > Look
//! by tier`: "Aircraft use these doors, not the Side A paths"), by the movement every air unit has. Once its centre is
//! over the door (`network::over_door`; the plan's D46: "it is taken in once it is over the door"), the door lets it
//! descend in its turn (`Sim::step_entries`), and it flies on to the door's centre as it does. Aircraft collide with
//! nothing, so no Failure rule ends its flight: one that comes to rest off the door before its descent (pushed off by
//! the air units' soft separation while it waits its turn, `movement::air`, or given up by the stuck rule while they
//! keep it off) flies back to it. **Placeholder (not in the design: an aircraft waiting its turn at the entry door)**.
//!
//! The behaviour only walks, and never completes the order itself: the step right after phase 5a does
//! (`Sim::step_entries`), by taking the unit off the map once it has walked in, or by ending the order of a unit
//! whose walk ended short of the path or whose target no longer suits it, with the events and the removal a
//! behaviour may not make.

use glam::Vec2;

use crate::ids::ObjectId;
use crate::math;
use crate::mover::{MoveGoal, MoveStatus};
use crate::network;
use crate::shape::Shape;

use super::state::Enter;
use super::world::{Unit, World};

/// An Enter order on `target` starts (`orders::start`): `unit` heads for the entry path's spot, where its box stands
/// against the wall. With no such Tunnel or building it stands, and its walk is over at once. (M6-SV) Its box is the
/// one it has driving in, facing the wall: a vehicle's Silhouette turned with it (slice M5-V1, `Shape::silhouette`),
/// so that a Bike's walk ends 28 su from a north- or south-facing side, its length against it, where it can stand; a
/// square Silhouette's is its own. (M6-SD) An air unit heads for the centre of a Tunnel's air entry door
/// ([`air_door`]).
pub(super) fn begin(target: ObjectId, unit: &mut Unit, world: &World) -> Enter {
    if let Some(door) = air_door(target, unit, world) {
        unit.mover.go_to(MoveGoal::Point(door), None, world.tick);
    } else if let Some(path) = world.entry_path(target)
        && unit.kind.def().silhouette().is_some()
    {
        let facing = math::vector_angle(-path.outward);
        let half = Shape::silhouette(unit.kind, Vec2::ZERO, facing).bound_half();
        unit.mover.go_to(MoveGoal::Point(path.stand(half)), None, world.tick);
    }
    Enter::default()
}

/// One tick of the walk. It is over once the move ended, whatever its outcome (`MoveStatus::Arrived`, `Failed`: the
/// normal Failure rule, design change 0247). (M6-SD) An air unit's flight never is: at rest off the air entry door
/// before its descent, it flies back (module docs). Never complete (module docs).
pub(super) fn run(target: ObjectId, state: &mut Enter, unit: &mut Unit, world: &World) -> bool {
    if unit.mover.status == MoveStatus::Moving {
        return false;
    }
    match air_door(target, unit, world) {
        None => state.walked = true,
        Some(door) => {
            if state.descent.is_none() && !network::over_door(door, unit.pos) {
                unit.mover.go_to(MoveGoal::Point(door), None, world.tick);
            }
        }
    }
    false
}

/// (M6-SD) Where at the air entry door `unit` flies to for an Enter on `target`: an air unit's, at a Tunnel
/// (`network::uses_air_doors`, `network::EntryPath::air_door`), the door's centre, inside the map
/// (`network::door_spot`); `None` for a ground unit, and at a building that holds units.
fn air_door(target: ObjectId, unit: &Unit, world: &World) -> Option<Vec2> {
    let door = world.entry_path(target)?.air_door?;
    let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    network::uses_air_doors(unit.kind).then(|| network::door_spot(world.map, door, half))
}
