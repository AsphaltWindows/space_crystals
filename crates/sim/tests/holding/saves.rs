//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): the units inside a building, and the
//! mark of those Eject All lets out, are part of the state.

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

/// A game with Recruits walking in, inside, streaming out by Eject All, and evacuated from a destroyed Armory goes on
/// alike after a save at every stage, tick by tick, and its replay plays back to the same state.
#[test]
fn a_game_with_units_inside_goes_on_alike_after_a_save_and_in_a_replay() {
    // Player 1 keeps a Storage (2), and so isn't defeated with the Armory.
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    let mut game = Game { t, saves: Vec::new() };
    game.until(200, "some are in", |t| !inside(t, A1).is_empty());
    game.save(); // Some inside, some walking in.
    game.until(200, "all are in", |t| inside(t, A1).len() == 10);
    game.save();
    let eject = game.t.sim().tick() + 1;
    game.t.at(eject, P1, eject_all(&[A1]));
    game.step();
    game.save(); // One in the doorway, nine marked.
    game.until(200, "some are out", |t| exits(t).len() == 4);
    game.save();
    game.until(200, "all are out", |t| exits(t).len() == 10);
    let back = game.t.sim().tick() + 1;
    game.t.at(back, P1, enter(&recruits[..5], A1));
    game.until(200, "five are back in", |t| inside(t, A1).len() == 5);
    game.save();
    let gone = game.t.sim().tick() + 1;
    remove_at(&mut game.t, gone, A1);
    game.step();
    game.save(); // Evacuated.
    for _ in 0..32 {
        game.step();
    }
    let evacuated = (game.t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Evacuated { .. }))
        .count();
    assert_eq!((entries(&game.t).len(), exits(&game.t).len(), evacuated), (15, 10, 5));
    assert_eq!(game.saves.len(), 6);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
