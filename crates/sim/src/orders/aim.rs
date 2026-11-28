//! (M6-S6a) `systems/controls/command_behaviors.md > AttackingObject > Can't aim from where it stands` (design change
//! 0454, answering Q0179) and `systems/controls/command_behaviors.md > AttackingLocation` (design change 0460,
//! answering Q0184: "**Can't aim from where it stands** applies"): "A unit in range that **can't bring its weapon to
//! bear from where it stands** (e.g. a hull-aiming unit whose turn is stopped by a wall or cliff beside it)" "moves to
//! a nearby spot from which it can aim (in range, beyond MinRange, with room to turn), then aims and fires. It doesn't
//! wait in place, and doesn't give up for this reason". "How the spot is chosen is the Director's call": this module's,
//! a placeholder throughout (the numbers are `tuning::orders::Orders`'s `aim_*`). The behaviours that close in on a
//! target run it (`super::engage::close_in`, and `super::attack_ground` for a location: [`Mark::point`]); it reads the
//! tiles only, as the owner knows them ([`World::plan_grid`]): units move, and yield (design change 0035).
//!
//! - **Who can't aim from where it stands** ([`stuck`]), its mark in range. Every other unit can: infantry turns on the
//!   spot, and a turret that turns all the way round needs no turn of its base.
//!   - **A unit that aims with its hull** (`combat::turret::aims_by_hull`: the Hand of Heaven) and has its mark out of
//!     its turret's arc, when the turn that brings the mark into the arc, the shorter way (as its spin turns,
//!     `movement::tracked`), is cut by the tiles where it stands, and everywhere its spin's creep toward the mark would
//!     take it: along the way to the mark, as far as the tiles let its Silhouette go, at most the half diagonal of its
//!     Silhouette, the farthest its corners reach. A turn cut there that the creep frees isn't stuck: the spin does it
//!     (`tests/hand_of_heaven/hull.rs`, beside a wall).
//!   - **A unit that can't turn in place, with a narrow turret** (`super::engage::brakes_into_arc`: the Assault Bike,
//!     the Raptor), when its mark lies inside either of the circles it turns on at its MinimumTurnRadius, or less than
//!     the margin beyond their rims (`margin`: `Orders::aim_turn_margin_su`, and its weapon's MinRange): driving
//!     round, it never points the mark into its arc, or only right on top of it, or comes to rest inside its MinRange.
//!     Farther out it comes round onto the mark: the approach does that (step 4).
//! - **The spot** ([`spot`]): the nearest the search finds, within `Orders::aim_spot_radius_gu` of the unit, the
//!   search a step of `Orders::aim_spot_step_su` at a time.
//!   - For a unit that aims with its hull, a spot with **room to turn**: no solid tile within the half diagonal of its
//!     Silhouette, so that it turns there whatever way it faces as it arrives, from which its attack reaches the mark,
//!     facing it (in range, beyond MinRange), and a **nearby** one: the straight way there is clear of the tiles for
//!     its box as the paths plan it (`nav::los`), so that its way there is no longer than the search's reach (a spot
//!     just beyond a wall can be a long way round). On rings round the unit, the nearest first; along each, from the
//!     point toward the mark on, further and further from it both ways, of two as far the clockwise one first.
//!   - For one that can't turn in place, a spot **straight ahead** (as it drives without turning; never back: "A
//!     vehicle **reverses only on the Reverse order** (never by itself, e.g. no automatic three-point turns)",
//!     `systems/units.md > UnitCollision`), from which its mark lies outside both of its circles by the margin, and by
//!     its weapon's MinRange (`margin`: coming round at speed, it brakes straight on): the nearest first, its way there
//!     clear of the tiles; for an order's own target, out to `Orders::aim_spot_far_gu`, since all along the line the
//!     mark stays within twice its MinimumTurnRadius and the margin of it, in its sight. From there it comes round onto
//!     the mark as it closes in again.
//!   - **None near** ([`far`]): an order's own target ("**Explicit Attack**: it **keeps trying** [...] It doesn't wait
//!     in place, and doesn't give up for this reason") takes the nearest tile's centre within `Orders::aim_spot_far_gu`
//!     with room to turn, its way there however long round, from which the attack of a unit that aims with its hull
//!     reaches the mark facing it, and from which the mark of one that can't turn in place lies outside both of its
//!     circles by the margin whatever way it faces there (twice its MinimumTurnRadius off, and the margin), a spot
//!     outside the circles it turns on where it stands (its way there, round a loop, took its mark out of its sight). A
//!     unit with none there either, hemmed in, does as before: the hull turns as far as it can, the wheeled base closes
//!     in.
//! - **The leash** ([`Leash`]): for an auto-acquired target, the spot lies within the leash too ("the same
//!   repositioning, under the normal leash rules": IdleLeashDistance from the IdleOrigin, AttackMoveLeashDistance from
//!   the leg's PathReference), and near; one with no spot within it disengages, as one past its leash does.

