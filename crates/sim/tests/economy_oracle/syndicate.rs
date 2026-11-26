//! The Syndicate (slice M4-L): the economy check of `factions/syndicate/units/agent.md > Gathering > Space Crystals >
//! Syndicate economy check` (design changes 0259 and 0294) on the reference layouts of `sc_sim::testing::syndicate`,
//! played through the gate, with `TestSim`'s invariants after every tick:
//! - **The table** ([`DESIGN`]), recomputed by 0294 for 4 seconds of mining: one Agent's trip, its income, its
//!   payback and the Agents that saturate a Tunnel, from its steady trip; the income of 2, 3 and 4 Agents. "Tolerance:
//!   ±15% on every figure" ([`TOLERANCE`]), checked "from 3 GU outward"; the rows 0 to 2 are "theoretical (with
//!   collision, Agents may need the player's help there)", measured for the record only (the ignored
//!   [`syndicate_table`]).
//! - **The ceiling**: "the 12.5 Space Crystals per second ceiling per Tunnel, which must hold exactly (hard rule: one
//!   drop-off per 4 s)": in every game measured, no two deliveries at a Tunnel come less than DropOffDuration apart.
//! - **The opening**: "3 Agents saturate a Tunnel around 0:40, i.e. 0:34–0:46, with the patch 3 GU from the hatch",
//!   played from the Starting Conditions (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: "With 100,
//!   the second Agent can start immediately and a starting Tunnel is saturated (3 Agents, 12.5 Space Crystals per
//!   second) around 0:40").
//! - **Stacked Tunnels** "double a field's intake" (`factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule >
//!   Crystal intake and stacked Tunnels`).
//! - **The lowest one-base saturation**: "the Syndicate's one-base saturation income is the lowest of the measured
//!   factions" (`factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`: "the Syndicate should have the lowest
//!   one-base saturation income").
//!
//! How each figure is measured:
//! - **A trip**: one Agent on [`reference_field`], the gap between its deliveries once it has settled (from its third
//!   on, all the same).
//! - **An income**: the Agents sent to the patch with one command, as one right-click sends a selection; the average
//!   over minutes 5 to 10 of a ten-minute game ([`INCOME_FROM`] to [`GAME`]), as `tests/agent_gathering/` measures it:
//!   with collision the Agents take minutes to settle into their lanes.
//! - **Out of tolerance** figures are brought to the designer with the measured table (design change 0108) and pinned
//!   meanwhile ([`OUT_OF_TOLERANCE`]): a change that moves one in or out of the tolerance fails here.
//!
//! `cargo test -p sc_sim --test economy_oracle syndicate -- --nocapture` prints the table from 3 grid units, the
//! stacked Tunnels and the opening; `cargo test -p sc_sim --release --test economy_oracle syndicate_table -- --ignored
//! --nocapture` the whole table. The results are in `docs/economy_oracle.md`.

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::testing::syndicate::{
    FIELD_PATCH, FIELD_PLAYER, FIELD_TUNNEL, STACKED_PATCH, STACKED_TUNNEL, SupplyBooks, Syndicate, field_agents,
    opening_field, reference_field, stacked_agents, stacked_field,
};
use sc_sim::tunnel::Resource;
use sc_sim::{ObjectId, ObjectKind, OrderKind, SimEvent, Target, Tick};

use super::{TOLERANCE, at, clock, seconds};

/// `factions/syndicate/units/agent.md > Gathering > Space Crystals`: 50 Space Crystals a load; DropOffDuration 64
/// frames, the hatch's one drop-off at a time, so "a Tunnel takes in at most 50 Space Crystals per 4 seconds = 12.5
/// Space Crystals per second".
const LOAD: f64 = 50.0;
const DROP_OFF: Tick = 64;
const CEILING: f64 = 12.5;

/// One row of the table: the trip (seconds), an Agent's income (Space Crystals per second), its payback (seconds to
/// earn its 100), the Agents that saturate a Tunnel, and the income of 2, 3 and 4 Agents.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Row {
    trip: f64,
    per_agent: f64,
    payback: f64,
    saturate: f64,
    incomes: [f64; 3],
}

