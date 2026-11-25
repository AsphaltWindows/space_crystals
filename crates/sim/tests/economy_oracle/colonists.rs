//! The Colonists' economy (slice M3-K1) against `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`: the
//! Colonists' column of its table, within ±15% (design change 0108), and its targets 1 to 4, against the GDO's
//! reference line (`played::plates_first`, slice M2-J1) measured the same way. Nothing here is tuned to the table:
//! a figure out of the tolerance, or a target that fails, goes to the Designer as a question with the measured table
//! (0108), and stays pinned here as measured until the answer comes.
//!
//! **The line** (`sc_sim::testing::colonists`): player 1 on the reference field (`colonists::reference_field`, design
//! change 0107) from the Colonists' Starting Conditions, one Robo Portal and four Drones with 100 Space Crystals. The
//! portal trains Drones without a break and is rallied on the middle patch; the four starting Drones are sent there
//! with one command; the occupancy rule deals them all out over the row. A Drone deploys whenever the room left
//! under Beacon Capacity falls to 1 as it comes out: the 5th, 11th, 17th and 23rd made, "every 6th" (`factions/
//! colonists/units.md > Drone`), at sites beside the portal. Neither money nor Beacon Capacity ever holds the portal
//! up: the k-th Drone comes out at 10·k seconds, exactly.
//!
//! **Income** at a time is what the Drones delivered over the 21 seconds centred on it ([`WINDOW`], 7 of a plate's
//! cycles and about 4 of a Drone's trips), the GDO's what its plates paid over the same window: an average that
//! neither lags a growing income nor runs ahead of it. At 0:00, which the design gives as the starting economy's
//! rate ("Two plates (≈6.7 Space Crystals per second) roughly match the Colonists' four starting Drones (≈6 per
//! second at typical distances)"), the GDO's is its first window and the Colonists' is the four starting Drones'
//! own, from each one's first delivery over its next [`TRIPS`] trips: their walk out and first load are a start-up,
//! not their rate.
//!
//! **Measured** (`--nocapture` prints the table), with the Drone's numbers of design change 0205 (slice M3-K3:
//! thrust and drag ×1.5, the same top speed reached sooner), Colonists against the design, with the line's working
//! Drones and Beacons, and the GDO:
//!
//! | Time | Colonists | Design | Delta | Drones | Beacons | GDO | Design |
//! |---|---|---|---|---|---|---|---|
//! | 0:00 | 6.93 | 6.1 | +13.5% | 4 | 0 | 6.67 | 6.7 |
//! | 0:40 | 11.43 | 12.1 | −5.5% | 8 | 0 | 10.00 | 10.0 |
//! | 1:00 | 13.33 | 15.0 | −11.1% | 9 | 1 | 12.86 | 13.3 |
//! | 1:20 | 16.00 | 16.5 | −3.0% | 11 | 1 | 15.24 | 16.7 |
//! | 1:40 | 18.67 | 19.2 | −2.8% | 13 | 1 | 17.62 | 20.0 |
//! | 2:20 | 22.86 | 23.1 | −1.1% | 16 | 2 | 23.33 | 23.3 |
//! | 2:40 | 25.14 | 25.7 | −2.2% | 18 | 2 | 26.19 | 26.7 |
//! | 3:00 | 26.29 | 27.9 | −5.8% | 19 | 3 | 26.67 | 26.7 |
//! | 4:00 | 27.81 | 31.9 | −12.8% | 24 | 4 | 26.67 | 26.7 |
//!
//! Every figure is within ±15%. The counts are the design's ("Colonists at 2 Drones per patch ≈ 23 Space Crystals
//! per second for 18 Drones [...] including 2 Beacons"; "~24 working Drones" at full saturation). **Targets** (each
//! checked below as it reads): 2, 3 and 4 hold; **1 fails**, at 2 Drones per patch only: the Colonists are ahead at
//! 0:40 (+1.4) and 1:00 (+0.5) and grow faster than the GDO to 1:40, but by 2:20 they have grown less (+15.9
//! against +16.7), as they start higher (6.93, above the GDO's 6.67 and the table's 6.1). Target 4 holds at the
//! design's ~24 working Drones (27.8 at 4:00, and 32.0 for 24 Drones dealt out in `curve.rs`, against the GDO's
//! 26.7). The line passes the GDO's ceiling between 3:40 and 4:00, with 23 to 24 working Drones.
//!
//! **Why** (the figures for Q0055, which stays open): a Drone's trip is 73 ticks at the middle patches and 75, 83 and
//! 95 from there out (1.75 down to 1.35 per second and Drone), so three Drones' mining, 96 ticks, covers even an end
//! patch's trip and 8 patches are mined without a break with 24 Drones. A new Drone delivers first 139 to 154 ticks
//! (about 9 s) after it comes out, a few more where it is sent on from patch to patch: its walk from the portal is
//! long, at top speed either way, and only its starts and stops are shorter. So, with a Drone every 10 seconds, one
//! is always on its first trip, about 1.5 per second the table seems to count. Dealing every Drone to the patch with
//! the fewest instead ([`Colonists::deal`], printed by the ignored `measure_the_colonists_lines`) gives a little more
//! at 1:00 (13.71) and a little less from 2:40 (24.76; 27.43 at 4:00).
//!
//! **Before design change 0205** (slice M3-K1, the question Q0055): 1:00, 3:00 and 4:00 were outside ±15% (12.57,
//! 23.62 and 24.38) and 0:40 just in (10.29); targets 1 and 4 failed, the line passing the GDO's ceiling at about
//! 4:40, with 28 working Drones. The trips were 81 ticks at the middle patches and 82, 92 and 104 from there out
//! (1.58 to 1.23 per second and Drone), and 8 patches needed 27 Drones; a new Drone delivered first about 150 ticks
//! after it came out (`docs/economy_oracle.md` has both tables).

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, BuildOrder, Entry, Order};
use sc_sim::testing::colonists::{Colonists, FIELD_DEAL, FIELD_MIDDLE, FIELD_SITES, field_drones, reference_field};
use sc_sim::{ObjectId, ObjectKind, SimConfig, SimEvent, Target, Tick};

