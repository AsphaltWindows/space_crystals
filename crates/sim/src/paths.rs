//! The sim's navigation state and path requests — `docs/movement.md > 4` (phases 5b and 5c), `docs/movement.md > 5.5`,
//! `docs/movement.md > 5.7`, and `systems/controls/behaviors.md > PathKnowledge`.
//!
//! [`NavState`] holds two kinds of `nav` grids, both derived data: never hashed, serialized or compared, rebuilt
//! when a `Sim` is created or loaded.
//!
//! - **The true grids** ([`NavState::nav`]): the map and every blocking footprint. Units **collide** on them: the
//!   movement step (5d), the spawn and teleport checks of the command gate and invariant I2 read these and nothing
//!   else. Phase 5b ([`sync`]) rebuilds them when the blocking footprints change and bumps the sim's `nav_version`.
//! - **Each player's known grids** ([`NavState::known`]): the map and the footprints that player knows of
//!   ([`knowledge`]: its own, those it has seen and not seen gone, the map's resources). Units **plan** on their
//!   owner's: path requests and the staggered checks of phase 5c ([`serve()`]), a chase's line of sight, the nearest
//!   reachable point of a destination (`orders::destinations`). They are brought up to date once per tick, at its
//!   end ([`sync_known`]), and each change bumps that player's `PlayerKnowledge::version`. Built lazily: only for
//!   a player with a unit that walks on a grid.
//!
//! A unit without an owner (neutral; a defeated player's units, design change 0101) has nobody's knowledge to plan
//! on and sees nothing, so it plans on the true grids, with `nav_version` as its version.
//!
//! Invariant: between ticks the true grids match the objects, and each player's match what it knows, so a loaded
//! save plans exactly like the running game. A phase after 5b that adds or removes a blocking footprint must call
//! [`sync`] again, and nothing may change the objects or the vision after [`sync_known`] (`Sim::step` checks both
//! in debug builds).

mod checks;
pub mod knowledge;
mod misplaced;
mod serve;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;

use glam::Vec2;

