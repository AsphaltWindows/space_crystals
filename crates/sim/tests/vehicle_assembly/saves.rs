//! Saves, replays and the hash (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`) with the Vehicle Assembly and its
//! Bikes: a site under construction, Bikes in training, inside the network, queued at the exit path and driving out.

use sc_sim::{Replay, Sim};

use super::*;

/// A game with a second Vehicle Assembly under construction, Bikes in training, inside, queued to come out, ejected,
/// auto-ejected by a surface rally point and driving off, goes on alike after a save at each of those stages, tick by
/// tick, and its replay plays back to the same state.
#[test]
fn a_game_with_bikes_in_training_inside_queued_and_driving_out_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = assembly();
    let second = TilePos::new(9, 12);
    assert_eq!(gate(&t, P1, expand(T1, second)), Ok(()));
    t.at(2, P1, expand(T1, second));
    for _ in 0..3 {
        t.at(2, P1, train_bike(VA));
    }
    let (first, others) = (id(5), [id(6), id(7)]);
    t.at(250, P1, eject(T1, BIKE));
    t.order_at(
        300,
        P1,
        &[first],
        OrderKind::Move(Target::Point(Vec2::new(926.0, 160.0))),
        false,
    );
    for tick in [722, 722] {
        t.at(tick, P1, eject(T1, BIKE));
    }
    t.order_at(
        760,
        P1,
        &others,
        OrderKind::Move(Target::Point(Vec2::new(1200.0, 160.0))),
        false,
    );
    t.at(761, P1, rally(VA, Target::Point(Vec2::new(400.0, 300.0))));
    t.at(761, P1, train_bike(VA));
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    while t.sim().tick() < 1100 {
        if [100, 245, 251, 723, 1001].contains(&t.sim().tick()) {
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
    // The game had every stage: the site, Bikes inside and queued at a save, and the fourth auto-ejected.
    let site = id(4);
    let completed = (t.events().iter()).any(|(_, e)| *e == SimEvent::ExpansionCompleted { tunnel: T1, site });
    assert!(completed && object(&t, site).kind == ASSEMBLY);
    let produced: Vec<ObjectId> = produced(&t, VA).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(produced, [first, others[0], others[1], id(8)]);
    let out: Vec<(Tick, ObjectId)> = exits(&t, T1);
    assert_eq!(out.len(), 4, "{out:?}");
    assert_eq!(
        (out[0], out[1].0, out[3]),
        ((250, first), 722, (1000, id(8))),
        "{out:?}"
    );
    assert!(inside(&t).is_empty() && queued(&t, T1).is_empty());
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
