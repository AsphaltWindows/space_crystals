//! One Recruit's trips against the design's formula (`factions/cults/units/recruit.md > Gathering > Expected
//! throughput`): "With a 24 su Silhouette and 8 su reach on each side, a Recruit walks 24 su (6 frames) each way, so a
//! round trip is 22 frames (6 + 8 mining + 6 + 2 drop-off) ≈ 1.45 SC/s per Recruit." Nothing is tuned to it: the trips
//! are pinned as measured, and slice M5-L's oracle asserts them.

use super::*;

/// The design's formula at a gap of `tiles` between the patch and the Storage (plan §3.3 D: "walk = gap − 24 − 16 su,
/// at 4 su per frame"): the round trip, frames.
fn formula(tiles: u32) -> f32 {
    let walk = (64 * tiles - 24 - 16) as f32;
    2.0 * walk / 4.0 + (MINING + DROP_OFF) as f32
}

/// One Recruit's trip as measured, ticks, at gaps of 1, 2 and 3 tiles: 8 of mining, 2 of unloading and two legs, each
/// a step longer than the formula's. A Recruit moves 4 su a step and stops at the first step that brings it within
/// reach, a few su inside it (6.6 su from the patch and 5.5 from the Storage, where the formula has 8 and 8), so it
/// walks 28 su each way and not 24: 7 steps. 1.33, 0.57 and 0.36 Space Crystals per second. (A Recruit whose stops
/// fall exactly on the reaches makes the formula's 22 frames: `rules`' two in their lanes, which start on a tile's
/// centre.)
const TRIP: [Tick; 3] = [24, 56, 88];

/// Plan §3.3 D's acceptance: "The single-Recruit rates are within ±15% of the formula": 1.45, 0.59 and 0.37 Space
/// Crystals per second at gaps of 1, 2 and 3 tiles. Measured: 8.3%, 3.6% and 2.3% under.
#[test]
fn one_recruits_rate_follows_the_formula_at_gaps_of_one_to_three_tiles() {
    let table = [1.45, 0.59, 0.37];
    for (i, &trip) in TRIP.iter().enumerate() {
        let tiles = i as u32 + 1;
        let (mut t, _) = face_at_work(1, tiles as i32, 1, 0);
        t.run_until(1600);
        let delivered = deliveries(&t);
        assert!(
            delivered.iter().all(|d| (d.1, d.2, d.3) == (3, 1, LOAD)),
            "{tiles} tiles"
        );
        let gaps: Vec<Tick> = delivered.windows(2).map(|d| d[1].0 - d[0].0).collect();
        assert!(gaps.len() >= 10, "{tiles} tiles: {gaps:?}");
        // The first trip comes from its start; then every trip is the same.
        assert!(
            gaps[1..].iter().all(|&gap| gap == trip),
            "{tiles} tiles: {gaps:?}, not {trip}"
        );
        let per_second = |frames: f32| LOAD as f32 * 16.0 / frames;
        let (rate, expected) = (per_second(trip as f32), per_second(formula(tiles)));
        assert!(
            (expected - table[i]).abs() < 0.005,
            "{tiles} tiles: the formula gives {expected}"
        );
        let off = (rate - expected) / expected;
        assert!(
            off.abs() <= 0.15,
            "{tiles} tiles: {rate} against the formula's {expected} Space Crystals per second"
        );
        assert_eq!(bank(&t), LOAD * delivered.len() as u32, "the Cults start with 0");
        assert_eq!(remaining(&t, 2) + load(&t, 3) + bank(&t), 1_000_000, "nothing lost");
        assert_eq!(
            order(&t, 3),
            Some(OrderKind::Gather(id(2))),
            "a Gather never ends on its own"
        );
    }
}

/// One trip, phase by phase, at a gap of 1 tile: MiningDuration 8 frames within MiningReach of the patch, then the
/// load of 2 as the mining ends; DropOffDuration 2 frames within DropOffReach of the Storage's edge, then the bank
/// (`economy::gathering`, by the recipe timing).
#[test]
fn a_recruit_mines_for_8_frames_and_unloads_for_2() {
    let (mut t, _) = face_at_work(1, 1, 1, 0);
    let gap_to = |t: &TestSim, n: u32| Shape::of(object(t, 3)).gap(&Shape::of(object(t, n)));
    // Its first load comes from the patch's far side, where it came from; from then on it works between the two.
    run_to(&mut t, 300, "the first load is in", |t| deliveries(t).len() == 1);
    let began = run_to(&mut t, 200, "it mines", |t| is_mining(t, 3));
    assert!(gap_to(&t, 2) <= REACH, "within MiningReach");
    t.run_until(began + MINING - 2);
    assert_eq!(
        (load(&t, 3), phase(&t, 3)),
        (0, Some(GatherPhase::Mining { done: MINING - 1 }))
    );
    t.step();
    assert_eq!((load(&t, 3), remaining(&t, 2)), (LOAD, 1_000_000 - 2 * LOAD));
    let unloading = run_to(&mut t, 100, "it unloads", |t| is_unloading(t, 3));
    assert!(gap_to(&t, 1) <= REACH, "within DropOffReach of the Storage's edge");
    assert_eq!(
        unloading - (began + MINING - 1),
        8,
        "7 steps of 4 su, and the tick it sets off in"
    );
    assert_eq!((bank(&t), load(&t, 3)), (LOAD, LOAD));
    t.run(DROP_OFF - 1);
    assert_eq!(deliveries(&t)[1], (unloading + DROP_OFF - 1, 3, 1, LOAD));
    assert_eq!((bank(&t), load(&t, 3)), (2 * LOAD, 0));
}
