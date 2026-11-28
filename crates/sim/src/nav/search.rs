//! Tile A\* and the Dijkstra field — `docs/movement.md > 5.3`, `> 5.5`.
//!
//! All arithmetic is `u32`: no float enters a search. `docs/architecture/tick-pipeline.md > Tick pipeline` lists the lessons this
//! follows: a correct closed set, a best-cost map, deterministic tie-breaking, and no parent overwritten at equal
//! cost.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{NavCounters, NavGrid};
use crate::ruleset::rules;
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
    /// Per tile: the generation in which it was a target of [`Search::field_to`].
    targets: Vec<u32>,
    counters: NavCounters,
}

/// What a search heads for: it decides the heuristic and when the search stops.
#[derive(Copy, Clone)]
enum Aim {
    /// A\*: the octile distance to this tile; stops when it is expanded.
    Tile(u32),
    /// Dijkstra over the whole component: no heuristic, never stops early.
    Everything,
    /// (G) A field for `left` target tiles (marked in `Search::targets`) inside the inclusive tile rectangle
    /// `x0..=x1 × y0..=y1`: the octile distance to the rectangle; stops when every target is expanded.
    Targets {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        left: u32,
    },
}

/// The costs of an orthogonal step and of a diagonal one (`tuning::paths::Paths::step_cost`, `diagonal_cost`), read
/// once per search by its caller, so that the search loop reads no rules.
fn costs() -> (u32, u32) {
    let paths = &rules().tuning.paths;
    (paths.step_cost, paths.diagonal_cost)
}

/// The octile distance of `dx` by `dy` tiles, at `step` an orthogonal step and `diagonal` a diagonal one.
#[inline]
fn octile(dx: u32, dy: u32, (step, diagonal): (u32, u32)) -> u32 {
    step * dx.max(dy) + (diagonal - step) * dx.min(dy)
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

    /// The one search loop, after [`Search::begin`]. Towards [`Aim::Tile`] it is A\* with the octile heuristic and
    /// stops when the goal is expanded; with [`Aim::Everything`] the heuristic is 0 and it runs until the
    /// component is exhausted (Dijkstra); towards [`Aim::Targets`] the heuristic is the octile distance to the
    /// targets' rectangle, and it stops when every target is expanded. Each heuristic is consistent, so an
    /// expanded node has its final cost and parent. Returns whether it stopped at what it aimed for.
    ///
    /// `limit`: the search gives up at the first node whose `f` is beyond it. Nodes come off the heap by ascending
    /// `f`, and `f` never exceeds the cost of the cheapest path, so it gives up exactly when that path costs more
    /// than `limit`, having expanded what the unlimited search expands up to there.
    fn run(&mut self, grid: &NavGrid, start: u32, aim: Aim, limit: u32, costs: (u32, u32)) -> bool {
        let (w, h) = (grid.width() as i32, grid.height() as i32);
        let generation = self.generation;
        let goal = match aim {
            Aim::Tile(goal) => Some(goal),
            Aim::Everything | Aim::Targets { .. } => None,
        };
        let mut targets_left = match aim {
            Aim::Targets { left, .. } => left,
            Aim::Tile(_) | Aim::Everything => 0,
        };
        let heuristic = |x: i32, y: i32| match aim {
            Aim::Tile(goal) => {
                let (gx, gy) = (goal as i32 % w, goal as i32 / w);
                octile((x - gx).unsigned_abs(), (y - gy).unsigned_abs(), costs)
            }
            Aim::Everything => 0,
            Aim::Targets { x0, y0, x1, y1, .. } => octile(
                (x0 - x).max(x - x1).max(0) as u32,
                (y0 - y).max(y - y1).max(0) as u32,
                costs,
            ),
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
            if f > limit {
                return false;
            }
            self.nodes[index as usize].closed = generation;
            self.counters.expansions += 1;
            if goal == Some(index) {
                return true;
            }
            if targets_left > 0 && self.targets[index as usize] == generation {
                targets_left -= 1;
                if targets_left == 0 {
                    return true;
                }
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
                    costs.0
                } else if free[orthogonal[0]] && free[orthogonal[1]] {
                    costs.1
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
        self.astar_within(grid, start, goal, avoid, u32::MAX)
    }

    /// [`Search::astar`] that gives up, with `None`, once the cheapest path is known to cost more than `max_cost`
    /// (`Paths::step_cost` per orthogonal step): the same search up to there, so a path it returns is the one
    /// [`Search::astar`] returns. It expands only tiles through which a path of at most `max_cost` could lead.
    pub fn astar_within(
        &mut self,
        grid: &NavGrid,
        start: TilePos,
        goal: TilePos,
        avoid: &[TilePos],
        max_cost: u32,
    ) -> Option<TilePath> {
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
        self.begin((grid.width() * grid.height()) as usize);
        if !self.run(grid, start, Aim::Tile(goal), max_cost, costs()) {
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
        self.begin((grid.width() * grid.height()) as usize);
        self.run(grid, anchor, Aim::Everything, u32::MAX, costs());
        self.field = Some((anchor, grid.width(), grid.height()));
        true
    }

    /// (G) `docs/movement.md > 5.5`: the field of [`Search::field`], built only as far as the tiles in `targets`
    /// need it: the search heads for the targets' bounding rectangle and stops when each of them is expanded.
    /// [`Search::field_path`] then answers for the targets (and for whatever else was expanded on the way) with
    /// paths of the same cost as the whole field's. A group standing together far from its anchor pays for the
    /// tiles between the two, not for the map. A target that is blocked or in another component is never
    /// reached: the search then covers the whole component, like [`Search::field`]. Returns `false`, without
    /// searching, if the anchor is blocked.
    pub fn field_to(&mut self, grid: &NavGrid, anchor: TilePos, targets: &[TilePos]) -> bool {
        let Some(anchor) = grid.index(anchor).filter(|&i| !grid.blocked_at(i)) else {
            return false;
        };
        self.avoid.clear();
        self.counters.fields += 1;
        let len = (grid.width() * grid.height()) as usize;
        self.begin(len);
        // `begin` starts the generations again when the grid's size changes: so do the marks.
        if self.targets.len() != len || self.generation == 1 {
            self.targets.clear();
            self.targets.resize(len, 0);
        }
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        let mut left = 0;
        for &tile in targets {
            let Some(index) = grid.index(tile) else {
                continue;
            };
            (x0, y0, x1, y1) = (x0.min(tile.x), y0.min(tile.y), x1.max(tile.x), y1.max(tile.y));
            if self.targets[index as usize] != self.generation {
                self.targets[index as usize] = self.generation;
                left += 1;
            }
        }
        let aim = if left == 0 {
            Aim::Everything
        } else {
            Aim::Targets { x0, y0, x1, y1, left }
        };
        self.run(grid, anchor, aim, u32::MAX, costs());
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