use super::played::{Played, plates_first};
use super::{P1, at, clock, seconds};

use ObjectKind::{Beacon, Drone, RoboPortal};

/// `> Economy Targets (GDO vs Colonists)`'s table: the time, the Colonists' income and the GDO's.
const TABLE: [(Tick, f64, f64); 9] = [
    (at(0, 0), 6.1, 6.7),
    (at(0, 40), 12.1, 10.0),
    (at(1, 0), 15.0, 13.3),
    (at(1, 20), 16.5, 16.7),
    (at(1, 40), 19.2, 20.0),
    (at(2, 20), 23.1, 23.3),
    (at(2, 40), 25.7, 26.7),
    (at(3, 0), 27.9, 26.7),
    (at(4, 0), 31.9, 26.7),
];
/// The table's row "2 Drones per patch".
const TWO_PER_PATCH: Tick = at(2, 20);
/// The table's last row, "~24 working Drones" (`> Reference check`: "Colonists fully saturated ≈ 32 per second (~24
/// working Drones)").
const LAST: Tick = at(4, 0);
/// Design change 0108: each income figure "within ±15% of the table".
const TOLERANCE: f64 = 0.15;
/// The figures measured outside [`TOLERANCE`] (module docs): pinned, so that a change that brings one in, or takes
/// another out, is noticed and reported. None since design change 0205; before it, 1:00, 3:00 and 4:00 (Q0055).
const OUT_OF_TOLERANCE: [Tick; 0] = [];
/// The line's working Drones and Beacons at the table's times: the design's counts where it gives them, 16 and 2 at
/// 2 Drones per patch and 24 working at 4:00.
const COUNTS: [(u32, u32); 9] = [
    (4, 0),
    (8, 0),
    (9, 1),
    (11, 1),
    (13, 1),
    (16, 2),
    (18, 2),
    (19, 3),
    (24, 4),
];
/// Target 1 fails as measured, and target 4 holds at the design's ~24 working Drones (module docs): pinned, as
/// [`OUT_OF_TOLERANCE`] is. Before design change 0205 target 4 failed there too (Q0055).
const TARGET_1_HOLDS: bool = false;
const TARGET_4_HOLDS_AT_24_DRONES: bool = true;
/// The income window: the 21 seconds centred on the time measured.
const WINDOW: Tick = 336;
/// The lines are played to the end of the last window.
const END: Tick = LAST + WINDOW / 2;
/// The starting Drones' trips their income at 0:00 is measured over.
const TRIPS: usize = 4;
/// "every 6th Drone becomes a Beacon": the Drones made that deploy, by their number.
const DEPLOYED: [usize; 4] = [5, 11, 17, 23];
/// The design's costs: `factions/colonists/structures.md > Robo Portal > Produces` ("Drone: 50 Space Crystals") and
/// `factions/gdo/economy.md > ExtractionFacility > Constructs` ("Extraction Plate: 75 Space Crystals").
const DRONE_COST: u32 = 50;
const PLATE_COST: u32 = 75;

