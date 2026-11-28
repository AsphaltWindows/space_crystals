//! Saves, replays and the hash (`ARCHITECTURE.md > Replays and golden hashes`) with passengers aboard: held at
//! `network::Place::Carrier`, their carrier on the map or held in the network itself.

use sc_sim::{Replay, Sim};

use super::*;

/// Two Guards board a Transport, the second a while after the first, and it drives into a Tier 2 Tunnel with them, is
/// ejected and unloads them, a third Guard sent onto it as it unloads. The game goes on alike after a save at each
/// stage (one aboard and the other on its way; both aboard, driving; inside the network; out, loaded; the tick of the
/// unload), tick by tick, and its replay plays back to the same state.
#[test]
fn a_game_with_passengers_aboard_goes_on_alike_after_a_save_and_in_a_replay() {
    let (mut t, transport, guards) = network(&[], 3, 2);
    let first = t.sim().tick() + 1;
    t.at(first, P1, order(&guards[..1], OrderKind::Enter(transport)));
    t.at(first + 20, P1, order(&guards[1..2], OrderKind::Enter(transport)));
    let mut stages: Vec<Tick> = Vec::new();
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    let mut step = |t: &mut TestSim, stages: &[Tick]| {
        let tick = t.sim().tick();
        if stages.contains(&tick) && saves.iter().all(|&(at, _)| at != tick) {
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
    };
    while aboard(&t, transport).is_empty() {
        step(&mut t, &stages);
        assert!(t.sim().tick() < 300, "the first boards");
    }
    stages.push(t.sim().tick());
    while aboard(&t, transport).len() < 2 {
        step(&mut t, &stages);
        assert!(t.sim().tick() < 400, "the second boards");
    }
    let drive = t.sim().tick() + 1;
    t.at(drive, P1, order(&[transport], OrderKind::Enter(T1)));
    stages.push(drive + 5);
    while !inside(&t).contains(&transport) {
        step(&mut t, &stages);
        assert!(t.sim().tick() < drive + 400, "it drives in");
    }
    let came_in = t.sim().tick();
    stages.push(came_in);
    let ejected = came_in + 40;
    t.at(ejected, P1, eject(T1, TRANSPORT));
    while left(&t, transport).is_none() {
        step(&mut t, &stages);
        assert!(t.sim().tick() < ejected + 120, "it comes out");
    }
    let out = t.sim().tick();
    stages.push(out);
    let unload = out + 2;
    t.at(unload, P1, order(&[transport], OrderKind::UnloadAll));
    t.at(unload, P1, order(&guards[2..], OrderKind::Enter(transport)));
    stages.push(unload);
    while t.sim().tick() < unload + 200 {
        step(&mut t, &stages);
    }
    assert_eq!(aboard(&t, transport), guards[2..], "the third aboard");
    assert_eq!(unloaded(&t, transport).len(), 2);
    assert_eq!(saves.len(), stages.len(), "a save at each stage: {stages:?}");
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
