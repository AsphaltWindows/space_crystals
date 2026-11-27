//! The clouds in the state (`Sim::clouds`; plan decision T77: "hashed only while not empty"): a replay with clouds
//! splitting and merging reproduces the hash; the clouds are saved and loaded with the game, and left out of the hash
//! and of the files while there are none, so that a game without them hashes as it did before Shroud (the goldens,
//! `tests/golden/`, stand unchanged).

use sc_sim::{Replay, Sim, Target};

use super::*;

/// Four Disciples of player 1's in a row, 3.5 grid units apart (1–4), with a Militant of player 1's under each (5–8)
/// that one of player 2's shoots from outside the clouds (9–12). 1 and 3, 7 grid units apart, channel apart; 2,
/// between them, joins them at [`JOIN`]; 4 joins the three; 2's Stop splits the cloud in two; 3 walks off, which ends
/// its channel; 1 runs dry in tick 178, its 50 energy spent; 3 channels again farther on, which joins it to 4 again;
/// and 4's Stop leaves 3's cloud alone. (M6-CM) Each channel that ends leaves its circle 24 ticks where it was (design
/// change 0448), so that each change comes 24 ticks after the end: 2's circle holds the cloud together until tick 124.
fn splits_and_merges() -> TestSim {
    let mut builder = flat(32, 14);
    for row in [400.0, 480.0, 700.0] {
        for x in [300.0, 524.0, 748.0, 972.0] {
            let owner = if row == 700.0 { P2 } else { P1 };
            let kind = if row == 400.0 { DISCIPLE } else { MILITANT };
            builder = builder.spawn(kind, Some(owner), point(x, row));
        }
    }
    let mut t = quiet(builder.start());
    order(&mut t, 1, P1, &[1, 3], OrderKind::Shroud);
    for (shooter, target) in [(9, 5), (10, 6), (11, 7), (12, 8)] {
        order(&mut t, 2, P2, &[shooter], OrderKind::Attack(ObjectId(target)));
    }
    order(&mut t, JOIN, P1, &[2], OrderKind::Shroud);
    order(&mut t, 60, P1, &[4], OrderKind::Shroud);
    order(&mut t, 100, P1, &[2], OrderKind::Stop);
    order(
        &mut t,
        130,
        P1,
        &[3],
        OrderKind::Move(Target::Point(Vec2::new(748.0, 250.0))),
    );
    order(&mut t, 190, P1, &[3], OrderKind::Shroud);
    order(&mut t, 220, P1, &[4], OrderKind::Stop);
    t.run_until(JOIN - 1);
    assert_eq!(clouds(&t), [vec![1], vec![3]]);
    let holding = [1, 3].iter().filter(|&&first| buffer(&t, first) > 0.0).count();
    t.step();
    assert_eq!(clouds(&t), [vec![1, 2, 3]], "2 joins them");
    let at = |t: &TestSim, tick: Tick| releases(t).into_iter().filter(|r| r.0 == tick).count();
    assert_eq!(at(&t, JOIN), holding, "each with a buffer releases it");
    for (tick, expected) in [
        (60, vec![vec![1, 2, 3, 4]]),
        (100, vec![vec![1, 2, 3, 4]]),
        (123, vec![vec![1, 2, 3, 4]]),
        (124, vec![vec![1], vec![3, 4]]),
        (130, vec![vec![1], vec![3, 4]]),
        (154, vec![vec![1], vec![4]]),
        (178, vec![vec![1], vec![4]]),
        (190, vec![vec![1], vec![3, 4]]),
        (202, vec![vec![3, 4]]),
        (220, vec![vec![3, 4]]),
        (244, vec![vec![3]]),
    ] {
        t.run_until(tick);
        assert_eq!(clouds(&t), expected, "tick {tick}");
    }
    t.run_until(260);
    t
}

/// [`splits_and_merges`]: Disciple 2 channels between 1 and 3 in this tick.
const JOIN: Tick = 30;

