//! Saves, replays and the hash (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`): a Chop Shop's queue with the pairs of
//! Recruits it reserves, and a Seeker driving out of its doorway, its momentum with it, are part of the state.

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

/// A game with two Seekers queued on five Recruits, the first out and driving through its doorway to the rally point,
/// the second in training, goes on alike after a save at every stage, tick by tick, and its replay plays back to the
/// same state.
#[test]
fn a_game_with_seekers_goes_on_alike_after_a_save_and_in_a_replay() {
    let (t, order) = filled(5);
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Inside.
    let start = game.t.sim().tick() + 1;
    game.t.at(start, P1, rally(&[CS], RALLY));
    game.t.at(start, P1, train(&[CS], RecruitKind::Plain));
    game.t.at(start, P1, train(&[CS], RecruitKind::Plain));
    game.step();
    game.save(); // Two entries reserving two Recruits each.
    game.until(FRAMES, "the first Seeker out", |t| !produced(t).is_empty());
    game.step();
    game.step();
    assert_eq!(game.t.mover(order[0].0).exit, Some(CS), "in its doorway");
    assert!(game.t.mover(order[0].0).momentum.length() > 0.0, "driving");
    game.save(); // A Seeker driving out of its doorway.
    game.until(FRAMES, "at the rally point", |t| {
        t.pos(order[0].0).distance(RALLY) < 64.0
    });
    game.save(); // Mid-training, a Seeker at its rally point.
    game.until(FRAMES, "the second Seeker out", |t| produced(t).len() == 2);
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 4);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
