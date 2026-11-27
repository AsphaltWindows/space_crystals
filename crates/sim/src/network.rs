//! The Tunnel Network — `factions/syndicate/tunnels.md > Tunnel Network`, `factions/syndicate/tunnels.md > Tunnel >
//! SymmetryType - ABCD` (Side A's paths), `factions/syndicate/tunnel_menus.md > EjectMenu`;
//! `systems/controls/commands.md > Enter`; slice M4-C. "The Tunnel Network is the collective of all Tunnels owned by a
//! Syndicate player. Units inside the Tunnel Network can travel between Tunnels freely, but can only enter or exit the
//! network through a Tunnel whose tier is sufficient for their unit base category."
//!
//! - **Units off the map** (plan decision T30): a unit inside the network is not among the sim's objects. It is
//!   [`Held`] in one store beside them (`Sim::held`), with its id, at a [`Place`]. So every system that walks the
//!   objects (movement, collision, vision, targeting, picking, records) passes it by without a filter, and a
//!   forgotten case can't show, move or hit a unit that isn't there. What must count the held units says so:
//!   Tunnel Space (`Sim::army_cap`), the invariants (`testing::invariants`), the hash and the saves, the client's
//!   control groups.
//! - **Inside** (design change 0249): the unit is inert. Its orders are cleared, "queued ones included", its move and
//!   its attack end; an Agent "keeps its load". It regenerates a share of its MaxHP every second, the Tunnel's
//!   `TunnelDef::regen_per_second` ([`Held::regenerate`]).
//! - **The transit delay** (design changes 0245 and 0246): a unit that walked in "can exit only once" the Tunnel's
//!   `TunnelDef::transit_delay` has passed, from any Tunnel; a unit produced inside has none ([`Held::may_exit`]).
//!   These two numbers are the Tunnel Network's, so the Tunnel type's: every Tunnel is one, at any tier (plan decision
//!   T39; [`rules`]).
//! - **Side A's two paths** ([`Paths`], design changes 0225 and 0248, `network/paths.rs`): "the exit path is the half
//!   of Side A nearer the crystal hatch (Side B), the entry path the half nearer the Supplies hatch (Side D). Both turn
//!   and flip with the Tunnel." Each has one fixed spot on the side's line, `Tunnel::path_offset_su` from its middle,
//!   where the client draws the path (`object_render::tunnel`).
//! - (M6-SD) **The air doors** ([`Paths::air_entry`], [`Paths::air_exit`], plan decision T96): an air unit goes in and
//!   comes out only through the two doors on a Tier 3 Tunnel's roof, never by Side A's paths
//!   (`factions/syndicate/tunnels.md > Tunnel > Look by tier`: "Aircraft use these doors, not the Side A paths";
//!   [`uses_air_doors`]). It rises out of the exit door, and descends into the entry door, for its type's
//!   `UnitDef::air_door_ticks`, on the map meanwhile, an air target (design change 0431); each door lets one aircraft
//!   through every `Tunnel::air_door_interval_ticks` (`TunnelState::air_out`, `TunnelState::air_in`), beside the ground
//!   lane (`sim/network/ejects.rs`, `sim/network/entries.rs`).
//! - **Which Tunnels** ([`admits`]): one whose tier's Transit lists the unit's base, at "the tier it had before the
//!   upgrade began" while it upgrades (0249; `TunnelState::tier` is that tier until an upgrade completes, slice
//!   M4-G): (M6-SD) an air unit at Tier 3, through the air doors.
//! - (M6-SN) **The services** ([`services`], plan decision T98): the manned units inside take their turns in their
//!   owner's Null Chambers ([`Held::service`]), and leave the network buffed once their turn is done.
//!
//! The steps that take units in and let them out are the sim's (`sim/network.rs`); the walk to the entry path, and the
//! flight to the air entry door, are the Enter order's (`orders::enter`).

mod paths;
pub mod services;

#[cfg(test)]
pub(crate) use paths::WALL_CLEARANCE;
pub use paths::{EntryPath, Paths, door_spot, over_door};
pub use services::Service;

use serde::{Deserialize, Serialize};

