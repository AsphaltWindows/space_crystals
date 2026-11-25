//! The Tunnel Network — `factions/syndicate/tunnels.md > Tunnel Network`, `> Tunnel > SymmetryType - ABCD` (Side A's
//! paths), `> EjectMenu`; `systems/controls/commands.md > Enter`; slice M4-C. "The Tunnel Network is the collective of
//! all Tunnels owned by a Syndicate player. Units inside the Tunnel Network can travel between Tunnels freely, but can
//! only enter or exit the network through a Tunnel whose tier is sufficient for their unit base category."
//!
//! - **Units off the map** (plan decision T30): a unit inside the network is not among the sim's objects. It is
//!   [`Held`] in one store beside them (`Sim::held`), with its id, at a [`Place`]. So every system that walks the
//!   objects (movement, collision, vision, targeting, picking, records) passes it by without a filter, and a
//!   forgotten case can't show, move or hit a unit that isn't there. What must count the held units says so:
//!   Tunnel Space (`Sim::army_cap`), the invariants (`testing::invariants`), the hash and the saves, the client's
//!   control groups.
//! - **Inside** (design change 0249): the unit is inert. Its orders are cleared, "queued ones included", its move and
//!   its attack end; an Agent "keeps its load". It regenerates [`REGEN_PER_SECOND`] of its MaxHP ([`Held::regenerate`]).
//! - **The transit delay** (design changes 0245 and 0246): a unit that walked in "can exit only once"
//!   [`TRANSIT_DELAY_TICKS`] have passed, from any Tunnel; a unit produced inside has none ([`Held::may_exit`]).
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

use crate::defs::{ObjectKind, Transit};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::{Object, Placement};
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::tunnel;
use crate::units::{SU_PER_GU, TICKS_PER_SECOND, Tick};

/// `factions/syndicate/tunnels.md > Tunnel Network > Transit delay` (design change 0245): "after a unit **enters** the
/// network, there is a **small delay before it can exit**: **2 seconds (32 frames), placeholder**". Flat, "not by
/// distance".
pub const TRANSIT_DELAY_TICKS: Tick = 32;

/// `> Tunnel Network > Regeneration inside` (design change 0249): "every unit inside the Tunnel Network slowly
/// regenerates HP: **1% of its MaxHP per second, placeholder**".
pub const REGEN_PER_SECOND: f32 = 0.01;

/// How far Side A's paths lie from the middle of the side, space units: the exit path towards Side B, the entry path
/// towards Side D (design change 0248). The middles of the two paths the client draws (`object_render::tunnel`, which
/// reads this), 60 su apart, so that a unit up to 60 su wide on one never touches one on the other ("entering and
/// exiting units never block each other"). **Placeholder**: the design gives the halves, not where in them (question
/// Q0063 point 2 proposed "centred on its half"; the drawing of slice M4-A came first).
pub const PATH_OFFSET_SU: f32 = 30.0;

/// Question Q0063 point 2, as proposed (design change 0247: "As proposed"): "A unit goes in when its silhouette is
/// within 8 su of the entry spot (DropOffReach's value, as a placeholder)".
pub const ENTRY_REACH_SU: f32 = 8.0;

/// `> EjectMenu` (design change 0250): "With no rally point, a unit walks about 1 grid unit straight out of Side A and
/// idles".
pub const WALK_OUT_SU: f32 = SU_PER_GU;

/// How far from the Tunnel's wall a unit stands on a path's spot, space units, so that no rounding makes its box
/// overlap the footprint (a technical margin: a sixty-fourth of a space unit, as a doorway's).
const WALL_CLEARANCE: f32 = 1.0 / 64.0;

/// Whether air units pass through a Tunnel: **not in M4** (the Director's decision for slice M4-C). No Syndicate air
/// unit exists yet (the Phantom is deferred, the Specter a concept), so Enter and Eject refuse every air unit at every
/// tier, Tier 3 included.
///
/// **The rule to come with the first Syndicate air unit** (design changes 0250 and 0251): "Air units leave through
/// the Tier 3 air exit door, in parallel with ground units; since aircraft don't collide, the door has its own exit
/// rate: one aircraft per 8 frames (0.5 s), placeholder (the same rate for the air entry door)"; "the air doors are on
/// the Tunnel's roof: the entry door over the entry path's half of the Tunnel, the exit door over the exit path's
/// half". They never use Side A's paths, and never wait for the exit spot.
pub const AIR_DOORS: bool = false;

