//! The `Sim`'s navigation side: the true grids, each player's path knowledge and planning grids, and the counters
//! of the path and movement phases (`paths`, `docs/movement.md > 5`).

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::Sim;
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::movement::MoveCounters;
use crate::nav::{Nav, NavClass, NavCounters, NavGrid};
use crate::network::Held;
use crate::object::Object;
use crate::paths::knowledge::PathKnowledge;
use crate::paths::{self, NavState};

/// The keys of the grids the units in a game walk on (`paths::walking_keys`), the held ones among them: what a game's
/// grids are built for at its start and on a load (`ARCHITECTURE.md > Object model > Units off the map: the held
/// store`: what must count held units says so).
pub(super) fn walking_keys(
    objects: &BTreeMap<ObjectId, Object>,
    held: &BTreeMap<ObjectId, Held>,
) -> Vec<(NavClass, u8)> {
    paths::walking_keys(objects.values(), held.values().map(|unit| &unit.object))
}

impl Sim {
    /// The starting objects are in: every player knows the neutral objects the map placed, from records of them
    /// (design change 0159: "known from the start (pathing and memory), like terrain", `Vision::seed`), and nothing
    /// of the others' starting layouts; tick 0 already shows what they see, and remembers what it saw; and the grids
    /// are built from all of it (`paths::knowledge`), for the units the game starts with.
    pub(super) fn settle_start(&mut self) {
        self.vision
            .seed(self.objects.values().filter(|o| o.owner.is_none()), self.tick);
        self.vision.update(&self.map, self.objects.values(), self.tick);
        self.vision.remember(self.tick, |id| self.objects.get(&id));
        self.path_knowledge = PathKnowledge::start(self.players.iter().map(|p| p.id));
        let keys = walking_keys(&self.objects, &self.held);
        self.nav = NavState::build(&self.map, &self.objects, &keys, &self.vision, &self.path_knowledge);
        // Scenario spawns emit no events (`ARCHITECTURE.md > Commands, events and replays`).
        self.events.clear();
    }

    /// Slice M5-Z2 (`paths`, *Which grids*): the grids of every unit in the game, on the map or held off it, and of
    /// every unit a producer has queued, those that are missing built now (`NavState::cover`). The grid sync steps
    /// call it, phase 5b and the end of the tick (`sim/phase.rs`), so that every unit that came into the game since
    /// has its grid before it moves or plans, and the spot a queued or held unit comes out at is checked on its grid.
    pub(super) fn cover_grids(&mut self) {
        let keys = walking_keys(&self.objects, &self.held);
        self.nav.cover(&self.map, keys);
    }

    /// The grid a unit of `kind` walks on, built now if it is missing (slice M5-Z2, `NavState::cover`): for a unit
    /// that comes into the game, or comes to walk on another grid, between two grid syncs, whose grid may be read
    /// meanwhile (a debug spawn, `Sim::apply_command`; a change of kind, `Sim::change_kind`).
    pub(in crate::sim) fn cover_kind(&mut self, kind: ObjectKind) {
        self.nav.cover(&self.map, paths::key_of(kind));
    }

    /// The grids of every unit type there is, as every game had them before slice M5-Z2, built now: for the tests
    /// that compare grids built late with grids built from the start.
    #[cfg(test)]
    pub(crate) fn build_every_grid(&mut self) {
        self.nav.cover(&self.map, paths::every_key());
    }

    /// The true navigation grids (`docs/movement.md > 5.2`): derived from the map and every blocking footprint,
    /// and in step with them between ticks. Units collide on these; they plan on [`Sim::known_nav`].
    pub fn nav(&self) -> &Nav {
        self.nav.nav()
    }

    /// The grids `player`'s units plan their paths on (`systems/controls/behaviors.md > PathKnowledge`): the map
    /// and the footprints the player knows of, as of the end of the latest tick. `None` for a player who isn't in
    /// the game or has no unit that walks on a grid (they are built lazily).
    pub fn known_nav(&self, player: PlayerId) -> Option<&Nav> {
        self.nav.known(player)
    }

    /// What each player knows for its paths beyond its vision's records: the version of its knowledge, and the map
    /// resources an older save still has it assume (`systems/controls/behaviors.md > PathKnowledge`).
    pub fn path_knowledge(&self) -> &PathKnowledge {
        &self.path_knowledge
    }

    /// The grids and whose they are, for the tests of `paths`.
    #[cfg(test)]
    pub(crate) fn nav_state(&self) -> &NavState {
        &self.nav
    }

    /// The true grid of `(class, size)`: the game's ([`Sim::nav`]), or, with no unit in the game that walks on it, the
    /// one the first would build (slice M5-Z2, `NavState::fresh_grid`), as of the end of the latest tick: for the
    /// tests and the tools that look at the ground of a class whose units the game may not have.
    pub fn true_grid(&self, class: NavClass, size: u8) -> Cow<'_, NavGrid> {
        match self.nav.nav().grid(class, size) {
            Some(grid) => Cow::Borrowed(grid),
            None => Cow::Owned(self.nav.fresh_grid(&self.map, class, size)),
        }
    }

    /// Checks each player's planning grids against a fresh build from the state (`testing::invariants`, K1).
    pub(crate) fn check_path_knowledge(&self) -> Result<(), String> {
        self.nav.check_known(&self.objects, &self.vision, &self.path_knowledge)
    }

    /// How many sets of navigation grids the most recent [`Sim::step`] rebuilt, the true ones and each player's
    /// counted once each: output for tests and timings, not state.
    pub fn nav_rebuilds(&self) -> u32 {
        self.nav.rebuilds()
    }

    /// How often the true grids were rebuilt since the game started (phase 5b). A player's units go by the version
    /// of what that player knows instead (`PathKnowledge::version`).
    pub fn nav_version(&self) -> u32 {
        self.nav_version
    }

    /// The path searches of the most recent [`Sim::step`]: output for tests and overlays, not state.
    pub fn nav_counters(&self) -> NavCounters {
        self.nav.counters()
    }

    /// The work of AttackMove's reach rule in the most recent [`Sim::step`] (`orders::reach`, design change 0230): the
    /// paths planned to judge enemies. Output for tests and timings, not state.
    pub fn reach_counters(&self) -> crate::orders::ReachCounters {
        self.nav.reach_counters()
    }

    /// The collision work of the most recent [`Sim::step`]'s movement phase: output for tests and overlays, not
    /// state.
    pub fn move_counters(&self) -> MoveCounters {
        self.nav.move_counters()
    }
}