/// What player 1 has at the end of a tick.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Counts {
    /// Drones, those sent to deploy among them.
    drones: u32,
    /// Drones with a Deploy order, on their way to their site.
    walking: u32,
    /// Beacons, those under deployment among them.
    beacons: u32,
}

impl Counts {
    fn of(t: &TestSim) -> Self {
        let sim = t.sim();
        let own = |kind| sim.objects().filter(|o| o.owner == Some(P1) && o.kind == kind).count() as u32;
        Self {
            drones: own(Drone),
            walking: sim.pending_deploys(P1).len() as u32,
            beacons: own(Beacon),
        }
    }

    /// The Drones that gather.
    fn working(&self) -> u32 {
        self.drones - self.walking
    }
}

/// The Colonists' reference line played: the game, the line, and player 1's books and counts.
struct Line {
    t: TestSim,
    line: Colonists,
    /// Player 1's books: `books[i]` is the step to tick `i + 1`'s.
    books: Vec<Entry>,
    /// What player 1 has as each step left it, the start's first: `counts[tick]`.
    counts: Vec<Counts>,
}

impl Line {
    /// The reference line (module docs) played to `until`; with `deal`, each Drone is dealt to a patch of its own
    /// instead ([`Colonists::deal`]). Every command the line gives is one the gate takes, and player 1's books balance
    /// every step.
    fn play(until: Tick, deal: bool) -> Self {
        let mut t = TestSim::with_config(reference_field(4), SimConfig::default());
        let line = Colonists::new(BuildOrder::new(P1).track(vec![Order::new(RoboPortal, Drone); 40]))
            .rally(RoboPortal, Target::Object(FIELD_MIDDLE))
            .deploy_at(FIELD_SITES);
        let mut line = if deal {
            line.deal(FIELD_DEAL)
        } else {
            line.gather(FIELD_MIDDLE)
        };
        let (mut books, mut counts) = (Vec::new(), vec![Counts::of(&t)]);
        while t.sim().tick() < until {
            let tick = t.sim().tick() + 1;
            let open = Books::open(t.sim(), P1);
            for c in line.commands(t.sim()) {
                if let Err(reject) = t.sim().validate(&c) {
                    panic!("tick {tick}: the gate refuses {c:?}: {reject:?}");
                }
                t.at(tick, c.player, c.command);
            }
            t.step();
            books.push(open.close(t.sim(), line.spent_in(tick)));
            counts.push(Counts::of(&t));
        }
        Self { t, line, books, counts }
    }

    /// What player 1's Drones delivered from tick `from + 1` to tick `to`.
    fn delivered(&self, from: Tick, to: Tick) -> u32 {
        self.books[from as usize..to as usize].iter().map(|e| e.delivered).sum()
    }

