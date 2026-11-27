//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): the scans under way are history, each
//! player's (`sc_sim::vision::scans`), and the cooldowns are the stations' state (`Object::cooldown`); a save leaves
//! out both while there are none, so that the games without a Scan save, and hash, as they did before them.

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
        self.t.step();
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

    fn run_until(&mut self, tick: Tick) {
        while self.t.sim().tick() < tick {
            self.step();
        }
    }
}

/// Player 1's saved scans and station `n`'s saved cooldown, each absent while there is none.
fn saved(t: &TestSim, n: u32) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    let json = serde_json::to_value(t.sim()).unwrap();
    let scans = json["vision"][0].get("scans").cloned();
    let objects = json["objects"].as_array().unwrap();
    let o = objects.iter().find(|o| o["id"] == n).expect("on the map");
    (scans, o.get("cooldown").cloned())
}

/// A game with two scans under way, one cast in the tick of a save, the cooldowns running, a scan over and its tiles
/// lingering, and a second Scan from the first station once its cooldown is over, goes on alike after a save at every
/// stage, tick by tick, and its replay plays back to the same state. The scans and the cooldowns are saved while there
/// are some.
#[test]
fn a_game_with_scans_and_cooldowns_goes_on_alike_after_a_save_and_in_its_replay() {
    let t = started(base());
    let mut game = Game { t, saves: Vec::new() };
    assert_eq!(saved(&game.t, S1), (None, None), "none, not saved");
    game.save(); // None.
    let a = game.t.sim().tick() + 1;
    game.t.at(a, P1, scan(&[S1], FAR));
    game.step();
    let (scans, cooldown) = saved(&game.t, S1);
    let until = a + SCAN_TICKS;
    assert_eq!(
        scans,
        Some(serde_json::json!([{"at": [FAR.x, FAR.y], "radius": 6.0, "until": until}]))
    );
    assert_eq!(cooldown, Some(serde_json::json!({"progress": 65_536, "total": 960})));
    game.save(); // One scan.
    game.run_until(a + 40);
    let b = game.t.sim().tick() + 1;
    game.t.at(b, P1, scan(&[S2], Vec2::new(40.0 * 64.0, 8.0 * 64.0)));
    game.step();
    game.save(); // Two.
    game.run_until(until + 2);
    game.save(); // One, the other's tiles lingering.
    game.run_until(b + SCAN_TICKS + FOG_DELAY_TICKS + 1);
    assert_eq!(saved(&game.t, S1).0, None, "over, not saved");
    game.save(); // Both over, the cooldowns still running.
    game.run_until(a + COOLDOWN_TICKS - 1);
    assert_eq!(saved(&game.t, S1), (None, None), "its cooldown over, not saved");
    let c = game.t.sim().tick() + 1;
    game.t.at(c, P1, scan(&[S1], Vec2::new(20.0 * 64.0, 28.0 * 64.0)));
    game.step();
    game.save(); // The first station's second.
    game.run_until(c + 40);
    assert_eq!(casts(&game.t).len(), 3);
    assert_eq!(game.saves.len(), 6);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}

/// A save is checked as it loads (`vision::load`): a scan off the map, of no radius, or over by the tick of the save,
/// or out of the order the scans end in, is refused, and the save as it was loads.
#[test]
fn a_save_with_a_scan_off_the_map_or_over_is_refused() {
    let mut t = started(base());
    let a = next(&mut t, scan(&[S1], FAR));
    t.run_until(a + 10);
    let b = next(&mut t, scan(&[S2], Vec2::new(40.0 * 64.0, 8.0 * 64.0)));
    let json = serde_json::to_value(t.sim()).unwrap();
    assert!(serde_json::from_value::<Sim>(json.clone()).is_ok());
    let edits: [(&str, serde_json::Value); 6] = [
        ("off the map", serde_json::json!([-10.0, 300.0])),
        ("off the map", serde_json::json!([300.0, 32.0 * 64.0 + 10.0])),
        ("no radius", serde_json::json!(0.0)),
        ("a radius not finite", serde_json::json!(f32::MAX as f64 * 4.0)),
        ("over", serde_json::json!(b)),
        ("out of order", serde_json::json!(b + SCAN_TICKS + 1)),
    ];
    for (what, value) in edits {
        let mut bad = json.clone();
        let scan = &mut bad["vision"][0]["scans"][0];
        match what {
            "off the map" => scan["at"] = value,
            "no radius" | "a radius not finite" => scan["radius"] = value,
            _ => scan["until"] = value,
        }
        assert!(serde_json::from_value::<Sim>(bad).is_err(), "{what}");
    }
}
