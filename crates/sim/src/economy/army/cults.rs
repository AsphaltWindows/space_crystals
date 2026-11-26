//! The Cults' Unit Control (slice M5-C, plan decision T56 as the Director amended it; `factions/cults/overview.md >
//! CultsResources > Unit Control`): what the HUD shows, `factions/cults/economy.md > RecruitmentHall > Cults Unit
//! Control HUD Display`'s "(sum of all LocalUnitControlUsage across all active Recruitment Halls and orphaned units) /
//! (sum of all LocalUnitControlCapacity across all active Recruitment Halls)", from `crate::recruitment`'s figures.
//!
//! - **Available**: the sum of the player's completed Halls' capacities, each rounded down (design change 0341 point
//!   2: "LocalUnitControlCapacity is rounded down everywhere (production and HUD)"): two Halls of 33 tiles each provide
//!   12 (6 + 6). A Hall under construction claims nothing, so it provides nothing (0341 point 4), and a destroyed one's
//!   capacity leaves the sum in the tick it is gone (`factions/cults/economy.md > RecruitmentHall > On Destruction`:
//!   "LocalUnitControlCapacity is removed from the global sum").
//! - **Used**: the usage of the player's Halls, the entries of every unit's lineage that name them and their Recruits
//!   in production (design change 0342: "the Recruit in production counts toward LocalUnitControlUsage from the start
//!   of its production"; its Verification, "At 19/20 with a Recruit in production, the HUD shows 20/20"), plus the
//!   player's orphaned points: the entries of their units whose Hall is gone (`factions/cults/economy.md >
//!   RecruitmentHall > On Destruction`: "Surviving units produced by this hall remain on the field and continue to
//!   count toward the global usage sum") and the points of their Cults units' costs that no lineage entry stands for,
//!   the whole cost of one without a lineage (design change 0344: "Recruits no Hall produced [...] are orphans from
//!   the start"; `Lineages::of`). The units held off the map count as those on it.
//!
//! It is for display only: no production waits for it (design change 0344: "Global total is for display only";
//! `ArmyResource::reserves`).

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
    let (mut used, mut available) = (lineages.orphaned(player), 0u32);
    let halls = (objects.iter())
        .filter(|o| o.owner == Some(player))
        .filter_map(|o| Some((o.id, recruitment::def_of(o.kind)?)));
    for (hall, def) in halls {
        used = used.saturating_add(lineages.usage(hall));
        available = available.saturating_add(Capacity::of(def, claims.tiles(hall)).floor());
    }
    ArmyCap {
        resource: ArmyResource::CultsUnitControl,
        used,
        available,
    }
}
