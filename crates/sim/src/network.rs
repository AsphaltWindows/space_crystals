//! The Tunnel Network — `factions/syndicate/tunnels.md > Tunnel Network`, `factions/syndicate/tunnels.md > Tunnel >
//! SymmetryType - ABCD` (Side A's paths), `factions/syndicate/tunnels.md > EjectMenu`; `systems/controls/commands.md >
//! Enter`; slice M4-C. "The Tunnel Network is the collective of all Tunnels owned by a Syndicate player. Units inside
//! the Tunnel Network can travel between Tunnels freely, but can only enter or exit the network through a Tunnel whose
//! tier is sufficient for their unit base category."
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
//! - **Side A's two paths** ([`Paths`], design changes 0225 and 0248): "the exit path is the half of Side A nearer the
//!   crystal hatch (Side B), the entry path the half nearer the Supplies hatch (Side D). Both turn and flip with the
//!   Tunnel." Each has one fixed spot on the side's line, [`PATH_OFFSET_SU`] from its middle, where the client draws
//!   the path (`object_render::tunnel`).
//! - **Which Tunnels** ([`admits`]): one whose tier's Transit lists the unit's base, at "the tier it had before the
//!   upgrade began" while it upgrades (0249; `TunnelState::tier` is that tier until an upgrade completes, slice
//!   M4-G). **No air unit passes in M4** ([`AIR_DOORS`]).
//!
//! The steps that take units in and let them out are the sim's (`sim/network.rs`); the walk to the entry path is the
//! Enter order's (`orders::enter`).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Transit, TunnelDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::{Object, Placement};
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::tuning::tunnel::{AIR_DOORS, ENTRY_REACH_SU, WALK_OUT_SU};
use crate::tunnel;
use crate::units::{SU_PER_GU, TICKS_PER_SECOND, Tick};

/// A tunable of `tuning::tunnel`, re-exported here for the client, which draws the paths by it
/// (`object_render::tunnel`).
pub use crate::tuning::tunnel::PATH_OFFSET_SU;

/// The Tunnel Network's rules (module docs), the Tunnel type's def: its transit delay and its regeneration
/// (`factions/syndicate/tunnels.md > Tunnel Network > Transit delay`, `factions/syndicate/tunnels.md > Regeneration
/// inside`).
pub fn rules() -> &'static TunnelDef {
    tunnel::def_of(ObjectKind::Tunnel).expect("the Tunnel has a tier table")
}

/// How far from the Tunnel's wall a unit stands on a path's spot, space units, so that no rounding makes its box
/// overlap the footprint (a technical margin: a sixty-fourth of a space unit, as a doorway's).
pub(crate) const WALL_CLEARANCE: f32 = 1.0 / 64.0;

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
    /// The tick it came to its place: `factions/syndicate/tunnels.md > EjectMenu`'s "the one that has been inside the
    /// network longest" comes out first (design change 0251); in a building, the pool's order
    /// (`Sim::held_in`).
    pub since: Tick,
    /// Whether it walked in (design change 0245: "Each unit inside stores when it entered"), so that the transit
    /// delay applies from `since`; `false` for a unit produced inside ("no delay", 0246), rallied into the network
    /// included, and for the Agent that finished a Tunnel (design change 0301, answering Q0087: "The Agent that
    /// completes a Tunnel can be ejected at once [...] produced units rallied into the network have no delay
    /// either").
    pub entered: bool,
    /// (M5-F1) Ejected from the building it is in (`factions/cults/structures/armory.md > Eject All`: "ejects all
    /// stored Recruits from the exit side in a rapid stream, one after another"): it comes out through the building's
    /// exit doorway once the doorway is free, after the units ejected before it (`Sim::step_holding`). Inside until
    /// then. Saved and hashed only while set, so the states of games that never eject hash as they did before the
    /// field existed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub ejected: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Held {
    /// Whether the unit may leave the network in `tick` (design changes 0245 and 0246): at once if it was produced
    /// inside, the transit delay ([`rules`]) after it walked in otherwise.
    pub fn may_exit(&self, tick: Tick) -> bool {
        !self.entered || tick >= self.since.saturating_add(rules().transit_delay)
    }

    /// One tick of `factions/syndicate/tunnels.md > Regeneration inside` (design change 0249): the network's share of
    /// its MaxHP a second ([`rules`]), up to its MaxHP. Only "inside the Tunnel Network": a unit held elsewhere
    /// regenerates nothing.
    pub(crate) fn regenerate(&mut self) {
        match self.place {
            Place::Network => {}
            Place::Site(_) | Place::Building(_) => return,
        }
        let (Some(hp), Some(max)) = (self.object.hp, self.object.max_hp()) else {
            return;
        };
        if hp < max {
            let gain = max * (rules().regen_per_second / TICKS_PER_SECOND as f32);
            self.object.hp = Some((hp + gain).min(max));
        }
    }
}