use crate::augmentation::Research;
use crate::defs::{ObjectKind, Transit, TunnelDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::state::{Behaviour, Enter};
use crate::units::Tick;
use crate::{regeneration, tunnel};

/// The Tunnel Network's rules (module docs), the Tunnel type's def: its transit delay and its regeneration
/// (`factions/syndicate/tunnels.md > Tunnel Network > Transit delay`, `factions/syndicate/tunnels.md > Regeneration
/// inside`).
pub fn rules() -> &'static TunnelDef {
    tunnel::def_of(ObjectKind::Tunnel).expect("the Tunnel has a tier table")
}

/// Where a held unit is (plan decision T30). Match it exhaustively, and never assume a held unit's place is on the
/// map: a transport's passengers (design changes 0221 and 0222, deferred) will be held at their carrier, which may
/// itself be held.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
    /// In its owner's Tunnel Network.
    Network,
    /// Inside the site it builds, this object (slice M4-E, `construction::site`; `factions/syndicate/units/agent.md >
    /// Building > Tunnel Construction Flow`: "the partially-built Tunnel appears at the location with the Agent inside
    /// it"): not in the network, so no Eject brings it out and it doesn't regenerate. It comes out at the site's
    /// centre if the site goes, into the network once it completes (`sim/sites.rs`).
    Site(ObjectId),
    /// Inside this building that holds units (slice M5-F1, `crate::holding`; `systems/controls/commands.md > Enter`:
    /// "The unit walks to the structure's entrance side and goes inside"): an Armory's internal pool
    /// (`factions/cults/structures/armory.md > InternalRecruitCapacity`). It comes out through the building's exit
    /// doorway (Eject All), or around its footprint, alive, when the building is destroyed
    /// (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`; `sim/holding.rs`). Not in
    /// any network: it doesn't regenerate.
    Building(ObjectId),
}

/// A unit off the map (plan decision T30, module docs): the object, with its id, where it is and since when.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Held {
    /// The unit, as it left the map but for what entering clears (module docs): its `pos` is where it stood last,
    /// and means nothing while it is held.
    pub object: Object,
    pub place: Place,
    /// The tick it came to its place: `factions/syndicate/tunnel_menus.md > EjectMenu`'s "the one that has been inside
    /// the network longest" comes out first (design change 0251); in a building, the pool's order
    /// (`Sim::held_in`).
    pub since: Tick,
    /// Whether it walked in (design change 0245: "Each unit inside stores when it entered"), so that the transit
    /// delay applies from `since`; `false` for a unit produced inside ("no delay", 0246), rallied into the network
    /// included, and for the Agent that finished a Tunnel (design change 0301, answering Q0087: "The Agent that
    /// completes a Tunnel can be ejected at once [...] produced units rallied into the network have no delay
    /// either").
    pub entered: bool,
    /// (M5-F1, M5-F2) The tick from which it is ready to leave the building it is in through the building's exit
    /// doorway: ejected (`factions/cults/structures/armory.md > Eject All`: "ejects all stored Recruits from the exit
    /// side in a rapid stream, one after another"), or reserved by the entry at the front of the building's queue,
    /// which finished in that tick (a Release lets it out unchanged, a training trains it as it comes out:
    /// `sim/producers/pool.rs`). The building lets its ready units out one at a time, "in the order they are ready"
    /// (design change 0348, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`), each
    /// once its doorway is free (`Sim::step_holding`). Inside until then. Saved and hashed only while set, so the
    /// states of games in which no unit leaves a building hash as they did before the field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaving: Option<Tick>,
    /// (M6-SN) Its turns at its network's services (plan decision T98, [`services`]): its Null Chamber turn, under way
    /// or done, while it is inside. Saved and hashed only while it has one, so the states of games in which no unit
    /// takes a turn hash as they did before the field existed.
    #[serde(default, skip_serializing_if = "Service::is_none")]
    pub service: Service,
}

impl Held {
    /// Whether the unit may leave the network in `tick` (design changes 0245 and 0246): at once if it was produced
    /// inside, the transit delay ([`rules`]) after it walked in otherwise.
    pub fn may_exit(&self, tick: Tick) -> bool {
        !self.entered || tick >= self.since.saturating_add(rules().transit_delay)
    }

