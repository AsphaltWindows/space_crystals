//! Scenario editing helpers for tools such as the map editor: player slots and map resizing. They keep the
//! invariants a game needs where they can (player ids `1..=n`) and report what they had to drop.

use super::{PlayerSetup, Scenario, Spawn, locate};
use crate::ids::PlayerId;
use crate::map::Map;
use crate::player::Faction;

/// What [`Scenario::resize_map`] dropped because it fell outside the new map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResizeReport {
    /// The removed spawns, each with its index in `Scenario::spawns` before the resize.
    pub removed_spawns: Vec<(usize, Spawn)>,
    /// Players whose start location was cleared.
    pub cleared_start_locations: Vec<PlayerId>,
    /// Players whose camera start was cleared.
    pub cleared_camera_starts: Vec<PlayerId>,
}

impl ResizeReport {
    /// Nothing was dropped.
    pub fn is_empty(&self) -> bool {
        self.removed_spawns.is_empty()
            && self.cleared_start_locations.is_empty()
            && self.cleared_camera_starts.is_empty()
    }
}

impl Scenario {
    /// Adds a player slot numbered `n + 1`, so the ids stay `1..=n` (`core/entities.md > Player > PlayerNumber`).
    /// The slot has no start location and no camera start. `None`, changing nothing, if the ids are used up.
    pub fn add_player(&mut self, name: impl Into<String>, faction: Faction) -> Option<PlayerId> {
        let id = PlayerId(u8::try_from(self.players.len() + 1).ok()?);
        self.players.push(PlayerSetup {
            id,
            name: name.into(),
            faction,
            camera_start: None,
            start_location: None,
        });
        Some(id)
    }

    /// Removes player `id` and numbers the players above it one lower, so the ids stay `1..=n`. Spawns follow
    /// their owner's new number, and the removed player's spawns become neutral. Returns how many became
    /// neutral, or `None`, changing nothing, if no player has this id.
    pub fn remove_player(&mut self, id: PlayerId) -> Option<usize> {
        let index = self.players.iter().position(|p| p.id == id)?;
        self.players.remove(index);
        let renumber = |p: PlayerId| if p.0 > id.0 { PlayerId(p.0 - 1) } else { p };
        for p in &mut self.players {
            p.id = renumber(p.id);
        }
        let mut neutralized = 0;
        for s in &mut self.spawns {
            match s.owner {
                Some(owner) if owner == id => {
                    s.owner = None;
                    neutralized += 1;
                }
                Some(owner) => s.owner = Some(renumber(owner)),
                None => {}
            }
        }
        Some(neutralized)
    }

