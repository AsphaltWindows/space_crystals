//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`) with the Shadebringer's tanks
//! (`Object::tanks`), its trail (`Sim::patches`), the slow (`BuffKind::Sludge`) and the Vault's refill.

use sc_sim::{Replay, Sim};
use serde_json::{Value, json};

use super::*;

/// One Shadebringer's life in a game with a Vault: laying its trail east along y = 200 from tick 2, across which
/// player 2's Peacekeeper walks, slowed; parked, its tank empty in tick 161 and its last patch gone 10 s later; driven
/// into the network, its tank filling at the Vault. It goes on alike after a save at each stage (laying, the
/// Peacekeeper slowed, empty, its last patch's last tick, gone, in, filling, full), tick by tick, and its replay plays
/// back to the same state.
#[test]
fn a_game_with_a_trail_a_slowed_unit_and_a_tank_filling_goes_on_alike_after_a_save_and_in_a_replay() {
    let (mut t, sbs) = network(&[(VAULT, VAULT_AT)], 1);
    let sb = sbs[0];
    let pk = id(sb.0 + 1);
    t.at(2, P1, press(&[sb], sludge(true)));
    t.at(2, P1, order(&[sb], move_to(1300.0, 200.0)));
    t.debug_at(2, spawn(ObjectKind::Peacekeeper, P2, point(700.0, 60.0)));
    t.at(30, P2, order(&[pk], move_to(700.0, 400.0)));
    t.at(170, P1, order(&[sb], OrderKind::Enter(T1)));
    let mut stages = vec![40, 161, 320, 321];
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    let mut slowed_seen = false;
    let mut came_in = None;
    while came_in.is_none_or(|tick| t.sim().tick() < tick + FULL_FILL + 4) {
        let tick = t.sim().tick();
        if !slowed_seen && t.sim().object(pk).is_some() && slow_until(&t, pk).is_some() {
            slowed_seen = true;
            stages.push(tick);
        }
        if came_in.is_none() && t.sim().held_unit(sb).is_some() {
            came_in = Some(tick);
            stages.extend([tick, tick + 100, tick + FULL_FILL]);
        }
        if stages.contains(&tick) {
            let saved = serde_json::to_string(t.sim()).unwrap();
            let loaded: Sim = serde_json::from_str(&saved).unwrap();
            assert_eq!(&loaded, t.sim(), "tick {tick}");
            saves.push((tick, loaded));
        }
        t.step();
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
    assert_eq!(
        saves.len(),
        8,
        "every stage: {:?}",
        saves.iter().map(|(tick, _)| tick).collect::<Vec<_>>()
    );
    assert_eq!(tank(&t, sb).used, 0, "full");
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// Object `n` of the save `json`.
fn object_json(json: &mut Value, n: u32) -> &mut Value {
    let objects = json["objects"].as_array_mut().expect("the objects are a list");
    objects.iter_mut().find(|o| o["id"] == n).expect("in the save")
}

/// The state of `json` loaded, hashed.
fn hash(json: &Value) -> u64 {
    serde_json::from_value::<Sim>(json.clone()).unwrap().state_hash()
}

/// The tanks and the patches are saved and hashed only while there are any (`Object::tanks`, Object tag 24;
/// `Sim::patches`, Sim tag 4), so that the states of games without a Shadebringer, and so every golden game's, are as
/// they were before them: no object of a game without one saves `tanks`, and no state without a patch saves `patches`;
/// a Shadebringer's tanks full and off save as `{}`. Switched on, its tank saves what it has used, its toggle and its
/// trail's newest patch, and the state its patch; and a state that differs only in a patch's end, in having no patch,
/// or in a tank's charge hashes otherwise.
#[test]
fn the_tanks_and_the_patches_are_in_the_save_and_the_hash_only_while_there_are_any() {
    let mut none = started(field().spawn(ObjectKind::Peacekeeper, Some(P2), point(500.0, 500.0)));
    run_until(&mut none, 2);
    let json = serde_json::to_value(none.sim()).unwrap();
    assert_eq!(json.get("patches"), None, "no patch, nothing saved");
    let objects = json["objects"].as_array().unwrap();
    assert!(objects.iter().all(|o| o.get("tanks").is_none()), "no tanks");

    let mut t = started(field().spawn(SHADEBRINGER, Some(P1), point(320.0, 800.0)));
    run_until(&mut t, 1);
    let mut json = serde_json::to_value(t.sim()).unwrap();
    assert_eq!(json.get("patches"), None);
    assert_eq!(object_json(&mut json, 1)["tanks"], json!({}), "full and off");
    t.at(2, P1, press(&[id(1)], sludge(true)));
    run_until(&mut t, 2);
    let mut json = serde_json::to_value(t.sim()).unwrap();
    let laid = json!({ "kind": "Sludge", "by": 1, "centre": [292.0, 800.0], "radius": 24.0, "until": 2 + LIFE });
    assert_eq!(json["patches"], json!([laid]));
    let tanks = json!({ "sludge": { "used": DRAIN, "on": true, "laid": [292.0, 800.0] } });
    assert_eq!(object_json(&mut json, 1)["tanks"], tanks);
    assert_eq!(hash(&json), t.sim().state_hash());
    let mut later = json.clone();
    later["patches"][0]["until"] = (3 + LIFE).into();
    assert_ne!(hash(&later), hash(&json), "a patch's end");
    let mut gone = json.clone();
    gone.as_object_mut().unwrap().remove("patches");
    assert_ne!(hash(&gone), hash(&json), "no patch");
    let mut fuller = json.clone();
    object_json(&mut fuller, 1)["tanks"]["sludge"]["used"] = 0.into();
    assert_ne!(hash(&fuller), hash(&json), "a tank's charge");
}
