//! The `Sim`'s navigation side: the true grids, each player's path knowledge and planning grids, and the counters
//! of the path and movement phases (`paths`, `docs/movement.md > 5`).

use super::Sim;
use crate::ids::PlayerId;
use crate::movement::MoveCounters;
use crate::nav::{Nav, NavCounters};
use crate::paths::NavState;
use crate::paths::knowledge::PathKnowledge;

impl Sim {
    /// The starting objects are in: every player knows the neutral objects the map placed, from records of them
    /// (design change 0159: "known from the start (pathing and memory), like terrain", `Vision::seed`), and nothing
    /// of the others' starting layouts; tick 0 already shows what they see, and remembers what it saw; and the grids
    /// are built from all of it (`paths::knowledge`).
    pub(super) fn settle_start(&mut self) {
        self.vision
            .seed(self.objects.values().filter(|o| o.owner.is_none()), self.tick);
        self.vision.update(&self.map, self.objects.values());
        self.vision.remember(self.tick, |id| self.objects.get(&id));
        self.path_knowledge = PathKnowledge::start(self.players.iter().map(|p| p.id));
        self.nav = NavState::build(&self.map, &self.objects, &self.vision, &self.path_knowledge);
        // Scenario spawns emit no events (`ARCHITECTURE.md > Commands, events and replays`).
        self.events.clear();
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

    /// The collision work of the most recent [`Sim::step`]'s movement phase: output for tests and overlays, not
    /// state.
    pub fn move_counters(&self) -> MoveCounters {
        self.nav.move_counters()
    }
}