use glam::Vec2;

use super::engage::brakes_into_arc;
use super::reach;
use super::world::{Unit, View, World};
use crate::combat::{reaches, reaches_point, turret};
use crate::defs::{AttackAttrs, MovementAttrs, TurretAttrs, UnitDef};
use crate::elevation::Elevation;
use crate::math;
use crate::mover::MoveGoal;
use crate::nav::{self, NavGrid};
use crate::ruleset::rules;
use crate::shape::{Shape, bounding_half};
use crate::units::{SU_PER_GU, TilePos, gu};

/// What a unit aims at ([`spot`]): an object, as its owner sees it, or (design change 0460) an Attack Ground's
/// location.
#[derive(Copy, Clone, Debug)]
pub(super) struct Mark {
    /// Where the unit aims: the object's centre, or the location.
    pos: Vec2,
    /// The object's shape; `None` for a location.
    shape: Option<Shape>,
    elevation: Elevation,
}

impl Mark {
    pub(super) fn of(view: &View) -> Self {
        Self {
            pos: view.pos,
            shape: Some(view.shape),
            elevation: view.elevation,
        }
    }

    /// An Attack Ground's location `at`, its ground at `ground`.
    pub(super) fn point(at: Vec2, ground: Elevation) -> Self {
        Self {
            pos: at,
            shape: None,
            elevation: ground,
        }
    }

    /// Whether an attack of `attrs` reaches it from the shape `from`, at `source`: in range, beyond MinRange
    /// (`combat::reaches`; `combat::reaches_point` for a location, measured from the attacker's edge to the point).
    fn reached(&self, attrs: &AttackAttrs, from: &Shape, source: Elevation) -> bool {
        match &self.shape {
            Some(shape) => reaches(attrs, from, source, shape, self.elevation),
            None => reaches_point(attrs, from, source, self.pos, self.elevation),
        }
    }
}

/// Where a spot for an auto-acquired target must lie (module docs): an order's own target has no leash.
#[derive(Copy, Clone, Debug)]
pub(super) enum Leash<'a> {
    /// An Attack's, or an Attack Ground's: anywhere.
    Free,
    /// An idle unit's (`systems/controls/behaviors.md > BaseAutoTargeting`): within IdleLeashDistance of its
    /// IdleOrigin, this point.
    Idle(Vec2),
    /// An AttackMove's or a Patrol's leg's (`systems/controls/command_behaviors.md > AttackMovingToLocation`): within
    /// AttackMoveLeashDistance of its PathReference (`super::reach::within_leash`).
    Route(&'a [Vec2]),
}

impl Leash<'_> {
    /// Whether `spot` lies within it.
    fn allows(&self, spot: Vec2) -> bool {
        match *self {
            Leash::Free => true,
            Leash::Idle(origin) => spot.distance(origin) <= gu(rules().tuning.orders.idle_leash_gu),
            Leash::Route(reference) => reach::within_leash(spot, reference),
        }
    }

    /// Whether a unit with no spot within it disengages (module docs): it is a leash.
    pub(super) fn disengages(&self) -> bool {
        !matches!(self, Leash::Free)
    }
}

