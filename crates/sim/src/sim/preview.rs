//! A lenient way to build a `Sim`, for tools that must show scenarios that aren't valid yet (the map editor).

use super::Sim;
use crate::ids::ObjectId;
use crate::scenario::{Scenario, locate_oriented};

impl Sim {
    /// A sim showing as much of `scenario` as can exist, for editors. Unlike [`Sim::try_new`] it accepts invalid
    /// scenarios: setup problems are ignored, and every pre-placed object and Starting Layout object whose footprint
    /// or point fits the map is created, even if footprints overlap or the owner is invalid. A slot's objects
    /// belong to the player who takes it, and are left out if nobody does. The second value maps each pre-placed
    /// object, in order, to its object, or to `None` if it couldn't be placed. For a valid scenario the sim equals
    /// [`Sim::try_new`]'s, game options included.
    ///
    /// Never start a game from it: games must use [`Sim::try_new`].
    pub fn preview(scenario: &Scenario) -> (Sim, Vec<Option<ObjectId>>) {
        let mut sim = Self::empty(scenario);
        let setup = &scenario.setup;
        let ids = scenario
            .map
            .spawns
            .iter()
            .map(|s| {
                let owner = match s.owner {
                    None => None,
                    Some(slot) => Some(setup.player_at(slot)?),
                };
                let (pos, placement) = locate_oriented(&sim.map, s.kind.def().footprint, s.at, s.orientation).ok()?;
                Some(sim.insert_object(s.kind, owner, pos, placement, s.settings.as_ref()))
            })
            .collect();
        for (player, p) in setup.player_ids().zip(&setup.players) {
            let Some(slot) = scenario.map.slot(p.slot) else {
                continue;
            };
            for o in slot.layouts.get(p.faction) {
                if let Ok((pos, placement)) = locate_oriented(&sim.map, o.kind.def().footprint, o.at, o.orientation) {
                    sim.insert_object(o.kind, Some(player), pos, placement, None);
                }
            }
        }
        sim.events.clear();
        (sim, ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::ids::{PlayerId, SlotId};
    use crate::scenario::{LayoutObject, Spawn, SpawnAt};
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
        let count = scenario.map.spawns.len() as u32;
        let expected: Vec<Option<ObjectId>> = (1..=count).map(|i| Some(ObjectId(i))).collect();
        assert_eq!(ids, expected);
    }

    #[test]
    fn invalid_scenarios_show_everything_that_fits_the_map() {
        let mut scenario = scenarios::empty(6, 4);
        scenario.setup.players[1].slot = SlotId(9); // not a slot of the map
        scenario.map.slots[0].start = TilePos::new(50, 50); // off the map
        let mut owned = patch(4, 3);
        owned.owner = Some(SlotId(1)); // resources can't be owned
        let mut unowned = patch(5, 3);
        unowned.owner = Some(SlotId(2)); // nobody takes slot 2
        scenario.map.spawns = vec![patch(1, 1), patch(1, 1), patch(9, 9), owned, unowned];
        // A layout object of a resource kind (invalid: its player would own it), in player 1's layout.
        scenario.map.slots[0]
            .layouts
            .get_mut(scenario.setup.players[0].faction)
            .push(LayoutObject {
                kind: ObjectKind::CrystalPatch,
                at: SpawnAt::Tile(TilePos::new(0, 0)),
                orientation: Default::default(),
            });
        assert!(Sim::try_new(&scenario).is_err());

        let (sim, ids) = Sim::preview(&scenario);
        assert_eq!(
            ids,
            [Some(ObjectId(1)), Some(ObjectId(2)), None, Some(ObjectId(3)), None]
        );
        assert_eq!(sim.objects().count(), 4);
        assert_eq!(
            sim.object(ObjectId(1)).unwrap().pos,
            sim.object(ObjectId(2)).unwrap().pos
        );
        assert_eq!(sim.object(ObjectId(3)).unwrap().owner, Some(PlayerId(1)));
        assert_eq!(
            sim.object(ObjectId(4)).unwrap().owner,
            Some(PlayerId(1)),
            "the layout object"
        );
        assert_eq!(sim.players().len(), 2);
        assert!(sim.events().is_empty());
    }
}
