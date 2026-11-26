//! Which grids there are (slice M5-Z2, `docs/movement.md > 5.2`): one for each `(class, size class)` that a unit in
//! the game walks on, in the true grids and in each player's built set, built as the first such unit comes and never
//! dropped (`paths` module docs, *Which grids*), and the readers' one way to a unit's grid.

use std::collections::BTreeMap;

use glam::Vec2;

use super::{Known, NavState};
use crate::defs::{ObjectKind, UnitBaseDef};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::nav::{self, Nav, NavClass, NavGrid, TileRect};
use crate::object::Object;

/// The `(class, size class)` of the grid a unit of `kind` walks on (`docs/movement.md > 5.2`); `None` for a kind that
/// walks on none: not a unit, or an air unit (question Q0011, gap G14).
pub(crate) fn key_of(kind: ObjectKind) -> Option<(NavClass, u8)> {
    let def = kind.def();
    let class = def.unit.as_ref()?.base_def().nav_class()?;
    Some((class, nav::size_class(def.silhouette()?)))
}

/// The `(class, size class)` of every unit in the game that walks on a grid, and of every unit about to come into it,
/// sorted, each once: each unit of `objects` (on the map) and of `held` (off it: in the Tunnel Network, a site or a
/// building, `Sim::held`), and each unit a producer of `objects` has queued (`Production::queue`: a pool producer's
/// entry trains a held unit into its kind as it comes out), a Recruitment Hall's Recruit included from the start, as
/// the Hall queues it by itself in the producers' step (`Sim::start_recruits`). So the spot a unit comes out at is
/// checked on its grid: a producer's doorway (`Sim::doorway_spot`), the network's exit and a building's evacuation.
pub(crate) fn walking_keys<'a>(
    objects: impl IntoIterator<Item = &'a Object>,
    held: impl IntoIterator<Item = &'a Object>,
) -> Vec<(NavClass, u8)> {
    // Each kind is looked at once: a game has hundreds of units of a few kinds.
    let mut seen = [false; ObjectKind::ALL.len()];
    let mut keys = Vec::new();
    let mut note = |kind: ObjectKind| {
        if !std::mem::replace(&mut seen[kind as usize], true)
            && let Some(key) = key_of(kind)
        {
            keys.push(key);
        }
    };
    for o in objects {
        if o.mover.is_some() {
            note(o.kind);
        }
        if let Some(production) = &o.production {
            production
                .queue
                .iter()
                .filter_map(|entry| entry.unit())
                .for_each(&mut note);
            let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
            if let Some(recipe) = def
                .filter(|def| def.recruitment.is_some())
                .and_then(|def| def.recipes.first())
            {
                note(recipe.kind);
            }
        }
    }
    for o in held {
        note(o.kind);
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// The grid of `nav` that a unit of base `base` with silhouette `silhouette` walks on; `None` for one that walks on
/// none (an air unit). Every unit in the game has its grid ([`NavState::cover`]), so a missing one is a bug, and a
/// debug build doesn't let it pass: its readers (the movement step, a doorway's or an evacuation's spot, the
/// destinations, the gathering) would take the unit's ground for open.
pub(crate) fn unit_grid<'a>(nav: &'a Nav, base: &UnitBaseDef, silhouette: Vec2) -> Option<&'a NavGrid> {
    grid_of(nav, base.nav_class()?, nav::size_class(silhouette))
}

/// [`unit_grid`] by the grid's key: the grid of `nav` of a unit that walks on `(class, size)`.
pub(super) fn grid_of(nav: &Nav, class: NavClass, size: u8) -> Option<&NavGrid> {
    let grid = nav.grid(class, size);
    debug_assert!(
        grid.is_some(),
        "no grid of ({class:?}, {size}) for a unit that walks on it: the grids cover every unit in the game \
         (`paths::NavState::cover`)"
    );
    grid
}

impl NavState {
    /// Builds the grids of `keys` that are missing (module docs): the true one of the footprints of the true grids
    /// beside it, and one of each player's built set of the footprints that player's grids have, so that a grid built
    /// late is the one that would have been there from the start (`NavGrid::restamp` gives what `NavGrid::build`
    /// gives). A player's set built later is the true grids restamped, this one with them. Never drops a grid, so a
    /// class whose units come and go doesn't churn; and no rebuild ([`NavState::rebuilds`]) or version: what blocks
    /// the ground is as it was.
    pub(crate) fn cover(&mut self, map: &Map, keys: impl IntoIterator<Item = (NavClass, u8)>) {
        for (class, size) in keys {
            if self.nav.grid(class, size).is_some() {
                continue;
            }
            let grid = self.fresh_grid(map, class, size);
            for Known { blockers, nav, .. } in &mut self.known {
                if let Some(nav) = nav {
                    let mut known = grid.clone();
                    known.restamp(blockers);
                    nav.insert(known);
                }
            }
            self.nav.insert(grid);
        }
    }

    /// The true grid of `(class, size)` as [`NavState::cover`] would build it, of the footprints of the true grids:
    /// for the gate's check of a debug spawn of the first unit of its class when the gate is only asked
    /// (`Sim::validate`), which builds no grid. `map` is the map they were built on.
    pub(crate) fn fresh_grid(&self, map: &Map, class: NavClass, size: u8) -> NavGrid {
        NavGrid::build(map, class, size, &self.blockers, self.nav.rules())
    }
}

/// Every set of grids: the true ones first, then each player's that is built, in player order.
pub(super) fn navs<'a>(truth: &'a Nav, known: &'a [Known]) -> Vec<&'a Nav> {
    std::iter::once(truth)
        .chain(known.iter().filter_map(|k| k.nav.as_ref()))
        .collect()
}

/// The grids of `truth`'s map under `blockers` instead of its own.
pub(super) fn stamped(truth: &Nav, blockers: &[TileRect]) -> Nav {
    let mut nav = truth.clone();
    nav.restamp(blockers);
    nav
}

/// The players with a unit that walks on a grid, ascending: those whose known grids are built.
pub(super) fn walkers(objects: &BTreeMap<ObjectId, Object>) -> Vec<PlayerId> {
    let mut walkers: Vec<PlayerId> = objects
        .values()
        .filter(|o| {
            o.mover.is_some()
                && o.def()
                    .unit
                    .as_ref()
                    .is_some_and(|unit| unit.base_def().nav_class().is_some())
        })
        .filter_map(|o| o.owner)
        .collect();
    walkers.sort_unstable();
    walkers.dedup();
    walkers
}
