//! The jam at the hatch (found by slice M4-M's soak, `tests/syndicate_soak.rs`'s `TURRET_BUILDER`; diagnosed by slice
//! M4-W, question Q0109): four or more Agents whose patch lies **to the side of the hatch**, so that they come to it
//! along the Tunnel's wall, stop delivering, in many layouts for good.
//!
//! **Design change 0323's contact step ends it** (slice M4-W3; step 1 of design change 0324, "keep the 1-square hatch
//! and 0315's rule. Implement 0323's sidestep (90°, one body width). Measure it"; `docs/movement.md > 6.3`): a
//! blocked Agent leaves its lane by a whole width where it used to shuffle a step along the Agent in its way, and the
//! pocket below opens. In none of the 30 layouts does the income stop any more
//! ([`the_income_never_stops_and_no_agent_goes_a_minute_without_delivering`]). In six an Agent still goes a minute
//! without a load ([`WAITS`]): none of those is a lock-up, and design change 0336 accepts them (an oversaturated
//! hatch is the player's job; steps 2 and 3 of design change 0324 are dropped):
//! - **facing the hatch at 1 grid unit** (4, 5 and 6 Agents): two Agents mine and unload from one place each,
//!   between the patch and the hatch square, and keep the hatch at 10.7 to 12.5 Space Crystals per second; the others
//!   stand loaded behind them, 40 to 70 su from the hatch and out of its reach, so they aren't in its line (0255) and
//!   get no turn;
//! - **six Agents** at 1 and 3 grid units to the side and at 4 facing it: they take turns at a saturated hatch (11.6
//!   to 12.5 per second, 17 to 30 loads each), and the longest wait for a turn is 66 to 76 seconds.
//!
//! The rest of these notes is the jam as slices M4-W and M4-W2 found it, before 0323.
//!
//! **Design change 0315 was built first, and didn't end it** (slice M4-W2; `factions/syndicate/units/agent.md > Gathering >
//! Gathering rules`, "Waiting at the hatch": "an Agent waits only once it is within reach of the hatch [...] Until it
//! is within reach it is never 'arrived': it keeps actively moving toward the hatch"; built in
//! `orders::gather::legs::return_leg`). The user chose to try it alone first; option (a) of Q0109, waiting Agents
//! yielding like idle units, isn't adopted. Measured over ten minutes (the table is in `docs/playtest/m4.md > Known
//! issues`): the layouts in which the income stopped for a minute or more were others than before, and no fewer
//! (nine); and 0315's Verification, "no Agent should go 60 s without delivering", failed in 20 of the 30.
//!
//! **What happens under 0315** (traced in [`north_west`]'s layout at 3 grid units with five Agents, from tick 2500,
//! and in the soak with six Agents on crystals, from tick 4788 to its end):
//! 1. Every Agent's leg back heads for the point nearest to itself against the wall within DropOffReach of the hatch
//!    square. Coming along the wall, that is the same point for all of them: the near end of the stretch within reach.
//!    (Facing the hatch, as in design change 0259's reference layout and the economy oracle's, the points spread over
//!    the square's front.)
//! 2. The Agent that has just unloaded stands at the wall, at that near end or in front of the square, and wants back
//!    to the patch: through the loaded Agents coming for its place. One stops north of it against the wall, one or
//!    two west of it. It is in a pocket. Its steering never steps back along a face (`docs/movement.md > 6.3`, rule
//!    4), so it doesn't leave by the open south, and its attempts at another route bring back the path it has
//!    (`docs/movement.md > 6.5`, rule 3: the Agents in its way stand in its own tile or the next). Its leg to the
//!    patch ends as arrived or failed, it waits "at its patch" where it stands, and tries again every
//!    `tuning::gathering::Gathering::recheck_ticks`, for ever.
//! 3. The loaded Agents are out of reach (in the soak the nearest one by 1 space unit, held by the corner of the
//!    next), and under 0315 they **press on** where until then they stood and set off again every 32 ticks. Pressing
//!    isn't making way: each heads for a point the pocketed Agent's Silhouette covers, or for a place beside it that
//!    another loaded Agent is in the way of, and none of them heads away from it.
//! 4. Nobody yields: all of them have an order (`systems/units.md > UnitCollision > Ground Collision > Who yields`:
//!    "only units with no active command yield"). The hatch is free and nobody is within reach of it.
//! 5. **In the worst cases the pressing Agents don't even look for another route.** The pocketed Agent's legs to the
//!    patch begin again every second and shuffle a few space units each, which is progress, and those blocked by it
//!    take over its progress clock (`docs/movement.md > 6.5`: waiting behind a unit that is getting on). In the
//!    soak the five loaded Agents were each on one leg from about tick 4,800 to the end, without a step. Where the
//!    clocks do stand still, a leg fails after 104 ticks and the next begins at once from where the Agent is by
//!    then; those legs creep along the pocketed Agent's face, and that is what opens some jams after a while.
//!
//! **At 1 grid unit** the patch's faces and the stretch within reach of the hatch are the same few places: the loaded
//! Agents stand where they mined, each head-on with an unloaded one that wants that place, and no load is ever
//! delivered. **Facing the hatch at 1 grid unit**, four and five Agents lock up the same way but for one, which mines
//! and unloads from one place and keeps half the income going: the income never stops, and the others never deliver
//! (five Agents brought the full income before 0315).
//!
//! **No existing rule resolves it**: the steering and the re-route are for a way that a step aside or the tiles
//! show, and the yielding is for idle units. What would is a rule the design doesn't have (who makes way for whom).