/// The two paths of a placed Tunnel's Side A (module docs): the points on the side's line where units go in and come
/// out, and the direction the side faces.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Paths {
    /// The unit vector out of Side A.
    pub outward: Vec2,
    /// The entry path's spot: on Side A's line, [`PATH_OFFSET_SU`] from its middle towards Side D.
    pub entry: Vec2,
    /// The exit path's spot: on Side A's line, [`PATH_OFFSET_SU`] from its middle towards Side B.
    pub exit: Vec2,
}

impl Paths {
    /// The paths of a `kind` structure placed at `placement`: `None` unless its sides are lettered A, B and D (the
    /// Tunnel's ABCD), wherever they face (`crate::sides::sides`).
    pub fn of(kind: ObjectKind, placement: &Placement) -> Option<Paths> {
        let letters = sides(kind, placement.orientation())?;
        let facing = |letter: SideLetter| {
            let i = letters.iter().position(|&l| l == letter)?;
            let (x, y) = DIRECTIONS[i];
            Some(Vec2::new(x as f32, y as f32))
        };
        let (outward, towards_b, towards_d) = (facing(SideLetter::A)?, facing(SideLetter::B)?, facing(SideLetter::D)?);
        let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        let middle = placement.center() + outward * outward.abs().dot(half);
        Some(Paths {
            outward,
            entry: middle + towards_d * PATH_OFFSET_SU,
            exit: middle + towards_b * PATH_OFFSET_SU,
        })
    }

    /// The paths of `tunnel`, a placed Tunnel.
    pub fn of_object(tunnel: &Object) -> Option<Paths> {
        tunnel.tunnel.as_ref()?;
        Paths::of(tunnel.kind, tunnel.placement.as_ref()?)
    }

    /// The entry path, as every way in is ([`EntryPath`]).
    pub fn entry_path(&self) -> EntryPath {
        EntryPath {
            outward: self.outward,
            spot: self.entry,
            standoff: 0.0,
        }
    }

    /// Where an entering unit with half extents `half` walks to: on the entry path's spot, against the wall.
    pub fn entry_stand(&self, half: Vec2) -> Vec2 {
        self.entry_path().stand(half)
    }

    /// Where an ejected unit with half extents `half` appears (design change 0250: "Each unit appears at the exit
    /// path's spot just outside Side A"): on the exit path's spot, against the wall.
    pub fn exit_stand(&self, half: Vec2) -> Vec2 {
        let exit = EntryPath {
            spot: self.exit,
            ..self.entry_path()
        };
        exit.stand(half)
    }

    /// Whether a unit with half extents `half` at `pos` has walked into the entry path ([`EntryPath::reached`]).
    pub fn reached_entry(&self, pos: Vec2, half: Vec2) -> bool {
        self.entry_path().reached(pos, half)
    }

    /// Where a unit that appeared at `stand` with no order walks to: [`WALK_OUT_SU`] "straight out of Side A".
    pub fn walk_out(&self, stand: Vec2) -> Vec2 {
        stand + self.outward * WALK_OUT_SU
    }
}

/// The way into a structure that units Enter (`systems/controls/commands.md > Enter`: "The unit walks to the
/// structure's entrance side and goes inside"), M4-C's entry path generalized (slice M5-F1): a spot on the line of the
/// side units go in by, and the direction out of that side. A Tunnel's is its entry path on Side A
/// ([`Paths::entry_path`]); a building that holds units has one in the middle of its entrance side
/// (`holding::entry_path`). The Enter order walks to it (`orders::enter`) and the step right after 5a takes in the
/// units that reached it (`Sim::step_entries`), one rule for both.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntryPath {
    /// The unit vector out of the side.
    pub outward: Vec2,
    /// The spot, on the side's line.
    pub spot: Vec2,
    /// How far clear of the wall an entering unit's walk ends ([`EntryPath::stand`]), space units: none for a
    /// Tunnel's, whose units walk up against it (M4-C); `holding::ENTRY_STANDOFF_SU` for a building that holds units.
    pub standoff: f32,
}

