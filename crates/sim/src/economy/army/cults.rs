//! The Cults' Unit Control (slice M5-C, plan decision T56 as the Director amended it; `factions/cults/overview.md >
//! CultsResources > Unit Control`): what the HUD shows, `factions/cults/economy.md > RecruitmentHall > Cults Unit
//! Control HUD Display`'s "(sum of all LocalUnitControlUsage across all active Recruitment Halls and orphaned units) /
//! (sum of all LocalUnitControlCapacity across all active Recruitment Halls)", from `crate::recruitment`'s figures.
//!
//! - **Available**: the sum of the player's completed Halls' capacities, exact, then rounded down (**Placeholder
//!   (Q0099 point 2)**): two Halls of 33 tiles each provide 13 (6.6 + 6.6), not 12. A Hall under construction claims
//!   nothing, so it provides nothing (**Placeholder (Q0099 point 4)**), and a destroyed one's capacity leaves the sum
//!   in the tick it is gone (`factions/cults/economy.md > RecruitmentHall > On Destruction`: "LocalUnitControlCapacity
//!   is removed from the global sum").
//! - **Used**: the usage of the player's Halls, the entries of every unit's lineage that name them, plus the player's
//!   orphaned points: the entries of their units whose Hall is gone (`factions/cults/economy.md > RecruitmentHall >
//!   On Destruction`: "Surviving units produced by this hall remain on the field and continue to count toward the
//!   global usage sum") and the whole cost of their Cults units without a lineage (**Placeholder (Q0101 point 2)**).
//!   The units held off the map count as those on it. A Recruit in production counts once it exists (**Placeholder
//!   (Q0100 point 3)**): the HUD shows 19/20 while a Hall makes its 20th.
//!
//! It is for display only: no production waits for it (**Placeholder (Q0101 point 1)**, `ArmyResource::reserves`).

use super::{ArmyCap, ArmyResource};
use crate::ids::PlayerId;
use crate::map::Map;
use crate::object::Object;
use crate::recruitment::{self, Capacity, Claims, Lineages};

/// `player`'s Unit Control among `objects`, which are every object of the game, the units held off the map included,
/// on `map` (module docs).
pub(super) fn cap<'a>(objects: impl IntoIterator<Item = &'a Object>, map: &Map, player: PlayerId) -> ArmyCap {
    let objects: Vec<&Object> = objects.into_iter().collect();
    let claims = Claims::of(objects.iter().copied(), map);
    let lineages = Lineages::of(objects.iter().copied());
    let (mut used, mut available) = (lineages.orphaned(player), Capacity::ZERO);
    let halls = (objects.iter())
        .filter(|o| o.owner == Some(player))
        .filter_map(|o| Some((o.id, recruitment::def_of(o.kind)?)));
    for (hall, def) in halls {
        used = used.saturating_add(lineages.usage(hall));
        available = available.plus(Capacity::of(def, claims.tiles(hall)));
    }
    ArmyCap {
        resource: ArmyResource::CultsUnitControl,
        used,
        available: available.floor(),
    }
}
