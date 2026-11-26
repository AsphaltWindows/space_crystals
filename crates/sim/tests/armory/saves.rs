//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): the entries of a building's queue with
//! the units they reserve, the mark of a unit ready to leave, and a unit's augmentations are part of the state.

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

/// A game with Militants training, a Release queued, a trained Recruit waiting for the doorway behind Eject All's
/// stream, one cancelled, an augmented Recruit, and the Armory destroyed with reserved units inside goes on alike
/// after a save at every stage, tick by tick, and its replay plays back to the same state. (That the hash sees the
/// reservations and the augmentations is the sim's unit tests'.)
#[test]
fn a_game_with_training_goes_on_alike_after_a_save_and_in_a_replay() {
    // Player 1 keeps a Storage (2), and so isn't defeated with the Armory.
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 8, 4, Vec2::new(1150.0, 704.0));
    augment(&mut t, 2, recruits[7], RecruitKind::Both);
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, 8);
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Inside, one augmented.
    let start = game.t.sim().tick() + 1;
    game.t.at(start, P1, rally(&[A1], RALLY));
    game.t.at(start, P1, train(&[A1], RecruitKind::Plain));
    game.t.at(start, P1, release(&[A1], RecruitKind::Both));
    game.t.at(start, P1, train(&[A1], RecruitKind::Plain));
    game.t.at(start, P1, train(&[A1], RecruitKind::Plain));
    game.t.at(start + 2, P1, cancel(&[A1]));
    game.step();
    game.save(); // Four entries reserving their Recruits.
    game.until(10, "training", |t| t.sim().tick() == start + 10);
    game.save(); // Mid-training, one cancelled.
    game.t.at(start + 25, P1, eject_all(&[A1]));
    game.until(30, "a Militant ready", |t| t.sim().tick() == start + 31);
    assert!(
        game.t
            .sim()
            .held_unit(recruits[0])
            .is_some_and(|held| held.leaving.is_some())
    );
    game.save(); // A trained Recruit waiting for the doorway, Eject All's stream under way.
    game.until(200, "the Militant out", |t| !produced(t).is_empty());
    game.save();
    game.until(200, "the Release's Recruit out", |t| {
        exits(t).iter().any(|&(_, unit)| unit == recruits[7])
    });
    game.save();
    let gone = game.t.sim().tick() + 1;
    remove_at(&mut game.t, gone, A1);
    game.step();
    game.save(); // Evacuated, the last training refunded.
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 7);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
