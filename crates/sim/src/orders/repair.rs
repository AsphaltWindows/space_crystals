//! A Repair (slice M5-N8, plan decision T83, `OrderKind::Repair`; `crate::repair`): a repairer restores a damaged own
//! building or vehicle — `factions/cults/units/recruit.md > Repair`, `factions/syndicate/units/agent.md > Repair`
//! ("How an Agent reaches a pop-up turret or a Tunnel (side, reach) follows the Recruit/Field Engineer repair flow;
//! details: Director's discretion").
//!
//! The behaviour ([`RepairWork`]) takes the repairer to its target; the steps around phase 5a and the repairs' step do
//! the rest, as for an Assist (`orders::assist`):
//! - **At a building** it holds its own spot beside the footprint ([`RepairWork::Spot`]), which the spots' step gives it
//!   and walks it to (`crate::spot_work`, `Sim::step_spots`): the first free spot of design change 0339's search round
//!   the footprint, on any side, a Tunnel's and a lowered Laser Turret's alike, within reach of the edge
//!   (`tuning::construction::BUILD_REACH_SU`). A lowered turret is repaired as it is: nothing of a repair raises it.
//!   The spots' step ends an order whose building no longer suits it (`spot_work::suits`: not its owner's any more, at
//!   full HP).
//! - **At a vehicle** it closes in and follows it, as a Move to a unit does (`orders::travel::approach`, Follow: it
//!   stops within proximity, `tuning::orders::PROXIMITY_SU`, and sets off again once the vehicle leaves it), so that a
//!   vehicle that drives off takes its repairers along ([`RepairWork::Follow`]). **Placeholder (not in the design: how
//!   a repairer keeps up with a vehicle)**: T83's "as a melee attacker does" read as the Follow's proximity, which is
//!   the reach a repairer works within; a Follow whose move fails ends the order, as a Move's.
//! - **The repair** is the repairs' step's (`Sim::step_repairs`), in the economy phase: each repairer at work, within
//!   reach of its target's edge with its walk over, restores its share of the HP.
//! - **The end** (design change 0329, `factions/cults/units/recruit.md > Repair`: "repairers **stop once the target is
//!   at full HP**; if the player **runs out of Space Crystals** mid-repair, the order **ends**"): the repairs' step
//!   ends the order at full HP, when a crystal it would pay can't be paid, or when the target is no longer one the
//!   unit repairs (`crate::repair::can_repair`); the removal path when the target leaves the game. Its queued orders go
//!   on from the next tick. Never complete by itself.

use super::state::{Approach, RepairWork, SpotWork};
use super::travel;
use super::world::{Unit, World};
use crate::ids::ObjectId;
use crate::tuning::construction::BUILD_REACH_SU;
use crate::tuning::orders::PROXIMITY_SU;

// A repairer that a vehicle's Follow stops within proximity of stands within reach of it, and is at work.
const _: () = assert!(PROXIMITY_SU <= BUILD_REACH_SU);

/// A Repair of `target` starts (`orders::start`): at a unit, a vehicle, its Follow from a standstill; at anything else,
/// a building, the repairer stands until the spots' step sends it to its spot (module docs).
pub(super) fn begin(target: ObjectId, world: &World) -> RepairWork {
    match world.get(target) {
        Some(view) if view.is_unit => RepairWork::Follow(Approach::default()),
        _ => RepairWork::Spot(SpotWork::default()),
    }
}

/// One tick of a Repair of `target` (module docs): a vehicle's repairer follows it; a building's has nothing to do. It
/// completes only when a vehicle's Follow does, its move failed.
pub(super) fn run(unit: &mut Unit, target: ObjectId, state: &mut RepairWork, world: &World) -> bool {
    match state {
        RepairWork::Spot(_) => false,
        RepairWork::Follow(approach) => travel::approach(unit, target, approach, world),
    }
}
