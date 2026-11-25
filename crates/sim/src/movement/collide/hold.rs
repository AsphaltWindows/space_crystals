//! (M3-G2) The move of a unit that waits in its queue — `docs/movement.md > 6.5`, rule 2.

use glam::Vec2;

use super::{Moved, World};
use crate::mover::Mover;
use crate::units::Tick;

impl World<'_> {
    /// (M3-G2) The move of a unit that waits in its queue (`stuck::waits_in_queue`): [`World::resolve_for`]
    /// without rule 4. Behind a unit that stands or comes its way it stays where it is, and makes no step round it;
    /// everything else is as ever, so it goes on the moment the way straight ahead is free.
    pub(in crate::movement) fn hold(
        &mut self,
        pos: Vec2,
        d: Vec2,
        snap: Option<Vec2>,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
    ) -> Moved {
        if self.tiles_free(pos, d)
            && let Some(hit) = self.first_unit(pos, d)
            && self.bodies[hit.index].velocity.dot(d) <= 0.0
        {
            touched.push(hit.index);
            return Moved {
                pos,
                forward: false,
                blocker: Some(hit.index),
                aside: false,
                slid: false,
            };
        }
        self.resolve_for(pos, d, snap, mover, tick, touched, false)
    }
}
