//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`): a neutral expansion's decay
//! (`Object::decay`) is part of the state, hashed and saved only while it is set.

use sc_sim::{Replay, Sim};

use super::*;

/// [`base`] with player 1's second Tunnel (3) far east and its Agent 4 south of [`T1`], with 10 Supplies: Tunnel 1
/// is removed in tick 3, and the Agent builds a Tunnel in its place from tick 4, which takes the Hideout back.
fn lost_and_rebuilt() -> TestSim {
    let mut t = base()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 3
        .spawn(AGENT, Some(P1), point(6, 16)) // 4
        .start();
    t.without_auto_targeting()
        .at(2, P1, bank(P1, 0, 10))
        .at(3, P1, remove(T1))
        .at(4, P1, build(&[4], at(4, 8)));
    t
}

/// A game in which a Hideout turns neutral, decays and is taken back goes on alike after a save at every stage, tick
/// by tick, and its replay plays back to the same state.
#[test]
fn a_game_with_a_decaying_hideout_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = lost_and_rebuilt();
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    while completed_at(&t).is_none_or(|done| t.sim().tick() < done + 20) {
        assert!(t.sim().tick() < 1000, "the new Tunnel completes");
        if [2, 3, 4, 100, 300].contains(&t.sim().tick()) || completed_at(&t).is_some_and(|done| t.sim().tick() == done)
        {
            let saved = serde_json::to_string(t.sim()).unwrap();
            let loaded: Sim = serde_json::from_str(&saved).unwrap();
            assert_eq!(&loaded, t.sim(), "tick {}", t.sim().tick());
            saves.push((t.sim().tick(), loaded));
        }
        step(&mut t);
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
    assert_eq!((owner(&t, H1), object(&t, H1).decay), (Some(P1), None), "taken back");
    assert_eq!(saves.len(), 6);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// The tick the Tunnel of [`lost_and_rebuilt`] completed in, if it has.
fn completed_at(t: &TestSim) -> Option<Tick> {
    t.events().iter().find_map(|(tick, e)| match e {
        SimEvent::SiteCompleted { .. } => Some(*tick),
        _ => None,
    })
}

/// The decay is hashed, and left out of the hash and of the files while it isn't set (the Director's rule for the
/// goldens: a state without it is as it was before it existed).
#[test]
fn the_decay_is_hashed_and_saved_only_while_it_is_set() {
    let mut t = lost_and_rebuilt();
    run_until(&mut t, 2);
    let json = serde_json::to_value(t.sim()).unwrap();
    // The Hideout's place among the objects, which are saved in id order.
    let place_of = |json: &serde_json::Value| {
        let objects = json["objects"].as_array().expect("the objects");
        objects.iter().position(|o| o["id"] == H1.0).expect("the Hideout")
    };
    assert!(
        json["objects"][place_of(&json)].get("decay").is_none(),
        "no key while unset"
    );
    run_until(&mut t, 10);
    let json = serde_json::to_value(t.sim()).unwrap();
    let i = place_of(&json);
    let decay = &json["objects"][i]["decay"];
    assert_eq!(
        *decay,
        serde_json::json!({ "progress": 8 * 65536, "total": DECAY_TICKS })
    );
    // Two states that differ only in the decay hash differently.
    let with = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut json = json.clone();
        edit(&mut json);
        serde_json::from_value::<Sim>(json).unwrap().state_hash()
    };
    let same = with(&|_| {});
    assert_eq!(same, t.sim().state_hash());
    assert_ne!(
        with(&|json| json["objects"][i]["decay"]["progress"] = (9 * 65536).into()),
        same
    );
    let without = with(&|json| {
        json["objects"][i].as_object_mut().expect("an object").remove("decay");
    });
    assert_ne!(without, same);
}
