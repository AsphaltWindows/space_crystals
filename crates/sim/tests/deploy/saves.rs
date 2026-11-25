//! Saves and replays (`ARCHITECTURE.md > Replays and golden hashes`): a game saved in the middle of a
//! deployment goes on exactly as the running one, and so does its replay.

use super::*;

/// Saves and replays (`ARCHITECTURE.md > Replays and golden hashes`): a game where Drone 2 deploys with a load, Drone
/// 6 fails on arrival (its site, on player 2's Egress Node 3, never seen), and Drone 5, sent once Drone 2 deploys,
/// deploys and is cancelled goes on alike after a save in the middle of Drone 2's deployment, tick by tick, and its
/// replay file plays back to the same state.
#[test]
fn deployments_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = at_capacity().spawn(CrystalPatch, None, tile(6, 13)).start();
    // The patch is object 14.
    t.order_at(1, P1, &[id(2)], OrderKind::Gather(id(14)), false).order_at(
        1,
        P1,
        &[id(6)],
        deploy_at(at(27, 15)),
        false,
    );
    while object(&t, 2).worker.is_some_and(|worker| worker.load == 0) {
        assert!(t.sim().tick() < 400, "it mines");
        t.step();
    }
    next(&mut t, order(&[2], deploy_at(SITE), false));
    walk(&mut t, 2);
    let tick = t.sim().tick();
    t.order_at(tick + 1, P1, &[id(5)], deploy_at(at(14, 14)), false);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(object(&t, 2).transform.as_ref().unwrap().load, 8);
    let mut cancelled = false;
    while t.sim().tick() < tick + 500 {
        if !cancelled && object(&t, 5).is_deploying() {
            t.at(t.sim().tick() + 1, P1, cancel(&[5]));
            cancelled = true;
        }
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert!(cancelled);
    assert_eq!(object(&t, 2).kind, Beacon);
    assert_eq!(object(&t, 5).kind, Drone, "cancelled");
    let failed: Vec<ObjectId> = t
        .events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::DeployFailed { drone, .. } => Some(*drone),
            _ => None,
        })
        .collect();
    assert_eq!(failed, [id(6)]);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