/// Where a held unit is (plan decision T30). Match it exhaustively, and never assume a held unit's place is on the
/// map: slice M4-E adds the site an Agent builds from inside, and a transport's passengers (design changes 0221 and
/// 0222, deferred) will be held at their carrier, which may itself be held.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
    /// In its owner's Tunnel Network.
    Network,
}

/// A unit off the map (plan decision T30, module docs): the object, with its id, where it is and since when.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Held {
    /// The unit, as it left the map but for what entering clears (module docs): its `pos` is where it stood last,
    /// and means nothing while it is held.
    pub object: Object,
    pub place: Place,
    /// The tick it came to its place: `> EjectMenu`'s "the one that has been inside the network longest" comes out
    /// first (design change 0251).
    pub since: Tick,
    /// Whether it walked in (design change 0245: "Each unit inside stores when it entered"), so that the transit
    /// delay applies from `since`; `false` for a unit produced inside ("no delay", 0246).
    pub entered: bool,
}

impl Held {
    /// Whether the unit may leave the network in `tick` (design changes 0245 and 0246): at once if it was produced
    /// inside, [`TRANSIT_DELAY_TICKS`] after it walked in otherwise.
    pub fn may_exit(&self, tick: Tick) -> bool {
        !self.entered || tick >= self.since.saturating_add(TRANSIT_DELAY_TICKS)
    }

