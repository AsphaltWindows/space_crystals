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
//! - **The claims** ([`Claims`], `factions/cults/economy.md > RecruitmentHall > TileClaiming`, whose Claiming details
//!   design change 0341 wrote): each Recruitable tile of an area goes to the first of the Halls whose area has it, in
//!   the order they were completed ("**Priority** goes to the Hall **completed first**"); of Halls completed in the
//!   same tick, the one whose centre is nearer the tile, and at exactly equal distance the lower object id ("each
//!   contested tile goes to the Hall whose centre is nearer; at exactly equal distance, to the Hall with the lower
//!   object id"). Only a completed Hall claims: "A Hall under construction claims nothing, provides no capacity and
//!   produces nothing until it is complete" (`Object::is_operational`). Every Hall competes, whoever owns it ("Halls of
//!   different players compete for tiles the same way as a player's own Halls"). The terrain alone says which tiles
//!   are Recruitable (`TilePreset::recruitable`):
//!   "Buildings do not affect recruitability". Being derived, the tiles a destroyed Hall claimed go to the next Halls
//!   in the order in the tick it is gone ("if multiple halls contest a newly freed tile, the one that was built first
//!   claims it"), and their capacity rises with them.
//! - **Effectiveness and capacity** ([`Capacity`], `factions/cults/economy.md > RecruitmentHall > Effectiveness`,
//!   `factions/cults/economy.md > RecruitmentHall > BaseUnitControlCapacity - 20`): the claimed tiles over the area's,
//!   and BaseUnitControlCapacity times that, rounded down everywhere (design change 0341 point 2; the Claiming details:
//!   "LocalUnitControlCapacity is **rounded down** everywhere (production and HUD): 33 claimed tiles of 100 gives 6.6,
//!   so 6"): a Hall produces while its usage is below the rounded figure.
//! - **Usage** ([`Lineages`], `factions/cults/economy.md > RecruitmentHall > Unit Control Tracking`, the Director's
//!   amendment to T56): each entry of a unit's lineage is a point of Unit Control at the Hall it names, a Hall's
//!   LocalUnitControlUsage being the entries that name it over every unit, those held off the map included, and its
//!   Recruit in production (design change 0342: "the Recruit in production counts toward LocalUnitControlUsage from
//!   the start of its production"). An entry whose Hall is gone is an orphaned point (`factions/cults/economy.md >
//!   RecruitmentHall > On Destruction`), and so is every point of the cost of a Cults unit without a lineage, a map's
//!   or a debug spawn's (design change 0344: "Recruits no Hall produced [...] are orphans from the start"). A
//!   player's used Unit Control is their Halls' usage plus their orphaned points (`factions/cults/economy.md >
//!   RecruitmentHall > Cults Unit Control HUD Display`, `economy::army`).

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
    /// The claims of the Halls among `objects` on `map`: each Recruitable tile of a completed Hall's area goes to the
    /// Hall of the least (completion tick, distance from its centre, id) whose area has it (module docs, design change
    /// 0341).
    pub fn of<'a>(objects: impl IntoIterator<Item = &'a Object>, map: &Map) -> Claims {
        let mut claims = BTreeMap::new();
        let mut first: BTreeMap<TilePos, (Tick, u64, ObjectId)> = BTreeMap::new();
        for o in objects.into_iter().filter(|o| o.is_operational()) {
            let (Some(production), Some(placement), Some(def)) = (&o.production, &o.placement, def_of(o.kind)) else {
                continue;
            };
            claims.insert(o.id, 0u32);
            for tile in area(placement, def) {
                if map.tile_preset(tile).is_some_and(|preset| preset.recruitable) {
                    let key = (production.completed_at, distance_squared(placement, tile), o.id);
                    first.entry(tile).and_modify(|k| *k = (*k).min(key)).or_insert(key);
                }
            }
        }
        for (_, _, hall) in first.into_values() {
            *claims.entry(hall).or_default() += 1;
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

/// The squared distance from the centre of the footprint `placement` to the centre of `tile`, in half tiles, so exact:
/// the rule that splits the tiles of Halls completed in the same tick (design change 0341: "each contested tile goes
/// to the Hall whose centre is nearer").
fn distance_squared(placement: &Placement, tile: TilePos) -> u64 {
    let (w, h) = (i64::from(placement.w), i64::from(placement.h));
    let dx = 2 * i64::from(tile.x) + 1 - (2 * i64::from(placement.origin.x) + w);
    let dy = 2 * i64::from(tile.y) + 1 - (2 * i64::from(placement.origin.y) + h);
    (dx * dx + dy * dy).unsigned_abs()
}

/// A Hall's LocalUnitControlCapacity, as an exact fraction of Unit Control (module docs;
/// `factions/cults/economy.md > RecruitmentHall > RecruitmentHallInstanceState`: "LocalUnitControlCapacity: number
/// (BaseUnitControlCapacity * Effectiveness)"): BaseUnitControlCapacity × the claimed tiles, over the area's tiles.
/// It counts rounded down everywhere ([`Capacity::floor`]; design change 0341 point 2: "LocalUnitControlCapacity is
/// rounded down everywhere (production and HUD)"): a Hall produces while its usage is below that
/// ([`Capacity::has_room`]), and a player's Halls provide the sum of theirs (`economy::army`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Capacity {
    /// The fraction's numerator and its denominator, in lowest terms: the denominator is never 0.
    num: u64,
    den: u64,
}

impl Capacity {
    /// The capacity of a Hall of `def` that claims `claimed` tiles (`factions/cults/economy.md > RecruitmentHall >
    /// BaseUnitControlCapacity - 20`: "At reduced effectiveness, capacity and production time scale linearly.
    /// Example: at 50% effectiveness, capacity = 10").
    pub fn of(def: &RecruitmentDef, claimed: u32) -> Capacity {
        Capacity::reduced(
            u64::from(def.capacity) * u64::from(claimed),
            u64::from(def.tiles()).max(1),
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

    /// Whether a usage of `usage` is below it, rounded down: the Hall produces "until LocalUnitControlUsage reaches
    /// LocalUnitControlCapacity" (`factions/cults/economy.md > RecruitmentHall > Production Behavior`), and a Hall
    /// with 33 claimed tiles, 6.6, stops at 6 (design change 0341 point 2). Plan decision T57's start rule.
    pub fn has_room(self, usage: u32) -> bool {
        usage < self.floor()
    }

    /// Rounded down, as it counts everywhere (design change 0341 point 2: "6.6 becomes 6, in production and on the HUD").
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
    /// LocalUnitControlUsage: the entries of the units' lineages that name it, and its Recruit in production
    /// ([`Lineages`]).
    pub usage: u32,
    /// LocalUnitControlCapacity, exactly; it counts rounded down ([`Capacity::floor`]).
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
/// Tracking`, the Director's amendment to T56): each Hall's usage, its Recruit in production included (design change
/// 0342), and each player's orphaned points. Derived ([`Lineages::of`]), never stored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lineages {
    /// LocalUnitControlUsage, by the Hall's id: the entries that name it, over every unit, and its Recruit in
    /// production.
    usage: BTreeMap<ObjectId, u32>,
    /// By the owner of the units: their entries whose Hall is gone, and the points of their Cults units' costs that no
    /// entry names: the whole cost of a unit without a lineage.
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
        let mut usage: BTreeMap<ObjectId, u32> = BTreeMap::new();
        for o in objects {
            if def_of(o.kind).is_some() {
                halls.insert(o.id);
                // Design change 0342: "the Recruit in production counts toward LocalUnitControlUsage from the start of
                // its production": a Hall's queue is its one Recruit, which starts as it is queued.
                let producing = (o.production.iter().flat_map(|p| &p.queue))
                    .filter_map(|entry| entry.unit())
                    .map(|kind| ArmyResource::CultsUnitControl.cost_of(kind))
                    .fold(0u32, u32::saturating_add);
                if producing > 0 {
                    *usage.entry(o.id).or_default() += producing;
                }
            }
            // Design change 0344: "Recruits no Hall produced [...] are orphans from the start": the whole cost of a unit
            // without a lineage, and of a unit trained of several Recruits (slice M5-F2, `Sim::train`: one entry for
            // each Hall's Recruit) the point of each that no Hall produced.
            let cost = ArmyResource::CultsUnitControl.cost_of(o.kind);
            let unnamed = cost.saturating_sub(u32::try_from(o.lineage.len()).unwrap_or(u32::MAX));
            if let Some(owner) = o.owner.filter(|_| unnamed > 0) {
                *orphaned.entry(owner).or_default() += unnamed;
            }
            entries.extend(o.lineage.iter().map(|&hall| (o.owner, hall)));
        }
        for (owner, hall) in entries {
            if halls.contains(&hall) {
                *usage.entry(hall).or_default() += 1;
            } else if let Some(owner) = owner {
                *orphaned.entry(owner).or_default() += 1;
            }
        }
        Lineages { usage, orphaned }
    }

    /// `hall`'s LocalUnitControlUsage: the entries that name it, and its Recruit in production.
    pub fn usage(&self, hall: ObjectId) -> u32 {
        self.usage.get(&hall).copied().unwrap_or(0)
    }

    /// `player`'s orphaned points: the entries of their units whose Hall is gone, and the points of their Cults units'
    /// costs that no entry names.
    pub fn orphaned(&self, player: PlayerId) -> u32 {
        self.orphaned.get(&player).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
