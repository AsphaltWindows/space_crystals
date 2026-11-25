//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): the held store and the eject queues
//! are part of the state, and of nothing while they are empty.

use sc_sim::network::TRANSIT_DELAY_TICKS;
use sc_sim::{ProductionCommand, Replay, Sim, Target};

use super::*;

/// A game with units walking in, inside, queued and coming out goes on alike after a save at every stage, tick by
/// tick, and its replay plays back to the same state.
#[test]
fn a_game_with_units_inside_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = base().spawn(HIDEOUT, Some(P1), tile(16, 11)).start(); // 2
    t.without_auto_targeting().debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 500,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let rally = ProductionCommand::SetRally(Target::Point(Vec2::new(500.0, 300.0)));
    for (i, command) in [rally, ProductionCommand::Train(AGENT)].into_iter().enumerate() {
        let objects = vec![id(2)];
        t.at(2 + i as Tick, P1, Command::Production { objects, command });
    }
    let guards: Vec<ObjectId> = (0..6).map(|i| id(3 + i)).collect();
    for i in 0..6 {
        t.spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 580.0 - 40.0 * i as f32));
    }
    t.at(2, P1, enter(&guards, T1));
    // Ejects while some are still in transit (refused), then once they may leave.
    for tick in [20, 30, 60, 61, 62, 100] {
        t.at(tick, P1, eject(T1, GUARD));
    }
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    while t.sim().tick() < 300 {
        if [10, 25, 61, 63, 241].contains(&t.sim().tick()) {
            let saved = serde_json::to_string(t.sim()).unwrap();
            let loaded: Sim = serde_json::from_str(&saved).unwrap();
            assert_eq!(&loaded, t.sim(), "tick {}", t.sim().tick());
            saves.push((t.sim().tick(), loaded));
        }
        step_checked(&mut t);
        for (since, loaded) in &mut saves {
            loaded.step(t.replay().inputs.last().unwrap());
            assert_eq!(
                loaded.state_hash(),
                t.sim().state_hash(),
                "tick {}, saved in {since}",
                t.sim().tick()
            );
        }
    }
    // The game had every stage: units inside at a save, a queue at a save, an auto-ejected Agent.
    assert_eq!(entries(&t).len(), 6);
    assert_eq!(exits(&t, T1).len(), 5, "four Guards and the Agent");
    assert!(saves.len() == 5 && entries(&t)[5].0 + TRANSIT_DELAY_TICKS > 61);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// The held units and the eject queues are hashed, and left out of the hash and of the files while there are none
/// (the Director's decision for the goldens: a state without them is as it was before they existed).
#[test]
fn the_held_store_is_hashed_and_saved_only_while_it_holds_a_unit() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 2
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 520.0)); // 3
    run_checked(&mut t, 1);
    let json = serde_json::to_value(t.sim()).unwrap();
    assert!(json.get("held").is_none(), "no key while empty");
    let tunnel = &json["objects"][0]["tunnel"];
    assert!(tunnel.get("eject").is_none(), "{tunnel}");
    t.at(2, P1, enter(&[id(2), id(3)], T1));
    run_checked(&mut t, 40);
    assert_eq!(inside(&t), [id(2), id(3)]);
    let json = serde_json::to_value(t.sim()).unwrap();
    assert_eq!(json["held"].as_array().map(Vec::len), Some(2));
    // Two states that differ only in a held unit, or only in a queue, hash differently.
    let with = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut json = json.clone();
        edit(&mut json);
        serde_json::from_value::<Sim>(json).unwrap().state_hash()
    };
    let same = with(&|_| {});
    assert_eq!(same, t.sim().state_hash());
    assert_ne!(with(&|json| json["held"][0]["object"]["hp"] = 79.0.into()), same);
    assert_ne!(with(&|json| json["held"][1]["since"] = 1.into()), same);
    assert_ne!(with(&|json| json["held"][1]["entered"] = false.into()), same);
    let queued = serde_json::json!([{ "unit": 2, "order": null }]);
    assert_ne!(
        with(&|json| json["objects"][0]["tunnel"]["eject"] = queued.clone()),
        same
    );
    // A held unit's id is no object's, and below the next id.
    let clash = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut json = json.clone();
        edit(&mut json);
        serde_json::from_value::<Sim>(json).is_err()
    };
    assert!(
        clash(&|json| json["held"][0]["object"]["id"] = 1.into()),
        "an object's id"
    );
    assert!(clash(&|json| json["held"][0]["object"]["id"] = 3.into()), "twice");
    assert!(
        clash(&|json| json["held"][0]["object"]["id"] = 99.into()),
        "beyond the next id"
    );
}