impl EntryPath {
    /// The entry path of `target`: a placed Tunnel's ([`Paths::of_object`]), or a placed building's that holds units
    /// (`holding::entry_path`); `None` for anything else.
    pub fn of_object(target: &Object) -> Option<EntryPath> {
        match Paths::of_object(target) {
            Some(paths) => Some(paths.entry_path()),
            None => crate::holding::entry_path(target),
        }
    }

    /// Where an entering unit with half extents `half` walks to: on the spot, "just outside" the side, its box
    /// [`EntryPath::standoff`] clear of the wall (against it, for a Tunnel).
    pub fn stand(&self, half: Vec2) -> Vec2 {
        self.spot + self.outward * (self.outward.abs().dot(half) + WALL_CLEARANCE + self.standoff)
    }

    /// Whether a unit with half extents `half` at `pos` has walked into the path: its box is within [`ENTRY_REACH_SU`]
    /// of the spot.
    pub fn reached(&self, pos: Vec2, half: Vec2) -> bool {
        let apart = (pos - self.spot).abs() - half;
        apart.max(Vec2::ZERO).length() <= ENTRY_REACH_SU
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
    /// Its tier's Transit doesn't list the unit's base (`factions/syndicate/tunnels.md > Transit Tier Requirements`).
    Tier,
    /// An air unit: no air doors in M4 ([`AIR_DOORS`]).
    Air,
    /// (M5-F1) A building that holds units, whose type doesn't take units of the kind in (`HoldsDef::accepts`;
    /// `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only").
    NotAccepted,
}

/// Whether `target` lets a unit of `kind` in: a building that holds units by its own rule (`holding::admits`, slice
/// M5-F1), and a Tunnel into or out of the network by
/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: it is a Tunnel in operation
/// ([`is_exit`]) whose tier's Transit lists the unit's base: "Tier 1+: Infantry", "Tier 2+: Vehicles", "Tier 3+: Air
/// units", the last not in M4 ([`AIR_DOORS`]). The tier is the one the Tunnel has (`TunnelState::tier`), which an
/// upgrade under way leaves alone until it completes (design change 0249: "until the upgrade has fully finished, a
/// Tunnel only accepts, and only lets out, units supported by the tier it had before the upgrade began").
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
    let class = Transit::of(unit.base);
    if passes(row.transit, class) {
        Ok(())
    } else if class == Transit::Air && !AIR_DOORS {
        Err(NoTransit::Air)
    } else {
        Err(NoTransit::Tier)
    }
}

/// Whether a Tunnel whose tier's Transit is `transit` lets a unit of `class` through: the tier lists the class, and
/// the unit isn't an air unit while there are no air doors ([`AIR_DOORS`]). The one rule of the gate, the steps and
/// the default command (`orders::resolution`).
pub fn passes(transit: &[Transit], class: Transit) -> bool {
    transit.contains(&class) && (class != Transit::Air || AIR_DOORS)
}

/// Whether `tunnel` is a way into and out of the network: a Tunnel with an owner, in operation
/// (`factions/syndicate/tunnels.md > Losing Tunnels`: "A Tunnel that is still under construction does not count as an
/// exit"; `Object::is_operational`, slice M4-E).
pub fn is_exit(tunnel: &Object) -> bool {
    tunnel.tunnel.is_some() && tunnel.placement.is_some() && tunnel.owner.is_some() && tunnel.is_operational()
}

