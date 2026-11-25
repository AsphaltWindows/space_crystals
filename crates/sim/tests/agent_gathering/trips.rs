//! Trip times and incomes against the design's table (`factions/syndicate/units.md > Agent > Gathering > Space
//! Crystals`; design change 0259, the Syndicate economy check: "Distance in the table is measured edge to edge, from
//! the patch's edge to the Tunnel's crystal hatch", "Tolerance: ±15% on every figure, except the 12.5 Space Crystals
//! per second ceiling per Tunnel, which must hold exactly"). Nothing is tuned to the table: the trips are pinned as
//! measured, and slice M4-L's oracle asserts them.

use super::*;

/// The design's table: the trip time in seconds at 0 to 6 grid units from the hatch.
const TABLE_TRIP_S: [f32; 7] = [9.0, 10.3, 11.7, 13.0, 14.3, 15.7, 17.0];
/// One Agent's trip as measured, ticks, at 0 to 6 grid units: 80 of mining, 64 of unloading and the two legs. At 0
/// the Agent stands within both reaches at once and never walks. The legs are shorter than the table's (which walks
/// the whole distance both ways at 1.5 grid units per second): the Agent's own 36 su and the two reaches come off.
const TRIP: [Tick; 7] = [144, 150, 172, 192, 214, 236, 256];

/// Slice D's acceptance: "One Agent at 0–6 GU from Side B: trip times measured and recorded against the table."
/// Each is within the check's 15% of the table's; the loads are 50, and the bank gets them.
#[test]
fn one_agent_trips_follow_the_table_at_every_distance() {
    for (distance, (&trip, &table)) in TRIP.iter().zip(&TABLE_TRIP_S).enumerate() {
        let mut t = field(1, distance as i32).start();
        let start = bank(&t);
        gather(&mut t, 1, &[3], 2);
        t.run_until(8 * 256);
        let delivered = deliveries(&t);
        assert!(delivered.len() >= 7, "{distance} GU: {delivered:?}");
        assert!(
            delivered.iter().all(|d| (d.1, d.2, d.3) == (3, 1, LOAD)),
            "{distance} GU"
        );
        let gaps: Vec<Tick> = delivered.windows(2).map(|d| d[1].0 - d[0].0).collect();
        // The first trip or two settle (the Agent comes from its start, and finds its spots); then every trip is
        // the same.
        assert!(
            gaps[2..].iter().all(|&gap| gap == trip),
            "{distance} GU: {gaps:?}, not {trip}"
        );
        let seconds = trip as f32 / 16.0;
        let off = (seconds - table) / table;
        assert!(
            off.abs() <= 0.15,
            "{distance} GU: {seconds} s against the table's {table} s"
        );
        assert_eq!(bank(&t), start + LOAD * delivered.len() as u32);
        assert_eq!(
            remaining(&t, 2) + load(&t, 3) + bank(&t) - start,
            1_000_000,
            "nothing lost"
        );
        assert_eq!(
            order(&t, 3),
            Some(OrderKind::Gather(id(2))),
            "a Gather never ends on its own"
        );
    }
}

/// One trip, phase by phase, at 2 grid units: MiningDuration 80 frames, then the load of 50 as the mining ends;
/// DropOffDuration 64 frames at the hatch, then the bank (`economy::gathering`, by the recipe timing).
#[test]
fn an_agent_mines_for_80_frames_and_unloads_for_64() {
    let mut t = field(1, 2).start();
    let start = bank(&t);
    gather(&mut t, 1, &[3], 2);
    let began = run_to(&mut t, 200, "it mines", |t| is_mining(t, 3));
    t.run_until(began + MINING - 2);
    assert_eq!(
        (load(&t, 3), phase(&t, 3)),
        (0, Some(GatherPhase::Mining { done: MINING - 1 }))
    );
    t.step();
    assert_eq!((load(&t, 3), remaining(&t, 2)), (LOAD, 1_000_000 - LOAD));
    assert_eq!(
        object(&t, 3).worker.unwrap().drop_off,
        None,
        "no Tunnel before the first return starts"
    );
    let unloading = run_to(&mut t, 100, "it unloads", |t| is_unloading(t, 3));
    assert_eq!(object(&t, 3).worker.unwrap().drop_off, Some(id(1)));
    assert!(gap_to_hatch(&t, 3, 1) <= REACH, "within DropOffReach of the hatch");
    t.run(DROP_OFF - 2);
    assert_eq!(
        (load(&t, 3), bank(&t)),
        (LOAD, start),
        "nothing enters the pool before the drop-off ends"
    );
    t.step();
    assert_eq!((load(&t, 3), bank(&t)), (0, start + LOAD));
    assert_eq!(deliveries(&t), [(unloading + DROP_OFF - 1, 3, 1, LOAD)]);
}

