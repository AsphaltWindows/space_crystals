//! The energy in the state (`Object::energy`; plan decision T75: "hashed only while set"): in the hash and the files
//! only for an object that has an energy pool, so that a game without one hashes as it did before the field existed
//! (the goldens, `tests/golden/`, stand unchanged), and saved and loaded with the game, old saves included.

use sc_sim::Sim;

use super::*;

/// A Disciple (1) and a Recruit (2) of player 1's, 40 ticks into the game.
fn game() -> TestSim {
    let mut t = cults()
        .spawn(DISCIPLE, Some(P1), point(300.0, 300.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(600.0, 300.0))
        .start();
    t.run(40);
    t
}

/// `sim` saved, with `edit` made to the object `id` of the save.
fn edited(sim: &Sim, id: u32, edit: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>)) -> serde_json::Value {
    let mut json = serde_json::to_value(sim).expect("a state serializes");
    let objects = json["objects"].as_array_mut().expect("the objects are a list");
    let object = objects.iter_mut().find(|o| o["id"] == id).expect("in the state");
    edit(object.as_object_mut().expect("an object"));
    json
}

/// The files: the Disciple's energy is saved, a whole number of units; the Recruit, which has no pool, saves no
/// energy at all, as before the field existed. The hash sees the energy: one unit more is another state.
#[test]
fn the_energy_is_saved_and_hashed_only_where_there_is_a_pool() {
    let t = game();
    let json = serde_json::to_value(t.sim()).unwrap();
    let objects = json["objects"].as_array().unwrap();
    let field = |id: u32| objects.iter().find(|o| o["id"] == id).unwrap().get("energy").cloned();
    assert_eq!(field(1), Some(serde_json::json!(50 * ONE + 40 * A_TICK)));
    assert_eq!(field(2), None, "no pool, no field");
    let more = edited(t.sim(), 1, |o| {
        o.insert("energy".into(), serde_json::json!(50 * ONE + 40 * A_TICK + 1));
    });
    let more: Sim = serde_json::from_value(more).unwrap();
    let same: Sim = serde_json::from_value(json).unwrap();
    assert_eq!(same.state_hash(), t.sim().state_hash());
    assert_ne!(more.state_hash(), t.sim().state_hash(), "the hash sees one unit");
}

/// The plan's acceptance, "saved and loaded": a save loads to the same game, which goes on alike, its energy too; and a
/// save from before the field existed, its Disciple without energy, loads with the Disciple at its start
/// (`sim/load.rs`), the Recruit without.
#[test]
fn a_save_goes_on_alike_and_an_old_one_loads_at_the_start() {
    let mut t = game();
    let mut loaded: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&loaded, t.sim());
    for _ in 0..100 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(raw(&t, ObjectId(1)), Some(50 * ONE + 140 * A_TICK));
    let old = edited(t.sim(), 1, |o| {
        o.remove("energy");
    });
    let old: Sim = serde_json::from_value(old).unwrap();
    let disciple = old.object(ObjectId(1)).unwrap();
    assert_eq!(disciple.energy, sc_sim::energy::start(DISCIPLE));
    assert_eq!(disciple.energy, Some(50 * ONE));
    assert_eq!(old.object(ObjectId(2)).unwrap().energy, None);
}
