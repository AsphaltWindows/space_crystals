//! The spatial grids and the box geometry of the movement step — `docs/movement.md > 6.1`, `docs/movement.md > 3`
//! (collision shape). The grid of a layer is [`Layer`] (`space/layer.rs`, split out with the kinds of body it lists,
//! plan decision T28).
//!
//! Everything here is **scratch**: rebuilt at the start of every phase 5d from the objects, so nothing carries over
//! between ticks and nothing is hashed or saved (`docs/movement.md > 3`).
//!
//! Float rules (`docs/movement.md > 7`): the one overlap predicate is [`overlaps`], strict, so touching is free. The
//! sweep works on the same differences (`a − b` against `half_a + half_b`) as the predicate, so a pair the predicate
//! calls apart is never "already inside" for the sweep. Contact positions are assigned ([`resting`]), not
//! accumulated.
//!
//! **(M5-V1) Oriented Silhouettes** (plan decision T78; `systems/units.md > UnitCollision > Silhouette Rotation`): a
//! body whose Silhouette turns with its facing ([`Body::oriented`]) collides with its turned shape
//! (`shape::Shape`): [`overlaps`] and [`contact`] take the separating-axis test for any pair with such a body in it,
//! "a square box against an oriented one" too, and the box test of two boxes as before. Its [`Body::half`] is the
//! bounding box of its turned shape: what the grids list it under, what the tiles are tested against first (where
//! that box isn't clear of them, the shape itself: `shape::Shape::clear_on`), and what the heuristics that read a box
//! (lanes, the contact step, yielding, a way round a row) see. Its contacts are found by
//! the sweep of the two shapes ([`contact`]); where the sweep's contact rests is the halving of `World::up_to`, not
//! [`resting`], whose exact values are the boxes'.

use glam::Vec2;

use super::crush;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::paths::boxes_overlap;
use crate::shape::{self, Shape};
use crate::units::Tick;

mod layer;

pub(crate) use layer::{KIND_CRUSHABLE, KIND_YIELDS, Layer, kinds_against};

/// A unit as the collision step sees it: a copy, so the step needs no second borrow of the object map
/// (`docs/movement.md > 4`). Units already moved this tick are listed at their new position, the rest at their old
/// one.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Body {
    pub id: ObjectId,
    /// For friendly yielding (`super::yielding`) and the exceptions of [`blocks`] (crushing spares own and allied
    /// units).
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    /// The half extents of its box: half its Silhouette, or (M5-V1) for one that turns with its facing
    /// ([`Body::oriented`]) of the bounding box of the turned Silhouette (`shape::bounding_half`).
    pub half: Vec2,
    /// (M5-V1) For a unit whose Silhouette turns with its facing (`defs::ObjectDef::silhouette_turns`): how it lies.
    /// `None` for a square one, whose shape is its box, `half`.
    pub oriented: Option<Oriented>,
    /// Its latest displacement: this tick's if it already moved, else last tick's.
    pub velocity: Vec2,
    /// (G) Where it is heading: the vector from it to its steering waypoint (any length), zero for a unit without
    /// a goal or waiting for its path. Unlike `velocity` it doesn't turn with a step around a blocker, so it tells
    /// an oncoming unit from one that only steps aside (`super::collide::World::lane`, [`Body::heads_against`]).
    pub heading: Vec2,
    /// `Mover::is_stationary`.
    pub stationary: bool,
    /// For a unit on its way (it has a goal): the tick of its latest progress, `Mover::last_progress`. Units
    /// waiting behind it share it (`docs/movement.md > 6.5`).
    pub progress: Option<Tick>,
    /// For a unit whose latest move ended `Arrived`: that move's order group (`Mover::group`).
    pub arrived_with: Option<u32>,
    /// Whether it steps aside for a blocked unit of its owner (`super::yielding::can_yield`, design changes 0022
    /// and 0023).
    pub yields: bool,
    /// Whether it is a Drone on a Gather job (`orders::gather::on_gather_job`, design change 0106): two such
    /// bodies don't collide ([`blocks`]).
    pub gathers: bool,
    /// (M3-H2) Whether those waiting behind it share its `progress` (`super::stuck::leads`): it has got on since
    /// its leg began.
    pub leads: bool,
    /// (M3-G2) `Mover::last_progress`, whether or not the unit is on its way: for one that stands, about when it
    /// got there (its last leg's latest progress, or the tick that leg began). A unit queueing behind it reads
    /// from it whether it came to stand in its way since its last attempt at a way round (`super::stuck`).
    pub moved: Tick,
    /// (M5-V4) Whether its move this tick crushes (`super::crush::drives`): set on the moving unit's own body for its
    /// turn alone, false on every other body, as [`blocks`] reads only the mover's.
    pub crushes: bool,
    /// (M5-V4) `UnitBaseDef::crushable`: a crusher's move goes through it if it is an enemy's ([`blocks`]).
    pub crushable: bool,
}

