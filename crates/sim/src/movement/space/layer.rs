//! The spatial grid of one collision layer — `docs/movement.md > 6.1` (split out of `super`, plan decision T28): a
//! cell per tile listing the bodies whose box reaches it, and per cell the kinds of body listed in it
//! ([`Body::kinds`]), so that a query skips the cells that can't hold what it looks for ([`Layer::near_any`]).

use glam::Vec2;

use super::Body;
use crate::units::SU_PER_GU;

/// One layer's grid (`docs/movement.md > 6.1`): a cell per tile, each unit listed in every cell its box overlaps or
/// touches, and per cell the kinds of unit listed in it.
#[derive(Debug, Default)]
pub(crate) struct Layer {
    width: i32,
    height: i32,
    cells: Vec<Vec<u32>>,
    /// (G) Per cell: what kinds of body were listed in it since the last reset ([`Body::kinds`]). A bit is never
    /// taken back when a body leaves or changes, so it says "maybe", and lets a query skip cells that can't hold
    /// what it looks for ([`Layer::near_any`]).
    kinds: Vec<u16>,
    /// Cells that got an entry since the last reset, to empty only those.
    used: Vec<u32>,
    /// (G) The largest half extent of the bodies listed since the last reset ([`Layer::widest`]).
    widest: f32,
}

/// [`Body::kinds`]: the body steps aside for a blocked unit of its owner (`Body::yields`).
pub(crate) const KIND_YIELDS: u16 = 1 << 8;

/// (M5-V4) [`Body::kinds`]: the body is crushable (`Body::crushable`), for the crush step's query
/// (`super::super::crush`).
pub(crate) const KIND_CRUSHABLE: u16 = 1 << 9;

/// The middles of the eight 45° sectors of headings, sector `k` from `45k°` to `45k + 45°` (0 is east, clockwise
/// on screen): `(cos, sin)` of `45k + 22.5°`.
const SECTORS: [Vec2; 8] = {
    const C: f32 = 0.923_879_5;
    const S: f32 = 0.382_683_43;
    [
        Vec2::new(C, S),
        Vec2::new(S, C),
        Vec2::new(-S, C),
        Vec2::new(-C, S),
        Vec2::new(-C, -S),
        Vec2::new(-S, -C),
        Vec2::new(S, -C),
        Vec2::new(C, -S),
    ]
};

/// The sector of [`SECTORS`] a heading lies in.
fn sector(heading: Vec2) -> usize {
    let wide = heading.x.abs() > heading.y.abs();
    match (heading.y >= 0.0, heading.x >= 0.0) {
        (true, true) => usize::from(!wide),
        (true, false) => 2 + usize::from(wide),
        (false, false) => 4 + usize::from(!wide),
        (false, true) => 6 + usize::from(wide),
    }
}

/// The kinds ([`Body::kinds`]) of the bodies whose heading may be against `ahead` (a unit vector) by more than the
/// cosine `cos`: every sector that holds such a heading. A heading is within 22.5° of its sector's middle, so the
/// sectors are those whose middle is nearly as much against `ahead`. It errs on the side of "maybe".
pub(crate) fn kinds_against(ahead: Vec2, cos: f32) -> u16 {
    // Two cosines differ by at most the angle between the two directions: 22.5° is 0.393 rad.
    let bound = cos + 0.4;
    let mut kinds = 0;
    for (k, middle) in SECTORS.iter().enumerate() {
        if middle.dot(ahead) < bound {
            kinds |= 1 << k;
        }
    }
    kinds
}

impl Body {
    /// (G) Whether the body's heading is against the unit vector `ahead` by more than the cosine `cos`, which is
    /// negative: the angle between the two is wider than that cosine's. In squares, so without a root.
    pub(crate) fn heads_against(&self, ahead: Vec2, cos: f32) -> bool {
        let along = self.heading.dot(ahead);
        along < 0.0 && along * along > cos * cos * self.heading.length_squared()
    }

