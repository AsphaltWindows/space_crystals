//! Each station's cooldown (`factions/gdo/structures/radar_station.md > Placeholder stats`: "**cooldown 60 seconds**
//! (960 frames); free. Each Radar Station has its own cooldown"; `sc_sim::cooldown`), and design change 0422's part in
//! the gate: "Groupable **false** [...] each Scan press scans from one station, as the Shrine; Q, Tab, Q scans two
//! places". The client sends each press to the ActiveGroup's one station (its own tests,
//! `crates/game/tests/commands/radar_station.rs`); the gate scans from the first station named that is ready.

use super::*;

/// The plan's acceptance, "the same station's next Scan is refused for 960 ticks", with 0422's Verification in the
/// gate: station 3 scans in tick 2, and a second press for it in the same tick is refused, its cooldown begun
/// (`Reject::CoolingDown`); station 4 scans in that tick too, for its own cooldown. Station 3's next Scan is refused
/// up to tick 961, the 960 ticks from its cast's, and taken in tick 962; it scans nothing in between. Its cooldown
/// counts a tick in each, from its cast's (`Object::cooldown`), and is gone once over. The Scans are free: the bank
/// stays as it was.
#[test]
fn a_station_scans_again_960_ticks_after_its_scan_and_another_meanwhile() {
    let mut t = started(base());
    let bank = t.sim().player(P1).unwrap().crystals;
    let other = Vec2::new(30.0 * 64.0, 10.0 * 64.0);
    let c = t.sim().tick() + 1;
    t.at(c, P1, scan(&[S1], FAR))
        .at(c, P1, scan(&[S1], other))
        .at(c, P1, scan(&[S2], other))
        .run_until(c);
    assert_eq!(casts(&t), [(c, S1, FAR), (c, S2, other)], "a press each");
    let progress = cooldown(&t, S1).expect("cooling down");
    assert_eq!(
        (progress.total, progress.ticks_done()),
        (COOLDOWN_TICKS, 1),
        "the cast's tick the first"
    );
    assert_eq!(gate(&t, P1, scan(&[S1], other)), Err(Reject::CoolingDown));
    assert_eq!(gate(&t, P1, scan(&[S1, S2], other)), Err(Reject::CoolingDown));
    for tick in c + 1..c + COOLDOWN_TICKS {
        t.at(tick, P1, scan(&[S1], other));
    }
    t.run_until(c + COOLDOWN_TICKS - 2);
    assert_eq!(cooldown(&t, S1).map(|p| p.ticks_done()), Some(COOLDOWN_TICKS - 1));
    assert_eq!(
        gate(&t, P1, scan(&[S1], other)),
        Err(Reject::CoolingDown),
        "in tick {}",
        c + 959
    );
    t.run_until(c + COOLDOWN_TICKS - 1);
    assert_eq!(cooldown(&t, S1), None, "over");
    assert_eq!(gate(&t, P1, scan(&[S1], other)), Ok(()), "in tick {}", c + 960);
    assert_eq!(casts(&t).len(), 2, "every press between refused");
    let again = next(&mut t, scan(&[S1], other));
    assert_eq!(again, c + COOLDOWN_TICKS);
    assert_eq!(casts(&t)[2..], [(c + COOLDOWN_TICKS, S1, other)]);
    assert_eq!(t.sim().player(P1).unwrap().crystals, bank, "free");
}

/// A press that names both stations (**Placeholder (not in the design: a command naming several casters)**, as the
/// Shrine's): it scans from the first in id order that is ready, once. Station 4 scans in tick 2, then a press for both
/// in tick 102 scans from station 3, one in tick 103 is refused, both cooling down, and one in tick 962 scans from
/// station 4, its cooldown over and station 3's not.
#[test]
fn a_press_for_both_scans_from_the_first_one_ready() {
    let mut t = started(base());
    let a = next(&mut t, scan(&[S2], FAR));
    t.run_until(a + 99);
    let b = next(&mut t, scan(&[S1, S2], FAR));
    assert_eq!(casts(&t), [(a, S2, FAR), (b, S1, FAR)]);
    assert_eq!(gate(&t, P1, scan(&[S1, S2], FAR)), Err(Reject::CoolingDown));
    t.run_until(a + COOLDOWN_TICKS - 1);
    assert!(cooldown(&t, S2).is_none() && cooldown(&t, S1).is_some());
    let c = next(&mut t, scan(&[S1, S2], FAR));
    assert_eq!(casts(&t)[2..], [(c, S2, FAR)]);
}
