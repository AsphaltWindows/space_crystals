//! Saves, replays and the hash (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`): a Shrine's energy, its participants
//! and the buffs under way (`Object::buffs`) are part of the state; a unit's buffs are left out of a save while it has
//! none, so that the games without a cast save, and hash, as they did before them (the goldens).

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

/// Unit `unit`'s saved buffs, absent while it has none.
fn saved_buffs(t: &TestSim, unit: ObjectId) -> Option<serde_json::Value> {
    let json = serde_json::to_value(t.sim()).unwrap();
    let objects = json["objects"].as_array().unwrap();
    let o = objects.iter().find(|o| o["id"] == unit.0).expect("on the map");
    o.get("buffs").cloned()
}

/// A game with a charged Shrine, a Q on a wounded Militant and a W on a Disciple under way, a recast, the participants
/// ejected and back, and the Shrine destroyed with energy goes on alike after a save at every stage, tick by tick, and
/// its replay plays back to the same state. A unit's buffs are saved while it has one.
#[test]
fn a_game_with_casts_goes_on_alike_after_a_save_and_in_a_replay() {
    let (mut t, participants) = manned(8);
    let militant = spawn(&mut t, MILITANT, P1, Vec2::new(900.0, 600.0));
    let disciple = spawn(&mut t, DISCIPLE, P1, Vec2::new(900.0, 680.0));
    wound(&mut t, &[(militant, 12.0)]);
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Manned, charging.
    game.until(1000, "charged", |t| energy(t, S1) >= 125 * ONE);
    assert_eq!(saved_buffs(&game.t, militant), None, "none, not saved");
    let c = game.t.sim().tick() + 1;
    game.t.at(c, P1, cast(&[S1], Q, AT));
    game.t.at(c, P1, cast(&[S1], W, AT));
    game.until(10, "cast", |t| t.sim().tick() == c + 5);
    let saved = saved_buffs(&game.t, militant).expect("saved");
    assert_eq!(
        saved,
        serde_json::json!([{"ability": "HealthRegeneration", "until": c + Q_TICKS}])
    );
    assert!(saved_buffs(&game.t, disciple).is_some());
    game.save(); // Both under way.
    let out = game.t.sim().tick() + 1;
    game.t.at(out, P1, eject_all(&[S1]));
    game.until(200, "the participants out", |t| exits(t).len() == 8);
    game.save(); // Out, the energy kept.
    let back = game.t.sim().tick() + 1;
    game.t.at(back, P1, enter(&participants, S1));
    game.until(300, "the participants back", |t| inside(t, S1).len() == 8);
    game.until(1000, "charged again", |t| energy(t, S1) >= 75 * ONE);
    let recast = game.t.sim().tick() + 1;
    game.t.at(recast, P1, cast(&[S1], Q, AT));
    game.step();
    game.save(); // Recast.
    game.until(Q_TICKS, "over", |t| buffs(t, militant).is_empty());
    assert_eq!(saved_buffs(&game.t, militant), None, "over, not saved");
    game.save(); // Over.
    let gone = game.t.sim().tick() + 1;
    remove_at(&mut game.t, gone, S1);
    game.step();
    game.save(); // Destroyed: its participants evacuated, its energy gone.
    for _ in 0..32 {
        game.step();
    }
    assert_eq!(game.saves.len(), 6);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