use crate::defs::{Category, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::movement;
use crate::nav::{self, BlockerKind, Nav, NavClass, NavCounters, NavGrid, NavRules, Search, TileRect};
use crate::object::{Object, Placement};
use crate::orders::reach::Reach;
use crate::popup;
use crate::units::SU_PER_GU;
use crate::vision::Vision;

use knowledge::{PathKnowledge, known_blockers, own_blockers};
pub(crate) use misplaced::misplaced;
pub(crate) use serve::serve;

/// What one player plans its paths on (`systems/controls/behaviors.md > PathKnowledge`).
#[derive(Clone)]
struct Known {
    player: PlayerId,
    /// The footprints the player knows to block the ground, as [`known_blockers`] listed them at the latest
    /// [`sync_known`]: compared first, because most ticks change nothing.
    listed: Vec<TileRect>,
    /// The same footprints sorted, each once ([`canonical`]): what the grids are built from. It changes only when
    /// the player knows other tiles to be blocked, not when a map resource it assumed becomes one it remembers.
    blockers: Vec<TileRect>,
    /// The grids of `blockers`, while the player has a unit that walks on a grid.
    nav: Option<Nav>,
}

/// The navigation grids and the search scratch (`docs/movement.md > 3`: derived, and scratch).
pub(crate) struct NavState {
    /// The true grids: every blocking footprint. Collision reads these.
    nav: Nav,
    /// The blocking footprints `nav` was built from, in object id order.
    blockers: Vec<TileRect>,
    /// Each player's grids for planning, in the sim's player order.
    known: Vec<Known>,
    search: Search,
    /// The work of the latest phase 5c: output, never state.
    counters: NavCounters,
    /// How many sets of grids the latest step rebuilt (the true ones and each player's count one each): output.
    rebuilds: u32,
    /// The scratch of phase 5d (`movement::step`): spatial grids, rebuilt every tick.
    moves: movement::Scratch,
    /// The scratch and the counters of AttackMove's reach rule in phase 5a (`orders::reach`, design change 0230).
    reach: Reach,
}

impl NavState {
    /// No grids yet: a stand-in while a `Sim` is put together, replaced by [`NavState::build`].
    pub(crate) fn unbuilt(map: &Map) -> Self {
        let rules = NavRules::default();
        Self {
            nav: Nav::build(map, &[], &[], rules),
            blockers: Vec::new(),
            known: Vec::new(),
            search: Search::new(),
            counters: NavCounters::default(),
            rebuilds: 0,
            moves: movement::Scratch::default(),
            reach: Reach::default(),
        }
    }

    /// The grids of every unit type's class and size class: the true ones for `map` and the footprints of
    /// `objects`, and those of what each player of `knowledge` knows.
    pub(crate) fn build(
        map: &Map,
        objects: &BTreeMap<ObjectId, Object>,
        vision: &Vision,
        knowledge: &PathKnowledge,
    ) -> Self {
        let rules = NavRules::default();
        let blockers: Vec<TileRect> = blockers(objects, rules).collect();
        let nav = Nav::build(map, &blockers, &grid_keys(), rules);
        let own = own_blockers(objects, rules);
        let walkers = walkers(objects);
        let known = knowledge
            .players()
            .iter()
            .map(|player| {
                let mut listed = Vec::new();
                known_blockers(player, &own, vision, rules, &mut listed);
                let blockers = canonical(&listed);
                let nav = walkers.contains(&player.player()).then(|| stamped(&nav, &blockers));
                Known {
                    player: player.player(),
                    listed,
                    blockers,
                    nav,
                }
            })
            .collect();
        Self {
            nav,
            blockers,
            known,
            search: Search::new(),
            counters: NavCounters::default(),
            rebuilds: 0,
            moves: movement::Scratch::default(),
            reach: Reach::default(),
        }
    }

    /// The true grids.
    pub(crate) fn nav(&self) -> &Nav {
        &self.nav
    }

    /// The grids `player` plans on, if they are built: it is in the game and has a unit that walks on a grid.
    pub(crate) fn known(&self, player: PlayerId) -> Option<&Nav> {
        self.known.iter().find(|k| k.player == player)?.nav.as_ref()
    }

    /// Builds `player`'s grids now if they are missing: for a plan made before the end of the tick in which the
    /// player got its first walking unit (an order in the tick of a spawn).
    pub(crate) fn ensure_known(&mut self, player: PlayerId) {
        if let Some(known) = self.known.iter_mut().find(|k| k.player == player)
            && known.nav.is_none()
        {
            known.nav = Some(stamped(&self.nav, &known.blockers));
        }
    }

    /// The grids the units of `owner` plan on: its known ones, or the true ones for a unit without an owner (see
    /// the module docs) and for an owner whose grids aren't built.
    pub(crate) fn planning(&self, owner: Option<PlayerId>) -> &Nav {
        owner.and_then(|player| self.known(player)).unwrap_or(&self.nav)
    }

    /// The index into [`navs`] of the grids a unit of `owner` at `pos` plans on, and its grid there.
    /// A unit standing on a tile that its owner's knowledge blocks (possible only by a debug teleport onto a
    /// remembered footprint that is gone) plans on the true grid until it has seen where it stands.
    pub(crate) fn plan_grid(
        &self,
        owner: Option<PlayerId>,
        class: NavClass,
        size: u8,
        pos: Vec2,
    ) -> Option<(usize, &NavGrid)> {
        let truth = (0, self.nav.grid(class, size)?);
        let Some(player) = owner else {
            return Some(truth);
        };
        let mut index = 0;
        for known in &self.known {
            let Some(nav) = &known.nav else {
                continue;
            };
            index += 1;
            if known.player == player {
                let grid = nav.grid(class, size)?;
                return Some(if grid.is_blocked(grid.tile_of(pos)) {
                    truth
                } else {
                    (index, grid)
                });
            }
        }
        Some(truth)
    }

    /// The scratch of the reach rule, for phase 5a, which reads the grids meanwhile: [`NavState::put_reach`] brings
    /// it back.
    pub(crate) fn take_reach(&mut self) -> Reach {
        std::mem::take(&mut self.reach)
    }

    pub(crate) fn put_reach(&mut self, reach: Reach) {
        self.reach = reach;
    }

    /// The work of the reach rule in the latest phase 5a.
    pub(crate) fn reach_counters(&self) -> crate::orders::ReachCounters {
        self.reach.counters()
    }

    /// The true grids, and the scratch of the movement step that reads them.
    pub(crate) fn for_movement(&mut self) -> (&Nav, &mut movement::Scratch) {
        (&self.nav, &mut self.moves)
    }

    /// The work of the latest phase 5d.
    pub(crate) fn move_counters(&self) -> movement::MoveCounters {
        self.moves.counters()
    }

    /// The angle each unit turned in this tick's phase 5d (`movement::Turns`), for phase 5e.
    pub(crate) fn turns(&self) -> &movement::Turns {
        self.moves.turns()
    }

    pub(crate) fn counters(&self) -> NavCounters {
        self.counters
    }

    /// How many sets of grids the latest step rebuilt.
    pub(crate) fn rebuilds(&self) -> u32 {
        self.rebuilds
    }

    /// A new tick: its rebuilds count from zero.
    pub(crate) fn begin_tick(&mut self) {
        self.rebuilds = 0;
    }

    /// Whether the true grids were built from the footprints the objects have now.
    pub(crate) fn matches(&self, objects: &BTreeMap<ObjectId, Object>) -> bool {
        blockers(objects, self.nav.rules()).eq(self.blockers.iter().copied())
    }

    /// Checks the players' grids against a fresh build from the state (`testing::invariants`, K1): each player's
    /// footprints are what it knows now, its grids are built if it has a unit that walks on one, and built grids
    /// are exactly those of its footprints. The first difference.
    pub(crate) fn check_known(
        &self,
        objects: &BTreeMap<ObjectId, Object>,
        vision: &Vision,
        knowledge: &PathKnowledge,
    ) -> Result<(), String> {
        let rules = self.nav.rules();
        let own = own_blockers(objects, rules);
        let walkers = walkers(objects);
        let mut list = Vec::new();
        if !self
            .known
            .iter()
            .map(|k| k.player)
            .eq(knowledge.players().iter().map(|p| p.player()))
        {
            return Err("the known grids aren't those of the game's players".into());
        }
        for (known, player) in self.known.iter().zip(knowledge.players()) {
            let id = known.player.0;
            known_blockers(player, &own, vision, rules, &mut list);
            let blockers = canonical(&list);
            if known.blockers != blockers {
                return Err(format!(
                    "player {id}'s grids are of the footprints {:?}, it knows {blockers:?}",
                    known.blockers
                ));
            }
            match &known.nav {
                None if walkers.contains(&known.player) => {
                    return Err(format!("player {id} has units that walk, but no grids to plan on"));
                }
                Some(nav) if *nav != stamped(&self.nav, &blockers) => {
                    return Err(format!(
                        "player {id}'s grids differ from a fresh build of what it knows"
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Clone for NavState {
    /// The grids and the footprints; the scratch starts empty.
    fn clone(&self) -> Self {
        Self {
            nav: self.nav.clone(),
            blockers: self.blockers.clone(),
            known: self.known.clone(),
            search: Search::new(),
            counters: self.counters,
            rebuilds: self.rebuilds,
            moves: movement::Scratch::default(),
            reach: Reach::default(),
        }
    }
}

impl PartialEq for NavState {
    /// Derived data: two sims with equal state have equal grids, so the grids never make them differ.
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl fmt::Debug for NavState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let known: Vec<(PlayerId, usize, bool)> = self
            .known
            .iter()
            .map(|k| (k.player, k.blockers.len(), k.nav.is_some()))
            .collect();
        f.debug_struct("NavState")
            .field("grids", &self.nav.grids().len())
            .field("blockers", &self.blockers)
            .field("known (player, footprints, built)", &known)
            .finish_non_exhaustive()
    }
}

/// The `(class, size class)` of every unit type that walks on a grid (`docs/movement.md > 5.2`), from the type
/// definitions, so the grids don't depend on which units exist.
fn grid_keys() -> Vec<(NavClass, u8)> {
    ObjectKind::ALL
        .iter()
        .filter_map(|kind| {
            let def = kind.def();
            let class = def.unit.as_ref()?.base_def().nav_class()?;
            Some((class, nav::size_class(def.silhouette()?)))
        })
        .collect()
}

/// Every set of grids: the true ones first, then each player's that is built, in player order.
fn navs<'a>(truth: &'a Nav, known: &'a [Known]) -> Vec<&'a Nav> {
    std::iter::once(truth)
        .chain(known.iter().filter_map(|k| k.nav.as_ref()))
        .collect()
}

/// `listed` sorted, each footprint once: equal for equal sets of footprints, however they came to be known.
fn canonical(listed: &[TileRect]) -> Vec<TileRect> {
    let mut blockers = listed.to_vec();
    blockers.sort_unstable_by_key(|r| (r.origin.y, r.origin.x, r.w, r.h));
    blockers.dedup();
    blockers
}

/// The grids of `truth`'s map under `blockers` instead of its own.
fn stamped(truth: &Nav, blockers: &[TileRect]) -> Nav {
    let mut nav = truth.clone();
    nav.restamp(blockers);
    nav
}

/// The players with a unit that walks on a grid, ascending: those whose known grids are built.
fn walkers(objects: &BTreeMap<ObjectId, Object>) -> Vec<PlayerId> {
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

/// The footprint of an object of `kind` placed at `placement`, if it blocks surface units
/// (`docs/movement.md > 5.2`; design change 0038: map resources block). An underground structure's never does
/// (`BlockerKind::UndergroundExpansion`, plan decision T32): the one test, so that the true grids, what each player
/// knows of the obstacles (`knowledge`), the unit-start rule (`misplaced`) and the debug spawns all let units walk
/// over it.
pub(crate) fn blocking_footprint(kind: ObjectKind, placement: &Placement, rules: NavRules) -> Option<TileRect> {
    let def = kind.def();
    let blocker = match def.category {
        Category::Structure if def.structure.as_ref().is_some_and(|s| s.underground) => {
            BlockerKind::UndergroundExpansion
        }
        Category::Structure => BlockerKind::Structure,
        Category::Resource => BlockerKind::Resource,
        Category::Unit => return None,
    };
    rules
        .footprint_blocks(blocker)
        .then(|| TileRect::new(placement.origin, placement.w, placement.h))
}

/// The footprints that block surface units, in object id order.
fn blockers(objects: &BTreeMap<ObjectId, Object>, rules: NavRules) -> impl Iterator<Item = TileRect> + '_ {
    objects.values().filter_map(move |o| blocking(o, rules))
}

/// The footprint of the object `o` if it blocks surface units now: [`blocking_footprint`] by its kind, but for a
/// pop-up turret while it is fully lowered (slice M4-S2, `popup::blocks`; `factions/syndicate/structures/laser_turret.md
/// > Pop-up rules`: "A lowered one never blocks (units walk over it like an expansion)"). The one test of a live
/// object's footprint: the true grids, what its owner knows to block (`knowledge::own_blockers`) and the gate's spawn
/// and teleport checks ask it. A record's is [`remembered_blocking`].
pub(crate) fn blocking(o: &Object, rules: NavRules) -> Option<TileRect> {
    blocking_footprint(o.kind, o.placement.as_ref()?, rules).filter(|_| popup::blocks(o))
}

/// The footprint of a record of an object of `kind`, `owner`'s when last seen, at `placement`, if it blocks surface
/// units by what the player who remembers it knows: [`blocking_footprint`] by its kind, but for a neutral turret's,
/// which is always lowered (design change 0360; `popup::remembered_lowered`). A record of an owned turret is of one
/// seen, so not lowered, and blocks by its kind.
pub(crate) fn remembered_blocking(
    kind: ObjectKind,
    owner: Option<PlayerId>,
    placement: &Placement,
    rules: NavRules,
) -> Option<TileRect> {
    blocking_footprint(kind, placement, rules).filter(|_| !popup::remembered_lowered(kind, owner))
}

/// Whether a box centred on `pos` with half extents `half` overlaps the tiles of `rect`. Touching is free.
pub(crate) fn box_overlaps_rect(pos: Vec2, half: Vec2, rect: TileRect) -> bool {
    let lo = rect.origin.corner();
    let hi = lo + Vec2::new(rect.w as f32, rect.h as f32) * SU_PER_GU;
    pos.x - half.x < hi.x && pos.x + half.x > lo.x && pos.y - half.y < hi.y && pos.y + half.y > lo.y
}

/// Whether two boxes overlap (`docs/movement.md > 3`): strictly, so touching is free.
pub(crate) fn boxes_overlap(a: Vec2, a_half: Vec2, b: Vec2, b_half: Vec2) -> bool {
    (a.x - b.x).abs() < a_half.x + b_half.x && (a.y - b.y).abs() < a_half.y + b_half.y
}

/// Phase 5b (`docs/movement.md > 4`, `docs/movement.md > 5.7`): if a blocking footprint appeared, moved or went away,
/// rebuild the true grids and bump `nav_version`. The players' grids follow at the end of the tick ([`sync_known`]).
pub(crate) fn sync(state: &mut NavState, objects: &BTreeMap<ObjectId, Object>, nav_version: &mut u32) {
    if state.matches(objects) {
        return;
    }
    state.blockers.clear();
    state.blockers.extend(blockers(objects, state.nav.rules()));
    state.nav.restamp(&state.blockers);
    state.rebuilds += 1;
    *nav_version = nav_version.wrapping_add(1);
}

/// The end of a tick, after the vision and `PathKnowledge::observe` (`systems/controls/behaviors.md >
/// PathKnowledge`): each player whose known footprints changed gets a new version and, if it has a unit that walks
/// on a grid, its grids rebuilt: at most once per player and tick (`docs/movement.md > 5.7`).
pub(crate) fn sync_known(
    state: &mut NavState,
    objects: &BTreeMap<ObjectId, Object>,
    vision: &Vision,
    knowledge: &mut PathKnowledge,
) {
    let rules = state.nav.rules();
    let own = own_blockers(objects, rules);
    let walkers = walkers(objects);
    let mut list = Vec::new();
    for i in 0..state.known.len() {
        let Some(player) = knowledge
            .players()
            .get(i)
            .filter(|p| p.player() == state.known[i].player)
        else {
            continue;
        };
        known_blockers(player, &own, vision, rules, &mut list);
        let known = &mut state.known[i];
        let walks = walkers.binary_search(&known.player).is_ok();
        let mut changed = false;
        if known.listed != list {
            std::mem::swap(&mut known.listed, &mut list);
            let blockers = canonical(&known.listed);
            if known.blockers != blockers {
                known.blockers = blockers;
                knowledge.bump(known.player);
                changed = true;
            }
        }
        match (&mut known.nav, walks) {
            (Some(nav), true) if changed => {
                nav.restamp(&known.blockers);
                state.rebuilds += 1;
            }
            (None, true) => {
                known.nav = Some(stamped(&state.nav, &known.blockers));
                state.rebuilds += 1;
            }
            // Nobody plans on them: they are built again when somebody does.
            (Some(_), false) if changed => known.nav = None,
            _ => {}
        }
    }
}
