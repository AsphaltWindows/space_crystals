//! (M5-V4) Crushing — `systems/units.md > UnitCollision > Crushing`, plan decision T86 with design changes 0418 and
//! 0419 (`docs/movement.md > 6.13`): "Units whose UnitBase can crush (TrackedVehicle, Mech, and a **DrillUnit in its
//! above-ground mode** [...]; never underground) crush enemy units whose UnitBase has Crushable = true (Light
//! Infantry)" (`UnitBaseDef::crushes`, `UnitBaseDef::crushable`). "**Effect**: instant death of the crushed unit."
//!
//! - **Whose move crushes** ([`drives`]): "**Trigger**: the crusher's Silhouette overlaps a crushable enemy **while
//!   the crusher has any speed** (Locomotion = Moving, Reversing or **Stopping**: a braking or coasting crusher still
//!   crushes)" (design change 0453, answering Q0178), and "**Turning in place crushes**" (design change 0418:
//!   "Silhouette overlap while Moving, Reversing **or Stationary + Turning**"). It is decided as the unit's turn of
//!   phase 5d begins, before its move, which every part of asks the collision exceptions (`space::blocks`): the probe,
//!   the lanes, the turn's every part. Every move a crusher makes in its turn is one of those: it sets Moving,
//!   Reversing or Stopping (braking to rest with no way to steer, `super::model::Model::coast`, after a Stop or as it
//!   comes to its goal; braking against its leg before it sets off the other way, `super::tracked`), or spins it in
//!   place. So a crusher's move always crushes, but for one at rest that its attack holds, which makes none. A
//!   crusher that brakes rolls over the enemy infantry in its way rather than halt against them, and is a hard
//!   obstacle only once it stands (Stationary) and doesn't turn: "a crusher with no speed (Stationary and not turning
//!   [...]) remains a normal hard obstacle" (`systems/units.md > UnitCollision > Crushing`).
//! - **Collision** (`space::blocks`, rule 3): such a move passes through the crushable enemies of its layer ("those
//!   enemies are not obstacles to them"; a spin's "don't block the spin"), and nothing else changes: a crusher that
//!   stands and doesn't turn blocks them ("a normal hard obstacle; crushable units cannot walk into it"), they keep
//!   avoiding crushers, and own and allied crushables block a crusher ("No friendly crushing"), its spin too, which
//!   "stops at contact". So a crusher's follower goes round its own infantry and straight through the enemy's.
//! - **The kill** ([`victims`]): at its commit, the crushable enemies its Silhouette overlaps where it ends, or
//!   swept on the way (its turn, then its move: `Shape` and the rotated shape, `systems/units.md > UnitCollision >
//!   Silhouette Rotation`), found through the movement grid's query, leave the layer at once: the units after it in
//!   the tick no longer meet them, and those not yet moved don't move. They die through the removal path in the same
//!   phase (`Sim::remove_objects`, which frees their Unit Control and their lineage with them), each with a
//!   `SimEvent::Crushed` in place of a `Died`. It isn't damage: no Hit, no armour, no reveal, no Shields. So no
//!   overlap with a crushed unit outlives its tick, and the no-overlap invariant I1, checked between ticks, needs no
//!   exception.
//! - **Enemies** (`economy::defeat::allied`): another player's units. Neutral ones are nobody's enemy, and block.
//!
//! (M6-ST1) Whether a type crushes is its own flag over its base's (`UnitDef::crushes`): the DrillUnit base's is false,
//! since underground it never crushes, and the Subterranean Transport's above-ground type crushes by its own (design
//! change 0419: "Above ground it **crushes** enemy Light Infantry like any tracked vehicle").

use std::collections::BTreeMap;

use glam::Vec2;

use super::space::{Body, KIND_CRUSHABLE, Layer};
use crate::economy::defeat;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::mover::Mover;
use crate::object::Object;
use crate::ruleset::rules;
use crate::shape::Shape;
use crate::vision::Hiding;

/// One crush of the tick (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Crush {
    /// The crushed unit: an index into the tick's bodies.
    pub(super) victim: usize,
    /// Its crusher, likewise.
    pub(super) by: usize,
}

/// Whether the players `a` and `b` are enemies: each a player, of another side (`economy::defeat::allied`). A neutral
/// unit is nobody's enemy.
fn enemies(a: Option<PlayerId>, b: Option<PlayerId>) -> bool {
    a.zip(b).is_some_and(|(a, b)| !defeat::allied(a, b))
}

/// `space::blocks`'s rule 3: whether the move of `mover` goes through `other` (module docs): its move this tick
/// crushes ([`Body::crushes`]), and `other` is a crushable enemy of its.
pub(super) fn crushes(mover: &Body, other: &Body) -> bool {
    mover.crushes && other.crushable && enemies(mover.owner, other.owner)
}

/// Whether the move a unit whose type crushes if `crushes` (`UnitDef::crushes`: (M6-ST1) its own flag, else its
/// base's) makes in its turn of this tick crushes (module docs; [`Body::crushes`]), as its turn begins, `mover` as it
/// is: a crusher's always does, Moving, Reversing, Stopping (design change 0453) or a spin, but for one at rest that
/// its attack holds (`held`), which makes no move.
pub(super) fn drives(crushes: bool, mover: &Mover, held: bool) -> bool {
    crushes && !(held && mover.momentum == Vec2::ZERO)
}