/// Whether `unit` can't bring its weapon, `attrs`, to bear from where it stands on its mark at `at`, which its attack
/// reaches (module docs). For a unit that aims with its hull, only while `at` is out of its turret's arc.
pub(super) fn stuck(unit: &Unit, attrs: &AttackAttrs, at: Vec2, world: &World) -> bool {
    let Some(def) = unit.unit_def() else {
        return false;
    };
    if turret::aims_by_hull(def) {
        return !unit.in_arc(at)
            && world
                .plan_grid(unit)
                .is_some_and(|grid| !turns_on_the_spot(unit, def, at, grid));
    }
    brakes_into_arc(def).is_some() && inside_its_circles(def, attrs, unit.pos, unit.facing, at)
}

/// The spot `unit` repositions to so as to bring its weapon to bear on `mark`, attacking with `attrs`, within `leash`
/// (module docs): the nearest near it, else, for an order's own target, the nearest farther ([`far`]); `None` if the
/// search finds none.
pub(super) fn spot(unit: &Unit, attrs: &AttackAttrs, mark: Mark, leash: Leash, world: &World) -> Option<Vec2> {
    let def = unit.unit_def()?;
    let grid = world.plan_grid(unit)?;
    let home = grid.component(grid.tile_of(unit.pos))?;
    let reachable = |spot: Vec2| grid.component(grid.tile_of(spot)) == Some(home);
    let farther = |aims: &dyn Fn(Vec2) -> bool| far(unit, grid, leash, |spot| reachable(spot) && aims(spot));
    if turret::aims_by_hull(def) {
        let half = unit.kind.def().half_extents()?;
        // Its attack reaches the mark from there, facing it.
        let aims = |spot: Vec2| {
            let silhouette = Shape::silhouette(unit.kind, spot, math::vector_angle(mark.pos - spot));
            mark.reached(attrs, &silhouette, Elevation::of(world.map, unit.kind, spot))
        };
        let fits = |spot: Vec2| {
            leash.allows(spot)
                && aims(spot)
                && room_to_turn(grid, spot, half.length())
                && reachable(spot)
                && nav::los(grid, half, unit.pos, spot)
        };
        return rings(unit.pos, mark.pos)
            .find(|&spot| fits(spot))
            .or_else(|| farther(&aims));
    }
    brakes_into_arc(def)?;
    let ahead = math::facing_vector(unit.facing);
    let half = bounding_half(unit.kind.def().half_extents()?, ahead);
    let fits = |spot: Vec2| {
        !inside_its_circles(def, attrs, spot, unit.facing, mark.pos)
            && leash.allows(spot)
            && reachable(spot)
            && nav::los(grid, half, unit.pos, spot)
    };
    let tuning = &rules().tuning.orders;
    // An order's own target: on along the line as far as the search goes farther (module docs).
    let reach = if leash.disengages() {
        tuning.aim_spot_radius_gu
    } else {
        tuning.aim_spot_far_gu
    };
    let steps = (gu(reach) / tuning.aim_spot_step_su).floor() as u32;
    let line = (1..=steps)
        .map(|n| unit.pos + ahead * (tuning.aim_spot_step_su * n as f32))
        .find(|&spot| fits(spot));
    line.or_else(|| {
        farther(&|spot| {
            beyond_its_circles(def, attrs, spot, mark.pos) && !near_its_circles(def, unit.pos, unit.facing, spot, 0.0)
        })
    })
}

