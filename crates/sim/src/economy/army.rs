//! Army caps (plan decision T20): what a player's army may take up, by faction. Derived from the objects, never
//! stored (plan decision T7), like power: a unit takes its cost for as long as it is in the game, and a building
//! provides capacity from the tick it is created until the tick it is removed.
//!
//! - **GDO: Unit Control** (`factions/gdo/overview.md > GDOResources > Unit Control`): "Hard cap of 200, always fully
//!   available with no buildings required to unlock it. Each GDO unit has a Unit Control cost."
//! - **Colonists: Beacon Capacity** (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`): "Each
//!   Beacon provides a certain amount of Beacon Capacity. Total Beacon Capacity caps at 200. Each Colonist unit has
//!   a Beacon Capacity cost." Every Colonists building provides 5 (`factions/colonists/economy.md > Warp In > Beacon
//!   Capacity Retention`), so what is available is min(200, 5 × the player's Colonists structures), a Beacon under
//!   deployment left out: its 5 comes with completion (`factions/colonists/economy.md > Beacon Deployment > Beacon
//!   Capacity during deployment`, design change 0112), while the Drone it was costs nothing from the deployment's first
//!   frame on, being a Beacon.
//! - **Syndicate: Tunnel Space** (slice M4-B; `factions/syndicate/overview.md > SyndicateResources > Tunnel Space`,
//!   `factions/syndicate/tunnels.md > Tunnel > Tunnel Space`): "Each Syndicate unit has a Tunnel Space cost", and what
//!   is available is min(200, ⌊the unused tunnel-region cells of the player's Tunnels / 4⌋), derived from the Tunnels
//!   and the expansions where they stand (`tunnel_space.rs`).
//! - **The Cults: Unit Control** (slice M5-C, plan decision T56; `factions/cults/overview.md > CultsResources > Unit
//!   Control`: "Provided by Recruitment Halls in proportion to the number of Recruitable tiles each hall is actively
//!   recruiting from. No hard cap"; `cults.rs`): what is available is the sum of the completed Halls' capacities,
//!   rounded down, and what is used is the Halls' usage plus the orphaned points, by the units' lineages
//!   (`factions/cults/economy.md > RecruitmentHall > Cults Unit Control HUD Display`; `crate::recruitment`). It counts
//!   the units held off the map as the others do.
//!
//! **A unit costs only in its own faction's resource** ([`ArmyResource::cost_of`]; the Director's decision, slice
//! M5-C): another faction's unit, which only a debug or a Custom game can give a player, takes none of it. So a Cults
//! unit of a GDO player's takes none of their Unit Control, nor a GDO unit of a Cults player's any of theirs.
//!
//! **Used** is what the player's units cost, plus their producers' units in production (design change 0143, for the
//! GDO, the Colonists and the Syndicate: `factions/gdo/overview.md > GDOResources > Unit Control`, "Reserved when
//! production starts": the HUD's "used" count "includes in-production units, not merely queued ones";
//! `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts` alike). A unit takes its cost
//! from the tick its producer begins progress on it (`Production::in_production`) for as long as it lives; a unit only
//! queued takes none, nor does one waiting at 0% for room. The Cults' Unit Control reserves nothing at production
//! starts ([`ArmyResource::reserves`]): a Hall counts its own Recruit in production in its LocalUnitControlUsage, from
//! the start of its production (design change 0342, `crate::recruitment::Lineages`), and every other Cults unit counts
//! the Recruits it was made of, which counted as they were.
//!
//! **The check** is made when production starts, not when a unit is queued (0143, which replaces design change 0082's
//! reservation at queue time and with it M2's and M3-C's refusal of a `Train`): the unit at the front of a queue
//! starts only while `available − used ≥ cost` ([`ArmyCap::fits`]); else its producer waits with it at 0% and starts
//! by itself once there is room (`sim/producers/step.rs`, `Production::blocked`). A unit whose cost is 0 always starts,
//! even over capacity (design change 0176 point 3). The client names the faction's resource in the wait's line and
//! feedback ([`ArmyResource::name`]). The Cults' Unit Control is never checked: it is for display only, each Hall
//! producing by its own usage and capacity (design change 0344: "Global total is for display only";
//! `factions/cults/economy.md > RecruitmentHall > Production Behavior`: "Production logic is entirely local — global
//! Unit Control state does not affect production decisions").
//!
//! **Over capacity** (`factions/colonists/overview.md > Beacon Capacity > Over capacity`, design change 0112 as 0143
//! amends it): when a Colonists building is lost, or a Beacon's deployment is cancelled, more can be used than is
//! available. That is allowed: nothing is killed or removed, a unit in production finishes (Q0041 point 2), and the
//! units after it wait until there is room again. Being derived, the cap needs no rule of its own for it; the client
//! shows it in red. Tunnel Space alike, when an expansion takes up tunnel-region (`factions/syndicate/tunnels.md >
//! Tunnel Space`: "usage is simply displayed over capacity (e.g. 12/10)").
//!
//! **The objects** [`cap`] counts are those it is given: `Sim::army_cap` gives the map's. The units inside the Tunnel
//! Network and an Agent inside its site are alive and use Tunnel Space (design change 0265: "units count from
//! production for life"): slice M4-C chains its store of them into that iterator.

