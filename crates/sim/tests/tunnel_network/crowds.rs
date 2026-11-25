//! Lines and crowds at Side A (design change 0246's Verification; the plan's single file and its 40 Guards).

use sc_sim::network::TRANSIT_DELAY_TICKS;

use super::*;

/// `count` Guards of player 1's in rows of `per_row`, 40 su apart, the first row's middle at `front`, the rows
/// behind it to the north; their ids, from the next free one.
fn block(t: &mut TestSim, tick: Tick, count: u32, per_row: u32, front: Vec2) -> Vec<ObjectId> {
    let first = next_id(t);
    for i in 0..count {
        let (row, col) = (i / per_row, i % per_row);
        let x = front.x + 40.0 * (col as f32 - (per_row - 1) as f32 / 2.0);
        t.spawn_unit_at(tick, GUARD, P1, Vec2::new(x, front.y - 40.0 * row as f32));
    }
    (0..count).map(|i| id(first + i)).collect()
}

/// Design change 0246's Verification: "20 Guards ordered to Enter file in continuously, without pausing between
/// units, and the whole line is in within a few seconds." A column of 20, 4 su between two Guards, 3 grid units from
/// the entry path: each goes in as soon as it gets there, one every 8 ticks (40 su at 5 su a tick), with no hold
/// between units, and none gives up.
#[test]
fn twenty_guards_file_in_without_pausing() {
    // A taller map than `base`'s, for the column: the Tunnel on tiles (12, 20) to (15, 23), Side A at y = 1280.
    let mut t = syndicate(&flat(30, 28)).spawn(TUNNEL, Some(P1), tile(12, 20)).start();
    t.without_auto_targeting();
    let guards = block(&mut t, 1, 20, 1, Vec2::new(866.0, 1088.0));
    t.at(2, P1, enter(&guards, T1));
    run_checked(&mut t, 2);
    while inside(&t).len() < 20 {
        assert!(t.sim().tick() < 400, "{} are in", inside(&t).len());
        step_checked(&mut t);
    }
    let went_in = entries(&t);
    let order: Vec<ObjectId> = went_in.iter().map(|&(_, unit)| unit).collect();
    assert_eq!(order, guards, "single file, in the line's order");
    let gaps: Vec<Tick> = went_in.windows(2).map(|w| w[1].0 - w[0].0).collect();
    assert!(
        gaps.iter().all(|&gap| gap == 8),
        "the line's own spacing, no hold: {gaps:?}"
    );
    // The first has 166 su to walk (33 ticks), the last 19 x 40 su more: all are in within 12 s.
    let (first, last) = (went_in[0].0, went_in[19].0);
    assert!(first <= 2 + 34 && last == first + 19 * 8, "{first} {last}");
    assert_eq!(gave_up(&t), []);
}