/// The plan's acceptance, "A replay with clouds splitting and merging reproduces the hash": the game joins and splits
/// its clouds as it says, and its replay, played back and through a JSON round trip, ends in the same state.
#[test]
fn a_replay_with_clouds_splitting_and_merging_reproduces_the_hash() {
    let t = splits_and_merges();
    assert!(releases(&t).len() >= 10, "{:?}", releases(&t));
    let played = t.replay().run();
    assert_eq!(played.state_hash(), t.sim().state_hash());
    assert_eq!(&played, t.sim());
    let json = serde_json::to_string(t.replay()).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.run().state_hash(), t.sim().state_hash());
}

/// The clouds go into the files only while there are any, and a save loads to the same game, which goes on alike
/// through its releases; the hash sees a cloud's buffer, and an empty `clouds` in a file is no cloud. A cloud naming
/// an id the game hasn't given out doesn't load. (M6-CM) A lingering circle too: its `until` in the file, and the
/// game loaded from it goes on alike until it vanishes; the hash sees it; and the files leave it out while the unit
/// channels.
#[test]
fn the_clouds_are_saved_and_loaded_and_left_out_while_there_are_none() {
    let mut t = quiet(
        flat(20, 14)
            .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
            .spawn(MILITANT, Some(P2), point(400.0, 620.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 2, P2, &[2], OrderKind::Attack(ObjectId(1)));
    let file = |sim: &Sim| serde_json::to_value(sim).unwrap();
    assert_eq!(file(t.sim()).get("clouds"), None, "none before");
    step_until(&mut t, 60, "a buffer", |t| buffer(t, 1) > 0.0);
    let json = file(t.sim());
    assert!(
        json["clouds"]["clouds"][0]["buffer"].as_f64().is_some_and(|b| b > 0.0),
        "{}",
        json["clouds"]
    );
    let channelling = &json["clouds"]["clouds"][0]["members"][0];
    assert_eq!(channelling.get("until"), None, "{channelling}");
    let mut loaded: Sim = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(&loaded, t.sim());
    for _ in 0..40 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert!(releases(&t).len() >= 2, "through its releases");
    let mut more = json.clone();
    more["clouds"]["clouds"][0]["buffer"] = serde_json::json!(1000.0);
    let more: Sim = serde_json::from_value(more).unwrap();
    let same: Sim = serde_json::from_value(json.clone()).unwrap();
    assert_ne!(more.state_hash(), same.state_hash(), "the hash sees the buffer");
    let mut unknown = json;
    unknown["clouds"]["clouds"][0]["members"][0]["id"] = serde_json::json!(99);
    assert!(serde_json::from_value::<Sim>(unknown).is_err(), "no object 99 yet");
    let moved = t.sim().tick() + 1;
    order(&mut t, moved, P1, &[1], OrderKind::Stop);
    order(&mut t, moved, P2, &[2], OrderKind::Stop);
    t.run(2);
    let json = file(t.sim());
    let lingering = &json["clouds"]["clouds"][0]["members"][0];
    assert_eq!(lingering["until"], serde_json::json!(moved + LINGER), "{lingering}");
    let mut loaded: Sim = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(&loaded, t.sim());
    while !t.sim().clouds().is_empty() {
        assert!(t.sim().tick() < moved + LINGER, "it vanishes in its tick");
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(t.sim().tick(), moved + LINGER);
    let mut later = json.clone();
    later["clouds"]["clouds"][0]["members"][0]["until"] = serde_json::json!(moved + LINGER + 1);
    let later: Sim = serde_json::from_value(later).unwrap();
    let same: Sim = serde_json::from_value(json).unwrap();
    assert_ne!(later.state_hash(), same.state_hash(), "the hash sees the linger");
    t.run(2);
    assert!(t.sim().clouds().is_empty());
    let mut none = file(t.sim());
    assert_eq!(none.get("clouds"), None, "none after");
    let hash = t.sim().state_hash();
    none["clouds"] = serde_json::json!({});
    let empty: Sim = serde_json::from_value(none).unwrap();
    assert_eq!(empty.state_hash(), hash, "an empty clouds is no cloud");
}