const fn row(trip: f64, per_agent: f64, payback: f64, saturate: f64, incomes: [f64; 3]) -> Row {
    Row {
        trip,
        per_agent,
        payback,
        saturate,
        incomes,
    }
}

/// The design's table, at 0 to 6 grid units from the hatch (design change 0294).
const DESIGN: [Row; 7] = [
    row(8.0, 6.25, 16.0, 2.00, [12.5, 12.5, 12.5]),
    row(9.3, 5.36, 19.0, 2.33, [10.7, 12.5, 12.5]),
    row(10.7, 4.69, 21.0, 2.67, [9.4, 12.5, 12.5]),
    row(12.0, 4.17, 24.0, 3.00, [8.3, 12.5, 12.5]),
    row(13.3, 3.75, 27.0, 3.33, [7.5, 11.3, 12.5]),
    row(14.7, 3.41, 29.0, 3.67, [6.8, 10.2, 12.5]),
    row(16.0, 3.12, 32.0, 4.00, [6.2, 9.4, 12.5]),
];
/// The figures of the checked rows ("the table from 3 GU outward", design change 0294: one test for each, from
/// [`syndicate_row_at_3_gu`] on) outside the tolerance, by distance and name ([`FIGURES`]), as measured: none.
const OUT_OF_TOLERANCE: [(i32, &str); 0] = [];
/// The figures' names, in a row's order.
const FIGURES: [&str; 7] = [
    "trip",
    "per Agent",
    "payback",
    "Agents to saturate",
    "2 Agents",
    "3 Agents",
    "4 Agents",
];

/// A ten-minute game; incomes are averaged over its second half.
const GAME: Tick = at(10, 0);
const INCOME_FROM: Tick = at(5, 0);

/// `(tick, worker, drop-off, amount)` of every delivery of Space Crystals so far.
fn deliveries(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId, u32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
                resource: Resource::Crystals,
            } => Some((*tick, worker, drop_off, amount)),
            _ => None,
        })
        .collect()
}

/// The ceiling (module docs): the deliveries at `tunnel` come no closer together than DropOffDuration.
fn assert_ceiling(t: &TestSim, tunnel: ObjectId, what: &str) {
    let ticks: Vec<Tick> = (deliveries(t).into_iter())
        .filter(|d| d.2 == tunnel)
        .map(|d| d.0)
        .collect();
    for pair in ticks.windows(2) {
        assert!(pair[1] - pair[0] >= DROP_OFF, "{what}: deliveries at {pair:?}");
    }
}

/// The income at `tunnel` (any, if `None`) of the deliveries after tick `from` up to tick `to`, per second.
fn income(t: &TestSim, tunnel: Option<ObjectId>, from: Tick, to: Tick) -> f64 {
    let amount: u32 = (deliveries(t).into_iter())
        .filter(|d| d.0 > from && d.0 <= to && tunnel.is_none_or(|tunnel| d.2 == tunnel))
        .map(|d| d.3)
        .sum();
    f64::from(amount) / seconds(to - from)
}

/// One Agent's steady trip at `distance`, ticks.
fn trip(distance: i32) -> Tick {
    let mut t = TestSim::new(reference_field(distance, 1));
    let agent = field_agents(1);
    t.order_at(1, FIELD_PLAYER, &agent, OrderKind::Gather(FIELD_PATCH), false);
    t.run_until(8 * 256);
    let gaps: Vec<Tick> = deliveries(&t).windows(2).map(|d| d[1].0 - d[0].0).collect();
    assert!(gaps.len() >= 6, "{distance} GU: {gaps:?}");
    assert!(gaps[2..].iter().all(|&gap| gap == gaps[2]), "{distance} GU: {gaps:?}");
    gaps[2]
}

/// The income of `agents` Agents at `distance` (module docs), with the ceiling checked.
fn income_of(distance: i32, agents: u32) -> f64 {
    let mut t = TestSim::new(reference_field(distance, agents));
    t.order_at(
        1,
        FIELD_PLAYER,
        &field_agents(agents),
        OrderKind::Gather(FIELD_PATCH),
        false,
    );
    t.run_until(GAME);
    assert_ceiling(&t, FIELD_TUNNEL, &format!("{agents} Agents at {distance} GU"));
    income(&t, None, INCOME_FROM, GAME)
}

