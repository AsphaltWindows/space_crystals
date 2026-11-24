//! Unit Control — `factions/gdo/overview.md > GDOResources > Unit Control`: "Hard cap of 200, always fully
//! available with no buildings required to unlock it. Each GDO unit has a Unit Control cost."
//!
//! Derived from the objects, never stored (plan decision T7): a unit takes its UnitControlCost for as long as it is in
//! the game, so a unit that dies or is removed gives it back at once. Units of the other factions have no
//! UnitControlCost (`UnitDef::unit_control_cost`) and take none.

use crate::ids::PlayerId;
use crate::object::Object;

/// `> Unit Control`: "Hard cap of 200". The player can't build units that would take the used Unit Control past it.
pub const UNIT_CONTROL_CAP: u32 = 200;

/// The Unit Control `player` uses: what the objects among `objects` that the player owns take up.
pub(crate) fn used<'a>(objects: impl IntoIterator<Item = &'a Object>, player: PlayerId) -> u32 {
    objects
        .into_iter()
        .filter(|o| o.owner == Some(player))
        .map(taken_by)
        .fold(0, u32::saturating_add)
}

/// The Unit Control `o` takes up for its owner: a unit its UnitControlCost, a producer what it holds reserved.
fn taken_by(o: &Object) -> u32 {
    let own = o
        .def()
        .unit
        .as_ref()
        .and_then(|unit| unit.unit_control_cost)
        .unwrap_or(0);
    own.saturating_add(reserved(o))
}

/// **Hook for slice M2-D2** (design change 0082, `> Unit Control > Reserved when queued`): "a unit takes its Unit
/// Control the moment it is queued", so the units in a producer's queue, the one in production and a finished one
/// waiting at the doorway included, are counted here, at their kind's UnitControlCost, until they leave the queue
/// (produced, cancelled, or refunded when the producer is destroyed). No object has a production queue yet.
fn reserved(_producer: &Object) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;

    #[test]
    fn the_cap_and_the_costs_follow_the_design() {
        assert_eq!(UNIT_CONTROL_CAP, 200);
        // `factions/gdo/units.md > Peacekeeper > UnitControlCost - 1`; the Syndicate's Guard has none.
        let cost = |kind: ObjectKind| kind.def().unit.as_ref().and_then(|u| u.unit_control_cost);
        assert_eq!(
            (cost(ObjectKind::Peacekeeper), cost(ObjectKind::Guard)),
            (Some(1), None)
        );
    }
}