/// The plan's single file (design change 0225's Verification): 10 Guards go in one after another through the entry
/// path while units ejected meanwhile "come out of the exit path, unhindered by those entering"; and the same units
/// come out of a second Tunnel (`> Tunnel Network`: "Units inside the Tunnel Network can travel between Tunnels
/// freely").
#[test]
fn units_come_out_unhindered_while_others_go_in() {
    let mut t = base().spawn(TUNNEL, Some(P1), tile(22, 2)).start(); // 2
    t.without_auto_targeting();
    // Four Guards go in first, and wait out their transit delay.
    let early = block(&mut t, 1, 4, 1, Vec2::new(866.0, 580.0));
    t.at(2, P1, enter(&early, T1));
    while inside(&t).len() < 4 {
        assert!(t.sim().tick() < 200);
        step_checked(&mut t);
    }
    let tick = t.sim().tick() + TRANSIT_DELAY_TICKS;
    run_checked(&mut t, tick);
    // Ten more file in; from the tick the first of them goes in, two of the early ones are ejected here and two at
    // the far Tunnel.
    let line = block(&mut t, tick + 1, 10, 1, Vec2::new(866.0, 580.0));
    t.at(tick + 2, P1, enter(&line, T1));
    while entries(&t).len() < 5 {
        assert!(t.sim().tick() < tick + 100);
        step_checked(&mut t);
    }
    let start = t.sim().tick() + 1;
    for tunnel in [T1, T1, id(2), id(2)] {
        t.at(start, P1, eject(tunnel, GUARD));
    }
    run_checked(&mut t, start + 200);
    // Those entering went in in the line's order, at about its pace (8 ticks a Guard): the two that came out here
    // walked 1 grid unit out and idle beside the line, and the first steps aside for the second (friendly yielding),
    // which may brush the line once. Getting them out of the way is the player's job (design change 0223).
    let went_in: Vec<(Tick, ObjectId)> = entries(&t).into_iter().skip(4).collect();
    assert_eq!(went_in.iter().map(|&(_, unit)| unit).collect::<Vec<_>>(), line);
    let gaps: Vec<Tick> = went_in.windows(2).map(|w| w[1].0 - w[0].0).collect();
    assert!(gaps.iter().all(|&gap| (7..=20).contains(&gap)), "{gaps:?}");
    assert!(gaps.iter().filter(|&&gap| gap > 9).count() <= 1, "{gaps:?}");
    assert_eq!(gave_up(&t), []);
    // Those ejected came out at the exit path's pace, as with nobody going in: here in the tick of the command and
    // 8 ticks later, at the far Tunnel alike, in the same ticks.
    let (near, far) = (exits(&t, T1), exits(&t, id(2)));
    assert_eq!(near, [(start, early[0]), (start + 8, early[1])]);
    assert_eq!(far, [(start, early[2]), (start + 8, early[3])]);
    assert_eq!(inside(&t), line);
}

/// The plan's crowd: 40 Guards enter one Tunnel and are then ejected. "Each Guard enters, or gives up by the Failure
/// rule and idles (0223), within a bounded number of ticks. None waits indefinitely. The test records how many give
/// up." Then all that went in come out again.
#[test]
fn forty_guards_enter_one_tunnel_and_come_out_again() {
    let mut t = base().start();
    t.without_auto_targeting();
    // A block of 8 x 5, its front row 3 grid units from Side A.
    let guards = block(&mut t, 1, 40, 8, Vec2::new(896.0, 448.0));
    t.at(2, P1, enter(&guards, T1));
    run_checked(&mut t, 2);
    let settled = |t: &TestSim| entries(t).len() + gave_up(t).len();
    while settled(&t) < 40 {
        assert!(
            t.sim().tick() < 1200,
            "{} in, {:?} gave up, the rest still wait",
            entries(&t).len(),
            gave_up(&t)
        );
        step_checked(&mut t);
    }
    let (went_in, gave_up) = (entries(&t), gave_up(&t));
    let (first, last) = (went_in[0].0, went_in.last().unwrap().0);
    println!(
        "40 Guards: {} entered (ticks {first} to {last}), {} gave up: {gave_up:?}",
        went_in.len(),
        gave_up.len()
    );
    // Those that gave up idle on the map.
    for unit in &gave_up {
        let o = t.sim().object(*unit).expect("on the map");
        assert!(o.orders.as_ref().unwrap().is_idle());
    }
    assert!(went_in.len() >= 30, "most of a crowd gets in: {}", went_in.len());
    // Every one of them out again, by 40 clicks at once: the queue holds them and lets them out one by one.
    let count = went_in.len();
    let tick = t.sim().tick().max(last + TRANSIT_DELAY_TICKS) + 1;
    for _ in 0..count {
        t.at(tick, P1, eject(T1, GUARD));
    }
    run_checked(&mut t, tick);
    assert!(t.sim().network_counts(P1).is_empty(), "all reserved");
    while !inside(&t).is_empty() {
        assert!(t.sim().tick() < tick + 40 * 60, "{} still inside", inside(&t).len());
        step_checked(&mut t);
    }
    let out = exits(&t, T1);
    assert_eq!(out.len(), count);
    println!("{count} Guards ejected in ticks {tick} to {}", out.last().unwrap().0);
}