mod cults;
mod tunnel_space;

use serde::{Deserialize, Serialize};

use self::tunnel_space::TunnelSpace;
// The caps are the factions' own rules, with their defs (`defs/<faction>.rs`).
use crate::defs::{BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING, Category, ObjectKind, UNIT_CONTROL_CAP};
#[cfg(test)]
use crate::defs::{CELLS_PER_TUNNEL_SPACE, TUNNEL_SPACE_CAP};
use crate::ids::PlayerId;
use crate::map::Map;
use crate::object::Object;
use crate::player::Faction;
use crate::production::Production;

/// The resource that caps a faction's army.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmyResource {
    /// The GDO's: `factions/gdo/overview.md > GDOResources > Unit Control`.
    UnitControl,
    /// The Colonists': `factions/colonists/overview.md > ColonistResources > Beacon Capacity`.
    BeaconCapacity,
    /// The Syndicate's: `factions/syndicate/overview.md > SyndicateResources > Tunnel Space` (slice M4-B).
    TunnelSpace,
    /// The Cults': `factions/cults/overview.md > CultsResources > Unit Control` (slice M5-C), which the Recruitment
    /// Halls provide (`cults.rs`).
    CultsUnitControl,
}

impl ArmyResource {
    /// `faction`'s: every faction has its own (the Cults' since slice M5-C).
    pub fn of(faction: Faction) -> ArmyResource {
        match faction {
            Faction::Gdo => ArmyResource::UnitControl,
            Faction::Colonists => ArmyResource::BeaconCapacity,
            Faction::Syndicate => ArmyResource::TunnelSpace,
            Faction::Cults => ArmyResource::CultsUnitControl,
        }
    }

    /// Its name, as the DisplayHuds give it (`factions/gdo/overview.md > DisplayHud`, `factions/colonists/
    /// overview.md > DisplayHud`, `factions/syndicate/overview.md > DisplayHud`, `factions/cults/overview.md >
    /// DisplayHud`): the GDO and the Cults both call theirs "Unit Control".
    pub fn name(self) -> &'static str {
        match self {
            ArmyResource::UnitControl | ArmyResource::CultsUnitControl => "Unit Control",
            ArmyResource::BeaconCapacity => "Beacon Capacity",
            ArmyResource::TunnelSpace => "Tunnel Space",
        }
    }

    /// What a unit of `kind` takes of it: its UnitControlCost (`factions/gdo/units/peacekeeper.md >
    /// UnitControlCost`; a Cults unit's, `factions/cults/overview.md > CultsResources > Unit Control`: "Each Cult unit
    /// has a Unit Control cost"), its BeaconCapacityCost (`factions/colonists/units/drone.md > BeaconCapacityCost`) or
    /// its TunnelSpaceCost (`factions/syndicate/units/agent.md > TunnelSpaceCost`); 0 for a kind without one, and for
    /// another faction's unit (module docs: a unit costs only in its own faction's resource).
    pub fn cost_of(self, kind: ObjectKind) -> u32 {
        let def = kind.def();
        if def.faction.map(ArmyResource::of) != Some(self) {
            return 0;
        }
        let unit = def.unit.as_ref();
        let cost = match self {
            ArmyResource::UnitControl | ArmyResource::CultsUnitControl => unit.and_then(|u| u.unit_control_cost),
            ArmyResource::BeaconCapacity => unit.and_then(|u| u.beacon_capacity_cost),
            ArmyResource::TunnelSpace => unit.and_then(|u| u.tunnel_space_cost),
        };
        cost.unwrap_or(0)
    }

    /// Whether a unit takes its cost from the start of its production, and its start waits for room (design change
    /// 0143, "Reserved when production starts"; module docs): the GDO's, the Colonists' and the Syndicate's. Not the
    /// Cults' Unit Control, whose Halls count their Recruits in production themselves (design change 0342,
    /// `crate::recruitment::Lineages`) and which makes no production wait (design change 0344: "Global total is for
    /// display only").
    pub fn reserves(self) -> bool {
        self != ArmyResource::CultsUnitControl
    }

    /// What `o` takes up of it for its owner: a unit its cost, and a producer its unit in production's if the resource
    /// [`reserves`](ArmyResource::reserves) (module docs).
    fn taken_by(self, o: &Object) -> u32 {
        let in_production = o
            .production
            .as_ref()
            .and_then(Production::in_production)
            .filter(|_| self.reserves())
            .map_or(0, |kind| self.cost_of(kind));
        self.cost_of(o.kind).saturating_add(in_production)
    }
}

