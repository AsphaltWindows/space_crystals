//! Saves and replays (`ARCHITECTURE.md > Replays and golden hashes`): a game saved in the middle of its Warp
//! Ins goes on exactly as the running one, and so does its replay.

use super::*;

/// A game saved in the middle of its Warp Ins loads and goes on exactly as the running one; the replay of its
/// commands, through its file, ends in the same state.
#[test]
fn warp_ins_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .debug_at(1, fund(P2, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal)))
        .at(2, P1, construction(&[2], warp(BioPortal)))
        .at(3, P2, construction(&[4], warp(EgressNode)))
        .at(250, P1, construction(&[2], CancelTransform))
        .at(260, P1, construction(&[2], warp(EgressNode)))
        .at(700, P1, train(&[1], Drone));
    t.run_until(200);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(transform(&t, 1).frames_done(), 200);
    while t.sim().tick() < 800 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(
        completions(&t),
        [(322, 4, EgressNode), (579, 2, EgressNode), (640, 1, RoboPortal)]
    );
    assert_eq!(cancellations(&t), [(250, 2, BioPortal, 112)]);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