    /// One tick of `factions/syndicate/tunnels.md > Regeneration inside` (design change 0249): the network's share of
    /// its MaxHP a second ([`rules`]), up to its MaxHP (`crate::regeneration`, the rule of the Tunnel expansions' too),
    /// (M5-N1a) its Augmentations' included, at its owner's levels in `research`. Only "inside the Tunnel Network": a
    /// unit held elsewhere regenerates nothing.
    pub(crate) fn regenerate(&mut self, research: Research) {
        match self.place {
            Place::Network => regeneration::regenerate(&mut self.object, rules().regen_per_second, research),
            Place::Site(_) | Place::Building(_) => {}
        }
    }
}

/// Why a structure doesn't let a unit in, or a Tunnel doesn't let one out ([`admits`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NoTransit {
    /// The object takes no units in: it is neither a Tunnel nor a building that holds units, or isn't one that works
    /// yet (under construction).
    NotATunnel,
    /// The kind isn't a unit.
    NotAUnit,
    /// Its tier's Transit doesn't list the unit's base (`factions/syndicate/tunnels.md > Transit Tier Requirements`):
    /// (M6-SD) an air unit's below Tier 3 too, the doors being Tier 3's.
    Tier,
    /// (M5-F1) A building that holds units, whose type doesn't take units of the kind in (`HoldsDef::accepts`;
    /// `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only").
    NotAccepted,
}

/// Whether `target` lets a unit of `kind` in: a building that holds units by its own rule (`holding::admits`, slice
/// M5-F1), and a Tunnel into or out of the network by
/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: it is a Tunnel in operation
/// ([`is_exit`]) whose tier's Transit lists the unit's base: "Tier 1+: Infantry", "Tier 2+: Vehicles", "Tier 3+: Air
/// units", (M6-SD) by its air doors. The tier is the one the Tunnel has (`TunnelState::tier`), which an upgrade under
/// way leaves alone until it completes (design change 0249: "until the upgrade has fully finished, a Tunnel only
/// accepts, and only lets out, units supported by the tier it had before the upgrade began"): a Tunnel upgrading to
/// Tier 3 lets no aircraft through ("not one mid-upgrade to Tier 3", `factions/syndicate/units/specter.md`).
pub fn admits(target: &Object, kind: ObjectKind) -> Result<(), NoTransit> {
    if crate::holding::holds(target).is_some() {
        return crate::holding::admits(target, kind);
    }
    let tunnel = target;
    if !is_exit(tunnel) {
        return Err(NoTransit::NotATunnel);
    }
    let row = tunnel::tier_def(tunnel).ok_or(NoTransit::NotATunnel)?;
    let unit = kind.def().unit.as_ref().ok_or(NoTransit::NotAUnit)?;
    if passes(row.transit, Transit::of(unit.base)) {
        Ok(())
    } else {
        Err(NoTransit::Tier)
    }
}

/// Whether a Tunnel whose tier's Transit is `transit` lets a unit of `class` through: the tier lists the class. The one
/// rule of the gate, the steps and the default command (`orders::resolution`). (M6-SD) Air units pass at Tier 3, by the
/// air doors, since slice M6-SD (until then the switch `Tunnel::air_doors` kept every one out).
pub fn passes(transit: &[Transit], class: Transit) -> bool {
    transit.contains(&class)
}

/// (M6-SD) Whether a unit of `kind` goes in and comes out by a Tunnel's air doors rather than Side A's paths: an air
/// unit, a Hover Craft's or a Glider's ("Tier 3+: Air units (Hover Craft, Glider)", `Transit::Air`; "Aircraft use these
/// doors, not the Side A paths").
pub fn uses_air_doors(kind: ObjectKind) -> bool {
    (kind.def().unit.as_ref()).is_some_and(|unit| Transit::of(unit.base) == Transit::Air)
}

/// (M6-SD) How far up out of an air door unit `o` is while it rises out of the exit door (`Mover::rise`) or descends
/// into the entry door (`Enter::descent`; design change 0431): from 0, in the door, to 1, in the air, by the whole
/// ticks of it done. `None` for a unit doing neither, in the air as any air unit is. The client draws its height by it
/// (`object_render::flight`).
pub fn door_lift(o: &Object) -> Option<f32> {
    if let Some(rise) = o.mover.as_ref().and_then(|mover| mover.rise) {
        return Some(rise.ratio());
    }
    let active = o.orders.as_ref()?.current.as_ref()?;
    match active.state {
        Behaviour::Enter(Enter {
            descent: Some(descent), ..
        }) => Some(1.0 - descent.ratio()),
        _ => None,
    }
}