    /// Income at `tick`, Space Crystals per second: what the Drones delivered over the [`WINDOW`] centred on it; at
    /// 0:00, the starting Drones' ([`Line::starting`]).
    fn income(&self, tick: Tick) -> f64 {
        if tick == 0 {
            return self.starting();
        }
        let from = tick - WINDOW / 2;
        f64::from(self.delivered(from, from + WINDOW)) / seconds(WINDOW)
    }

    /// The four starting Drones' income once they are under way: what each delivered over its [`TRIPS`] trips after
    /// its first delivery, over their time, summed.
    fn starting(&self) -> f64 {
        field_drones(4)
            .iter()
            .map(|&drone| {
                let deliveries = self.deliveries(drone);
                let (first, last) = (deliveries[0].0, deliveries[TRIPS].0);
                let amount: u32 = deliveries[1..=TRIPS].iter().map(|&(_, n)| n).sum();
                f64::from(amount) / seconds(last - first)
            })
            .sum()
    }

    /// `drone`'s deliveries: the tick, and how much.
    fn deliveries(&self, drone: ObjectId) -> Vec<(Tick, u32)> {
        self.books
            .iter()
            .enumerate()
            .flat_map(|(i, e)| {
                e.deliveries
                    .iter()
                    .filter(move |&&(worker, _)| worker == drone)
                    .map(move |&(_, n)| (i as Tick + 1, n))
            })
            .collect()
    }

    /// The Drones the Robo Portal made, in order: the tick each came out in, and the Drone.
    fn made(&self) -> Vec<(Tick, ObjectId)> {
        self.t
            .events()
            .iter()
            .filter_map(|(tick, e)| match *e {
                SimEvent::UnitProduced { unit, .. } => Some((*tick, unit)),
                _ => None,
            })
            .collect()
    }

    /// How many of the game's events `matches` picks.
    fn count(&self, matches: fn(&SimEvent) -> bool) -> usize {
        self.t.events().iter().filter(|(_, e)| matches(e)).count()
    }

    /// Prints the line: each Drone, when it came out and whether it deployed, when it first delivered and its trips;
    /// then the table.
    fn print(&self, name: &str, g: &Played) {
        println!("== {name}, played to {}", clock(self.books.len() as Tick));
        let starting = field_drones(4).into_iter().map(|d| (0, d));
        for (i, (tick, drone)) in starting.chain(self.made()).enumerate() {
            let deliveries = self.deliveries(drone);
            let deploy = self.line.deploys().iter().find(|d| d.drone == drone);
            let trips: Vec<Tick> = deliveries.windows(2).map(|w| w[1].0 - w[0].0).take(8).collect();
            let what = match (deploy, deliveries.first()) {
                (Some(d), _) => format!("deploys on ({}, {})", d.origin.x, d.origin.y),
                (None, Some(&(first, _))) => format!("first delivery {} ticks after, trips {trips:?}", first - tick),
                (None, None) => "no delivery yet".to_string(),
            };
            println!("  Drone {:2} (#{}) at {}: {what}", i + 1, drone.0, clock(tick));
        }
        println!("  time | Colonists design  delta | Drones Beacons | GDO   design  delta | bank");
        for (tick, colonists, gdo) in TABLE {
            let (c, gdo_measured) = (self.income(tick), gdo_income(g, tick));
            let counts = self.counts[tick as usize];
            println!(
                "  {} | {c:9.2} {colonists:6.1} {:+5.1}% | {:6} {:7} | {gdo_measured:5.2} {gdo:6.1} {:+5.1}% | {}",
                &clock(tick)[..4],
                100.0 * delta(c, colonists),
                counts.working(),
                counts.beacons,
                100.0 * delta(gdo_measured, gdo),
                tick.checked_sub(1).map_or(100, |i| self.books[i as usize].bank),
            );
        }
    }
}

/// The GDO's income on its reference line (`played::plates_first`), measured as the Colonists' is: over the
/// [`WINDOW`] centred on `tick`, at 0:00 over the first.
fn gdo_income(g: &Played, tick: Tick) -> f64 {
    let from = tick.saturating_sub(WINDOW / 2);
    f64::from(g.mined(from, from + WINDOW)) / seconds(WINDOW)
}

