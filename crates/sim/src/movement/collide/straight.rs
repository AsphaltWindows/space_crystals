//! (M3-D2) A move in a straight line, as far as it is free: the walk out of a portal's energy doorway (design change
//! 0145, `super::super::doorway::emerge`), which goes neither round nor along what is in its way.

use glam::Vec2;

use super::World;

/// How many times [`World::straight`] halves a move that a tile is in the way of before it gives up: down to a
/// 256th of it (a technical bound).
const HALVINGS: u32 = 8;

impl World<'_> {
    /// `pos` moved in a straight line towards `to`, as far as it is free: `to` itself if the whole move is; else, if a
    /// tile is in the way (`tiles_free`, so the doorway's exception holds), the longest of its halvings that isn't,
    /// down to [`HALVINGS`]; and of that, if a unit is in the way, the part up to the contact with it (`up_to`; the
    /// unit is noted in `touched`). Never sideways. Every position it returns passed [`World::is_free`]; `pos` itself
    /// if no part of the move is free.
    pub(in crate::movement) fn straight(&mut self, pos: Vec2, to: Vec2, touched: &mut Vec<usize>) -> Vec2 {
        let mut d = to - pos;
        let mut halvings = 0;
        while !self.tiles_free(pos, d) {
            if halvings == HALVINGS {
                return pos;
            }
            d *= 0.5;
            halvings += 1;
        }
        if let Some(hit) = self.first_unit(pos, d) {
            touched.push(hit.index);
            return self.up_to(pos, d, hit);
        }
        // The whole move ends exactly on `to`.
        if halvings == 0 && self.is_free(to) {
            return to;
        }
        let end = pos + d;
        if self.is_free(end) { end } else { pos }
    }
}
