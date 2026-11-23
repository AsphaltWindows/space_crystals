//! Tile A\* and the Dijkstra field — `docs/movement.md > 5.3`, `> 5.5`.
//!
//! All arithmetic is `u32`: no float enters a search. `ARCHITECTURE.md > Tick pipeline` lists the lessons this
//! follows: a correct closed set, a best-cost map, deterministic tie-breaking, and no parent overwritten at equal
//! cost.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{DIAGONAL_COST, NavCounters, NavGrid, STEP_COST};
use crate::units::TilePos;

/// Neighbour offsets in the fixed expansion order E, S, W, N, SE, SW, NW, NE (`docs/movement.md > 5.3`), with the
/// two orthogonal neighbours (indices into this table) a diagonal step needs free.
const NEIGHBOURS: [(i32, i32, [usize; 2]); 8] = [
    (1, 0, [0, 0]),
    (0, 1, [1, 1]),
    (-1, 0, [2, 2]),
    (0, -1, [3, 3]),
    (1, 1, [0, 1]),
    (-1, 1, [2, 1]),
    (-1, -1, [2, 3]),
    (1, -1, [0, 3]),
];

/// A path as tiles, first to last, each a neighbour of the one before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TilePath {
    pub tiles: Vec<TilePos>,
    /// 10 per orthogonal and 14 per diagonal step.
    pub cost: u32,
}

#[derive(Copy, Clone, Default)]
struct Node {
    g: u32,
    parent: u32,
    /// Generation in which `g` and `parent` were written.
    stamp: u32,
    /// Generation in which the node was expanded.
    closed: u32,
}

/// The scratch arrays of the searches. Reuse one for every search: a generation counter makes old entries
/// invalid, so nothing is cleared and nothing carries over into a result (`docs/movement.md > 3`: scratch).
#[derive(Default)]
pub struct Search {
    nodes: Vec<Node>,
    generation: u32,
    /// Min-heap on `(f, h, tile index)`.
    heap: BinaryHeap<Reverse<(u32, u32, u32)>>,
    /// Tile indices the running search treats as blocked.
    avoid: Vec<u32>,
    /// `(anchor index, width, height)` of the field the arrays hold now, if any.
    field: Option<(u32, u32, u32)>,
    counters: NavCounters,
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    /// Work done since the last [`Search::reset_counters`].
    pub fn counters(&self) -> NavCounters {
        self.counters
    }

    pub fn reset_counters(&mut self) {
        self.counters = NavCounters::default();
    }

    /// Starts a new generation for a grid of `len` tiles.
    fn begin(&mut self, len: usize) {
        if self.nodes.len() != len || self.generation == u32::MAX {
            self.nodes.clear();
            self.nodes.resize(len, Node::default());
            self.generation = 0;
        }
        self.generation += 1;
        self.heap.clear();
        self.field = None;
    }

    /// The one search loop. With a goal it is A\* with the octile heuristic and stops when the goal is expanded;
    /// without one the heuristic is 0 and it runs until the component is exhausted (Dijkstra).
    fn run(&mut self, grid: &NavGrid, start: u32, goal: Option<u32>) -> bool {
        let (w, h) = (grid.width() as i32, grid.height() as i32);
        self.begin((w * h) as usize);
        let generation = self.generation;
        let goal_xy = goal.map(|g| (g as i32 % w, g as i32 / w));
        let heuristic = |x: i32, y: i32| match goal_xy {
            Some((gx, gy)) => {
                let (dx, dy) = ((x - gx).unsigned_abs(), (y - gy).unsigned_abs());
                STEP_COST * dx.max(dy) + (DIAGONAL_COST - STEP_COST) * dx.min(dy)
            }
            None => 0,
        };
        let start_h = heuristic(start as i32 % w, start as i32 / w);
        self.nodes[start as usize] = Node {
            g: 0,
            parent: start,
            stamp: generation,
            closed: 0,
        };
        self.heap.push(Reverse((start_h, start_h, start)));
        while let Some(Reverse((f, node_h, index))) = self.heap.pop() {
            let node = self.nodes[index as usize];
            // A stale entry: the node was expanded already, or was pushed again with a lower cost.
            if node.closed == generation || f - node_h != node.g {
                continue;
            }
            self.nodes[index as usize].closed = generation;
            self.counters.expansions += 1;
            if goal == Some(index) {
                return true;
            }
            let (x, y) = (index as i32 % w, index as i32 / w);
            let mut free = [false; 4];
            for (n, &(dx, dy, orthogonal)) in NEIGHBOURS.iter().enumerate() {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let neighbour = (ny * w + nx) as u32;
                if grid.blocked_at(neighbour) || self.avoid.contains(&neighbour) {
                    continue;
                }
                let cost = if n < 4 {
                    free[n] = true;
                    STEP_COST
                } else if free[orthogonal[0]] && free[orthogonal[1]] {
                    DIAGONAL_COST
                } else {
                    // No corner cutting: nothing squeezes between diagonal obstacles.
                    continue;
                };
                let g = node.g + cost;
                let entry = &mut self.nodes[neighbour as usize];
                // Relax only on a strictly lower cost, so an equal-cost parent is never overwritten.
                if entry.stamp == generation && (entry.closed == generation || g >= entry.g) {
                    continue;
                }
                *entry = Node {
                    g,
                    parent: index,
                    stamp: generation,
                    closed: 0,
                };
                let neighbour_h = heuristic(nx, ny);
                self.heap.push(Reverse((g + neighbour_h, neighbour_h, neighbour)));
            }
        }
        false
    }