use super::*;

const TEN_MINUTES: Tick = 10 * 60 * 16;
/// Slice M4-W's acceptance: the income never stops for this long while crystals remain; and design change 0315's
/// Verification: "no Agent should go 60 s without delivering". 60 seconds.
const LONGEST_GAP: Tick = 60 * 16;

/// A layout of the measurements: whether the patch lies to the side of the hatch ([`north_west`]) or faces it
/// ([`field`]), its distance from the hatch in grid units, and the Agents.
type Layout = (bool, i32, u32);

/// The layouts in which some Agent still goes a minute or more without a load **with design change 0323's contact
/// step built** (module docs): an oversaturated hatch, where the Tunnel's income never stops and extra Agents wait
/// without a turn. **Accepted** (design change 0336, answering question Q0127 with its option (a); `factions/syndicate/
/// units/agent.md > Gathering`): sorting it out is the player's job, as with Recruits (design change 0320). Until 0323
/// they were 20 of the 30, nine of them with the income stopped for a minute or more.
/// The least an oversaturated hatch takes in, Space Crystals per second ([`WAITS`]): 85% of its full rate of 12.5.
const OVERSATURATED: f32 = 10.6;

const WAITS: [Layout; 6] = [
    (false, 1, 4),
    (false, 1, 5),
    (false, 1, 6),
    (true, 1, 6),
    (true, 3, 6),
    (false, 4, 6),
];

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

/// What ten minutes of gathering came to.
struct Measured {
    /// The longest stretch without a delivery, ticks: from the start to the first, between two, and from the last to
    /// the end.
    longest: Tick,
    /// The longest any one Agent went without delivering, ticks, likewise.
    worst_agent: Tick,
    /// The income after the first minute, Space Crystals per second.
    income: f32,
}

/// Ten minutes of `count` Agents on patch 2 of `t`.
fn ten_minutes(mut t: TestSim, count: u32) -> Measured {
    gather(&mut t, 1, &agents(count), 2);
    t.run(TEN_MINUTES);
    let delivered = deliveries(&t);
    let gap = |of: Option<u32>| {
        let mine = delivered.iter().filter(|d| of.is_none_or(|n| d.1 == n));
        let ticks: Vec<Tick> = [0].into_iter().chain(mine.map(|d| d.0)).chain([TEN_MINUTES]).collect();
        ticks
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .max()
            .unwrap_or(TEN_MINUTES)
    };
    Measured {
        longest: gap(None),
        worst_agent: agents(count)
            .into_iter()
            .map(|n| gap(Some(n)))
            .max()
            .unwrap_or(TEN_MINUTES),
        income: income(&t, 960, TEN_MINUTES),
    }
}