/// The income of `count` Agents at `distance` grid units over the second half of ten minutes, with the hatch's rules
/// checked in every tick ([`watch`]) and the ceiling on every pair of deliveries.
fn measured(count: u32, distance: i32) -> (f32, Vec<usize>) {
    const TICKS: Tick = 10 * 60 * 16;
    let mut t = field(count, distance).start();
    let agents = agents(count);
    gather(&mut t, 1, &agents, 2);
    watch(&mut t, TICKS, &agents, &[1]);
    assert_ceiling(&t, 1);
    let delivered = deliveries(&t);
    let each = (agents.iter())
        .map(|&n| delivered.iter().filter(|d| d.1 == n).count())
        .collect();
    (income(&t, TICKS / 2, TICKS), each)
}

/// Slice D's acceptance: "Two, three and four Agents at 1 GU: income measured. It never passes 12.5 SC per second
/// per Tunnel. Never two Agents drop off at one side at once."
///
/// **Measured** (Space Crystals per second; the table has 9.7, 12.5 and 12.5): two Agents 10.67; three and four
/// 5.33, one Agent's worth. With the patch 1 grid unit from the hatch the strip between them is 64 su wide, and an
/// Agent is 36: Agents can't pass each other in it. Each works in a lane of its own from the wall to the patch, and
/// one that comes to stand behind another in its lane blocks it head-on, for good: neither yields, having an order
/// (`systems/units.md > Ground Collision > Friendly yielding`: "Two moving units never yield to each other or swap
/// places"), and neither gives up (design change 0255). From these starts, south-east of the Tunnel, two of three
/// Agents end up so. The open issue is in the slice's report; the ceiling and the hatch's rules hold all the same.
#[test]
fn incomes_at_one_grid_unit_never_pass_the_ceiling() {
    let (two, each) = measured(2, 1);
    assert!((two - 9.7).abs() / 9.7 <= 0.15, "{two} against the table's 9.7");
    assert!(each.iter().all(|&count| count >= 60), "{each:?}");
    for count in [3, 4] {
        let (income, each) = measured(count, 1);
        assert!((5.0..=12.5).contains(&income), "{count} Agents: {income}, {each:?}");
    }
}

/// The same from 2 to 6 grid units, where Agents walk round each other: every Agent delivers its share, and the
/// income is within the check's 15% of the table's from 3 grid units on. At 2 grid units three and four Agents
/// bring 9.8 and 10.0 where the table has 12.5 and 12.5: they still get in each other's way at the hatch.
#[test]
fn incomes_from_two_grid_units_on_follow_the_table() {
    // (Agents, grid units, the table's income, whether it is within 15%).
    let rows = [
        (2, 2, 8.6, true),
        (3, 2, 12.5, false),
        (4, 2, 12.5, false),
        (2, 3, 7.7, true),
        (3, 3, 11.5, true),
        (4, 3, 12.5, true),
        (3, 4, 10.5, true),
        (4, 4, 12.5, true),
        (3, 6, 8.8, true),
        (4, 6, 11.8, true),
    ];
    for (count, distance, table, within) in rows {
        let (income, each) = measured(count, distance);
        let (least, most) = (each.iter().min().unwrap(), each.iter().max().unwrap());
        assert!(
            most - least <= 6,
            "{count} Agents at {distance} GU share alike: {each:?}"
        );
        let off = (income - table) / table;
        assert!(income <= 12.5, "{count} Agents at {distance} GU: {income}");
        assert_eq!(
            off.abs() <= 0.15,
            within,
            "{count} Agents at {distance} GU: {income} against {table}"
        );
    }
}
