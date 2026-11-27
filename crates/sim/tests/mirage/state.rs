//! The illusions in the state (`Object::illusion_of`: "Optional in files and left out of them while `None`, and hashed
//! only while set"): a game with illusions saved and loaded goes on alike, and its replay reproduces it; the link is in
//! the files and in the hash of an illusion only, so that a game without illusions hashes as it did before Mirage (the
//! goldens, `tests/golden/`, stand unchanged).

use sc_sim::{PlayerCommand, Replay, Sim, TickInputs};

use super::*;

/// Player 1's Disciple (1) casts in tick 1, its illusions 3 to 6; back to 10 energy it channels, the illusions
/// mirroring it, and player 2's Militant (2), which sees it, closes in to shoot it, into its cloud while it lasts: the
/// game in the channel's 30th tick.
fn mirroring() -> TestSim {
    let mut t = quiet(
        flat(20, 20)
            .spawn(DISCIPLE, Some(P1), point(AT.x, AT.y))
            .spawn(MILITANT, Some(P2), point(AT.x + 4.0 * GU, AT.y + 2.0 * GU))
            .start(),
    );
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    assert_eq!(illusions(&t, 1), [3, 4, 5, 6]);
    regenerate(&mut t, 1, 10.5);
    let channel = t.sim().tick() + 1;
    order(&mut t, channel, P1, &[1], OrderKind::Shroud);
    order(&mut t, channel, P2, &[2], OrderKind::Attack(ObjectId(1)));
    t.run_until(channel + 30);
    assert!([1, 3, 4, 5, 6].iter().all(|&id| stands(&t, id)), "mirroring");
    t
}

/// A save names each illusion's parent, and no other object has the field. Loaded, the game hashes as the saved one,
/// and both play on alike, tick by tick, until the Militant has killed the Disciple and its illusions have gone with
/// it; the replay from the start and through a JSON round trip ends on the same state. A save whose illusion names
/// another parent hashes otherwise: the hash sees the link.
#[test]
fn a_game_with_illusions_saves_loads_and_replays_alike() {
    let mut t = mirroring();
    let json = serde_json::to_value(t.sim()).unwrap();
    let objects = json["objects"].as_array().expect("the objects");
    for o in objects {
        let id = o["id"].as_u64().expect("an id");
        let link = o.get("illusion_of");
        match id {
            3..=6 => assert_eq!(link, Some(&serde_json::json!(1)), "#{id}"),
            _ => assert_eq!(link, None, "#{id}"),
        }
    }
    let mut loaded: Sim = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    let mut edited = json;
    let first = (edited["objects"].as_array_mut().expect("the objects").iter_mut())
        .find(|o| o["id"] == serde_json::json!(3))
        .expect("an illusion");
    first["illusion_of"] = serde_json::json!(99);
    let other: Sim = serde_json::from_value(edited).unwrap();
    assert_ne!(other.state_hash(), t.sim().state_hash(), "the hash sees the link");
    for _ in 0..300 {
        step_checked(&mut t);
        loaded.step(&TickInputs {
            commands: Vec::<PlayerCommand>::new(),
        });
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
        if t.sim().object(ObjectId(1)).is_none() {
            break;
        }
    }
    assert!(t.sim().object(ObjectId(1)).is_none(), "the Disciple killed");
    assert!(illusions(&t, 1).is_empty(), "its illusions with it");
    let played = t.replay().run();
    assert_eq!(&played, t.sim());
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