/// Whether `tunnel` is a way into and out of the network: a Tunnel with an owner, in operation
/// (`factions/syndicate/tunnels.md > Losing Tunnels`: "A Tunnel that is still under construction does not count as an
/// exit"; `Object::is_operational`, slice M4-E).
pub fn is_exit(tunnel: &Object) -> bool {
    tunnel.tunnel.is_some() && tunnel.placement.is_some() && tunnel.owner.is_some() && tunnel.is_operational()
}

/// One entry of a Tunnel's eject queue (`TunnelState::eject`; `factions/syndicate/tunnel_menus.md > EjectMenu`, design
/// change 0251: "each click reserves a specific unit [...]; the queue can't be cancelled").
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eject {
    /// The reserved unit, held in the Tunnel owner's network.
    pub unit: ObjectId,
    /// The order it gets as it comes out: a produced unit's default command on its producer's rally point (design
    /// changes 0136 and 0253; `factions/syndicate/structures/README.md > Rally Point Behavior`). `None` for a
    /// player's Eject, and for a rally order whose target left the game meanwhile: the unit then walks straight out
    /// and idles.
    pub order: Option<crate::orders::OrderKind>,
}

impl StateHash for Place {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Place::Network => h.tag(0),
            Place::Site(site) => {
                h.tag(1);
                site.hash(h);
            }
            // (M5-F1) A tag of its own: a state with no unit in a building hashes as it did before.
            Place::Building(building) => {
                h.tag(2);
                building.hash(h);
            }
        }
    }
}

impl StateHash for Held {
    fn hash(&self, h: &mut StateHasher) {
        let Held {
            object,
            place,
            since,
            entered,
            leaving,
            service,
        } = self;
        object.hash(h);
        place.hash(h);
        since.hash(h);
        entered.hash(h);
        // (M5-F1) Only while set, under a tag of its own: the goldens stand.
        if let Some(tick) = leaving {
            h.tag(1);
            tick.hash(h);
        }
        // (M6-SN) The same for its services' turns.
        if !service.is_none() {
            h.tag(2);
            service.hash(h);
        }
    }
}

impl StateHash for Eject {
    fn hash(&self, h: &mut StateHasher) {
        let Eject { unit, order } = self;
        unit.hash(h);
        order.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// A Guard, as a game has one.
    fn guard() -> Object {
        let mut t = crate::testing::ScenarioBuilder::new(&"........\n".repeat(6)).start();
        t.spawn_unit_at(1, ObjectKind::Guard, crate::ids::PlayerId(1), Vec2::new(100.0, 100.0))
            .run(1);
        t.sim().objects().next().expect("the Guard").clone()
    }

    /// Design changes 0245 and 0246: a unit that walked in may leave 32 frames later, a produced one at once.
    #[test]
    fn the_transit_delay_is_for_units_that_walked_in() {
        let object = guard();
        let held = |entered| Held {
            object: object.clone(),
            place: Place::Network,
            since: 10,
            entered,
            leaving: None,
            service: Service::default(),
        };
        assert!(!held(true).may_exit(10) && !held(true).may_exit(41) && held(true).may_exit(42));
        assert!(held(false).may_exit(10));
    }

    /// Design change 0249: 1% of MaxHP a second, never above MaxHP.
    #[test]
    fn a_held_unit_regenerates_a_hundredth_of_its_max_hp_a_second() {
        let mut held = Held {
            object: guard(),
            place: Place::Network,
            since: 0,
            entered: true,
            leaving: None,
            service: Service::default(),
        };
        held.object.hp = Some(40.0);
        for _ in 0..160 {
            held.regenerate(Research::NONE);
        }
        let hp = held.object.hp.unwrap();
        assert!((hp - 48.0).abs() < 1e-3, "10 s: 50% to 60% of 80, got {hp}");
        held.object.hp = Some(79.99);
        held.regenerate(Research::NONE);
        assert_eq!(held.object.hp, Some(80.0));
    }
}