    /// Resizes the map, anchored at the top-left corner ([`Map::resize`]), and drops what falls outside it:
    /// spawns that fitted the old map but not the new one, and start locations and camera starts that were on
    /// the old map but are off the new one. Things that were already off the map stay, with their validation
    /// problem. Panics if a side is 0.
    pub fn resize_map(&mut self, width: u32, height: u32) -> ResizeReport {
        let fits = |map: &Map, s: &Spawn| locate(map, s.kind.def().footprint, s.at).is_ok();
        let fitted: Vec<bool> = self.spawns.iter().map(|s| fits(&self.map, s)).collect();
        let starts_on: Vec<bool> = self
            .players
            .iter()
            .map(|p| p.start_location.is_some_and(|t| self.map.in_bounds(t)))
            .collect();
        let cameras_on: Vec<bool> = self
            .players
            .iter()
            .map(|p| p.camera_start.is_some_and(|c| self.map.contains_point(c)))
            .collect();
        self.map.resize(width, height);

        let mut report = ResizeReport::default();
        for (i, (spawn, fitted)) in std::mem::take(&mut self.spawns).into_iter().zip(fitted).enumerate() {
            if fitted && !fits(&self.map, &spawn) {
                report.removed_spawns.push((i, spawn));
            } else {
                self.spawns.push(spawn);
            }
        }
        for ((p, start_on), camera_on) in self.players.iter_mut().zip(starts_on).zip(cameras_on) {
            if start_on && p.start_location.is_some_and(|t| !self.map.in_bounds(t)) {
                p.start_location = None;
                report.cleared_start_locations.push(p.id);
            }
            if camera_on && p.camera_start.is_some_and(|c| !self.map.contains_point(c)) {
                p.camera_start = None;
                report.cleared_camera_starts.push(p.id);
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::ObjectKind;
    use crate::scenario::{ScenarioProblem, SpawnAt};
    use crate::scenarios;
    use crate::units::TilePos;

    fn spawn(kind: ObjectKind, owner: Option<u8>, x: i32, y: i32) -> Spawn {
        Spawn {
            kind,
            owner: owner.map(PlayerId),
            at: SpawnAt::Tile(TilePos::new(x, y)),
        }
    }

    fn ids(s: &Scenario) -> Vec<u8> {
        s.players.iter().map(|p| p.id.0).collect()
    }

    #[test]
    fn added_players_are_numbered_on() {
        let mut s = scenarios::empty(8, 8);
        assert_eq!(s.add_player("Third", Faction::Cults), Some(PlayerId(3)));
        assert_eq!(ids(&s), [1, 2, 3]);
        let third = &s.players[2];
        assert_eq!((third.name.as_str(), third.faction), ("Third", Faction::Cults));
        assert_eq!((third.start_location, third.camera_start), (None, None));
        assert!(s.validate().is_empty());
        while s.players.len() < 255 {
            s.add_player("More", Faction::Gdo).unwrap();
        }
        assert_eq!(s.add_player("Too many", Faction::Gdo), None);
        assert_eq!(s.players.len(), 255);
    }

    #[test]
    fn removing_a_player_renumbers_the_rest_and_their_objects() {
        let mut s = scenarios::empty(8, 8);
        s.add_player("Third", Faction::Syndicate);
        // Owned resources are invalid, but the owners still follow the renumbering.
        s.spawns = vec![
            spawn(ObjectKind::CrystalPatch, Some(1), 0, 0),
            spawn(ObjectKind::CrystalPatch, Some(2), 1, 0),
            spawn(ObjectKind::CrystalPatch, Some(3), 2, 0),
            spawn(ObjectKind::CrystalPatch, None, 3, 0),
        ];
        assert_eq!(s.remove_player(PlayerId(2)), Some(1));
        assert_eq!(ids(&s), [1, 2]);
        assert_eq!(s.players[1].name, "Third");
        let owners: Vec<Option<u8>> = s.spawns.iter().map(|s| s.owner.map(|p| p.0)).collect();
        assert_eq!(owners, [Some(1), None, Some(2), None]);
        assert_eq!(s.remove_player(PlayerId(7)), None);
        assert_eq!(ids(&s), [1, 2]);
    }

    #[test]
    fn resizing_drops_what_falls_outside_and_reports_it() {
        let mut s = scenarios::empty(10, 8);
        s.players[0].start_location = Some(TilePos::new(1, 1));
        s.players[0].camera_start = Some(Vec2::new(600.0, 100.0)); // tile (9, 1)
        s.players[1].start_location = Some(TilePos::new(8, 6));
        s.spawns = vec![
            spawn(ObjectKind::CrystalPatch, None, 2, 2),
            spawn(ObjectKind::SupplyDeliveryStation, None, 5, 1), // covers x 5..=6
            spawn(ObjectKind::CrystalPatch, None, 1, 6),
        ];
        let report = s.resize_map(6, 7);
        assert_eq!((s.map.width(), s.map.height()), (6, 7));
        let removed: Vec<usize> = report.removed_spawns.iter().map(|(i, _)| *i).collect();
        assert_eq!(removed, [1]);
        assert_eq!(report.removed_spawns[0].1.kind, ObjectKind::SupplyDeliveryStation);
        assert_eq!(s.spawns.len(), 2);
        assert_eq!(report.cleared_start_locations, [PlayerId(2)]);
        assert_eq!(report.cleared_camera_starts, [PlayerId(1)]);
        assert_eq!(s.players[0].start_location, Some(TilePos::new(1, 1)));
        assert!(s.validate().is_empty());
        assert!(s.resize_map(6, 7).is_empty(), "nothing more to drop");
    }

    #[test]
    fn resizing_keeps_what_was_already_off_the_map() {
        let mut s = scenarios::empty(4, 4);
        s.spawns = vec![spawn(ObjectKind::CrystalPatch, None, 9, 9)];
        s.players[0].start_location = Some(TilePos::new(-1, 0));
        let report = s.resize_map(3, 3);
        assert!(report.is_empty());
        assert_eq!(s.spawns.len(), 1);
        assert!(matches!(s.validate()[0], ScenarioProblem::StartLocationOffMap { .. }));
    }
}
