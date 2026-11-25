//! Saves and replays (`ARCHITECTURE.md > Replays and golden hashes`): a game saved in the middle of its
//! constructions goes on exactly as the running one, and so does its replay.

use super::*;

/// A game saved in the middle of its constructions loads and goes on exactly as the running one; the replay of
/// its commands, through its file, ends in the same state.
#[test]
fn constructions_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(ExtractionPlate)))
        .at(
            250,
            P1,
            construction(
                &[1],
                Place {
                    origin: at(6, 5),
                    orientation: BuildingOrientation {
                        rotation: Rotation::R270,
                        flip_h: true,
                        flip_v: false,
                    },
                },
            ),
        )
        .at(251, P1, construction(&[1], Construct(Barracks)))
        .at(300, P1, construction(&[1], CancelConstruction))
        .at(301, P1, construction(&[1], Construct(Generator)))
        .at(420, P1, construction(&[2], place(5, 7)))
        .at(421, P1, construction(&[2], Construct(ExtractionPlate)));
    t.run_until(200);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(state(&t, 1).frames_done(), Some(200));
    while t.sim().tick() < 1000 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(
        completions(&t),
        [
            (240, 1, Generator),
            (400, 2, ExtractionPlate),
            (540, 1, Generator),
            (820, 2, ExtractionPlate)
        ]
    );
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