/// The relative difference of `measured` from `design`.
fn delta(measured: f64, design: f64) -> f64 {
    measured / design - 1.0
}

/// The reference line against the Colonists' column of `> Economy Targets (GDO vs Colonists)` and the GDO's line,
/// with targets 1 to 4 (module docs):
/// - **the line as the design has it**: a Drone every 10 seconds, the portal never waiting for money or Beacon
///   Capacity; the 5th, 11th, 17th and 23rd Drones made deploy, and every deployment completes; the working Drones
///   and Beacons at the table's times are the design's where it gives them;
/// - **each figure** within ±15% of the table (design change 0108), but the ones measured outside it, pinned;
/// - **the targets** as they read, each pinned as measured.
#[test]
fn the_colonists_reference_line_against_the_targets() {
    let line = Line::play(END, false);
    let g = plates_first(END);
    line.print("The Colonists' reference line", &g);

    let made: Vec<Tick> = line.made().iter().map(|&(tick, _)| tick).collect();
    assert_eq!(
        made,
        (1..=made.len() as Tick).map(|k| 160 * k).collect::<Vec<_>>(),
        "a Drone every 10 s"
    );
    assert!(made.len() >= 25);
    let deployed: Vec<ObjectId> = line.line.deploys().iter().map(|d| d.drone).collect();
    let every_6th: Vec<ObjectId> = DEPLOYED.iter().map(|&k| line.made()[k - 1].1).collect();
    assert_eq!(deployed, every_6th, "the 5th, 11th, 17th and 23rd Drones deploy");
    assert_eq!(
        line.count(|e| matches!(e, SimEvent::ProductionWaiting { .. })),
        0,
        "never waits for room"
    );
    assert_eq!(line.count(|e| matches!(e, SimEvent::DeployFailed { .. })), 0);
    let completed = line.count(|e| matches!(e, SimEvent::TransformCompleted { kind: Beacon, .. }));
    assert_eq!(completed, DEPLOYED.len());
    let counts: Vec<(u32, u32)> = TABLE
        .iter()
        .map(|&(tick, ..)| line.counts[tick as usize])
        .map(|c| (c.working(), c.beacons))
        .collect();
    assert_eq!(counts, COUNTS, "working Drones and Beacons at the table's times");

    // 0108: each figure within ±15%, but those measured outside (the question draft).
    for (tick, design, _) in TABLE {
        let income = line.income(tick);
        let off = delta(income, design);
        let known = OUT_OF_TOLERANCE.contains(&tick);
        assert_eq!(
            off.abs() > TOLERANCE,
            known,
            "{}: {income:.2} per second against the table's {design}, {:+.1}%: {}",
            clock(tick),
            100.0 * off,
            if known {
                "within ±15% now, where the Designer was told it isn't: update OUT_OF_TOLERANCE and the docs, and tell \
                 the Director"
            } else {
                "outside ±15%: a question for the Designer (design change 0108); add it to OUT_OF_TOLERANCE"
            }
        );
    }

    let colonists = |tick: Tick| line.income(tick);
    let gdo = |tick: Tick| gdo_income(&g, tick);
    let working = |tick: Tick| line.counts[tick as usize].working();
    let grown = |income: &dyn Fn(Tick) -> f64, tick| income(tick) - income(0);

    // Target 1: "Early on, the Colonists' economy grows faster — until they reach about 2 Drones per patch"; the
    // table: "The Colonists pull ahead in the first minute". From 0:00 to every time of the table until 2 Drones per
    // patch the Colonists' income grows more than the GDO's, and in the first minute it is ahead.
    let early = TABLE
        .iter()
        .map(|&(tick, ..)| tick)
        .filter(|&t| t > 0 && t <= TWO_PER_PATCH);
    let faster = early.clone().all(|t| grown(&colonists, t) > grown(&gdo, t));
    let ahead = [at(0, 40), at(1, 0)].iter().all(|&t| colonists(t) > gdo(t));
    println!(
        "  target 1: grown by 0:40, 1:00, 1:20, 1:40, 2:20: Colonists {:?}, GDO {:?}; ahead at 0:40 and 1:00: {ahead}",
        early
            .clone()
            .map(|t| format!("{:.2}", grown(&colonists, t)))
            .collect::<Vec<_>>(),
        early.map(|t| format!("{:.2}", grown(&gdo, t))).collect::<Vec<_>>(),
    );
    assert_eq!(faster && ahead, TARGET_1_HOLDS, "target 1 (TARGET_1_HOLDS)");

    // Target 2: "After that, the Colonists grow more slowly (each extra Drone adds less, and every 6th Drone becomes
    // a Beacon)": from 2 Drones per patch to 4:00 less per second and less per working Drone added than before it;
    // every 6th Drone deploys (above).
    let per_second = |from: Tick, to: Tick| (colonists(to) - colonists(from)) / (seconds(to) - seconds(from));
    let per_drone = |from: Tick, to: Tick| (colonists(to) - colonists(from)) / f64::from(working(to) - working(from));
    let (before, after) = (per_drone(0, TWO_PER_PATCH), per_drone(TWO_PER_PATCH, LAST));
    println!("  target 2: per working Drone added {before:.2} before 2:20, {after:.2} after");
    assert!(after < before, "target 2: each extra Drone adds less");
    assert!(
        per_second(TWO_PER_PATCH, LAST) < per_second(0, TWO_PER_PATCH),
        "target 2: growing more slowly"
    );

    // Target 3: "GDO's full saturation yields more Space Crystals per second, and costs less, than the Colonists at 2
    // Drones per patch": the GDO's fully plated income above the Colonists' at 2:20, and its 8 plates cheaper than
    // the Colonists' Drones then, the Beacons' included (the design's "900 Space Crystals, including 2 Beacons").
    let at_two = line.counts[TWO_PER_PATCH as usize];
    let (gdo_cost, colonists_cost) = (8 * PLATE_COST, (at_two.drones + at_two.beacons) * DRONE_COST);
    println!(
        "  target 3: GDO fully plated {:.2} per second for {gdo_cost}, Colonists at 2 per patch {:.2} for \
         {colonists_cost}",
        gdo(LAST),
        colonists(TWO_PER_PATCH)
    );
    assert!(
        gdo(LAST) > colonists(TWO_PER_PATCH) && gdo_cost < colonists_cost,
        "target 3"
    );
    assert_eq!(colonists_cost, 900, "the design's 18 Drones");

    // Target 4: "The Colonists' full saturation yields more than GDO's full saturation", at the design's ~24 working
    // Drones (4:00). At full saturation, which takes more Drones: `curve.rs`.
    println!(
        "  target 4: Colonists at 4:00, 24 working Drones, {:.2}; GDO {:.2}",
        colonists(LAST),
        gdo(LAST)
    );
    assert_eq!(
        colonists(LAST) > gdo(LAST),
        TARGET_4_HOLDS_AT_24_DRONES,
        "target 4 at ~24 working Drones"
    );
}

/// The measurements behind the module docs, printed: both lines' Drones and tables (the reference line, and each
/// Drone dealt to a patch of its own), and the reference line to 6:00, every 20 seconds, as it passes the GDO's
/// ceiling and saturates.
#[test]
#[ignore = "a measurement: prints the Colonists' lines; run it in release"]
fn measure_the_colonists_lines() {
    let later = at(6, 0);
    let g = plates_first(END);
    let line = Line::play(later + WINDOW / 2, false);
    line.print("The Colonists' reference line", &g);
    Line::play(END, true).print("Each Drone dealt to a patch of its own", &g);
    println!("== The reference line on, against the GDO's ceiling of 26.67");
    for s in (200..=360).step_by(20) {
        let tick = at(0, s);
        let counts = line.counts[tick as usize];
        println!(
            "  {}: {:5.2} per second, {} working Drones, {} Beacons",
            &clock(tick)[..4],
            line.income(tick),
            counts.working(),
            counts.beacons
        );
    }
}