/// A player's army cap (`Sim::army_cap`): how much of their faction's [`ArmyResource`] their army uses, and how much
/// is available to it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ArmyCap {
    pub resource: ArmyResource,
    /// What the player's units and their producers' units in production take up (module docs; design change 0143).
    /// It may exceed `available` when Colonists buildings are lost or a deployment is cancelled
    /// (`factions/colonists/overview.md > Beacon Capacity > Over capacity`, design change 0112).
    pub used: u32,
    /// [`UNIT_CONTROL_CAP`] for Unit Control; min([`BEACON_CAPACITY_CAP`], [`BEACON_CAPACITY_PER_BUILDING`] × the
    /// player's Colonists structures) for Beacon Capacity; min([`crate::defs::TUNNEL_SPACE_CAP`], ⌊the unused
    /// tunnel-region cells of the player's Tunnels / [`crate::defs::CELLS_PER_TUNNEL_SPACE`]⌋) for Tunnel Space; the
    /// sum of the player's completed Halls' capacities, rounded down, for the Cults' Unit Control (`cults.rs`).
    pub available: u32,
}

impl ArmyCap {
    /// Whether one more unit that costs `cost` fits, `cost == 0 || available − used ≥ cost`: the start check of design
    /// change 0143 (the unit "needs free Beacon Capacity (available minus used) at least equal to its
    /// BeaconCapacityCost", `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`; the
    /// GDO's Unit Control alike), where a unit without a cost always fits, even over the cap (design change 0176 point
    /// 3, `factions/gdo/overview.md > Unit Control`: "a unit whose capacity cost is 0 never waits for capacity, even
    /// when used is over available"; every faction's cap). Today's units all cost something in their own faction's
    /// resource, so only another faction's unit, in a debug or a Custom game, costs 0 ([`ArmyResource::cost_of`]).
    /// Never asked of the Cults' Unit Control, which no production waits for ([`ArmyResource::reserves`]).
    pub fn fits(&self, cost: u32) -> bool {
        cost == 0 || self.used.saturating_add(cost) <= self.available
    }
}

/// The army cap of `player`, whose faction is `faction`, among `objects` (module docs: the map's, and the units held
/// out of it), on `map`. Tunnel Space looks at every object, as a neutral expansion takes up tunnel-region
/// (`tunnel_space.rs`), and so does the Cults' Unit Control, every Hall claiming tiles and every unit's lineage naming
/// its Halls (`cults.rs`); the other figures only at `player`'s.
pub(crate) fn cap<'a>(
    objects: impl IntoIterator<Item = &'a Object>,
    map: &Map,
    player: PlayerId,
    faction: Faction,
) -> ArmyCap {
    let resource = ArmyResource::of(faction);
    if resource == ArmyResource::CultsUnitControl {
        return cults::cap(objects, map, player);
    }
    let (mut used, mut providers) = (0u32, 0u32);
    let mut space = TunnelSpace::default();
    for o in objects {
        if resource == ArmyResource::TunnelSpace {
            space.see(o, player);
        }
        if o.owner != Some(player) {
            continue;
        }
        used = used.saturating_add(resource.taken_by(o));
        providers = providers.saturating_add(u32::from(provides_capacity(o)));
    }
    let available = match resource {
        ArmyResource::UnitControl => UNIT_CONTROL_CAP,
        ArmyResource::BeaconCapacity => providers
            .saturating_mul(BEACON_CAPACITY_PER_BUILDING)
            .min(BEACON_CAPACITY_CAP),
        ArmyResource::TunnelSpace => space.available(),
        // Returned above.
        ArmyResource::CultsUnitControl => 0,
    };
    ArmyCap {
        resource,
        used,
        available,
    }
}

