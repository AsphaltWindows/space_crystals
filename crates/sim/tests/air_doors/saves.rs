//! Saves, replays and the hash (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`) with the air doors: Phantoms in
//! training, inside, queued at the air exit door, rising out of it, flying in and descending into the air entry door,
//! the doors' clocks running (`TunnelState::air_out`, `TunnelState::air_in`).

use sc_sim::{Replay, Sim};

use super::*;

/// A game whose Aerospace Assembly trains three Phantoms, all ejected together, two then sent back in, goes on alike
/// after a save at each stage (in training, queued with the first rising, two rising at once, flying in, the first
/// descending, the second waiting its turn, both inside), tick by tick, and its replay plays back to the same state.
#[test]
fn a_game_with_phantoms_rising_and_descending_goes_on_alike_after_a_save_and_in_a_replay() {
    let mut t = hangar();
    for _ in 0..3 {
        t.at(2, P1, train(AA, PHANTOM));
    }
    let done = 1 + 3 * PHANTOM_FRAMES;
    for _ in 0..3 {
        t.at(done + 1, P1, eject(T1, PHANTOM));
    }
    let (first, second) = (id(7), id(8));
    t.at(done + 60, P1, order(&[first, second], OrderKind::Enter(T1)));
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    let mut stages = [false; 4];
    while t.sim().tick() < done + 400 {
        let tick = t.sim().tick();
        let sim = t.sim();
        let rising_now = sim
            .objects()
            .filter(|o| o.mover.as_ref().is_some_and(|m| m.rise.is_some()))
            .count();
        let descending_now = sim
            .objects()
            .filter(|o| network::door_lift(o).is_some() && o.mover.as_ref().is_some_and(|m| m.rise.is_none()))
            .count();
        let at_stage = [
            tick == 100,
            tick == done + 1 || rising_now == 2 && !stages[1],
            descending_now == 1 && !stages[2],
            tick == done + 399,
        ];
        if at_stage.iter().any(|&now| now) || [done + 9, done + 70].contains(&tick) {
            for (seen, now) in stages.iter_mut().zip(at_stage) {
                *seen |= now;
            }
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
    assert!(stages.iter().all(|&seen| seen), "{stages:?}");
    let made: Vec<ObjectId> = produced(&t, AA).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(made, [first, second, id(9)]);
    let out: Vec<Tick> = exits(&t, T1).iter().map(|&(tick, _)| tick).collect();
    assert_eq!(out, [done + 1, done + 9, done + 17], "one every 8 ticks");
    assert_eq!(inside(&t), [first, second], "both back in");
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
