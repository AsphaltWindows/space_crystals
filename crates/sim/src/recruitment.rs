//! The Recruitment Halls (`factions/cults/economy.md > RecruitmentHall`; plan decisions T56 and T57, slice M5-C): the
//! tiles each Hall claims, its effectiveness and its capacity, and the Unit Control the units of its lineage use.
//! Everything here is derived from the objects and the map each time it is read, never stored (plan decision T7, as
//! power and the other caps): the claims from the completed Halls and the terrain, the usage from the units'
//! lineages (`Object::lineage`). A Hall stores only the tick it was completed (`Production::completed_at`); its
//! production is `crate::production`'s, in an automatic mode (`sim/recruitment.rs`).
//!
//! - **The Recruitment Area** ([`area`], `factions/cults/economy.md > RecruitmentHall > RecruitmentArea`: "A 10x10 grid
//!   unit area centered on the 4x4 building"): the square of `RecruitmentDef::area` tiles a side centred on the
//!   footprint. Its tiles off the map count among its tiles, and can't be claimed (**Placeholder (Q0099 point 1)**).
//! - **The claims** ([`Claims`], `factions/cults/economy.md > RecruitmentHall > TileClaiming`): each Recruitable tile
//!   of an area goes to the first of the Halls whose area has it, in the order they were completed, ties by id
//!   (**Placeholder (Q0099 point 3)**: "constructed first" is completed first, and two Halls completed in the same tick
//!   go by the order they were placed). Only a completed Hall claims: one under construction claims nothing and
//!   produces nothing (**Placeholder (Q0099 point 4)**; `Object::is_operational`). Every Hall competes, whoever owns it
//!   (**Placeholder (Q0099 point 5)**: "another active Recruitment Hall", by the letter). The terrain alone says which
//!   tiles are Recruitable (`TilePreset::recruitable`): "Buildings do not affect recruitability". Being derived, the
//!   tiles a destroyed Hall claimed go to the next Halls in the order in the tick it is gone ("if multiple halls
//!   contest a newly freed tile, the one that was built first claims it"), and their capacity rises with them.
//! - **Effectiveness and capacity** ([`Capacity`], `factions/cults/economy.md > RecruitmentHall > Effectiveness`,
//!   `factions/cults/economy.md > RecruitmentHall > BaseUnitControlCapacity - 20`): the claimed tiles over the area's,
//!   and BaseUnitControlCapacity times that, kept as an exact fraction. A Hall produces while its usage is below it,
//!   compared exactly, and the interface shows it rounded down: 7 Recruits at 33 tiles' 6.6, shown as 6 (**Placeholder
//!   (Q0099 point 2)**).
//! - **Usage** ([`Lineages`], `factions/cults/economy.md > RecruitmentHall > Unit Control Tracking`, the Director's
//!   amendment to T56): each entry of a unit's lineage is a point of Unit Control at the Hall it names, a Hall's
//!   LocalUnitControlUsage being the entries that name it over every unit, those held off the map included. An entry
//!   whose Hall is gone is an orphaned point (`factions/cults/economy.md > RecruitmentHall > On Destruction`), and so
//!   is every point of the cost of a Cults unit without a lineage, a map's or a debug spawn's (**Placeholder (Q0101
//!   point 2)**). A player's used Unit Control is their Halls' usage plus their orphaned points
//!   (`factions/cults/economy.md > RecruitmentHall > Cults Unit Control HUD Display`, `economy::army`).

use std::collections::{BTreeMap, BTreeSet};

use crate::defs::{ObjectKind, RecruitmentDef};
use crate::economy::ArmyResource;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::{Object, Placement};
use crate::units::{Tick, TilePos};

/// How a `kind` Hall recruits (its `ProductionDef::recruitment`); `None` for every kind but the Recruitment Hall's.
pub fn def_of(kind: ObjectKind) -> Option<&'static RecruitmentDef> {
    kind.def().structure.as_ref()?.production.as_ref()?.recruitment.as_ref()
}

/// The tiles of the Recruitment Area of a Hall of `def` at `placement`, row by row, those off the map included (module
/// docs): the square of `def.area` tiles a side centred on the footprint. Where the two can't be centred exactly, the
/// odd tile is on the far side (east or south); the Hall's 10 and 4 centre exactly, 3 tiles on every side.
pub fn area(placement: &Placement, def: &RecruitmentDef) -> impl Iterator<Item = TilePos> + use<> {
    let side = i32::try_from(def.area).unwrap_or(i32::MAX);
    let start = |origin: i32, size: u32| origin - (side - i32::try_from(size).unwrap_or(side)).div_euclid(2);
    let (x0, y0) = (
        start(placement.origin.x, placement.w),
        start(placement.origin.y, placement.h),
    );
    (0..side).flat_map(move |dy| (0..side).map(move |dx| TilePos::new(x0 + dx, y0 + dy)))
}

