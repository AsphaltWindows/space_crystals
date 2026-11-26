//! The Shrine's energy (`factions/cults/structures/shrine.md > Energy`: "**Energy regeneration**: proportional to the
//! number of Recruits inside (0.25 per second per Recruit, placeholder). With no Recruits inside, the Shrine does not
//! regenerate energy"; `> Placeholder stats`: "MaxEnergy 200; starts at 0; regeneration **0.25 energy per second per
//! Recruit inside** (2 per second when full)"; `crate::energy`).

use super::*;

/// The plan's acceptance: "With 8 Recruits inside, 2 energy a second: the first Q is possible after 37.5 s, and 200 is
/// reached after 100 s". Eight Recruits go in, each adding 1024 a tick (0.25 a second) from its first tick inside;
/// full, the Shrine charges 8192 a tick up to its 200 and no further. Spent to nothing (Q, Q and W in one tick: "No
/// cooldowns"), it can cast Q again 600 ticks later, 37.5 s, not a tick before, and is full again 1600 ticks after,
/// 100 s.
#[test]
fn eight_recruits_charge_2_a_second_so_the_first_q_waits_37_5_s_and_200_takes_100_s() {
    let (mut t, _) = manned(8);
    let mut gains = Vec::new();
    while energy(&t, S1) < 200 * ONE {
        assert!(t.sim().tick() < 2000, "charged");
        let before = energy(&t, S1);
        step_checked(&mut t);
        gains.push(energy(&t, S1) - before);
    }
    let (last, full) = gains.split_last().unwrap();
    assert!(full.iter().all(|&gain| gain == FULL), "{full:?}");
    assert!(*last <= FULL);
    step_checked(&mut t);
    assert_eq!(energy(&t, S1), 200 * ONE, "no further");
    let now = t.sim().tick() + 1;
    let at = Vec2::new(448.0, 640.0); // The Shrine's centre, which it sees.
    t.at(now, P1, cast(&[S1], Q, at));
    t.at(now, P1, cast(&[S1], Q, at));
    t.at(now, P1, cast(&[S1], W, at));
    run_checked(&mut t, now);
    assert_eq!(casts(&t), [(now, S1, Q, at), (now, S1, Q, at), (now, S1, W, at)]);
    assert_eq!(energy(&t, S1), FULL, "spent to nothing, then a tick's charge");
    let q = cast(&[S1], Q, at);
    run_checked(&mut t, now + 598);
    assert_eq!(
        validate(&t, q.clone()),
        Err(Reject::NotEnoughEnergy),
        "599 ticks of charge"
    );
    step_checked(&mut t);
    assert_eq!(energy(&t, S1), 75 * ONE, "600 ticks, 37.5 s");
    assert_eq!(validate(&t, q), Ok(()));
    run_checked(&mut t, now + 1598);
    assert_eq!(energy(&t, S1), 200 * ONE - FULL);
    step_checked(&mut t);
    assert_eq!(energy(&t, S1), 200 * ONE, "1600 ticks, 100 s");
    step_checked(&mut t);
    assert_eq!(energy(&t, S1), 200 * ONE);
}

/// "With no Recruits inside, the Shrine does not regenerate energy", and it "starts at 0": a Shrine with no one inside
/// stays at 0; two Recruits charge it 2048 a tick, one 1024; once both are out, it keeps what it has.
#[test]
fn with_no_recruits_inside_it_keeps_its_energy_and_it_starts_at_0() {
    let mut t = base().start();
    t.without_auto_targeting();
    assert_eq!(energy(&t, S1), 0, "it starts at 0");
    run_checked(&mut t, 100);
    assert_eq!(energy(&t, S1), 0, "none inside");
    let recruits = block(&mut t, 101, RECRUIT, 2, Vec2::new(650.0, 640.0));
    run_checked(&mut t, 101);
    t.at(102, P1, enter(&recruits, S1));
    run_checked(&mut t, 102);
    let mut gains = Vec::new();
    while inside(&t, S1).len() < 2 {
        assert!(t.sim().tick() < 400, "both in");
        let before = energy(&t, S1);
        step_checked(&mut t);
        gains.push((inside(&t, S1).len(), energy(&t, S1) - before));
    }
    assert!(
        gains.iter().any(|&(inside, gain)| (inside, gain) == (1, 1024)),
        "{gains:?}"
    );
    assert!(
        gains.iter().all(|&(inside, gain)| gain == 1024 * inside as u32),
        "{gains:?}"
    );
    let before = energy(&t, S1);
    step_checked(&mut t);
    assert_eq!(energy(&t, S1) - before, 2048, "two inside");
    let out = t.sim().tick() + 1;
    t.at(out, P1, eject_all(&[S1]));
    while !inside(&t, S1).is_empty() {
        assert!(t.sim().tick() < out + 200, "both out");
        step_checked(&mut t);
    }
    let kept = energy(&t, S1);
    assert!(kept > 0);
    run_for(&mut t, 100);
    assert_eq!(energy(&t, S1), kept, "none inside: no regeneration");
}
