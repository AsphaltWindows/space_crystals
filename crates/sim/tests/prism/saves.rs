use sc_sim::Sim;
use serde_json::{Value, json};

use super::*;

/// Object `n` of the save `json`.
fn object_json(json: &mut Value, n: u32) -> &mut Value {
    let objects = json["objects"].as_array_mut().unwrap();
    objects.iter_mut().find(|o| o["id"] == n).expect("in the save")
}

/// The state of `json` loaded, hashed.
fn hash(json: &Value) -> u64 {
    serde_json::from_value::<Sim>(json.clone()).unwrap().state_hash()
}

/// A game saved in the middle of a burst, and of the Shift-queued thrust behind it, goes on alike after it is loaded:
/// the same states, the same hashes, and in a replay.
#[test]
fn a_game_saved_mid_burst_goes_on_alike() {
    let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)));
    t.at(2, P1, thrust(&[id(1)], Vec2::new(1280.0, 640.0), false));
    t.at(2, P1, thrust(&[id(1)], Vec2::new(832.0, 1280.0), true));
    t.run_until(8);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(&loaded, t.sim());
    while t.sim().tick() < 50 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(t.pos(1), Vec2::new(832.0, 832.0), "both thrusts");
    assert_eq!(loaded.object(id(1)).unwrap().pos, t.pos(1));
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// The thrusts are saved and hashed only while a unit has them (`Object::thrust`, Object tag 17), so that the states of
/// games without a Prism, and so every golden game's, are as they were before them: no object of a game without one
/// saves `thrust`, and a Prism's with none spent saves as `{}`. A thrust's charge and burst are saved, and a state that
/// differs only in a charge spent hashes otherwise. A Prism saved without its thrusts loads with every charge.
#[test]
fn the_thrusts_are_in_the_save_and_the_hash_only_while_a_unit_has_them() {
    let none = started(field().spawn(ObjectKind::Phantom, Some(P1), point(640.0, 640.0)));
    let json = serde_json::to_value(none.sim()).unwrap();
    let objects = json["objects"].as_array().unwrap();
    assert!(objects.iter().all(|o| o.get("thrust").is_none()), "no thrusts");

    let mut t = started(field().spawn(PRISM, Some(P1), point(640.0, 640.0)));
    let mut json = serde_json::to_value(t.sim()).unwrap();
    assert_eq!(object_json(&mut json, 1)["thrust"], json!({}), "none spent");
    let mut unsaved = json.clone();
    object_json(&mut unsaved, 1).as_object_mut().unwrap().remove("thrust");
    let loaded: Sim = serde_json::from_value(unsaved).unwrap();
    assert_eq!(&loaded, t.sim(), "every charge");
    t.at(2, P1, thrust(&[id(1)], Vec2::new(1280.0, 640.0), false));
    t.run_until(2);
    let mut json = serde_json::to_value(t.sim()).unwrap();
    let thrusts = json!({ "used": 1, "burst": { "step": [12.0, 0.0], "left": THRUST_TICKS - 1 } });
    assert_eq!(object_json(&mut json, 1)["thrust"], thrusts);
    assert_eq!(hash(&json), t.sim().state_hash());
    let mut unspent = json.clone();
    object_json(&mut unspent, 1)["thrust"]["used"] = 0.into();
    assert_ne!(hash(&unspent), hash(&json), "a charge spent");
    let mut later = json.clone();
    object_json(&mut later, 1)["thrust"]["burst"]["left"] = 3.into();
    assert_ne!(hash(&later), hash(&json), "the burst's ticks");
}