    /// The tiles from `from` along the parent chain to the search's start, and the cost of `from`.
    fn chain(&self, grid: &NavGrid, from: u32) -> TilePath {
        let mut tiles = vec![grid.tile(from)];
        let mut index = from;
        loop {
            let parent = self.nodes[index as usize].parent;
            if parent == index {
                break;
            }
            index = parent;
            tiles.push(grid.tile(index));
        }
        TilePath {
            tiles,
            cost: self.nodes[from as usize].g,
        }
    }

    /// The cheapest tile path from `start` to `goal`: 8-connected, no corner cutting, `avoid` tiles treated as
    /// blocked (the start tile never is).
    ///
    /// Returns `None` **without searching** when the start is blocked, the goal is blocked or in another
    /// component, or the goal is an avoid tile. Otherwise only avoid tiles can make it fail
    /// (`docs/movement.md > 5.3`); callers retarget an unreachable goal first ([`NavGrid::nearest_in_component`]).
    pub fn astar(&mut self, grid: &NavGrid, start: TilePos, goal: TilePos, avoid: &[TilePos]) -> Option<TilePath> {
        let (start, goal) = (grid.index(start)?, grid.index(goal)?);
        if grid.blocked_at(start) || grid.component_at(start) != grid.component_at(goal) {
            return None;
        }
        self.avoid.clear();
        self.avoid
            .extend(avoid.iter().filter_map(|&t| grid.index(t)).filter(|&i| i != start));
        if self.avoid.contains(&goal) {
            return None;
        }
        self.counters.searches += 1;
        if !self.run(grid, start, Some(goal)) {
            return None;
        }
        let mut path = self.chain(grid, goal);
        path.tiles.reverse();
        Some(path)
    }

    /// `docs/movement.md > 5.5`: one Dijkstra over the anchor's whole component (the same loop, `h = 0`). Until
    /// the next search, [`Search::field_path`] reads paths out of it. Returns `false`, without searching, if the
    /// anchor is blocked.
    pub fn field(&mut self, grid: &NavGrid, anchor: TilePos) -> bool {
        let Some(anchor) = grid.index(anchor).filter(|&i| !grid.blocked_at(i)) else {
            return false;
        };
        self.avoid.clear();
        self.counters.fields += 1;
        self.run(grid, anchor, None);
        self.field = Some((anchor, grid.width(), grid.height()));
        true
    }

    /// The cheapest tile path from `from` to the anchor of the last [`Search::field`], along the field's parent
    /// chain. `None` if `from` is outside the anchor's component, or the last search wasn't a field on this grid.
    pub fn field_path(&self, grid: &NavGrid, from: TilePos) -> Option<TilePath> {
        let (_, width, height) = self.field?;
        if (width, height) != (grid.width(), grid.height()) {
            return None;
        }
        let from = grid.index(from)?;
        (self.nodes[from as usize].closed == self.generation).then(|| self.chain(grid, from))
    }
}
