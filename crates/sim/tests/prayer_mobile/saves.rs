//! Saves, replays and the hash (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`): a Prayer-mobile's energy and the
//! Inspire and Suppress buffs under way (`Object::buffs`) are part of the state, a unit's buffs left out of a save while
//! it has none, so that the games without a cast save, and hash, as they did before them (the goldens).

use sc_sim::{OrderKind, Replay, Sim, SpawnAt, Target};

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

    /// Steps until tick `tick`.
    fn until(&mut self, tick: Tick) {
        while self.t.sim().tick() < tick {
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

/// Player 1's Prayer-mobile (1) charges to 175; it inspires its two Recruits (2, 3), which walk off, and in the next
/// tick suppresses its own Militant (4) and player 2's Peacekeeper (5) ahead of it as they fight. The game goes on alike
/// after a save at each stage, tick by tick: charged, both buffs under way, Suppress over, Inspire over; and its replay
/// plays back to the same state. A unit's buffs are saved while it has one.
#[test]
fn a_game_with_inspire_and_suppress_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = versus(Faction::Gdo, 30, 20)
        .spawn(PRAYER_MOBILE, Some(P1), SpawnAt::Point(Vec2::new(400.0, 640.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(400.0, 540.0)))
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(400.0, 740.0)))
        .spawn(MILITANT, Some(P1), SpawnAt::Point(Vec2::new(560.0, 640.0)))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(680.0, 640.0)),
        )
        .start();
    t.without_auto_targeting();
    let (pm, recruits, militant, peacekeeper) = (id(1), [id(2), id(3)], id(4), id(5));
    charge(&mut t, pm, 175);
    assert_eq!(saved_buffs(&t, recruits[0]), None, "none, not saved");
    let mut game = Game { t, saves: Vec::new() };
    game.save(); // Charged.
    let c = game.t.sim().tick() + 1;
    let inspire = cast(&game.t, pm, INSPIRE);
    game.t.at(c, P1, inspire);
    for unit in recruits {
        let to = Vec2::new(1400.0, game.t.sim().object(unit).expect("on the map").pos.y);
        game.t
            .order_at(c, P1, &[unit], OrderKind::Move(Target::Point(to)), false);
    }
    game.t
        .order_at(c, P1, &[militant], OrderKind::Attack(peacekeeper), false);
    game.t
        .order_at(c, P2, &[peacekeeper], OrderKind::Attack(militant), false);
    let suppress = cast(&game.t, pm, SUPPRESS);
    game.t.at(c + 1, P1, suppress);
    game.until(c + 5);
    assert_eq!(
        saved_buffs(&game.t, recruits[0]),
        Some(serde_json::json!([{"ability": "Inspire", "until": c + INSPIRE_TICKS}]))
    );
    assert_eq!(
        saved_buffs(&game.t, peacekeeper),
        Some(serde_json::json!([{"ability": "Suppress", "until": c + 1 + SUPPRESS_TICKS}]))
    );
    game.save(); // Both under way.
    game.until(c + 1 + SUPPRESS_TICKS);
    assert_eq!(saved_buffs(&game.t, peacekeeper), None, "over, not saved");
    assert!(saved_buffs(&game.t, recruits[1]).is_some());
    game.save(); // Suppress over.
    game.until(c + INSPIRE_TICKS);
    assert_eq!(saved_buffs(&game.t, recruits[1]), None, "over, not saved");
    game.save(); // Inspire over.
    game.until(c + INSPIRE_TICKS + 32);
    assert_eq!(game.saves.len(), 4);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(game.t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), game.t.sim().state_hash());
}
