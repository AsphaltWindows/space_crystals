//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): a Cathedral's queue with the Recruits
//! it reserves, and the Disciples' energy, are part of the state.

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

/// A game with two Disciples queued, the first trained and regenerating its energy, the second mid-training, goes on
/// alike after a save at every stage, tick by tick, its energy included, and its replay plays back to the same state.
#[test]
fn a_game_with_disciples_goes_on_alike_after_a_save_and_in_a_replay() {
    let (t, order) = filled(3);
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Inside.
    let start = game.t.sim().tick() + 1;
    game.t.at(start, P1, rally(&[C1], RALLY));
    game.t.at(start, P1, train(&[C1], RecruitKind::Plain));
    game.t.at(start, P1, train(&[C1], RecruitKind::Plain));
    game.step();
    game.save(); // Two entries reserving their Recruits.
    game.until(FRAMES, "the first Disciple out", |t| !produced(t).is_empty());
    assert_eq!(energy(&game.t, order[0]), Some(50.0));
    game.save(); // A Disciple at its start.
    game.until(FRAMES, "regenerating", |t| t.sim().tick() == start + FRAMES + 100);
    game.save(); // Mid-training, a Disciple regenerating.
    let saved = energy(&game.t, order[0]).unwrap();
    assert!(saved > 50.0 && saved < 200.0, "{saved}");
    game.until(FRAMES, "the second Disciple out", |t| produced(t).len() == 2);
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 4);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