/// The tiles each completed Hall claims (module docs, `factions/cults/economy.md > RecruitmentHall > TileClaiming`):
/// how many, by the Hall's id. Derived ([`Claims::of`]), never stored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Claims(BTreeMap<ObjectId, u32>);

impl Claims {
    /// The claims of the Halls among `objects` on `map`: the completed Halls, first completed first and ties by id
    /// (**Placeholder (Q0099 point 3)**), each taking the Recruitable tiles of its area that no Hall before it took.
    pub fn of<'a>(objects: impl IntoIterator<Item = &'a Object>, map: &Map) -> Claims {
        let mut halls: Vec<(Tick, ObjectId, &Placement, &RecruitmentDef)> = objects
            .into_iter()
            .filter(|o| o.is_operational())
            .filter_map(|o| {
                let completed = o.production.as_ref()?.completed_at;
                Some((completed, o.id, o.placement.as_ref()?, def_of(o.kind)?))
            })
            .collect();
        halls.sort_unstable_by_key(|&(completed, id, ..)| (completed, id));
        let mut taken = BTreeSet::new();
        let mut claims = BTreeMap::new();
        for (_, id, placement, def) in halls {
            let mut claimed = 0u32;
            for tile in area(placement, def) {
                let recruitable = map.tile_preset(tile).is_some_and(|preset| preset.recruitable);
                if recruitable && taken.insert(tile) {
                    claimed += 1;
                }
            }
            claims.insert(id, claimed);
        }
        Claims(claims)
    }

    /// The Recruitable tiles `hall` claims: 0 for an object that isn't a completed Hall.
    pub fn tiles(&self, hall: ObjectId) -> u32 {
        self.0.get(&hall).copied().unwrap_or(0)
    }

    /// The completed Halls and the tiles each claims, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (ObjectId, u32)> + '_ {
        self.0.iter().map(|(&id, &tiles)| (id, tiles))
    }
}

/// A Hall's LocalUnitControlCapacity, or a sum of them, as an exact fraction of Unit Control (module docs;
/// `factions/cults/economy.md > RecruitmentHall > RecruitmentHallInstanceState`: "LocalUnitControlCapacity: number
/// (BaseUnitControlCapacity * Effectiveness)"): BaseUnitControlCapacity × the claimed tiles, over the area's tiles. A
/// Hall produces while its usage is below it ([`Capacity::has_room`]), and it is shown rounded down
/// ([`Capacity::floor`]; **Placeholder (Q0099 point 2)**).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Capacity {
    /// The fraction's numerator and its denominator, in lowest terms: the denominator is never 0, and divides an
    /// area's tiles (every Hall's 100), so that a sum's stays as small.
    num: u64,
    den: u64,
}

impl Capacity {
    /// No capacity: no Hall's, or the sum of none.
    pub const ZERO: Capacity = Capacity { num: 0, den: 1 };

    /// The capacity of a Hall of `def` that claims `claimed` tiles (`factions/cults/economy.md > RecruitmentHall >
    /// BaseUnitControlCapacity - 20`: "At reduced effectiveness, capacity and production time scale linearly.
    /// Example: at 50% effectiveness, capacity = 10").
    pub fn of(def: &RecruitmentDef, claimed: u32) -> Capacity {
        Capacity::reduced(
            u64::from(def.capacity) * u64::from(claimed),
            u64::from(def.tiles()).max(1),
        )
    }

    /// The sum of the two, exactly: what a player's Halls provide together (`economy::army`).
    pub fn plus(self, other: Capacity) -> Capacity {
        let den = self.den / gcd(self.den, other.den) * other.den;
        Capacity::reduced(
            (self.num * (den / self.den)).saturating_add(other.num * (den / other.den)),
            den,
        )
    }

    /// `num / den` in lowest terms, `den` not 0.
    fn reduced(num: u64, den: u64) -> Capacity {
        let common = gcd(num, den);
        Capacity {
            num: num / common,
            den: den / common,
        }
    }

