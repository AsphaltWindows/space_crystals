//! The stacked-Tunnel soak (slice D's acceptance as the Director changed it for design change 0255): six Agents on
//! two Tier 1 Tunnels sharing one field for ten minutes, the Tunnels "on opposite sides of the field, each hatch
//! facing it, so every Agent's nearest Tunnel is its own" (design change 0259). Liveness: income keeps flowing, every
//! Agent keeps delivering, and no Agent stands at a hatch that is free for longer than a stated bound. Ignored by
//! default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test agent_gathering -- --ignored --nocapture
//! ```

use super::*;

const TICKS: Tick = 10 * 60 * 16;
/// The longest an Agent may stand with its load, within reach of its hatch or short of it, while nobody unloads
/// there: the walk in, and until design change 0315 a retry of the leg back (32 ticks; the leg to a hatch no longer
/// ends short behind standing Agents, `orders::gather::legs::return_leg`). Measured: see the test.
const FREE_HATCH_BOUND: Tick = 64;

/// Tunnel 1 unturned at tiles (6, 6) to (9, 9), crystal hatch (9, 8) facing east; the field, six patches at tiles
/// (12, 7) to (13, 9), 2 grid units from it; Tunnel 8 flipped left and right at (16, 6) to (19, 9), hatch (16, 8)
/// facing west, 2 grid units from the field's other side. Their Tier 1 Tunnel Areas (8 × 8) touch and don't overlap.
/// Agents 9 to 11 start south-west of the field, 12 to 14 south-east. Ids: patches 2 to 7.
fn stacked() -> TestSim {
    let mut builder = flat(30, 20).spawn(TUNNEL, Some(P1), tile(6, 6));
    for x in 12..14 {
        for y in 7..10 {
            builder = builder.spawn_with(patch_of(1_000_000), tile(x, y));
        }
    }
    builder = builder.spawn(TUNNEL, Some(P1), tile(16, 6));
    for x in [9, 10, 11, 14, 15, 16] {
        builder = builder.spawn(AGENT, Some(P1), centre(x, 13));
    }
    TestSim::new(turned(builder.build(), 7, Rotation::R0, true))
}

#[test]
#[ignore = "a ten-minute soak: run with --ignored"]
fn six_agents_on_two_stacked_tunnels_keep_delivering() {
    let mut t = stacked();
    let (west, east) = ([9, 10, 11], [12, 13, 14]);
    // Each side's Agents on the middle patch of its column.
    gather(&mut t, 1, &west, 3);
    gather(&mut t, 1, &east, 6);
    let agents: Vec<u32> = west.iter().chain(&east).copied().collect();
    let watched = watch(&mut t, TICKS, &agents, &[1, 8]);
    assert_ceiling(&t, 1);
    assert_ceiling(&t, 8);
    let delivered = deliveries(&t);
    for (tunnel, own) in [(1, west), (8, east)] {
        for n in own {
            let mine: Vec<&(Tick, u32, u32, u32)> = delivered.iter().filter(|d| d.1 == n).collect();
            assert!(
                mine.iter().all(|d| d.2 == tunnel),
                "Agent {n} keeps to its nearest Tunnel, {tunnel}"
            );
            let ticks: Vec<Tick> = mine.iter().map(|d| d.0).collect();
            let longest = ticks.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap_or(TICKS);
            println!(
                "Agent {n}: {} loads at Tunnel {tunnel}, longest gap {longest} ticks",
                ticks.len()
            );
            assert!(
                ticks.len() >= 30 && longest <= 60 * 16,
                "Agent {n}: {} loads, {longest}",
                ticks.len()
            );
            assert_eq!(
                order(&t, n).map(|kind| matches!(kind, OrderKind::Gather(_))),
                Some(true)
            );
        }
    }
    // Income keeps flowing: minute by minute, and "Two Tier 1 Tunnels may share a rich crystal field (doubling its
    // intake)".
    for minute in 1..10 {
        let rate = income(&t, minute * 960, (minute + 1) * 960);
        println!("minute {minute}: {rate:.2} Space Crystals per second");
        assert!((15.0..=25.0).contains(&rate), "minute {minute}: {rate}");
    }
    println!("{watched:?}; in all {:.2} per second", income(&t, 960, TICKS));
    assert!(
        watched.longest_idle_wait <= 1 && watched.longest_idle_stand <= FREE_HATCH_BOUND,
        "{watched:?}"
    );
}