/// With no spot near, the spot for an order's own target (module docs; none for an auto-acquired target's, `leash`):
/// the nearest tile's centre within `Orders::aim_spot_far_gu` of `unit` with room to turn for which `fits` holds, its
/// way there however long; of two as near, the first by row, then column.
fn far(unit: &Unit, grid: &NavGrid, leash: Leash, fits: impl Fn(Vec2) -> bool) -> Option<Vec2> {
    if leash.disengages() {
        return None;
    }
    let reach = gu(rules().tuning.orders.aim_spot_far_gu);
    let room = unit.kind.def().half_extents()?.length();
    let (at, tiles) = (TilePos::containing(unit.pos), (reach / SU_PER_GU).ceil() as i32);
    (-tiles..=tiles)
        .flat_map(|dy| (-tiles..=tiles).map(move |dx| TilePos::new(at.x + dx, at.y + dy).center()))
        .filter(|&spot| spot.distance(unit.pos) <= reach && fits(spot) && room_to_turn(grid, spot, room))
        .min_by(|a, b| a.distance_squared(unit.pos).total_cmp(&b.distance_squared(unit.pos)))
}

/// The unit sets off for the spot of [`spot`], driving forward (module docs).
pub(super) fn go(unit: &mut Unit, spot: Vec2, world: &World) {
    unit.mover.go_to(MoveGoal::Point(spot), None, world.tick);
}

/// The points of the search round `from` for a unit aiming at `toward` ([`spot`]), the nearest first: on rings
/// `Orders::aim_spot_step_su` apart out to `Orders::aim_spot_radius_gu`, along each from the point toward `toward` on,
/// further and further from it both ways at most that step apart, the clockwise one first (the angle growing, with `y`
/// pointing down the map).
fn rings(from: Vec2, toward: Vec2) -> impl Iterator<Item = Vec2> {
    let tuning = &rules().tuning.orders;
    let step = tuning.aim_spot_step_su;
    let bearing = math::vector_angle(toward - from);
    let count = (gu(tuning.aim_spot_radius_gu) / step).floor() as u32;
    (1..=count).flat_map(move |ring| {
        let radius = step * ring as f32;
        // An even number of points, at most a step apart along the ring.
        let points = ((math::TAU * radius / step / 2.0).ceil() as u32).max(4) * 2;
        let apart = math::TAU / points as f32;
        (0..points).map(move |i| {
            let way = if i % 2 == 1 { 1.0 } else { -1.0 };
            let angle = bearing + way * apart * i.div_ceil(2) as f32;
            from + math::facing_vector(angle) * radius
        })
    })
}

/// Whether `unit`, a unit that aims with its hull, brings `at` into its turret's arc turning on the spot, as far as
/// the tiles of `grid` let it (module docs): where it stands, or along its creep toward `at`.
fn turns_on_the_spot(unit: &Unit, def: &UnitDef, at: Vec2, grid: &NavGrid) -> bool {
    let Some(attrs) = def.turret else {
        return true;
    };
    let reach = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO).length();
    let way = (at - unit.pos).normalize_or_zero();
    let try_su = rules().tuning.movement.turn_try_su;
    let creeps = (reach / try_su).ceil() as u32;
    for n in 0..=creeps {
        let from = unit.pos + way * (try_su * n as f32).min(reach);
        if n > 0 && !Shape::silhouette(unit.kind, from, unit.facing).clear_on(grid) {
            return false;
        }
        if turns_clear(unit, &attrs, from, at, grid) {
            return true;
        }
    }
    false
}

/// Whether a unit of `unit`'s kind at `from`, facing as `unit` does, turns the shorter way until `at` is within the arc
/// of its turret, `attrs`, its Silhouette clear of the tiles of `grid` all the way: tried in parts along which its
/// corners move at most `Movement::turn_try_su`, as the movement step tries a turn (`movement::collide::turn`).
fn turns_clear(unit: &Unit, attrs: &TurretAttrs, from: Vec2, at: Vec2, grid: &NavGrid) -> bool {
    let to = at - from;
    if to == Vec2::ZERO {
        return true;
    }
    let off = math::wrap_angle(math::vector_angle(to) - unit.facing);
    let need = off.abs() - attrs.turn_angle / 2.0;
    if need <= 0.0 {
        return true;
    }
    let corner = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO).length();
    let parts = ((need * corner / rules().tuning.movement.turn_try_su).ceil() as u32).max(1);
    let turn = need.copysign(off);
    (1..=parts).all(|part| {
        let facing = unit.facing + turn * part as f32 / parts as f32;
        Shape::silhouette(unit.kind, from, facing).clear_on(grid)
    })
}

