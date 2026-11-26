//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): a Temple's research queued and in
//! progress, its staff, and the technologies a player has completed (`Player::techs`) are part of the state; a player's
//! technologies are left out of a save while they have none, so that the games without research save, and hash, as
//! they did before it (the goldens).

use sc_sim::{Replay, Sim};

use super::*;

/// The game of the test below: its sim, the copies loaded from its saves, each with the tick it was saved in.
struct Game {
    t: TestSim,
    saves: Vec<(Tick, Sim)>,
}

impl Game {
    /// Saves the game and loads the save, which must be the same game.
    fn save(&mut self) {
        let saved = serde_json::to_string(self.t.sim()).unwrap();
        let loaded: Sim = serde_json::from_str(&saved).unwrap();
        assert_eq!(&loaded, self.t.sim(), "tick {}", self.t.sim().tick());
        self.saves.push((self.t.sim().tick(), loaded));
    }

    /// One tick of the game and of every loaded copy, on the same inputs: they must stay the same game.
    fn step(&mut self) {
        step_checked(&mut self.t);
        let inputs = self.t.replay().inputs.last().unwrap();
        for (since, loaded) in &mut self.saves {
            loaded.step(inputs);
            assert_eq!(
                loaded.state_hash(),
                self.t.sim().state_hash(),
                "tick {}, saved in {since}",
                self.t.sim().tick()
            );
        }
    }

    /// Steps until `done` holds, at most `limit` ticks.
    fn until(&mut self, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
        let start = self.t.sim().tick();
        while !done(&self.t) {
            assert!(self.t.sim().tick() < start + limit, "{what}");
            self.step();
        }
    }
}

/// Player 1's saved technologies (`Player::techs`), absent while they have none.
fn saved_techs(t: &TestSim) -> Option<serde_json::Value> {
    let json = serde_json::to_value(t.sim()).unwrap();
    let players = json["players"].as_array().unwrap();
    let p1 = players.iter().find(|p| p["id"] == P1.0).unwrap();
    p1.get("techs").cloned()
}

/// A game with both lines' Level 1s queued, one cancelled and queued again, Enhanced Constitution Level 1 completed
/// with an Enhanced Constitution Recruit on the map, the staff ejected mid-research and back, and the Temple destroyed
/// with research queued, goes on alike after a save at every stage, tick by tick, and its replay plays back to the same
/// state. A player's technologies are saved once they have one.
#[test]
fn a_game_with_research_goes_on_alike_after_a_save_and_in_a_replay() {
    let (mut t, staff) = staffed(3);
    let now = t.sim().tick() + 1;
    let recruit = id(next_id(&t));
    t.spawn_unit_at(now, RECRUIT, P1, Vec2::new(1500.0, 300.0));
    set_augmentation(&mut t, now + 1, recruit, RecruitKind::Constitution);
    run_checked(&mut t, now + 1);
    let mut game = Game { t, saves: Vec::new() };
    assert_eq!(saved_techs(&game.t), None, "none, not saved");
    game.save(); // Staffed, nothing queued.
    let start = game.t.sim().tick() + 1;
    game.t.at(start, P1, research(&[T1], EC1));
    game.t.at(start, P1, research(&[T1], IM1));
    game.t.at(start + 5, P1, cancel(&[T1]));
    game.t.at(start + 6, P1, research(&[T1], IM1));
    game.until(10, "queued", |t| t.sim().tick() == start + 6);
    game.save(); // Both queued, one of them again.
    game.until(100, "under way", |t| t.sim().tick() == start + 100);
    game.save(); // In progress.
    game.until(L1_FRAMES, "Enhanced Constitution Level 1", |t| !completed(t).is_empty());
    assert_eq!(hp(&game.t, recruit), (80.0, 80.0));
    assert_eq!(
        saved_techs(&game.t),
        Some(serde_json::json!(["EnhancedConstitutionL1"]))
    );
    game.save(); // Completed, the next one under way.
    let out = game.t.sim().tick() + 1;
    game.t.at(out, P1, eject_all(&[T1]));
    game.until(100, "the staff out", |t| exits(t).len() == 3);
    game.save(); // The staff out, the research paused.
    let back = game.t.sim().tick() + 1;
    game.t.at(back, P1, enter(&staff, T1));
    game.until(300, "the staff back", |t| t.sim().staff(T1) == 3);
    game.save(); // Back at it.
    game.step();
    let gone = game.t.sim().tick() + 1;
    remove_at(&mut game.t, gone, T1);
    game.step();
    game.save(); // Destroyed: refunded, the staff evacuated.
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 7);
    assert_eq!(techs(&game.t, P1), [EC1], "completed research is permanent");
    assert_eq!(bank(&game.t, P1), BANK - L1_PRICE, "the one in progress refunded");
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