/// The row at `distance`, measured.
fn measure(distance: i32) -> Row {
    let trip = seconds(trip(distance));
    let per_agent = LOAD / trip;
    let incomes = [2, 3, 4].map(|agents| income_of(distance, agents));
    row(trip, per_agent, 100.0 / per_agent, trip / seconds(DROP_OFF), incomes)
}

fn figures(row: &Row) -> [f64; 7] {
    let [two, three, four] = row.incomes;
    [row.trip, row.per_agent, row.payback, row.saturate, two, three, four]
}

/// The row as the table prints it, each figure with its gap to the design's.
fn print(distance: i32, measured: &Row) {
    let design = figures(&DESIGN[distance as usize]);
    let cells: Vec<String> = (figures(measured).iter().zip(design))
        .map(|(&m, d)| format!("{m:6.2} ({:+5.1}%)", (m - d) / d * 100.0))
        .collect();
    println!("{distance} GU | {}", cells.join(" | "));
}

/// The figures of `measured` at `distance` outside the tolerance, by name.
fn off(distance: i32, measured: &Row) -> Vec<(i32, &'static str)> {
    let design = figures(&DESIGN[distance as usize]);
    (FIGURES.iter().zip(figures(measured).iter().zip(design)))
        .filter(|&(_, (&m, d))| ((m - d) / d).abs() > TOLERANCE)
        .map(|(&name, _)| (distance, name))
        .collect()
}

/// The check's row at `distance` (module docs): its figures within 15% of the design's, but for those pinned out of
/// it; the ceiling in each of its games; and no income above 12.5 per second, the Syndicate's one-base saturation,
/// which is below the GDO's and the Colonists' (`factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`,
/// "Reference check": "GDO fully plated = 26.7 per second for 8 plates", "Colonists fully saturated = 32 per second",
/// as the GDO and Colonists oracles measure them).
fn check_row(distance: i32) {
    let measured = measure(distance);
    print(distance, &measured);
    let pinned: Vec<(i32, &str)> = (OUT_OF_TOLERANCE.iter())
        .filter(|&&(at, _)| at == distance)
        .copied()
        .collect();
    assert_eq!(
        off(distance, &measured),
        pinned,
        "the figures outside the tolerance changed: bring the table to the designer (design change 0108), pin it here \
         and update docs/economy_oracle.md"
    );
    let best = measured.incomes.iter().copied().fold(0.0, f64::max);
    let (gdo, colonists) = (8.0 * 10.0 * 16.0 / 48.0, 8.0 * 4.0);
    println!(
        "{distance} GU: at most {best:.2} per second; the GDO fully plated {gdo:.2}, the Colonists {colonists:.2}"
    );
    assert!(
        best <= CEILING && best < gdo && gdo < colonists,
        "the Syndicate's is the lowest"
    );
}

#[test]
fn syndicate_row_at_3_gu() {
    check_row(3);
}

#[test]
fn syndicate_row_at_4_gu() {
    check_row(4);
}

#[test]
fn syndicate_row_at_5_gu() {
    check_row(5);
}

#[test]
fn syndicate_row_at_6_gu() {
    check_row(6);
}

/// The whole table, rows 0 to 2 included, for the record (design change 0294: "Rows 0–2 are theoretical, since
/// collision can cost income there"): nothing is checked but the ceiling.
#[test]
#[ignore = "the whole table, for docs/economy_oracle.md: run with --release --ignored --nocapture"]
fn syndicate_table() {
    println!("distance | trip | per Agent | payback | to saturate | 2 Agents | 3 Agents | 4 Agents");
    for distance in 0..=6 {
        print(distance, &measure(distance));
    }
}