    /// One tick of `> Regeneration inside` (design change 0249): [`REGEN_PER_SECOND`] of its MaxHP a second, up to its
    /// MaxHP. Only "inside the Tunnel Network": a unit held elsewhere regenerates nothing.
    pub(crate) fn regenerate(&mut self) {
        match self.place {
            Place::Network => {}
        }
        let (Some(hp), Some(max)) = (self.object.hp, self.object.max_hp()) else {
            return;
        };
        if hp < max {
            let gain = max * (REGEN_PER_SECOND / TICKS_PER_SECOND as f32);
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

    /// Where a unit with half extents `half` stands on `spot` (a path's): "just outside Side A", its box against
    /// the wall.
    fn stand(&self, spot: Vec2, half: Vec2) -> Vec2 {
        spot + self.outward * (self.outward.abs().dot(half) + WALL_CLEARANCE)
    }

    /// Where an entering unit with half extents `half` walks to: on the entry path's spot, against the wall.
    pub fn entry_stand(&self, half: Vec2) -> Vec2 {
        self.stand(self.entry, half)
    }

    /// Where an ejected unit with half extents `half` appears (design change 0250: "Each unit appears at the exit
    /// path's spot just outside Side A"): on the exit path's spot, against the wall.
    pub fn exit_stand(&self, half: Vec2) -> Vec2 {
        self.stand(self.exit, half)
    }

    /// Whether a unit with half extents `half` at `pos` has walked into the entry path: its box is within
    /// [`ENTRY_REACH_SU`] of the entry spot.
    pub fn reached_entry(&self, pos: Vec2, half: Vec2) -> bool {
        let apart = (pos - self.entry).abs() - half;
        apart.max(Vec2::ZERO).length() <= ENTRY_REACH_SU
    }

    /// Where a unit that appeared at `stand` with no order walks to: [`WALK_OUT_SU`] "straight out of Side A".
    pub fn walk_out(&self, stand: Vec2) -> Vec2 {
        stand + self.outward * WALK_OUT_SU
    }
}

/// Why a Tunnel doesn't let a unit in or out ([`admits`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NoTransit {
    /// The object isn't a Tunnel, or isn't one that works yet.
    NotATunnel,
    /// The kind isn't a unit.
    NotAUnit,
    /// Its tier's Transit doesn't list the unit's base (`> Transit Tier Requirements`).
    Tier,
    /// An air unit: no air doors in M4 ([`AIR_DOORS`]).
    Air,
}

/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: whether `tunnel` lets a unit of
/// `kind` into or out of the network. It is a Tunnel in operation ([`is_exit`]) whose tier's Transit lists the unit's
/// base: "Tier 1+: Infantry", "Tier 2+: Vehicles", "Tier 3+: Air units", the last not in M4 ([`AIR_DOORS`]). The tier
/// is the one the Tunnel has (`TunnelState::tier`), which an upgrade under way leaves alone until it completes (design
/// change 0249: "until the upgrade has fully finished, a Tunnel only accepts, and only lets out, units supported by
/// the tier it had before the upgrade began").
pub fn admits(tunnel: &Object, kind: ObjectKind) -> Result<(), NoTransit> {
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

/// A Tunnel's Tunnel Area (`factions/syndicate/tunnels.md > Tunnel Area`): "a square underground build zone centered
/// on the Tunnel. The radius extends outward from the Tunnel's 4x4 footprint in each direction", at its tier's Tunnel
/// Area Radius. As a footprint, which may reach off the map. `None` for anything but a placed Tunnel.
pub fn area(tunnel: &Object) -> Option<Placement> {
    let radius = tunnel::tier_def(tunnel)?.area_radius;
    let placement = tunnel.placement.as_ref()?;
    let reach = i32::try_from(radius).ok()?;
    Some(Placement {
        origin: crate::units::TilePos::new(placement.origin.x - reach, placement.origin.y - reach),
        w: placement.w + 2 * radius,
        h: placement.h + 2 * radius,
        ..*placement
    })
}

/// The parent Tunnel of `expansion`, an underground structure (plan decision T32: derived, never stored): "the
/// owner's Tunnel whose current area fully contains it", the first by id of `tunnels` (an owner's Tunnel Areas never
/// overlap, so there is at most one). `None` for an expansion without an owner or outside every area of its owner's
/// Tunnels in operation.
pub fn parent<'a>(expansion: &Object, tunnels: impl IntoIterator<Item = &'a Object>) -> Option<ObjectId> {
    let owner = expansion.owner?;
    let inside = expansion.placement.as_ref()?;
    let contains = |area: &Placement| {
        let (x0, y0) = (i64::from(area.origin.x), i64::from(area.origin.y));
        let (x1, y1) = (x0 + i64::from(area.w), y0 + i64::from(area.h));
        let (ix, iy) = (i64::from(inside.origin.x), i64::from(inside.origin.y));
        x0 <= ix && y0 <= iy && ix + i64::from(inside.w) <= x1 && iy + i64::from(inside.h) <= y1
    };
    tunnels
        .into_iter()
        .find(|t| t.owner == Some(owner) && is_exit(t) && area(t).is_some_and(|area| contains(&area)))
        .map(|t| t.id)
}

/// Whether `tunnel` is a way into and out of the network: a Tunnel with an owner. (`> Losing Tunnels`: "A Tunnel that
/// is still under construction does not count as an exit": slice M4-E's sites answer `false` here.)
pub fn is_exit(tunnel: &Object) -> bool {
    tunnel.tunnel.is_some() && tunnel.placement.is_some() && tunnel.owner.is_some()
}

/// One entry of a Tunnel's eject queue (`TunnelState::eject`; `> EjectMenu`, design change 0251: "each click reserves
/// a specific unit [...]; the queue can't be cancelled").
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eject {
    /// The reserved unit, held in the Tunnel owner's network.
    pub unit: ObjectId,
    /// The order it gets as it comes out: a produced unit's default command on its producer's rally point (design
    /// changes 0136 and 0253; `factions/syndicate/structures.md > Rally Point Behavior`). `None` for a player's Eject,
    /// and for a rally order whose target left the game meanwhile: the unit then walks straight out and idles.
    pub order: Option<crate::orders::OrderKind>,
}

impl StateHash for Place {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Place::Network => h.tag(0),
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
        } = self;
        object.hash(h);
        place.hash(h);
        since.hash(h);
        entered.hash(h);
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
