//! The per-tile passability grid and its connected components — `docs/movement.md > 5.2`.

use glam::Vec2;

use super::{NavClass, NavRules, TileRect};
use crate::map::Map;
use crate::units::{SU_PER_GU, TilePos};

/// Passability for one `(NavClass, size class)`: a blocked bit and a component label per tile.
///
/// Tiles are indexed row-major, `y * width + x`. Everything outside the map is blocked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavGrid {
    class: NavClass,
    size: u8,
    width: u32,
    height: u32,
    /// The tile itself is impassable for the class. Line of sight tests boxes against these.
    solid: Vec<bool>,
    /// (G) The tile and its eight neighbours are all on the map and none is solid ([`NavGrid::open_around`]).
    open: Vec<bool>,
    /// A unit of this size class can't be anchored here: for size 1 the same as `solid`.
    blocked: Vec<bool>,
    /// Component label, from 1; 0 on blocked tiles.
    component: Vec<u32>,
    components: u32,
    passable: u32,
}

impl NavGrid {
    /// Builds the grid of `class` and size class `size` (at least 1) from the map and the blocking footprints.
    ///
    /// - `Ground`: blocked if not Traversible, Rugged (when `rules.rugged_impassable`), or under a footprint.
    /// - `GroundRugged`: not Traversible, or under a footprint.
    /// - `Underground`: not Drillable; surface footprints don't matter.
    /// - Size class `k > 1`: a tile is passable if the `k × k` block anchored on it (it is the top-left) is.
    pub fn build(map: &Map, class: NavClass, size: u8, blockers: &[TileRect], rules: NavRules) -> Self {
        let size = size.max(1);
        let (width, height) = (map.width(), map.height());
        let (w, h) = (width as usize, height as usize);
        let mut solid = Vec::with_capacity(w * h);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let preset = map.tile_preset(TilePos::new(x, y)).expect("in bounds");
                solid.push(match class {
                    NavClass::Ground => !preset.traversible || (preset.rugged && rules.rugged_impassable),
                    NavClass::GroundRugged => !preset.traversible,
                    NavClass::Underground => !preset.drillable,
                });
            }
        }
        if class != NavClass::Underground {
            for rect in blockers {
                let x0 = i64::from(rect.origin.x).max(0);
                let y0 = i64::from(rect.origin.y).max(0);
                let x1 = (i64::from(rect.origin.x) + i64::from(rect.w)).min(i64::from(width));
                let y1 = (i64::from(rect.origin.y) + i64::from(rect.h)).min(i64::from(height));
                for y in y0..y1 {
                    for x in x0..x1 {
                        solid[y as usize * w + x as usize] = true;
                    }
                }
            }
        }
        let open = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                x > 0
                    && y > 0
                    && x + 1 < w
                    && y + 1 < h
                    && (y - 1..=y + 1).all(|ny| !solid[ny * w + x - 1..=ny * w + x + 1].iter().any(|&s| s))
            })
            .collect();
        let k = size as usize;
        let blocked = if k == 1 {
            solid.clone()
        } else {
            let mut blocked = vec![true; w * h];
            for y in 0..h {
                for x in 0..w {
                    let fits = x + k <= w && y + k <= h;
                    blocked[y * w + x] =
                        !fits || (y..y + k).any(|by| solid[by * w + x..by * w + x + k].iter().any(|&s| s));
                }
            }
            blocked
        };
        let mut grid = Self {
            class,
            size,
            width,
            height,
            solid,
            open,
            blocked,
            component: Vec::new(),
            components: 0,
            passable: 0,
        };
        grid.label_components();
        grid
    }

    /// Flood fill from row-major seeds, so labels are deterministic. Two tiles are connected when orthogonally
    /// adjacent: a diagonal step needs both orthogonal neighbours free (`docs/movement.md > 5.3`), so it never
    /// connects more than orthogonal steps do.
    fn label_components(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        self.component = vec![0; w * h];
        self.components = 0;
        self.passable = 0;
        let mut stack = Vec::new();
        for seed in 0..w * h {
            if self.blocked[seed] || self.component[seed] != 0 {
                continue;
            }
            self.components += 1;
            self.component[seed] = self.components;
            stack.push(seed);
            while let Some(i) = stack.pop() {
                self.passable += 1;
                let (x, y) = (i % w, i / w);
                let neighbours = [
                    (x + 1 < w).then(|| i + 1),
                    (y + 1 < h).then(|| i + w),
                    (x > 0).then(|| i - 1),
                    (y > 0).then(|| i - w),
                ];
                for n in neighbours.into_iter().flatten() {
                    if !self.blocked[n] && self.component[n] == 0 {
                        self.component[n] = self.components;
                        stack.push(n);
                    }
                }
            }
        }
    }

    /// (G) Whether the tile of point `p` and its eight neighbours are on the map and free of solid tiles. A box up
    /// to a tile wide centred within such a tile overlaps no solid tile, and neither does it on a move shorter
    /// than `SU_PER_GU - half extent` from there: [`super::box_clear`] and [`super::los`] hold without a look at
    /// the tiles. The movement step asks this first, for every unit and tick. `false` off the map and for points
    /// that aren't finite.
    pub fn open_around(&self, p: Vec2) -> bool {
        // On the map the cast is the floor (`floor` is a library call on the baseline x86-64 target, and this is
        // the hot path). A point left of or above the map, or not finite, is not on it.
        if !(p.x >= 0.0 && p.y >= 0.0 && p.is_finite()) {
            return false;
        }
        let tile = TilePos::new((p.x / SU_PER_GU) as i32, (p.y / SU_PER_GU) as i32);
        self.index(tile).is_some_and(|i| self.open[i as usize])
    }

    pub fn class(&self) -> NavClass {
        self.class
    }

    /// The size class, at least 1.
    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// How many tiles a unit of this grid can be anchored on.
    pub fn passable_count(&self) -> u32 {
        self.passable
    }

    /// How many components there are. Their labels are `1..=component_count()`.
    pub fn component_count(&self) -> u32 {
        self.components
    }

    pub fn in_bounds(&self, t: TilePos) -> bool {
        t.x >= 0 && t.y >= 0 && (t.x as u32) < self.width && (t.y as u32) < self.height
    }

    /// Row-major index of a tile on the map.
    pub fn index(&self, t: TilePos) -> Option<u32> {
        self.in_bounds(t).then(|| t.y as u32 * self.width + t.x as u32)
    }

    /// The tile of a row-major index.
    pub fn tile(&self, index: u32) -> TilePos {
        TilePos::new((index % self.width) as i32, (index / self.width) as i32)
    }

    /// Whether a unit of this grid can't be anchored on `t`. Tiles outside the map are blocked.
    pub fn is_blocked(&self, t: TilePos) -> bool {
        self.index(t).is_none_or(|i| self.blocked[i as usize])
    }

    /// Whether tile `t` itself is impassable for the class, whatever the size class. Outside the map: yes.
    pub fn is_solid(&self, t: TilePos) -> bool {
        self.index(t).is_none_or(|i| self.solid[i as usize])
    }

    /// The component label of a passable tile; `None` for a blocked one. Two tiles have a path between them
    /// exactly when their labels are equal.
    pub fn component(&self, t: TilePos) -> Option<u32> {
        let label = self.component[self.index(t)? as usize];
        (label != 0).then_some(label)
    }

    pub(super) fn blocked_at(&self, index: u32) -> bool {
        self.blocked[index as usize]
    }

    pub(super) fn component_at(&self, index: u32) -> u32 {
        self.component[index as usize]
    }

    /// Offset from the centre of a unit's anchor tile to the unit's centre: zero for size class 1.
    fn anchor_offset(&self) -> Vec2 {
        Vec2::splat(f32::from(self.size - 1) * (SU_PER_GU / 2.0))
    }

    /// The tile a unit centred on `p` is anchored on: for size class 1, the tile that contains `p`.
    pub fn tile_of(&self, p: Vec2) -> TilePos {
        TilePos::containing(p - self.anchor_offset())
    }

    /// Where a unit anchored on `t` is centred: for size class 1 the tile's centre, else its block's centre.
    pub fn center(&self, t: TilePos) -> Vec2 {
        t.center() + self.anchor_offset()
    }

    /// `t` moved onto the map by the shortest way.
    pub fn clamp(&self, t: TilePos) -> TilePos {
        TilePos::new(
            t.x.clamp(0, self.width as i32 - 1),
            t.y.clamp(0, self.height as i32 - 1),
        )
    }

    /// `docs/movement.md > 5.3`: the tile of `component` nearest to `goal`, which is `goal` itself when it
    /// belongs to the component. A ring scan outward from the goal: nearest by squared distance between tiles,
    /// ties by tile index. `None` only if the component doesn't exist. It never searches.
    pub fn nearest_in_component(&self, component: u32, goal: TilePos) -> Option<TilePos> {
        if component == 0 || component > self.components {
            return None;
        }
        let goal = self.clamp(goal);
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (gx, gy) = (i64::from(goal.x), i64::from(goal.y));
        // (squared distance, index) of the best tile so far.
        let mut best: Option<(i64, u32)> = None;
        for r in 0..w.max(h) {
            // Every tile of ring `r` or beyond is at least `r` away.
            if best.is_some_and(|(d2, _)| d2 < r * r) {
                break;
            }
            let mut visit = |x: i64, y: i64| {
                if x < 0 || y < 0 || x >= w || y >= h {
                    return;
                }
                let index = (y * w + x) as u32;
                if self.component[index as usize] == component {
                    let candidate = ((x - gx) * (x - gx) + (y - gy) * (y - gy), index);
                    if best.is_none_or(|b| candidate < b) {
                        best = Some(candidate);
                    }
                }
            };
            if r == 0 {
                visit(gx, gy);
                continue;
            }
            for x in gx - r..=gx + r {
                visit(x, gy - r);
                visit(x, gy + r);
            }
            for y in gy - r + 1..gy + r {
                visit(gx - r, y);
                visit(gx + r, y);
            }
        }
        best.map(|(_, index)| self.tile(index))
    }
}