/// Whether `at` lies inside either circle a unit of `def` at `pos`, facing `facing`, turns on, or less than the
/// [`margin`] of its weapon, `attrs`, beyond their rims (module docs). False for a unit that turns otherwise.
fn inside_its_circles(def: &UnitDef, attrs: &AttackAttrs, pos: Vec2, facing: f32, at: Vec2) -> bool {
    near_its_circles(def, pos, facing, at, margin(attrs))
}

/// Whether `at` lies inside either of the circles a unit of `def` at `pos`, facing `facing`, turns on, or less than
/// `margin` beyond its rim. False for a unit that turns otherwise.
fn near_its_circles(def: &UnitDef, pos: Vec2, facing: f32, at: Vec2, margin: f32) -> bool {
    let Some(radius) = turn_radius(def) else {
        return false;
    };
    let side = math::facing_vector(facing).perp() * radius;
    [pos + side, pos - side]
        .iter()
        .any(|centre| centre.distance(at) < radius + margin)
}

/// Whether `at` lies outside both circles a unit of `def` at `pos` turns on, by the [`margin`] of its weapon, `attrs`,
/// whatever way it faces there: twice its MinimumTurnRadius off, and the margin ([`far`]). True for a unit that turns
/// otherwise.
fn beyond_its_circles(def: &UnitDef, attrs: &AttackAttrs, pos: Vec2, at: Vec2) -> bool {
    turn_radius(def).is_none_or(|radius| pos.distance(at) >= 2.0 * radius + margin(attrs))
}

/// (M6-S6a) How far beyond the rims of the circles it turns on a unit that can't turn in place needs its mark to come
/// round onto it with its weapon, `attrs`: `Orders::aim_turn_margin_su`, and the weapon's MinRange (design change 0460:
/// the Assault Bike's 1 grid unit). **Placeholder (not in the design: how a vehicle comes round onto a mark beyond its
/// MinRange)**: coming round at speed, it brakes straight on, up to 78 su from the Bike's top speed, and from a mark
/// only the turn margin out it came to rest inside its MinRange, and backed off.
fn margin(attrs: &AttackAttrs) -> f32 {
    rules().tuning.orders.aim_turn_margin_su + gu(attrs.min_range)
}

/// The MinimumTurnRadius of a unit of `def` on FixedTurnRadiusMovement, which no Augmentation changes; `None` for a
/// unit that turns otherwise.
fn turn_radius(def: &UnitDef) -> Option<f32> {
    match def.movement {
        MovementAttrs::FixedTurnRadius {
            minimum_turn_radius, ..
        } => Some(minimum_turn_radius),
        _ => None,
    }
}

/// Whether no solid tile of `grid` lies within `room` of `spot`, and it all lies on the map: a unit there turns
/// whatever way it faces ([`spot`]).
fn room_to_turn(grid: &NavGrid, spot: Vec2, room: f32) -> bool {
    let (first, last) = (TilePos::containing(spot - room), TilePos::containing(spot + room));
    (first.y..=last.y).all(|y| {
        (first.x..=last.x).all(|x| {
            let tile = TilePos::new(x, y);
            let corner = tile.corner();
            let apart = (corner - spot).max(spot - (corner + SU_PER_GU)).max(Vec2::ZERO);
            !grid.is_solid(tile) || apart.length() >= room
        })
    })
}

#[cfg(test)]
mod tests;
