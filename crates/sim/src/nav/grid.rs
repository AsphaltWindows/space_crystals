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
    /// The terrain's part of `solid`: the tile is impassable for the class whatever stands on it. It changes only
    /// with the map, so a rebuild for other footprints starts from it ([`NavGrid::restamp`]).
    terrain: Vec<bool>,
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
        let mut terrain = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let preset = map.tile_preset(TilePos::new(x, y)).expect("in bounds");
                terrain.push(match class {
                    NavClass::Ground => !preset.traversible || (preset.rugged && rules.rugged_impassable),
                    NavClass::GroundRugged => !preset.traversible,
                    NavClass::Underground => !preset.drillable,
                });
            }
        }
        let mut grid = Self {
            class,
            size,
            width,
            height,
            terrain,
            solid: Vec::new(),
            open: Vec::new(),
            blocked: Vec::new(),
            component: Vec::new(),
            components: 0,
            passable: 0,
        };
        grid.restamp(blockers);
        grid
    }

    /// `docs/movement.md > 5.7`: the grid of the same terrain under other blocking footprints, as
    /// [`NavGrid::build`] would make it. The terrain's part is kept, so this is the cheap rebuild for a map that
    /// didn't change (timings: `tests/nav_perf/`).
    pub fn restamp(&mut self, blockers: &[TileRect]) {
        let (w, h) = (self.width as usize, self.height as usize);
        self.solid.clone_from(&self.terrain);
        if self.class != NavClass::Underground {
            for rect in blockers {
                let x0 = i64::from(rect.origin.x).clamp(0, w as i64) as usize;
                let y0 = i64::from(rect.origin.y).clamp(0, h as i64) as usize;
                let x1 = (i64::from(rect.origin.x) + i64::from(rect.w)).clamp(0, w as i64) as usize;
                let y1 = (i64::from(rect.origin.y) + i64::from(rect.h)).clamp(0, h as i64) as usize;
                if x0 < x1 {
                    for y in y0..y1 {
                        self.solid[y * w + x0..y * w + x1].fill(true);
                    }
                }
            }
        }
        self.mark_open();
        let k = self.size as usize;
        if k == 1 {
            self.blocked.clone_from(&self.solid);
        } else {
            self.blocked.clear();
            self.blocked.resize(w * h, true);
            for y in 0..h {
                for x in 0..w {
                    let fits = x + k <= w && y + k <= h;
                    self.blocked[y * w + x] =
                        !fits || (y..y + k).any(|by| self.solid[by * w + x..by * w + x + k].iter().any(|&s| s));
                }
            }
        }
        self.label_components();
    }

    /// `open` from `solid`: a tile is open when it isn't on the map's edge and no tile of the 3 × 3 block around it
    /// is solid. Two passes, along the rows and then across them, so the work is a few sweeps over the tiles.
    fn mark_open(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        self.open.clear();
        self.open.resize(w * h, false);
        if w < 3 || h < 3 {
            return;
        }
        // `open[i]` first holds "the tile and its two neighbours in the row are free", false in the edge columns.
        for (open, solid) in self.open.chunks_exact_mut(w).zip(self.solid.chunks_exact(w)) {
            for ((o, a), (b, c)) in open[1..w - 1]
                .iter_mut()
                .zip(&solid[..w - 2])
                .zip(solid[1..w - 1].iter().zip(&solid[2..]))
            {
                *o = !(*a | *b | *c);
            }
        }
        // Then the rows above and below must be as free: top-down, with a copy of the row above as it was. The
        // edge rows are never open.
        let mut above: Vec<bool> = self.open[..w].to_vec();
        let mut here: Vec<bool> = vec![false; w];
        self.open[..w].fill(false);
        for y in 1..h - 1 {
            here.copy_from_slice(&self.open[y * w..(y + 1) * w]);
            let (row, below) = self.open[y * w..(y + 2) * w].split_at_mut(w);
            for ((o, a), b) in row.iter_mut().zip(&above).zip(below.iter()) {
                *o &= *a & *b;
            }
            std::mem::swap(&mut above, &mut here);
        }
        self.open[(h - 1) * w..].fill(false);
    }

    /// The components, labelled in the row-major order of their first tiles, so labels are deterministic. Two
    /// tiles are connected when orthogonally adjacent: a diagonal step needs both orthogonal neighbours free
    /// (`docs/movement.md > 5.3`), so it never connects more than orthogonal steps do.
    ///
    /// It works on runs of free tiles: each run of a row joins the runs of the row above that it touches
    /// (union-find over provisional labels), then the runs get their component's final label in the order they
    /// were found. That gives the labels of a flood fill from row-major seeds in a fraction of its time, which
    /// matters because every grid is labelled again whenever a footprint changes, for the true map and for each
    /// player's knowledge (`paths::knowledge`).
    fn label_components(&mut self) {
        /// The root of `label`, halving the path to it.
        fn find(parent: &mut [u32], mut label: u32) -> u32 {
            while parent[label as usize] != label {
                let up = parent[parent[label as usize] as usize];
                parent[label as usize] = up;
                label = up;
            }
            label
        }
        let (w, h) = (self.width as usize, self.height as usize);
        self.component.clear();
        self.component.resize(w * h, 0);
        self.components = 0;
        self.passable = 0;
        // `parent[label]`: union-find over provisional labels from 1; a root is its own parent, and of two roots
        // joined the smaller label stays the root.
        let mut parent: Vec<u32> = vec![0];
        // Every run as (first tile index, length, provisional label), in row-major order.
        let mut runs: Vec<(u32, u32, u32)> = Vec::new();
        // The runs of the row above and of this row as (x0, x1, label), `x1` exclusive.
        let mut above: Vec<(u32, u32, u32)> = Vec::new();
        let mut here: Vec<(u32, u32, u32)> = Vec::new();
        for y in 0..h {
            let row = &self.blocked[y * w..(y + 1) * w];
            here.clear();
            let mut next_above = 0;
            let mut x = 0;
            while x < w {
                if row[x] {
                    x += 1;
                    continue;
                }
                let x0 = x;
                while x < w && !row[x] {
                    x += 1;
                }
                let (x0, x1) = (x0 as u32, x as u32);
                while next_above < above.len() && above[next_above].1 <= x0 {
                    next_above += 1;
                }
                let mut label = 0;
                let mut touching = next_above;
                while touching < above.len() && above[touching].0 < x1 {
                    let root = find(&mut parent, above[touching].2);
                    if label == 0 {
                        label = root;
                    } else if root != label {
                        let (low, high) = (label.min(root), label.max(root));
                        parent[high as usize] = low;
                        label = low;
                    }
                    touching += 1;
                }
                // The last run it touches may reach over the next run of this row too.
                next_above = touching.saturating_sub(1).max(next_above);
                if label == 0 {
                    label = parent.len() as u32;
                    parent.push(label);
                }
                here.push((x0, x1, label));
                runs.push(((y * w) as u32 + x0, x1 - x0, label));
            }
            std::mem::swap(&mut above, &mut here);
        }
        let mut finals: Vec<u32> = vec![0; parent.len()];
        for (start, len, label) in runs {
            let root = find(&mut parent, label) as usize;
            if finals[root] == 0 {
                self.components += 1;
                finals[root] = self.components;
            }
            self.component[start as usize..(start + len) as usize].fill(finals[root]);
            self.passable += len;
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

    /// (M5-F3) The terrain's part of [`NavGrid::is_solid`]: whether tile `t` is impassable for the class whatever
    /// stands on it, no footprint counted. Outside the map: yes.
    pub fn is_terrain_solid(&self, t: TilePos) -> bool {
        self.index(t).is_none_or(|i| self.terrain[i as usize])
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