/// Ten minutes of the layout `(sideways, distance, count)`.
fn measure((sideways, distance, count): Layout) -> Measured {
    let t = if sideways {
        north_west(count, distance)
    } else {
        field(count, distance).start()
    };
    ten_minutes(t, count)
}

/// Every layout of the measurements: 4, 5 and 6 Agents at one hatch, the patch 1 to 5 grid units from it, facing it
/// and to its side.
fn layouts() -> Vec<Layout> {
    let mut layouts = Vec::new();
    for distance in 1..=5 {
        for count in 4..=6 {
            layouts.extend([(false, distance, count), (true, distance, count)]);
        }
    }
    layouts
}

/// Slice M4-W's focused repro: four Agents, the patch 2 grid units north-west of the hatch. Until design change 0315
/// they delivered four loads, the last at tick 688, and then stood for the nine minutes left; with 0315 they locked
/// up again and again (4.35 Space Crystals per second, one Agent 5,392 ticks without a load). With design change
/// 0323's contact step they work: 11.85 per second, no stretch without a delivery beyond the walk to the first load,
/// and no Agent more than 30 seconds without one.
#[test]
fn four_agents_coming_along_the_wall_keep_delivering() {
    let measured = measure((true, 2, 4));
    assert!(
        measured.longest <= LONGEST_GAP && measured.worst_agent <= LONGEST_GAP,
        "no delivery for {} ticks, an Agent's {}; {:.2} Space Crystals per second",
        measured.longest,
        measured.worst_agent,
        measured.income
    );
}

/// Design change 0324's Verification, step 1 ("The Q0111 grid: count the layouts where an Agent goes 60 s or more
/// without delivering, against 20/30 now"), and slice M4-W's acceptance, in every layout of the measurements:
/// - **the income never stops for 60 seconds**, in all 30 (the longest stretch is 19 seconds: at 5 grid units to the side, the walk to the first
///   load at 5 grid units to the side; until 0323 it stopped in nine);
/// - **no Agent goes 60 seconds without delivering** (0315's Verification), in every layout but [`WAITS`];
/// - **in the layouts of [`WAITS`]** (design change 0336: accepted, the player's job): the income stays near the
///   hatch's full rate of 12.5 Space Crystals per second, [`OVERSATURATED`] at least (10.7 to 12.5 measured), and no
///   limit is set on an extra Agent's wait.
///
/// It prints the measurements of `docs/playtest/m4.md > Known issues`.
#[test]
fn the_income_never_stops_and_no_agent_goes_a_minute_without_delivering() {
    let (mut stops, mut waits) = (Vec::new(), Vec::new());
    for layout in layouts() {
        let measured = measure(layout);
        let (sideways, distance, count) = layout;
        let name = if sideways { "north-west" } else { "facing" };
        println!(
            "{name:>10}, {distance} GU, {count} Agents: {:5.2} per second, longest gap {:4} ticks, an Agent's {:4}",
            measured.income, measured.longest, measured.worst_agent
        );
        if measured.longest > LONGEST_GAP {
            stops.push((layout, measured.longest));
        }
        if WAITS.contains(&layout) {
            assert!(
                measured.income >= OVERSATURATED,
                "{layout:?}: {:.2} per second",
                measured.income
            );
        } else if measured.worst_agent > LONGEST_GAP {
            waits.push((layout, measured.worst_agent));
        }
    }
    assert_eq!(stops, [], "((sideways, grid units, Agents), the longest gap in ticks)");
    assert_eq!(
        waits,
        [],
        "((sideways, grid units, Agents), an Agent's longest gap in ticks)"
    );
}