/// The crushes of the crusher `me` (an index into `bodies`) in its turn of this tick, whose move crushed ([`drives`]),
/// noted in `crushed`: it moved from `from`, facing `start`, to where its body now lies (`bodies[me]`, as it
/// committed). Each crushable enemy of its layer that its shape overlaps there, or overlapped on the way: along its
/// turn, which every model makes before its move and where it stands, tried as `World::turn` tries it (in parts along
/// which its corners move at most `Movement::turn_try_su`), and along its move in the shape it ends in (`Shape::sweep`,
/// which counts touching as free). None if it neither moved nor turned ("a crusher that is stationary and **not**
/// turning stays a hard obstacle"). Those crushed earlier in the tick have left the layer. Once each, in the grid's
/// order.
pub(super) fn victims(bodies: &[Body], layer: &Layer, me: usize, from: Vec2, start: f32, crushed: &mut Vec<Crush>) {
    let now = bodies[me];
    let turned = match now.oriented {
        Some(lies) if lies.facing != start => math::wrap_angle(lies.facing - start),
        _ => 0.0,
    };
    if now.pos == from && turned == 0.0 {
        return;
    }
    // A shape that turns lies within its circumscribed circle at every facing.
    let reach = now.oriented.map_or(now.half, |lies| Vec2::splat(lies.half.length()));
    let first = crushed.len();
    // What it swept, worked out for the first unit it may have crushed: most crushers' moves come near none.
    let mut swept: Option<Swept> = None;
    layer.near_any(
        from.min(now.pos) - reach,
        from.max(now.pos) + reach,
        KIND_CRUSHABLE,
        |index| {
            let index = index as usize;
            let other = &bodies[index];
            if index == me
                || !other.crushable
                || !enemies(now.owner, other.owner)
                || crushed[first..].iter().any(|c| c.victim == index)
            {
                return;
            }
            let swept = swept.get_or_insert_with(|| Swept::of(&now, from, start, turned));
            if swept.reaches(&other.shape_at(other.pos)) {
                crushed.push(Crush { victim: index, by: me });
            }
        },
    );
}

/// What a crusher's move swept ([`victims`]): its shape where it ended (`ending`), the shape it ended in where it
/// started (`leaving`) and the move between (`moved`), and its turn where it started: from the facing `start`, by
/// `turned`, in `parts` as `World::turn` tries it, its Silhouette's half extents `half`.
struct Swept {
    ending: Shape,
    leaving: Shape,
    moved: Vec2,
    from: Vec2,
    half: Vec2,
    start: f32,
    turned: f32,
    parts: u32,
}

impl Swept {
    /// What the crusher whose body is now `now` swept, from `from` and the facing `start`, having turned by `turned`.
    fn of(now: &Body, from: Vec2, start: f32, turned: f32) -> Self {
        let tuning = &rules().tuning.movement;
        let (half, parts) = match now.oriented {
            Some(lies) if turned != 0.0 => {
                let parts = (turned.abs() * lies.half.length() / tuning.turn_try_su).ceil().max(1.0) as u32;
                (lies.half, parts)
            }
            _ => (Vec2::ZERO, 0),
        };
        Self {
            ending: now.shape_at(now.pos),
            leaving: now.shape_at(from),
            moved: now.pos - from,
            from,
            half,
            start,
            turned,
            parts,
        }
    }

    /// Whether it swept `shape`: its shape overlaps it where it ended, or swept it on its way (its move, which counts
    /// touching as free; the parts of its turn before the facing it ended at).
    fn reaches(&self, shape: &Shape) -> bool {
        self.ending.overlaps(shape)
            || (self.moved != Vec2::ZERO && self.leaving.sweep(self.moved, shape).is_some())
            || (1..self.parts).any(|part| self.turning(part).overlaps(shape))
    }

    /// Its shape at the end of part `part` of its turn.
    fn turning(&self, part: u32) -> Shape {
        let facing = math::wrap_angle(self.start + self.turned * (part as f32 / self.parts as f32));
        Shape::oriented(self.from, self.half, math::facing_vector(facing))
    }
}

/// The tick's crushes (`crushed`, of the tick's `bodies`) as the sim tells them: a `SimEvent::Crushed` for each, in
/// the order they came, the crushed unit where it died and as hidden as it was (`Hiding::of`), as a death is told.
/// Returns the crushed, for the removal path.
pub(super) fn tell(
    objects: &BTreeMap<ObjectId, Object>,
    bodies: &[Body],
    crushed: &[Crush],
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    let mut dead = Vec::with_capacity(crushed.len());
    for crush in crushed {
        let Some(o) = objects.get(&bodies[crush.victim].id) else {
            continue;
        };
        events.push(SimEvent::Crushed {
            id: o.id,
            kind: o.kind,
            owner: o.owner,
            pos: o.pos,
            hiding: Hiding::of(o),
            crusher: bodies[crush.by].id,
        });
        dead.push(o.id);
    }
    dead.sort_unstable();
    dead
}

#[cfg(test)]
mod tests;
