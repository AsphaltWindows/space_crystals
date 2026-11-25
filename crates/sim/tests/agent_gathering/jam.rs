//! The jam at the hatch (slice M4-W; found by M4-M's soak, `tests/syndicate_soak.rs`'s `TURRET_BUILDER`): four or
//! more Agents whose patch lies **to the side of the hatch**, so that they come to it along the Tunnel's wall, stop
//! delivering for good. Both tests are ignored: they fail as built, and the fix needs a design answer (below).
//!
//! **What happens** (traced in [`north_west`]'s layout at 2 and 3 grid units, and in the soak at 6:54):
//! 1. Every Agent's leg back heads for the point nearest to itself against the wall within DropOffReach of the hatch
//!    square (`orders::gather::legs::return_leg`). Coming along the wall, that is the same point for all of them: the
//!    near end of the stretch within reach. (Facing the hatch, as in design change 0259's reference layout and the
//!    economy oracle's, the points spread over the square's front, and nothing jams from 3 grid units on.)
//! 2. The first one there unloads. The next ones' legs end against it, short of the hatch and out of reach
//!    (`systems/controls/behaviors.md > MovingToLocation > Occupied target location`: arrived), on the side they came
//!    from: the side the patch is on. They stand, and set off again every `tuning::gathering::LEG_RETRY_TICKS` for
//!    the same point (design change 0255: "It waits as long as it takes; it never gives up").
//! 3. The one that unloaded wants back to the patch, and stands in a pocket: the wall behind it, a waiting Agent
//!    north of it and one west. Its steering never steps back along a face (`docs/movement.md > 6.3`, rule 4), so
//!    it doesn't leave by the open south. It is stuck, and its three attempts at another route
//!    (`docs/movement.md > 6.5`, rule 3) bring back the path it has: the Agents in its way stand in its own tile
//!    or the next, and the one avoid tile it notes is not on its straight line. It fails, waits at "its patch"
//!    where it stands, and tries again every `tuning::gathering::RECHECK_TICKS`, for ever.
//! 4. Nobody yields: all of them have an order (`systems/units.md > UnitCollision > Ground Collision > Who yields`:
//!    "only units with no active command yield"). The hatch is free, nobody is within reach of it, and the one
//!    Agent that would free the way can't get out past those waiting for its place.
//!
//! With three Agents one is always away mining, and two can't close the pocket. Where the waiting Agents' retries
//! creep along the unloader's face (a few space units per leg) the jam sometimes opens after some minutes; where
//! they don't, it is for good.
//!
//! **No existing rule resolves it**: the steering and the re-route are for a way that a step aside or the tiles
//! show, and the yielding is for idle units. What would is a rule the design doesn't have (who makes way for whom),
//! so slice M4-W stopped at the diagnosis (`docs/playtest/m4.md > Known issues`).

use super::*;

const TEN_MINUTES: Tick = 10 * 60 * 16;
/// The brief's acceptance: the income never stops for this long while crystals remain. 60 seconds.
const LONGEST_GAP: Tick = 60 * 16;

/// Tunnel 1 turned Side A south at tiles (6, 6) to (9, 9), its crystal hatch (6, 7) facing west, as player 1's in
/// `dev_skirmish`; patch 2 north-west of the hatch, `distance` grid units north of the hatch's row and one less west
/// of its wall (`dev_skirmish`'s patch on (3, 3), where the soak jams, is `distance` 3); the Agents, ids from 3, in a
/// row south of the Tunnel.
fn north_west(agents: u32, distance: i32) -> TestSim {
    let mut builder = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(1_000_000), tile(6 - distance, 6 - distance));
    for i in 0..agents as i32 {
        builder = builder.spawn(AGENT, Some(P1), centre(2 + i, 12));
    }
    TestSim::new(turned(builder.build(), 0, Rotation::R180, false))
}

/// Ten minutes of `count` Agents on patch 2 of `t`: the longest stretch without a delivery (the last one runs to the
/// end), ticks, and the income after the first minute, Space Crystals per second.
fn ten_minutes(mut t: TestSim, count: u32) -> (Tick, f32) {
    gather(&mut t, 1, &agents(count), 2);
    t.run(TEN_MINUTES);
    let mut ticks: Vec<Tick> = deliveries(&t).iter().map(|d| d.0).collect();
    ticks.insert(0, 0);
    ticks.push(TEN_MINUTES);
    let longest = ticks
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .max()
        .unwrap_or(TEN_MINUTES);
    (longest, income(&t, 960, TEN_MINUTES))
}

/// The focused repro (module docs): four Agents, the patch 2 grid units north-west of the hatch. As built they
/// deliver four loads, the last at tick 688, and then stand for the nine minutes left.
#[test]
#[ignore = "M4-W: fails as built (the jam at the hatch); the fix needs a design answer"]
fn four_agents_coming_along_the_wall_keep_delivering() {
    let (longest, income) = ten_minutes(north_west(4, 2), 4);
    assert!(
        longest <= LONGEST_GAP,
        "no delivery for {longest} ticks; {income:.2} Space Crystals per second"
    );
}

/// The brief's acceptance: 4, 5 and 6 Agents per hatch, the patch 1 to 5 grid units from it, facing it ([`field`])
/// and to its side ([`north_west`]), never stop delivering for 60 seconds. As built the facing layouts pass (longest
/// gap 225 ticks, the walk to the first load), and every sideways one with four or more Agents is far below its income, most of them over the
/// bound (measured: `docs/playtest/m4.md > Known issues`).
#[test]
#[ignore = "M4-W: fails as built (the jam at the hatch); the fix needs a design answer"]
fn four_to_six_agents_at_one_hatch_never_stop_delivering() {
    let mut jams = Vec::new();
    for distance in 1..=5 {
        for count in 4..=6 {
            let layouts = [
                ("facing", field(count, distance).start()),
                ("north-west", north_west(count, distance)),
            ];
            for (name, t) in layouts {
                let (longest, income) = ten_minutes(t, count);
                println!(
                    "{name:>10}, {distance} GU, {count} Agents: longest gap {longest:4} ticks, {income:5.2} per second"
                );
                if longest > LONGEST_GAP {
                    jams.push((name, distance, count, longest));
                }
            }
        }
    }
    assert_eq!(jams, [], "(layout, grid units, Agents, the longest gap in ticks)");
}