    /// Whether a usage of `usage` is below it: the Hall produces "until LocalUnitControlUsage reaches
    /// LocalUnitControlCapacity" (`factions/cults/economy.md > RecruitmentHall > Production Behavior`), compared
    /// exactly (**Placeholder (Q0099 point 2)**: 7 Recruits at 6.6). Plan decision T57's start rule, `100 × usage <
    /// 20 × claimed` for the Hall.
    pub fn has_room(self, usage: u32) -> bool {
        u64::from(usage) * self.den < self.num
    }

    /// Rounded down, as the interface shows it (**Placeholder (Q0099 point 2)**: 6.6 is shown as 6).
    pub fn floor(self) -> u32 {
        u32::try_from(self.num / self.den).unwrap_or(u32::MAX)
    }
}

/// The greatest common divisor of two numbers, `b` not 0.
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// What a Hall's state shows (`factions/cults/economy.md > RecruitmentHall > RecruitmentHallInstanceState`;
/// `Sim::hall_state`), at the end of the latest tick: its Effectiveness as the tiles it claims over its area's, its
/// LocalUnitControlUsage and its LocalUnitControlCapacity. Its ProductionProgress and its RallyPoint are its
/// production's (`Sim::production_status`, `Object::production`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HallState {
    /// The Recruitable tiles it claims.
    pub claimed: u32,
    /// Its area's tiles: "the total number of tiles in Recruitment Area".
    pub tiles: u32,
    /// LocalUnitControlUsage: the entries of the units' lineages that name it ([`Lineages`]).
    pub usage: u32,
    /// LocalUnitControlCapacity, exactly.
    pub capacity: Capacity,
}

impl HallState {
    /// Its Effectiveness in whole percent, rounded down: 100% only with every tile of its area claimed.
    pub fn effectiveness_percent(&self) -> u32 {
        let percent = u64::from(self.claimed) * 100 / u64::from(self.tiles.max(1));
        u32::try_from(percent).unwrap_or(u32::MAX)
    }
}

/// The Unit Control the units' lineages use (module docs; `factions/cults/economy.md > RecruitmentHall > Unit Control
/// Tracking`, the Director's amendment to T56): each Hall's usage, and each player's orphaned points. Derived
/// ([`Lineages::of`]), never stored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lineages {
    /// LocalUnitControlUsage, by the Hall's id: the entries that name it, over every unit.
    usage: BTreeMap<ObjectId, u32>,
    /// By the owner of the units: their entries whose Hall is gone, and the costs of their Cults units without a
    /// lineage.
    orphaned: BTreeMap<PlayerId, u32>,
}

impl Lineages {
    /// The usage of every object among `objects`, which are all the game's: the map's and the units held off it. An
    /// entry names a Hall that is gone if no Hall among `objects` has its id. A unit without an owner (a defeated
    /// player's) counts at the Halls its entries name, and as no one's orphan.
    pub fn of<'a>(objects: impl IntoIterator<Item = &'a Object>) -> Lineages {
        let mut halls = BTreeSet::new();
        let mut entries = Vec::new();
        let mut orphaned: BTreeMap<PlayerId, u32> = BTreeMap::new();
        for o in objects {
            if def_of(o.kind).is_some() {
                halls.insert(o.id);
            }
            if o.lineage.is_empty() {
                // **Placeholder (Q0101 point 2)**: no Hall produced it, so its whole cost is orphaned.
                let cost = ArmyResource::CultsUnitControl.cost_of(o.kind);
                if let Some(owner) = o.owner.filter(|_| cost > 0) {
                    *orphaned.entry(owner).or_default() += cost;
                }
            }
            entries.extend(o.lineage.iter().map(|&hall| (o.owner, hall)));
        }
        let mut usage: BTreeMap<ObjectId, u32> = BTreeMap::new();
        for (owner, hall) in entries {
            if halls.contains(&hall) {
                *usage.entry(hall).or_default() += 1;
            } else if let Some(owner) = owner {
                *orphaned.entry(owner).or_default() += 1;
            }
        }
        Lineages { usage, orphaned }
    }

    /// `hall`'s LocalUnitControlUsage: the entries that name it.
    pub fn usage(&self, hall: ObjectId) -> u32 {
        self.usage.get(&hall).copied().unwrap_or(0)
    }

    /// `player`'s orphaned points: the entries of their units whose Hall is gone, and the costs of their Cults units
    /// without a lineage.
    pub fn orphaned(&self, player: PlayerId) -> u32 {
        self.orphaned.get(&player).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
