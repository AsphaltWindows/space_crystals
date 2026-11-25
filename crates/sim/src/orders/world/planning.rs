//! What the behaviours read of path planning (`systems/controls/behaviors.md > PathKnowledge`): the version of
//! what a unit's owner knows, and the path a chase would get, for AttackMove's reach rule (`orders::reach`, design
//! change 0230).

use glam::Vec2;

use crate::ids::PlayerId;
use crate::nav::{self, Search};
use crate::paths::knowledge::PathKnowledge;

use super::{Unit, View, World};

impl<'a> World<'a> {
    /// The snapshot with the versions of what each player knows of the obstacles, as the last tick left them
    /// (`PlayerKnowledge::version`), and the sim's `nav_version` for the units without an owner. Without this, every
    /// version is 0.
    pub(crate) fn with_knowledge(mut self, knowledge: &'a PathKnowledge, nav_version: u32) -> Self {
        self.knowledge = Some(knowledge);
        self.nav_version = nav_version;
        self
    }

    /// The version of the obstacle knowledge the units of `owner` plan on (`paths::serve`): a path planned at
    /// another version may differ.
    pub(crate) fn knowledge_version(&self, owner: Option<PlayerId>) -> u32 {
        owner
            .zip(self.knowledge)
            .and_then(|(player, knowledge)| knowledge.version(player))
            .unwrap_or(self.nav_version)
    }

    /// The path `unit` plans when it goes after `target` (`MoveGoal::Object`, as phase 5c serves it,
    /// `paths::serve`): on the grid of what its owner knows, from where it stands to the point nearest the target's
    /// centre that it can stand on; straight for a unit that walks on no grid (an air unit).
    /// - `Err(())` if no path starts where the unit stands (a tile its grid blocks: the exit doorway of the building
    ///   that released it).
    /// - `Ok(None)` if the path is longer than `max_tiles` orthogonal steps: the search gives up there
    ///   (`nav::find_path_within`), so its cost is bounded by `max_tiles` and not by the map.
    ///
    /// One search on `search`, outside phase 5c's budget.
    pub(crate) fn chase_path(
        &self,
        unit: &Unit,
        target: &View,
        max_tiles: u32,
        search: &mut Search,
    ) -> Result<Option<Vec<Vec2>>, ()> {
        let def = unit.kind.def();
        let (unit_def, silhouette) = def.unit.as_ref().zip(def.silhouette()).ok_or(())?;
        let Some(class) = unit_def.base_def().nav_class() else {
            return Ok(Some(vec![unit.pos, target.pos]));
        };
        let half = silhouette / 2.0;
        let (_, grid) = self
            .nav
            .plan_grid(unit.owner, class, nav::size_class(silhouette), unit.pos)
            .ok_or(())?;
        let goal = grid
            .component(grid.tile_of(unit.pos))
            .and_then(|component| nav::reachable_point(grid, component, half, target.pos))
            .unwrap_or(target.pos);
        let max_cost = max_tiles.saturating_mul(nav::STEP_COST);
        nav::find_path_within(grid, search, half, unit.pos, goal, max_cost)
            .map(|path| path.map(|path| path.waypoints))
            .map_err(|_| ())
    }
}
