//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`) with the Null Chamber's turns and buff:
//! units waiting for a slot, taking their turns and done inside (`Held::service`), and out with the buff
//! (`BuffKind::NullChamber`).

use sc_sim::{Replay, Sim};
use serde_json::Value;

use super::*;

/// The plan's six Guards in a game: in together by six Tunnels, one chamber in the game, the four first done at 2 s and
/// ejected at 3 s, buffed, the other two going on with their turns. It goes on alike after a save at each stage (in,
/// four taking their turns and two waiting; the four done; the four out buffed and the two under way; all done; the
/// buffs' last tick; after their end), tick by tick, and its replay plays back to the same state.
#[test]
fn a_game_with_units_taking_their_turns_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = started(rows(6, 1), 1);
    let guards = row_guards(&mut t, &[42.0; 6]);
    let came_in = entered(&t, guards[0]);
    let out = came_in + 3 * 16;
    for k in 0..4 {
        t.at(out, P1, eject(row_tunnel(k), GUARD));
    }
    let stages = [
        came_in,
        came_in + TURN - 1,
        out,
        came_in + 2 * TURN,
        out + DURATION - 2,
        out + DURATION,
    ];
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    while t.sim().tick() <= out + DURATION + 8 {
        let tick = t.sim().tick();
        if stages.contains(&tick) {
            let saved = serde_json::to_string(t.sim()).unwrap();
            let loaded: Sim = serde_json::from_str(&saved).unwrap();
            assert_eq!(&loaded, t.sim(), "tick {tick}");
            saves.push((tick, loaded));
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
    assert_eq!(saves.len(), stages.len());
    assert_eq!(buffed(&t, &guards[..4]), [false; 4], "the buffs over");
    assert_eq!(turns(&t, &guards[4..]), [Some(TURN); 2]);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}

/// The held unit `id` of the save `json`.
fn held_json(json: &mut Value, id: ObjectId) -> &mut Value {
    let held = json["held"].as_array_mut().expect("the held units are a list");
    (held.iter_mut())
        .find(|held| held["object"]["id"] == id.0)
        .expect("held in the save")
}

/// A held unit's turn is saved and hashed only while it has one (`Held::service`): a unit inside with no chamber in
/// the game saves no `service`, so that the states of games in which no unit takes a turn are as they were before the
/// field existed; one taking its turn saves it, and a state that differs only in it hashes otherwise.
#[test]
fn a_held_unit_s_turn_is_saved_and_hashed_only_while_it_has_one() {
    let mut no_chamber = started(base(), 1);
    let guard = guards_in(&mut no_chamber, 1)[0];
    let mut json = serde_json::to_value(no_chamber.sim()).unwrap();
    assert_eq!(
        held_json(&mut json, guard).get("service"),
        None,
        "no turn, nothing saved"
    );
    let mut t = started(with_chamber(), 1);
    let guard = guards_in(&mut t, 1)[0];
    let came_in = entered(&t, guard);
    run_checked(&mut t, came_in + 4);
    let mut json = serde_json::to_value(t.sim()).unwrap();
    let saved = held_json(&mut json, guard)["service"].clone();
    let ratio_one = 1 << 16;
    assert_eq!(
        saved,
        serde_json::json!({ "chamber": { "progress": 5 * ratio_one, "total": TURN } })
    );
    let hash = |json: &Value| serde_json::from_value::<Sim>(json.clone()).unwrap().state_hash();
    assert_eq!(hash(&json), t.sim().state_hash());
    let mut further = json.clone();
    held_json(&mut further, guard)["service"]["chamber"]["progress"] = (6 * ratio_one).into();
    assert_ne!(hash(&further), hash(&json), "a tick further");
    let mut without = json.clone();
    let held = held_json(&mut without, guard).as_object_mut().unwrap();
    held.remove("service");
    assert_ne!(hash(&without), hash(&json), "no turn");
}