/// "Stacked Tunnels double a field's intake": [`stacked_field`] at 3 grid units with four Agents for each Tunnel,
/// each sent to its own side's patch, against twice the ceiling, within the tolerance; each Tunnel within it, and the
/// ceiling at each.
#[test]
fn stacked_tunnels_double_a_fields_intake() {
    let mut t = TestSim::new(stacked_field(3, 4));
    let (west, east) = stacked_agents(4);
    t.order_at(1, FIELD_PLAYER, &west, OrderKind::Gather(FIELD_PATCH), false);
    t.order_at(1, FIELD_PLAYER, &east, OrderKind::Gather(STACKED_PATCH), false);
    t.run_until(GAME);
    let mut total = 0.0;
    for (tunnel, agents) in [(FIELD_TUNNEL, &west), (STACKED_TUNNEL, &east)] {
        assert_ceiling(&t, tunnel, "stacked");
        let at_tunnel = deliveries(&t).into_iter().filter(|d| d.2 == tunnel);
        // (M4-W3) Four Agents work each Tunnel. Until design change 0323 they were the four sent to its side; now
        // one of each side has changed over to the other Tunnel, from its first load on: on its first way back it
        // steps a whole width round an Agent in its way (`docs/movement.md > 6.3`), presumably its leg begins
        // again from there, and the other Tunnel's hatch is the nearer. The field's intake is what it was.
        let mut workers: Vec<u32> = at_tunnel.clone().map(|d| d.1.0).collect();
        workers.sort_unstable();
        workers.dedup();
        assert_eq!(workers.len(), agents.len(), "four Agents work each Tunnel: {workers:?}");
        let each = income(&t, Some(tunnel), INCOME_FROM, GAME);
        println!("stacked at 3 GU: Tunnel {} takes in {each:.2} per second", tunnel.0);
        assert!(((each - CEILING) / CEILING).abs() <= TOLERANCE, "{each}");
        total += each;
    }
    let single = income_of(3, 4);
    println!(
        "stacked at 3 GU: {total:.2} per second in all, against {} and twice one Tunnel's with four Agents \
         ({single:.2})",
        2.0 * CEILING
    );
    assert!(
        ((total - 2.0 * CEILING) / (2.0 * CEILING)).abs() <= TOLERANCE,
        "{total}"
    );
    assert!(
        ((total - 2.0 * single) / (2.0 * single)).abs() <= TOLERANCE,
        "{total} against {single}"
    );
}

/// The opening (module docs) on [`opening_field`] at 3 grid units: the starting Agent is sent to the patch and the
/// Hideout rallied on it, and the Hideout trains an Agent whenever it is free and the bank covers it (the player's
/// commands through the gate, `Syndicate`); the books balance every tick. The second Agent starts at once, in tick 1,
/// and the Tunnel has its third Agent when that Agent comes out of the Tunnel to gather: within 0:34–0:46.
#[test]
fn the_opening_saturates_a_tunnel_around_0_40() {
    let line = BuildOrder::new(FIELD_PLAYER).track([Order::new(ObjectKind::Hideout, ObjectKind::Agent); 2]);
    let mut syndicate = Syndicate::new(line)
        .gather(FIELD_PATCH)
        .rally(Target::Object(FIELD_PATCH));
    let mut t = TestSim::new(opening_field(3));
    let mut out = Vec::new();
    while t.sim().tick() < at(1, 30) {
        let tick = t.sim().tick() + 1;
        for command in syndicate.commands(t.sim()) {
            t.at(tick, command.player, command.command);
        }
        let (books, supplies) = (
            Books::open(t.sim(), FIELD_PLAYER),
            SupplyBooks::open(t.sim(), FIELD_PLAYER),
        );
        t.step();
        books.close(t.sim(), syndicate.spent_in(tick));
        supplies.close(t.sim(), syndicate.supplies_spent_in(tick));
        let left = t.sim().events().iter().filter_map(|e| match *e {
            SimEvent::LeftNetwork { unit, tunnel } if tunnel == FIELD_TUNNEL => Some(unit),
            _ => None,
        });
        out.extend(left.map(|unit| (tick, unit)));
    }
    let trained: Vec<Tick> = syndicate.line().issued().iter().map(|i| i.tick).collect();
    println!("opening at 3 GU: Agents trained in ticks {trained:?}, out at {out:?}");
    assert_eq!(trained.first(), Some(&1), "the second Agent starts at once");
    assert_eq!(out.len(), 2, "both trained Agents come out to gather");
    let third = out[1].0;
    let income = income(&t, Some(FIELD_TUNNEL), at(1, 0), at(1, 30));
    println!(
        "opening at 3 GU: the third Agent out at {}; 1:00 to 1:30, {income:.2} per second",
        clock(third)
    );
    assert!((at(0, 34)..=at(0, 46)).contains(&third), "{}", clock(third));
}