/// (M5-V1) How a Silhouette that turns with its unit's facing lies (`systems/units.md > UnitCollision > Silhouette
/// Rotation`): its length along the facing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Oriented {
    /// Half its length and half its width (`defs::ObjectDef::half_extents`).
    pub half: Vec2,
    /// The facing it lies at, radians.
    pub facing: f32,
    /// That facing as a unit vector (`math::facing_vector`): the direction of its length.
    pub axis: Vec2,
}

impl Oriented {
    /// A Silhouette with half extents `half` turned to `facing` (radians).
    pub(crate) fn facing(half: Vec2, facing: f32) -> Self {
        Self {
            half,
            facing,
            axis: math::facing_vector(facing),
        }
    }

    /// The half extents of its bounding box (`shape::bounding_half`): [`Body::half`].
    pub(crate) fn bound_half(&self) -> Vec2 {
        shape::bounding_half(self.half, self.axis)
    }
}

/// `docs/movement.md > 6.1`: whether `other` is an obstacle for `mover`, both of one layer, for the move `mover`
/// makes this tick from where it stands (`mover.pos`). **The single place for collision exceptions.** Every other
/// unit of the layer blocks, friend or enemy, except:
///
/// 1. **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`,
///    design change 0106): two Drones on a Gather job ([`Body::gathers`]) pass through each other, of any player.
/// 2. **Units that already overlap** (`systems/units.md > Units that already overlap`, design change 0106): two units
///    whose boxes overlap as the mover's turn begins "ignore each other until they have separated; from then on, normal
///    collision applies between them. Nothing is pushed apart." It is read off the two boxes each tick and no pair
///    is remembered: [`overlaps`] is the predicate of invariant I1, so a pair is ignored exactly while I1 would
///    name it. Only rule 1 ever lets two boxes come to overlap, so the pairs this rule covers are those rule 1 left
///    behind (a Drone whose job ended inside another) and they only ever part. While it lasts, either may move
///    freely through the other, deeper in too. An idle one moves off by itself (design change 0126,
///    `super::separate`); nothing moves a unit that has a command.
///
/// 3. **(M5-V4) Crushing** (`systems/units.md > UnitCollision > Crushing`, plan decision T86 with design change
///    0418): a crusher whose move this tick crushes ([`Body::crushes`]: Moving or Reversing, or turning in place)
///    passes through the crushable units of its enemies ([`Body::crushable`]): "crushers may path and drive through
///    crushable enemies — those enemies are not obstacles to them", and a spin's crushable enemies "don't block the
///    spin". Those it overlaps or sweeps die in the same tick (`super::crush`). Every other pair collides as before: a
///    crusher that stands and doesn't turn ("a normal hard obstacle"), own and allied crushables ("No friendly
///    crushing"), and a crushable moving into a crusher ("Crushable units avoid crushers"): only the mover's flag
///    counts.
///
/// A unit ignored for a move is ignored for the whole of it: the sweeps, the lanes and [`overlaps`] at the
/// position it commits all ask here, and (M5-V4) so does a turn's every part (`super::collide::World::turn`). So a
/// move never brings two units to overlap unless one of the rules holds between them at its start, and under rule 3
/// the crushable doesn't outlive the tick.
pub(crate) fn blocks(mover: &Body, other: &Body) -> bool {
    mover.id != other.id
        && !(mover.gathers && other.gathers)
        && !crush::crushes(mover, other)
        && !overlaps(mover, mover.pos, other)
}

impl Body {
    /// (M5-V1) Its shape with its centre at `pos`: its Silhouette turned as it lies ([`Body::oriented`]), else its box.
    pub(crate) fn shape_at(&self, pos: Vec2) -> Shape {
        match self.oriented {
            Some(oriented) => Shape::oriented(pos, oriented.half, oriented.axis),
            None => Shape::rect(pos, self.half),
        }
    }

    /// (M5-V1) The body turned to lie as `oriented` (a Silhouette that turns with its facing): its box follows.
    pub(crate) fn turned(&self, oriented: Oriented) -> Self {
        Self {
            half: oriented.bound_half(),
            oriented: Some(oriented),
            ..*self
        }
    }
}

/// Where a moving box first meets a standing one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Contact {
    /// The part of the displacement travelled before contact, `0..1`.
    pub t: f32,
    /// The axis the boxes meet on: 0 for x, 1 for y. (M5-V1) For two shapes of which one turns with its facing, the
    /// one nearest to the edge direction they meet on ([`contact`]).
    pub axis: usize,
    /// (M5-V1) The edge direction two shapes meet on, a unit vector, if one of them turns with its facing: the face
    /// they meet on is then no axis. `None` for two boxes.
    pub normal: Option<Vec2>,
}

impl Contact {
    /// The face the two meet on ([`Face`]).
    pub(crate) fn face(&self) -> Face {
        Face {
            axis: self.axis,
            normal: self.normal,
        }
    }
}

/// (M5-V1) The face a moving unit met (`Contact::face`), for the rules that go round it
/// (`super::collide::World::around`): the parts of a vector straight at it and along it. For two boxes the face lies
/// along an axis, and the parts are a vector's coordinates, exactly as those rules always took them; for a shape that
/// turns with its facing, the projections on the edge direction they meet on and on the direction across it.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Face {
    axis: usize,
    normal: Option<Vec2>,
}

