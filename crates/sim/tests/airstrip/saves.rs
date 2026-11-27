//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): a Sparrow's takeoff run
//! (`Mover::takeoff`) is part of the state.

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
}

/// A game with two Sparrows queued, each taking off to a rally point of its own (moved as the first comes out), goes
/// on alike after a save at every stage, tick by tick: mid-training, on the first's takeoff run, with an order given
/// on it waiting for its end, and with the second on its run; and its replay plays back to the same state (the run in
/// the hash: the mover's unit tests).
#[test]
fn a_game_with_a_sparrow_on_its_takeoff_run_goes_on_alike_after_a_save_and_in_a_replay() {
    let (t, order) = filled(2);
    let mut game = Game { t, saves: Vec::new() };
    let start = game.t.sim().tick() + 1;
    game.t.at(start, P1, rally(&[A1], Vec2::new(1400.0, 400.0)));
    game.t.at(start, P1, train(&[A1], RecruitKind::Plain));
    game.t.at(start, P1, train(&[A1], RecruitKind::Plain));
    game.step();
    game.save(); // Two Sparrows queued.
    game.t.at(start + FRAMES, P1, rally(&[A1], Vec2::new(300.0, 1200.0)));
    while produced(&game.t).is_empty() {
        assert!(game.t.sim().tick() < start + FRAMES + 2, "the first Sparrow out");
        game.step();
    }
    let first = order[0];
    game.step();
    assert!(game.t.mover(first.0).takeoff.is_some());
    game.save(); // On its run.
    game.t.order_at(
        game.t.sim().tick() + 1,
        P1,
        &[first],
        OrderKind::Move(Target::Point(Vec2::new(200.0, 1300.0))),
        false,
    );
    for _ in 0..8 {
        game.step();
    }
    assert!(
        game.t.mover(first.0).takeoff.is_some(),
        "the order waits for the run's end"
    );
    game.save(); // An order waiting.
    while produced(&game.t).len() < 2 {
        assert!(game.t.sim().tick() < start + 2 * FRAMES + 2, "the second Sparrow out");
        game.step();
    }
    for _ in 0..6 {
        game.step();
    }
    assert!(game.t.mover(order[1].0).takeoff.is_some());
    game.save(); // The second on its run.
    for _ in 0..48 {
        game.step();
    }
    assert_eq!(game.saves.len(), 4);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
