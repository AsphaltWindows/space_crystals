//! A lenient way to build a `Sim`, for tools that must show scenarios that aren't valid yet (the map editor).

use super::Sim;
use crate::ids::ObjectId;
use crate::player::Player;
use crate::rng::Rng;
use crate::scenario::{Scenario, locate};

impl Sim {
    /// A sim showing as much of `scenario` as can exist, for editors. Unlike [`Sim::try_new`] it accepts invalid
    /// scenarios: player problems are ignored, and every spawn whose footprint or point fits the map is created,
    /// even if footprints overlap or the owner is invalid. The second value maps each spawn, in order, to its
    /// object, or to `None` if it couldn't be placed. For a valid scenario the sim equals [`Sim::try_new`]'s.
    ///
    /// Never start a game from it: games must use [`Sim::try_new`].
    pub fn preview(scenario: &Scenario) -> (Sim, Vec<Option<ObjectId>>) {
        let players = scenario
            .players
            .iter()
            .map(|p| Player {
                id: p.id,
                name: p.name.clone(),
                faction: p.faction,
            })
            .collect();
        let mut sim = Self {
            tick: 0,
            map: scenario.map.clone(),
            players,
            objects: Default::default(),
            next_id: 1,
            rng: Rng::from_seed(scenario.seed),
            events: Vec::new(),
        };
        let ids = scenario
            .spawns
            .iter()
            .map(|s| {
                let (pos, placement) = locate(&sim.map, s.kind.def().footprint, s.at).ok()?;
                Some(sim.insert_object(s.kind, s.owner, pos, placement, s.settings.as_ref()))
            })
            .collect();
        sim.events.clear();
        (sim, ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::{Spawn, SpawnAt};
    use crate::scenarios;
    use crate::units::TilePos;

    fn patch(x: i32, y: i32) -> Spawn {
        Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, y)))
    }

    #[test]
    fn a_valid_scenario_previews_as_the_game_starts() {
        let scenario = scenarios::dev_skirmish();
        let (preview, ids) = Sim::preview(&scenario);
        let game = Sim::try_new(&scenario).unwrap();
        assert_eq!(preview.state_hash(), game.state_hash());
        assert_eq!(preview, game);
        let expected: Vec<Option<ObjectId>> = (1..=scenario.spawns.len() as u32).map(|i| Some(ObjectId(i))).collect();
        assert_eq!(ids, expected);
    }

    #[test]
    fn invalid_scenarios_show_everything_that_fits_the_map() {
        let mut scenario = scenarios::empty(6, 4);
        scenario.players[1].id = PlayerId(9); // out of range
        scenario.players[0].start_location = Some(TilePos::new(50, 50)); // off the map
        let mut owned = patch(4, 3);
        owned.owner = Some(PlayerId(1)); // resources can't be owned
        scenario.spawns = vec![patch(1, 1), patch(1, 1), patch(9, 9), owned];
        assert!(Sim::try_new(&scenario).is_err());

        let (sim, ids) = Sim::preview(&scenario);
        assert_eq!(ids, [Some(ObjectId(1)), Some(ObjectId(2)), None, Some(ObjectId(3))]);
        assert_eq!(sim.objects().count(), 3);
        assert_eq!(
            sim.object(ObjectId(1)).unwrap().pos,
            sim.object(ObjectId(2)).unwrap().pos
        );
        assert_eq!(sim.object(ObjectId(3)).unwrap().owner, Some(PlayerId(1)));
        assert_eq!(sim.players().len(), 2);
        assert!(sim.events().is_empty());
    }
}