impl Face {
    /// The part of `v` straight at the face.
    pub(crate) fn at(&self, v: Vec2) -> f32 {
        match self.normal {
            None => v[self.axis],
            Some(n) => v.dot(n),
        }
    }

    /// The part of `v` along the face.
    pub(crate) fn along(&self, v: Vec2) -> f32 {
        match self.normal {
            None => v[1 - self.axis],
            Some(n) => v.dot(Vec2::new(-n.y, n.x)),
        }
    }

    /// The vector of length `s` straight at the face (signed).
    pub(crate) fn at_by(&self, s: f32) -> Vec2 {
        match self.normal {
            None => {
                let mut v = Vec2::ZERO;
                v[self.axis] = s;
                v
            }
            Some(n) => n * s,
        }
    }

    /// The vector of length `s` along the face (signed).
    pub(crate) fn along_by(&self, s: f32) -> Vec2 {
        match self.normal {
            None => {
                let mut v = Vec2::ZERO;
                v[1 - self.axis] = s;
                v
            }
            Some(n) => Vec2::new(-n.y, n.x) * s,
        }
    }
}

/// `docs/movement.md > 6.3`: sweeps the box at `pos` along `d` against the standing box at `other`; `reach` is the
/// sum of the two half extents. `None` if they never overlap on the way. Touching is free: a box sliding along
/// another, or leaving it, isn't in contact.
///
/// The slab test of the segment `pos → pos + d` against the box `other ± reach`, on the differences the overlap
/// predicate uses.
pub(crate) fn sweep(pos: Vec2, d: Vec2, other: Vec2, reach: Vec2) -> Option<Contact> {
    let r = pos - other;
    let (mut t_in, mut t_out) = (0.0_f32, 1.0_f32);
    // When each axis is entered; an axis without motion is never "entered".
    let mut entry = [f32::NEG_INFINITY; 2];
    for axis in 0..2 {
        let (r, d, reach) = (r[axis], d[axis], reach[axis]);
        if d == 0.0 {
            if r.abs() >= reach {
                return None;
            }
            continue;
        }
        let (t0, t1) = ((-reach - r) / d, (reach - r) / d);
        let (near, far) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
        t_in = t_in.max(near);
        t_out = t_out.min(far);
        entry[axis] = near;
    }
    if d == Vec2::ZERO || t_in >= t_out {
        return None;
    }
    // The contact axis is the one entered last. When both are entered at once (a corner), it is the one the box
    // moves less along, so that sliding keeps the larger part of the motion.
    let axis = if entry[0] > entry[1] {
        0
    } else if entry[1] > entry[0] {
        1
    } else {
        usize::from(d.y.abs() < d.x.abs())
    };
    Some(Contact {
        t: t_in,
        axis,
        normal: None,
    })
}

/// The coordinate at which a box rests against another on one axis (`docs/movement.md > 3`: contact positions are
/// assigned): `other ∓ reach`, on the low side of `other` if `low_side`. If rounding leaves that value inside
/// `reach` by the overlap predicate's own arithmetic, it is moved away float by float.
pub(crate) fn resting(other: f32, reach: f32, low_side: bool) -> f32 {
    let mut at = if low_side { other - reach } else { other + reach };
    for _ in 0..4 {
        if (at - other).abs() >= reach {
            break;
        }
        at = if low_side { at.next_down() } else { at.next_up() };
    }
    at
}

/// Whether `me`'s box at `pos` overlaps `other`'s where it is (`docs/movement.md > 3`): the predicate of invariant I1.
/// (M5-V1) Their shapes', if either turns with its facing (`Shape::overlaps`, the separating-axis test).
pub(crate) fn overlaps(me: &Body, pos: Vec2, other: &Body) -> bool {
    if me.oriented.is_none() && other.oriented.is_none() {
        return boxes_overlap(pos, me.half, other.pos, other.half);
    }
    me.shape_at(pos).overlaps(&other.shape_at(other.pos))
}

/// Where `me`, moving from `pos` along `d`, first meets `other`, which stands: [`sweep`] for two boxes; (M5-V1) the
/// sweep of their shapes if either turns with its facing (`Shape::sweep`, the separating-axis test along the move),
/// with the edge direction they meet on, the face that the rules going round it read (`Contact::face`), and the one of
/// `x` and `y` nearest to it. `None` if they never overlap on the way; touching is free.
pub(crate) fn contact(me: &Body, pos: Vec2, d: Vec2, other: &Body) -> Option<Contact> {
    if me.oriented.is_none() && other.oriented.is_none() {
        return sweep(pos, d, other.pos, me.half + other.half);
    }
    let (t, normal) = me.shape_at(pos).sweep(d, &other.shape_at(other.pos))?;
    Some(Contact {
        t,
        axis: usize::from(normal.y.abs() > normal.x.abs()),
        normal: Some(normal),
    })
}

#[cfg(test)]
mod tests;