/// The GDO's Unit Control `player` uses among `objects` (`Sim::unit_control`): the UnitControlCost of their GDO units
/// and of their producers' GDO units in production (design change 0143). Another faction's unit costs none of it
/// (module docs).
pub(crate) fn unit_control_used<'a>(objects: impl IntoIterator<Item = &'a Object>, player: PlayerId) -> u32 {
    objects
        .into_iter()
        .filter(|o| o.owner == Some(player))
        .map(|o| ArmyResource::UnitControl.taken_by(o))
        .fold(0, u32::saturating_add)
}

/// Whether `o` provides Beacon Capacity to its owner: a Colonists structure (`factions/colonists/economy.md > Warp In >
/// Beacon Capacity Retention`), complete or warping in ("throughout a Warp In", design change 0112; plan decision
/// T20), but for a Beacon under deployment ([`Object::is_deploying`], slice M3-F): a Beacon from its first frame (plan
/// decision T15, design change 0109), whose 5 arrives only on completion (`factions/colonists/economy.md > Beacon
/// Deployment > Beacon Capacity during deployment`, 0112: "The Beacon's 5 is provided only on completion").
fn provides_capacity(o: &Object) -> bool {
    let def = o.kind.def();
    def.category == Category::Structure && def.faction == Some(Faction::Colonists) && !o.is_deploying()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caps_and_the_costs_follow_the_design() {
        assert_eq!(UNIT_CONTROL_CAP, 200);
        // `factions/colonists/overview.md > ColonistResources > Beacon Capacity`: "caps at 200";
        // `factions/colonists/economy.md > Beacon
        // > BeaconCapacity provided - 5`.
        assert_eq!((BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING), (200, 5));
        // `factions/gdo/units/peacekeeper.md > UnitControlCost - 1`; the Syndicate's Guard has none.
        let cost = |kind: ObjectKind| kind.def().unit.as_ref().and_then(|u| u.unit_control_cost);
        assert_eq!(
            (cost(ObjectKind::Peacekeeper), cost(ObjectKind::Guard)),
            (Some(1), None)
        );
        // `factions/colonists/units/README.md`: the Drone's BeaconCapacityCost is 1, the Legionnaires' 2.
        let (uc, bc, ts, cu) = (
            ArmyResource::UnitControl,
            ArmyResource::BeaconCapacity,
            ArmyResource::TunnelSpace,
            ArmyResource::CultsUnitControl,
        );
        let costs = |resource: ArmyResource| {
            [
                ObjectKind::Peacekeeper,
                ObjectKind::Drone,
                ObjectKind::SabreLegionnaire,
                ObjectKind::Guard,
                ObjectKind::Agent,
                ObjectKind::Recruit,
            ]
            .map(|kind| resource.cost_of(kind))
        };
        // The Peacekeeper and the Recruit both have a UnitControlCost of 1, each in its own faction's resource alone
        // (module docs).
        assert_eq!(costs(uc), [1, 0, 0, 0, 0, 0]);
        assert_eq!(costs(bc), [0, 1, 2, 0, 0, 0]);
        // `factions/syndicate/units/guard.md` and `factions/syndicate/units/agent.md > TunnelSpaceCost - 1`.
        assert_eq!(costs(ts), [0, 0, 0, 1, 1, 0]);
        // `factions/cults/overview.md > CultsResources > Unit Control`: "Each Cult unit has a Unit Control cost".
        assert_eq!(costs(cu), [0, 0, 0, 0, 0, 1]);
        assert_eq!(cu.cost_of(ObjectKind::RecruitmentHall), 0);
        assert_eq!(bc.cost_of(ObjectKind::RoboPortal), 0, "a structure costs nothing");
        assert_eq!(ts.cost_of(ObjectKind::Hideout), 0);
        // `factions/syndicate/tunnels.md > Tunnel Space`: "floor((...) / 4) ... capped at a hard maximum of 200".
        assert_eq!((TUNNEL_SPACE_CAP, CELLS_PER_TUNNEL_SPACE), (200, 4));
    }

    /// Each faction has its own cap, named as its DisplayHud names it: the GDO's and the Cults' are both "Unit
    /// Control" (`factions/cults/overview.md > DisplayHud`), and only the Cults' reserves nothing.
    #[test]
    fn each_faction_has_its_own_cap() {
        let resources = Faction::ALL.iter().map(|&f| ArmyResource::of(f));
        let names: Vec<(&str, bool)> = resources.map(|r| (r.name(), r.reserves())).collect();
        assert_eq!(
            names,
            [
                ("Unit Control", true),
                ("Tunnel Space", true),
                ("Unit Control", false),
                ("Beacon Capacity", true)
            ]
        );
    }

    /// The Director's decision (slice M5-C; module docs): a unit costs only in its own faction's resource, both ways. A
    /// GDO player's Recruit takes none of their Unit Control, and a Cults player's Peacekeeper none of theirs, though
    /// both have a UnitControlCost; each player's own unit takes its 1 (the Recruit as an orphan: design change 0344,
    /// "Recruits no Hall produced [...] are orphans from the start").
    #[test]
    fn another_factions_unit_costs_nothing_in_either_direction() {
        use crate::ids::PlayerId;
        use crate::scenario::SpawnAt;
        use crate::sim::Sim;
        use crate::testing::ScenarioBuilder;
        use crate::units::TilePos;
        let (gdo, cults) = (PlayerId(1), PlayerId(2));
        let used = |spawns: &[(ObjectKind, PlayerId)]| {
            let mut builder = ScenarioBuilder::new(&vec![".".repeat(16); 16].join("\n"))
                .faction(gdo, Faction::Gdo)
                .faction(cults, Faction::Cults);
            for (i, &(kind, owner)) in (0..).zip(spawns) {
                builder = builder.spawn(kind, Some(owner), SpawnAt::Point(TilePos::new(2 + 2 * i, 2).center()));
            }
            let sim = Sim::new(&builder.build());
            let used = |player| sim.army_cap(player).map(|cap| (cap.resource, cap.used));
            (used(gdo), used(cults))
        };
        let (uc, cu) = (ArmyResource::UnitControl, ArmyResource::CultsUnitControl);
        let foreign = [(ObjectKind::Recruit, gdo), (ObjectKind::Peacekeeper, cults)];
        assert_eq!(used(&foreign), (Some((uc, 0)), Some((cu, 0))));
        let own = [(ObjectKind::Peacekeeper, gdo), (ObjectKind::Recruit, cults)];
        assert_eq!(used(&own), (Some((uc, 1)), Some((cu, 1))));
    }

    /// A unit fits while the used figure stays at most at the available one.
    #[test]
    fn a_unit_fits_up_to_the_cap() {
        let cap = |used| ArmyCap {
            resource: ArmyResource::BeaconCapacity,
            used,
            available: 10,
        };
        assert!(cap(8).fits(2) && !cap(9).fits(2));
        assert!(cap(9).fits(1) && !cap(10).fits(1));
        assert!(!cap(u32::MAX).fits(1), "saturates");
    }

    /// Design change 0176 point 3 (`factions/gdo/overview.md > Unit Control`: "a unit whose capacity cost is 0 never
    /// waits for capacity, even when used is over available"): the check is `cost == 0 || free >= cost`, for every
    /// faction's cap. At the cap and over it, a unit of cost 0 starts and one of cost 1 waits.
    #[test]
    fn a_unit_without_a_cost_fits_even_over_the_cap() {
        for resource in [
            ArmyResource::UnitControl,
            ArmyResource::BeaconCapacity,
            ArmyResource::TunnelSpace,
        ] {
            for (used, available) in [(10, 10), (11, 10), (15, 0), (u32::MAX, 0), (0, 0)] {
                let cap = ArmyCap {
                    resource,
                    used,
                    available,
                };
                assert!(cap.fits(0), "{cap:?}: cost 0 passes");
                assert!(!cap.fits(1), "{cap:?}: cost 1 waits");
            }
        }
    }
}