    /// (G) What a query may look for in a cell ([`Layer::near_any`]): the sector of the body's heading (bits 0 to 7;
    /// none for a body that isn't heading anywhere), [`KIND_YIELDS`] and (M5-V4) [`KIND_CRUSHABLE`].
    pub(crate) fn kinds(&self) -> u16 {
        let heading = if self.heading == Vec2::ZERO {
            0
        } else {
            1 << sector(self.heading)
        };
        heading | if self.yields { KIND_YIELDS } else { 0 } | if self.crushable { KIND_CRUSHABLE } else { 0 }
    }
}

impl Layer {
    /// Empties the grid, for a map of `width × height` tiles.
    pub(crate) fn reset(&mut self, width: u32, height: u32) {
        let (width, height) = (width as i32, height as i32);
        self.widest = 0.0;
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.cells = vec![Vec::new(); (width * height) as usize];
            self.kinds = vec![0; (width * height) as usize];
            self.used.clear();
            return;
        }
        for cell in self.used.drain(..) {
            self.cells[cell as usize].clear();
            self.kinds[cell as usize] = 0;
        }
    }

    /// The cells the closed box `lo..=hi` reaches, as inclusive tile ranges, clamped to the map.
    fn span(&self, lo: Vec2, hi: Vec2) -> (i32, i32, i32, i32) {
        // (G) The cast truncates, which is the floor for every tile on the map; left of or above the map both
        // clamp to tile 0. (`floor` is a library call on the baseline x86-64 target, and this is the hot path.)
        let tile = |v: f32, max: i32| ((v / SU_PER_GU) as i32).clamp(0, max - 1);
        (
            tile(lo.x, self.width),
            tile(lo.y, self.height),
            tile(hi.x, self.width),
            tile(hi.y, self.height),
        )
    }

    /// Lists body `index`, of these `kinds` ([`Body::kinds`]), in the cells its box at `pos` reaches.
    pub(crate) fn insert(&mut self, index: u32, pos: Vec2, half: Vec2, kinds: u16) {
        self.widest = self.widest.max(half.max_element());
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.width + x) as usize;
                self.cells[cell].push(index);
                self.kinds[cell] |= kinds;
                self.used.push(cell as u32);
            }
        }
    }

    /// (G) The largest half extent, on either axis, of the bodies listed since the last reset: no box reaches
    /// farther than `√2` times this from its centre in any direction.
    pub(crate) fn widest(&self) -> f32 {
        self.widest
    }

    /// (G) Notes that the body listed at `pos` now has these `kinds` ([`Body::kinds`]) too.
    pub(crate) fn mark(&mut self, pos: Vec2, half: Vec2, kinds: u16) {
        if kinds == 0 || self.kinds.is_empty() {
            return;
        }
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.kinds[(y * self.width + x) as usize] |= kinds;
            }
        }
    }

    /// (G) [`Layer::near`], if a cell the box reaches may list a body of one of `kinds` ([`Body::kinds`]);
    /// otherwise it visits nothing and returns `false`.
    pub(crate) fn near_any(&self, lo: Vec2, hi: Vec2, kinds: u16, mut visit: impl FnMut(u32)) -> bool {
        if self.cells.is_empty() {
            return false;
        }
        let (x0, y0, x1, y1) = self.span(lo, hi);
        if !(y0..=y1).any(|y| (x0..=x1).any(|x| self.kinds[(y * self.width + x) as usize] & kinds != 0)) {
            return false;
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &index in &self.cells[(y * self.width + x) as usize] {
                    visit(index);
                }
            }
        }
        true
    }

    pub(crate) fn remove(&mut self, index: u32, pos: Vec2, half: Vec2) {
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = &mut self.cells[(y * self.width + x) as usize];
                if let Some(at) = cell.iter().position(|&i| i == index) {
                    cell.swap_remove(at);
                }
            }
        }
    }

    /// Calls `visit` with every body listed in a cell the closed box `lo..=hi` reaches, in a fixed order. A body
    /// in several of the cells is visited once per cell.
    pub(crate) fn near(&self, lo: Vec2, hi: Vec2, mut visit: impl FnMut(u32)) {
        if self.cells.is_empty() {
            return;
        }
        let (x0, y0, x1, y1) = self.span(lo, hi);
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &index in &self.cells[(y * self.width + x) as usize] {
                    visit(index);
                }
            }
        }
    }
}
