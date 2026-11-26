//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): an Altar's augmentation orders with the
//! units they reserve, and the Augmentations they give, are part of the state.

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

/// A game with the three orders queued, a Release among them, one cancelled, a unit augmented and out, and the Altar
/// destroyed mid-order with reserved units inside goes on alike after a save at every stage, tick by tick, and its
/// replay plays back to the same state.
#[test]
fn a_game_with_augmentation_orders_goes_on_alike_after_a_save_and_in_a_replay() {
    // Player 1 keeps a Storage (2), and so isn't defeated with the Altar.
    let scenario = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2));
    let (mut t, units) = spawned_on(scenario, &[RECRUIT, MILITANT, GUNNER, DISCIPLE, RECRUIT]);
    set_augmentation(&mut t, 2, units[3], RecruitKind::Metabolism);
    go_in(&mut t, &units);
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Inside, one augmented.
    let start = game.t.sim().tick() + 1;
    let (plain, metabolism) = (RecruitKind::Plain, RecruitKind::Metabolism);
    let commands = [
        rally(&[A1], RALLY),
        augment(&[A1], RECRUIT, plain, AugmentOrder::Both),
        release(&[A1], GUNNER, plain),
        augment(&[A1], DISCIPLE, metabolism, AugmentOrder::EnhancedConstitution),
        augment(&[A1], MILITANT, plain, AugmentOrder::ImprovedMetabolism),
    ];
    for command in commands {
        game.t.at(start, P1, command);
    }
    game.t.at(start + 2, P1, cancel(&[A1]));
    game.step();
    game.save(); // Four orders reserving their units.
    game.until(100, "augmenting", |t| t.sim().tick() == start + 50);
    game.save(); // Mid-order, one cancelled.
    game.until(400, "the Recruit out", |t| !produced(t).is_empty());
    game.save(); // Augmented and out.
    game.until(40, "the Gunner out", |t| !exits(t).is_empty());
    game.save(); // Released.
    game.until(100, "the Disciple under way", |t| t.sim().tick() == start + 120);
    let gone = game.t.sim().tick() + 1;
    remove_at(&mut game.t, gone, A1);
    game.step();
    game.save(); // Evacuated, the last order refunded.
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 6);
    assert_eq!(bank(&game.t, P1), BANK - 50);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