/// One entry of a Tunnel's eject queue (`TunnelState::eject`; `factions/syndicate/tunnels.md > EjectMenu`, design
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
            ejected,
        } = self;
        object.hash(h);
        place.hash(h);
        since.hash(h);
        entered.hash(h);
        // (M5-F1) Only while set, under a tag of its own: the goldens stand.
        if *ejected {
            h.tag(1);
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
    use super::*;
    use crate::object::Rotation;
    use crate::scenario::BuildingOrientation;
    use crate::units::TilePos;

    fn tunnel_at(rotation: Rotation, flip_h: bool) -> Placement {
        let turned = BuildingOrientation {
            rotation,
            flip_h,
            flip_v: false,
        };
        Placement {
            rotation: turned.rotation,
            flip_h: turned.flip_h,
            ..Placement::new(TilePos::new(4, 4), ObjectKind::Tunnel.def().footprint)
        }
    }

    /// A Guard, as a game has one.
    fn guard() -> Object {
        let mut t = crate::testing::ScenarioBuilder::new(&"........\n".repeat(6)).start();
        t.spawn_unit_at(1, ObjectKind::Guard, crate::ids::PlayerId(1), Vec2::new(100.0, 100.0))
            .run(1);
        t.sim().objects().next().expect("the Guard").clone()
    }

    /// Design change 0248: unturned, Side A is the top; the exit path is its half nearer Side B (east), the entry
    /// path the half nearer Side D (west). The footprint covers tiles 4..8, so the side's line is y = 256 and its
    /// middle x = 384.
    #[test]
    fn the_exit_is_nearer_b_and_the_entry_nearer_d() {
        let paths = Paths::of(ObjectKind::Tunnel, &tunnel_at(Rotation::R0, false)).unwrap();
        assert_eq!(paths.outward, Vec2::new(0.0, -1.0));
        assert_eq!(paths.exit, Vec2::new(384.0 + PATH_OFFSET_SU, 256.0));
        assert_eq!(paths.entry, Vec2::new(384.0 - PATH_OFFSET_SU, 256.0));
        // A Guard (36 x 36) stands with its box against the wall.
        let half = Vec2::splat(18.0);
        assert_eq!(paths.exit_stand(half), Vec2::new(414.0, 256.0 - 18.0 - WALL_CLEARANCE));
        assert_eq!(paths.entry_stand(half), Vec2::new(354.0, 256.0 - 18.0 - WALL_CLEARANCE));
        assert!(paths.reached_entry(paths.entry_stand(half), half));
        assert!(
            !paths.reached_entry(paths.exit_stand(half), half),
            "the exit is no way in"
        );
        assert_eq!(
            paths.walk_out(paths.exit_stand(half)).y,
            256.0 - 18.0 - WALL_CLEARANCE - 64.0
        );
    }

    /// "Both turn and flip with the Tunnel": in each of the 8 orientations the two spots lie on the side A faces,
    /// the exit towards B and the entry towards D.
    #[test]
    fn the_paths_turn_and_flip_with_the_tunnel() {
        let centre = Vec2::new(384.0, 384.0);
        for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
            for flip_h in [false, true] {
                let placement = tunnel_at(rotation, flip_h);
                let letters = sides(ObjectKind::Tunnel, placement.orientation()).unwrap();
                let facing = |letter| {
                    let (x, y) = DIRECTIONS[letters.iter().position(|&l| l == letter).unwrap()];
                    Vec2::new(x as f32, y as f32)
                };
                let paths = Paths::of(ObjectKind::Tunnel, &placement).unwrap();
                assert_eq!(paths.outward, facing(SideLetter::A), "{rotation:?} {flip_h}");
                for (spot, towards) in [(paths.exit, SideLetter::B), (paths.entry, SideLetter::D)] {
                    let from_centre = spot - centre;
                    assert_eq!(from_centre.dot(paths.outward), 128.0, "on Side A's line");
                    assert_eq!(
                        from_centre.dot(facing(towards)),
                        PATH_OFFSET_SU,
                        "{rotation:?} {flip_h}"
                    );
                }
            }
        }
        // Nothing but a Tunnel has the two paths.
        let hideout = Placement::new(TilePos::new(0, 0), ObjectKind::Hideout.def().footprint);
        assert_eq!(Paths::of(ObjectKind::Hideout, &hideout), None);
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
            ejected: false,
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
            ejected: false,
        };
        held.object.hp = Some(40.0);
        for _ in 0..160 {
            held.regenerate();
        }
        let hp = held.object.hp.unwrap();
        assert!((hp - 48.0).abs() < 1e-3, "10 s: 50% to 60% of 80, got {hp}");
        held.object.hp = Some(79.99);
        held.regenerate();
        assert_eq!(held.object.hp, Some(80.0));
    }
}
