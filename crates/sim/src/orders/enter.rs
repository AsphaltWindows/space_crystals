//! A unit's walk into a Tunnel — `systems/controls/commands.md > Enter` ("The unit walks to Side A and enters the
//! Tunnel Network"), `factions/syndicate/tunnels.md > Tunnel Network > Walking in` (design changes 0246 and 0247),
//! slice M4-C (`crate::network`).
//!
//! "Units enter through the entry path by walking into it at their normal speed, one behind another with normal
//! spacing, like walking through any one-unit-wide passage; there is no fixed hold time per unit." The walk is a plain
//! move to the entry path's spot (`network::Paths::entry_stand`), by the movement every unit has: the units of a
//! line follow each other to the one spot, and nothing here orders them, looks for another spot or holds a unit
//! (`systems/controls/behaviors.md > Core principle: predictable, not optimal`, design change 0223). "No special
//! waiting rule: a unit heading for the entry path moves like any unit [...]; if the way is genuinely blocked, the
//! normal Failure rule applies".
//!
//! The behaviour only walks, and never completes the order itself: the step right after phase 5a does
//! (`Sim::step_entries`), by taking the unit off the map once it has walked in, or by ending the order of a unit
//! whose walk ended short of the path or whose Tunnel no longer suits it, with the events and the removal a
//! behaviour may not make.

use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};

use super::state::Enter;
use super::world::{Unit, World};

/// An Enter order on `tunnel` starts (`orders::start`): `unit` heads for the entry path's spot, where its box stands
/// against the Tunnel's wall. With no such Tunnel it stands, and its walk is over at once.
pub(super) fn begin(tunnel: ObjectId, unit: &mut Unit, world: &World) -> Enter {
    if let (Some(paths), Some(half)) = (world.tunnel_paths(tunnel), unit.kind.def().half_extents()) {
        unit.mover
            .go_to(MoveGoal::Point(paths.entry_stand(half)), None, world.tick);
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
