//! The economy oracle (slice M2-J1): GDO openings played from the real start of the reference base and checked
//! against the design's numbers, `factions/gdo/economy.md > Opening Economy Baseline` and the GDO column of `> Economy
//! Targets (GDO vs Colonists)`, with design changes 0098 (plates pay lumps of 10; the cycle timer advances by the power
//! ratio) and 0089 (the power penalty; the Deployment Center is never slowed).
//!
//! Each line is player 1's on `maps/dev_skirmish.json` from its Starting Conditions (a Deployment Center, an Extraction
//! Facility, two plates and 100 Space Crystals, all 8 patches of the field in range: the reference base), played by a
//! scripted build order (`sc_sim::testing::build_order`): the player's own commands through the gate, each order as
//! soon as the bank covers it and its builder is free, each building placed at once at the first spot the placement
//! check takes, plates onto patches. Player 2 does nothing. Player 1's books balance every tick ([`Books`]), every
//! lump is followed to its patch, and each line is played only to the last time it checks.
//!
//! The design computed its numbers with a trickle of income; the sim pays plates in lumps of 10 at the end of each
//! 48-frame cycle (0098) and counts whole frames. The tolerances follow from that, each named where it is used:
//! - **Income** at a time is the average over the 9 seconds (3 cycles, [`WINDOW`]) ending then: a plate that mined
//!   through the window counts exactly 10/3 per second, as in the design. A plate that lands within the 12 seconds
//!   before counts less, as its first lump comes 3 seconds after it lands.
//! - **A wait for money** ends up to one cycle (3 s) later than with a trickle: the time the lumps take to cover it.
//! - **A bank** lags a trickle's by less than one lump (10) per plate.
//! - **Costs, build times, lumps and the power penalty** are checked exactly against the design's numbers, to the
//!   whole frame, so that a wrong cost, rate, build time, penalty or refund fails here whatever the tolerances above.
//!
//! `cargo test -p sc_sim --test economy_oracle -- --nocapture` prints each line's timeline: when each order started,
//! finished and was placed, and the bank, income and power at the line's checkpoints.

use std::collections::BTreeMap;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, BuildOrder, Entry, Order, first_spot};
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, MapDef, ObjectId, ObjectKind, PlayerId, PowerSummary,
    ProductionCommand, Scenario, Sim, SimConfig, SimEvent, TICKS_PER_SECOND, Tick,
};

use ObjectKind::{
    Barracks, DeploymentCenter as DC, ExtractionFacility as EF, ExtractionPlate as Plate, Generator, Peacekeeper as PK,
};

const P1: PlayerId = PlayerId(1);

/// A plate's cycle at full power, ticks: "10 Space Crystals per 48 frames" (`factions/gdo/economy.md >
/// ExtractionPlate > MiningRate`), paid at its end (0098).
const CYCLE: Tick = 48;
/// The income window: three cycles, 9 seconds.
const WINDOW: Tick = 3 * CYCLE;
/// What a plate pays per cycle (`> MiningRate`).
const LUMP: u32 = 10;
/// A plate's income at full power, Space Crystals per second.
const PLATE_RATE: f64 = LUMP as f64 * TICKS_PER_SECOND as f64 / CYCLE as f64;
/// The design's costs: `> ExtractionFacility > Constructs`, `factions/gdo/structures.md > DeploymentCenter >
/// Constructs` and `> Barracks > Produces`.
const COSTS: [(ObjectKind, u32); 4] = [(Plate, 75), (Generator, 100), (Barracks, 200), (PK, 50)];
/// The design's build times, frames: the same sections.
const FRAMES: [(ObjectKind, u32); 4] = [(Plate, 400), (Generator, 240), (Barracks, 160), (PK, 80)];

/// The tick at `m` minutes and `s` seconds of the game.
const fn at(m: u32, s: u32) -> Tick {
    (m * 60 + s) * TICKS_PER_SECOND
}

fn seconds(tick: Tick) -> f64 {
    f64::from(tick) / f64::from(TICKS_PER_SECOND)
}

/// "m:ss.ss".
fn clock(tick: Tick) -> String {
    let s = seconds(tick);
    format!("{}:{:05.2}", (s / 60.0).floor(), s % 60.0)
}

fn design(table: &[(ObjectKind, u32)], kind: ObjectKind) -> u32 {
    table.iter().find(|(k, _)| *k == kind).unwrap().1
}

/// The power ratio by the design's formula, available / used, at most 1 (`> ExtractionPlate > PowerPenalty`, design
/// change 0089): independent of the sim's fixed point.
fn ratio(power: PowerSummary) -> f64 {
    if power.consumed == 0 {
        return 1.0;
    }
    (f64::from(power.generated) / f64::from(power.consumed)).min(1.0)
}

/// The reference base: `maps/dev_skirmish.json` as a game, from its real start.
fn reference_base() -> Scenario {
    let map: MapDef = serde_json::from_str(include_str!("../../../maps/dev_skirmish.json")).unwrap();
    Scenario::new(map)
}

/// One order of a line as it went: the tick it started in (its command applied; the recipe's first frame), the
/// tick at whose end it finished (ready to place, or the unit released) and, for a building, the tick it was placed
/// in.
#[derive(Clone, Debug)]
struct Run {
    kind: ObjectKind,
    cost: u32,
    start: Tick,
    done: Option<Tick>,
    placed: Option<Tick>,
}

impl Run {
    /// How many ticks it took: from its first frame to the end of its last.
    fn ticks(&self) -> Tick {
        self.done.expect("finished") - self.start + 1
    }
}

/// A line played: the game, its build order, and player 1's accounts.
struct Played {
    name: &'static str,
    t: TestSim,
    line: BuildOrder,
    /// Player 1's books: `books[i]` is the step to tick `i + 1`'s.
    books: Vec<Entry>,
    /// Player 1's power as each step left it, and so as it worked in that step: `power[i]` is tick `i + 1`'s.
    power: Vec<PowerSummary>,
    /// Each patch under a plate of player 1's, with the tick the plate was placed in (0 for the starting plates).
    plated: BTreeMap<ObjectId, Tick>,
}

impl Played {
    /// Plays `line` for player 1 on the reference base to tick `until`.
    fn play(name: &'static str, mut line: BuildOrder, until: Tick) -> Self {
        let mut t = TestSim::with_config(reference_base(), SimConfig::default());
        let mut plated: BTreeMap<ObjectId, Tick> = t
            .sim()
            .objects()
            .filter(|o| o.owner == Some(P1))
            .filter_map(|o| Some((o.extractor?.patch, 0)))
            .collect();
        assert_eq!(plated.len(), 2, "the GDO Starting Conditions' two plates");
        let (mut books, mut power) = (Vec::new(), Vec::new());
        while t.sim().tick() < until {
            let tick = t.sim().tick() + 1;
            let open = Books::open(t.sim(), P1);
            for c in line.commands(t.sim()) {
                t.at(tick, c.player, c.command);
            }
            let spent = line.issued().iter().filter(|i| i.tick == tick).map(|i| i.cost).sum();
            t.step();
            books.push(open.close(t.sim(), spent));
            power.push(t.sim().power(P1));
            for e in t.sim().events() {
                if let SimEvent::StructurePlaced { structure, .. } = e
                    && let Some(extractor) = t.sim().object(*structure).and_then(|o| o.extractor)
                {
                    plated.insert(extractor.patch, tick);
                }
            }
        }
        let refused = t
            .events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::PlacementRefused { .. }))
            .count();
        assert_eq!(refused, 0, "{name}: every Place applied");
        assert_eq!(line.stalled(), [], "{name}: every building found a spot");
        Self {
            name,
            t,
            line,
            books,
            power,
            plated,
        }
    }

    /// The bank at the end of `tick`.
    fn bank(&self, tick: Tick) -> u32 {
        self.books[tick as usize - 1].bank
    }

    /// What player 1's plates mined from tick `from + 1` to tick `to`.
    fn mined(&self, from: Tick, to: Tick) -> u32 {
        self.books[from as usize..to as usize].iter().map(|e| e.mined).sum()
    }

    /// Income at `tick`, Space Crystals per second: the average over the [`WINDOW`] ending then (over the first
    /// window for a time before its end).
    fn income(&self, tick: Tick) -> f64 {
        let end = tick.max(WINDOW);
        f64::from(self.mined(end - WINDOW, end)) / seconds(WINDOW)
    }

    fn power_at(&self, tick: Tick) -> PowerSummary {
        self.power[tick as usize - 1]
    }

    /// Each plate's payments, by its patch: the ticks it paid in, and how much.
    fn payments(&self) -> BTreeMap<ObjectId, Vec<(Tick, u32)>> {
        let mut payments: BTreeMap<ObjectId, Vec<(Tick, u32)>> = BTreeMap::new();
        for (i, e) in self.books.iter().enumerate() {
            for &(patch, amount) in &e.paid {
                payments.entry(patch).or_default().push((i as Tick + 1, amount));
            }
        }
        payments
    }

    /// The line's orders as they went, in the order they were issued.
    fn runs(&self) -> Vec<Run> {
        // Each builder finishes and places its orders one at a time, in the order it took them.
        let mut done: BTreeMap<ObjectId, Vec<Tick>> = BTreeMap::new();
        for (tick, e) in self.t.events() {
            if let SimEvent::ConstructionCompleted { constructor: by, .. }
            | SimEvent::UnitProduced { producer: by, .. } = *e
            {
                done.entry(by).or_default().push(*tick);
            }
        }
        let mut placed: BTreeMap<ObjectId, Vec<Tick>> = BTreeMap::new();
        for p in self.line.placed() {
            placed.entry(p.by).or_default().push(p.tick);
        }
        let mut taken: BTreeMap<ObjectId, usize> = BTreeMap::new();
        self.line
            .issued()
            .iter()
            .map(|i| {
                let n = taken.entry(i.by).or_default();
                let run = Run {
                    kind: i.order.kind,
                    cost: i.cost,
                    start: i.tick,
                    done: done.get(&i.by).and_then(|d| d.get(*n)).copied(),
                    placed: placed.get(&i.by).and_then(|p| p.get(*n)).copied(),
                };
                *n += 1;
                run
            })
            .collect()
    }

    /// The runs of `kind`.
    fn runs_of(&self, kind: ObjectKind) -> Vec<Run> {
        self.runs().into_iter().filter(|r| r.kind == kind).collect()
    }

    /// Whether player 1's power ratio was the same through ticks `from` to `to`: full power throughout, or the same
    /// penalty.
    fn steady(&self, from: Tick, to: Tick) -> bool {
        let speed = ratio(self.power_at(from));
        (from..=to).all(|t| ratio(self.power_at(t)) == speed)
    }

    /// The ticks in which player 1's power was negative.
    fn negative(&self) -> Vec<Tick> {
        (1..=self.power.len() as Tick)
            .filter(|&t| self.power_at(t).is_negative())
            .collect()
    }

    /// Exact checks of the design's numbers that hold on every line: each order paid the design's cost (the books
    /// saw the bank pay it), and, where the power ratio stayed the same throughout, each recipe and each plate's
    /// cycle took its frames at that ratio, to the whole frame: `frames / ratio` ticks, the last frame's tick
    /// counting whole. The Deployment Center is never slowed (0089); the Extraction Facility's plates, the Barracks'
    /// units and the plates' cycles are (`> ExtractionPlate > PowerPenalty`, 0098). Every lump is 10.
    fn check_design_numbers(&self) {
        for r in self.runs() {
            assert_eq!(
                r.cost,
                design(&COSTS, r.kind),
                "{}: the cost of a {:?}",
                self.name,
                r.kind
            );
            let Some(done) = r.done else { continue };
            let frames = f64::from(design(&FRAMES, r.kind));
            let slowed = r.kind != Generator && r.kind != Barracks; // the Deployment Center's
            if self.steady(r.start, done) {
                let speed = if slowed { ratio(self.power_at(r.start)) } else { 1.0 };
                let expected = frames / speed;
                let ticks = f64::from(r.ticks());
                assert!(
                    ticks >= expected - 1e-6 && ticks < expected + 1.0,
                    "{}: a {:?} from tick {} took {ticks} ticks, the design's {frames} frames at {speed:.3}: {expected}",
                    self.name,
                    r.kind,
                    r.start
                );
            }
            if let Some(placed) = r.placed {
                assert_eq!(placed, done + 1, "{}: placed at once", self.name);
            }
        }
        for (patch, payments) in self.payments() {
            let mut from = self.plated[&patch];
            for &(tick, amount) in &payments {
                assert_eq!(amount, LUMP, "{}: a lump of patch #{}", self.name, patch.0);
                if self.steady(from + 1, tick) {
                    let expected = f64::from(CYCLE) / ratio(self.power_at(tick));
                    let ticks = f64::from(tick - from);
                    // The timer keeps what a cycle's last tick added past its end, so a cycle may end a tick sooner.
                    assert!(
                        ticks > expected - 1.0 && ticks < expected + 1.0,
                        "{}: patch #{} paid in tick {tick}, {ticks} ticks after the last, at a power ratio {:.3}",
                        self.name,
                        patch.0,
                        ratio(self.power_at(tick))
                    );
                }
                from = tick;
            }
        }
    }

    /// Prints the timeline (shown with `--nocapture`): each order, then the bank, income and power at `checkpoints`.
    fn print(&self, checkpoints: &[Tick]) {
        println!("== {}, played to {}", self.name, clock(self.books.len() as Tick));
        let when = |tick: Option<Tick>| tick.map_or_else(|| "-".to_string(), |t| format!("{} (t{t})", clock(t)));
        for r in self.runs() {
            println!(
                "  {:<16} start {:<16} done {:<16} placed {}",
                format!("{:?}", r.kind),
                when(Some(r.start)),
                when(r.done),
                when(r.placed)
            );
        }
        for &tick in checkpoints {
            let power = self.power_at(tick);
            println!(
                "  at {}: bank {:>4}, income {:5.2}/s, power {:+}",
                clock(tick),
                self.bank(tick),
                self.income(tick),
                i64::from(power.generated) - i64::from(power.consumed)
            );
        }
    }
}

/// `> How the opening plays out`: "Plates first is best: a plate at frame 0, the first Generator once 100 Space
/// Crystals are banked", the Extraction Facility plating the field and the Deployment Center building Generators
/// (the "Generators-only Deployment Center"). The plates get the money first when both builders are free.
fn plates_first(until: Tick) -> Played {
    let line = BuildOrder::new(P1)
        .track(vec![Order::new(EF, Plate); 6])
        .track(vec![Order::new(DC, Generator); 20]);
    Played::play("Plates first (the reference line)", line, until)
}

/// The times of `> Economy Targets (GDO vs Colonists)`'s table and its GDO income there.
const TARGETS: [(Tick, f64); 9] = [
    (at(0, 0), 6.7),
    (at(0, 40), 10.0),
    (at(1, 0), 13.3),
    (at(1, 20), 16.7),
    (at(1, 40), 20.0),
    (at(2, 20), 23.3),
    (at(2, 40), 26.7),
    (at(3, 0), 26.7),
    (at(4, 0), 26.7),
];

/// The reference line against the GDO column of `> Economy Targets (GDO vs Colonists)` and `> Opening Economy
/// Baseline`: income at the table's times; the Extraction Facility plating non-stop but for money, the plates landing
/// every 25 seconds, "fully plated at ~2:30"; "Power is not a constraint in the opening"; and every cost, build time
/// and lump exactly the design's.
#[test]
fn the_plates_first_line_earns_the_targets_income() {
    let g = plates_first(at(4, 0));
    g.print(&TARGETS.map(|(t, _)| t.max(1)));
    g.check_design_numbers();

    // The table's model lands the k-th built plate at 25·k seconds ("6 plates to build at 25 seconds each"). Here
    // each lands then or up to one cycle later: the Extraction Facility waits for its 75 Space Crystals once a
    // Generator took the bank.
    let plates = g.runs_of(Plate);
    assert_eq!(plates.len(), 6);
    for (k, plate) in (1..).zip(&plates) {
        let landed = seconds(plate.placed.unwrap());
        let model = 25.0 * f64::from(k);
        assert!(
            (model..=model + 3.0).contains(&landed),
            "plate {k} landed at {landed} s, the design's at {model} s"
        );
    }
    // "Fully plated at ~2:30": every patch of the field under a plate.
    assert_eq!(g.plated.len(), 8, "all 8 patches plated");
    let full = seconds(plates[5].placed.unwrap());
    assert!((150.0..=153.0).contains(&full), "fully plated at {full} s");

    // Income at the table's times. A plate the table lands within the 15 seconds before (12 seconds for its first
    // lump and the window, 3 for the wait for money above) may not count yet: the measure may be that many plates
    // under the table's figure, never over it (the table rounds to 0.1).
    for (tick, target) in TARGETS {
        let landing = |k: u32| at(0, 25 * k);
        let late = (1..=6)
            .filter(|&k| landing(k) <= tick && landing(k) + at(0, 15) > tick)
            .count();
        let income = g.income(tick);
        let low = target - PLATE_RATE * late as f64 - 0.05;
        assert!(
            (low..=target + 0.05).contains(&income),
            "income at {}: {income:.2}/s, the table's {target} (down to {low:.2})",
            clock(tick)
        );
    }
    assert_eq!(
        g.negative(),
        Vec::<Tick>::new(),
        "\"Power is not a constraint in the opening\""
    );
}

/// `> How the opening plays out`: "from about 1:00 a Generators-only Deployment Center floats Space Crystals (~300
/// banked by 1:30, ~600 by 2:00)". The bank lags a trickle's by less than a lump per plate (5 plates at 1:30, 6 at
/// 2:00) and by 10 more for the Extraction Facility's wait for money at 0:50, which lands two plates 1.5 s later than
/// a trickle would; the design's figures are round, to 15 either way.
#[test]
fn a_generators_only_deployment_center_floats_crystals() {
    let g = plates_first(at(2, 0));
    g.print(&[at(1, 0), at(1, 30), at(2, 0)]);
    for (tick, target, plates) in [(at(1, 30), 300, 5), (at(2, 0), 600, 6)] {
        let bank = g.bank(tick);
        let low = target - LUMP * plates - 10 - 15;
        assert!(
            (low..=target + 15).contains(&bank),
            "the bank at {}: {bank}, the design's ~{target} (from {low})",
            clock(tick)
        );
    }
}

/// `> How the opening plays out`: "Opening with a Generator delays the first plate by ~11 seconds and costs ≈ 8% of
/// income by 3:00", against the reference line. The delay is a wait for 75 Space Crystals from two plates, 11.25 s
/// as a trickle and up to one cycle more with lumps. The income is what the plates mined by 3:00; each second the
/// later plates land later costs about half a point, so the lumps' up to 3 s give 2 points either way.
#[test]
fn opening_with_a_generator_delays_the_first_plate_and_costs_income() {
    let line = BuildOrder::new(P1)
        .track(
            [Order::new(DC, Generator)]
                .into_iter()
                .chain(vec![Order::new(EF, Plate); 6]),
        )
        .track(vec![Order::new(DC, Generator); 20]);
    let g = Played::play("Generator first", line, at(3, 0));
    g.print(&[at(1, 0), at(2, 0), at(3, 0)]);
    g.check_design_numbers();
    let reference = plates_first(at(3, 0));

    let first = |p: &Played| p.runs_of(Plate)[0].start;
    let delay = seconds(first(&g)) - seconds(first(&reference));
    assert!(
        (11.0..=14.25).contains(&delay),
        "the first plate starts {delay} s later, the design's ~11 s"
    );
    let (mined, reference_mined) = (g.mined(0, at(3, 0)), reference.mined(0, at(3, 0)));
    let loss = 100.0 * (1.0 - f64::from(mined) / f64::from(reference_mined));
    println!("  mined by 3:00: {mined}, the reference line {reference_mined}: {loss:.1}% less");
    assert!(
        (6.0..=10.0).contains(&loss),
        "{loss:.1}% less income by 3:00, the design's ≈8%"
    );
    assert_eq!(g.negative(), Vec::<Tick>::new());
}

/// `> How the opening plays out`: "Fastest clean Barracks (Plate, Generator, Generator, Barracks) starts at ~46
/// seconds and finishes at ~56 seconds", "≈48 s instead of 45.8 s" with lumps (`> ExtractionPlate > Paid in
/// lumps`): a wait for money, so up to one cycle after the trickle's 45.8 s. Power never goes negative ("building
/// one without going negative requires two Generators first"). Slice M2-D1 measured it first (its mini-oracle):
/// started in tick 769 and ready at the end of tick 928, pinned here.
#[test]
fn the_fastest_clean_barracks_starts_at_about_48_seconds() {
    let line = BuildOrder::new(P1).track([
        Order::new(EF, Plate),
        Order::new(DC, Generator),
        Order::new(DC, Generator),
        Order::new(DC, Barracks),
    ]);
    let g = Played::play("Fastest clean Barracks", line, 929);
    g.print(&[at(0, 30), at(0, 48), 929]);
    g.check_design_numbers();
    let starts: Vec<(Tick, ObjectKind)> = g.runs().iter().map(|r| (r.start, r.kind)).collect();
    assert_eq!(
        starts,
        [(1, Plate), (193, Generator), (433, Generator), (769, Barracks)]
    );
    let barracks = &g.runs_of(Barracks)[0];
    let start = seconds(barracks.start);
    assert!((45.8..=48.8).contains(&start), "the Barracks starts at {start} s");
    assert_eq!(barracks.done, Some(928));
    let finished = seconds(barracks.done.unwrap() + 1);
    assert!(
        (55.8..=58.8).contains(&finished),
        "the Barracks is ready at {finished} s"
    );
    assert_eq!(g.negative(), Vec::<Tick>::new());
    let power = g.power_at(929);
    assert_eq!((power.generated, power.consumed), (70, 54));
}

/// `> How the opening plays out`: "An all-in Barracks (halting everything else) can start at ~15 seconds, but
/// leaves power at −21, so plates and Peacekeeper production run at ~59% speed until Generators catch up." The line:
/// the Barracks first, nothing bought before it; then a Peacekeeper, a plate and a Peacekeeper at −21; then two
/// Generators, built at full speed all the same (0089), and a last Peacekeeper at −4, after the first Generator.
/// 30 / 51 is 58.8%: a Peacekeeper's 80 frames take 136 ticks, a plate's 400 take 680, and the plates' cycles 81.6
/// ticks.
#[test]
fn an_all_in_barracks_runs_plates_and_peacekeepers_at_59_percent() {
    let line = BuildOrder::new(P1).track([
        Order::new(DC, Barracks),
        Order::new(Barracks, PK),
        Order::new(EF, Plate),
        Order::new(Barracks, PK),
        Order::new(DC, Generator),
        Order::new(DC, Generator),
        Order::new(Barracks, PK),
    ]);
    let g = Played::play("All-in Barracks", line, at(2, 0));
    g.print(&[at(0, 15), at(0, 30), at(1, 0), at(1, 30), at(2, 0)]);
    g.check_design_numbers();
    let runs = g.runs();
    assert_eq!(runs.len(), 7, "the whole line");

    // The Barracks first, at 15 s: 100 more Space Crystals from two plates, 15 s as a trickle; the lumps bring them
    // at 15 s exactly, and the order goes in the next tick.
    let barracks = &runs[0];
    assert_eq!(barracks.kind, Barracks);
    let start = seconds(barracks.start);
    assert!((15.0..=18.0).contains(&start), "the Barracks starts at {start} s");
    // Placed: −21.
    let placed = barracks.placed.unwrap();
    let power = g.power_at(placed);
    assert_eq!((power.generated, power.consumed), (30, 51), "power at −21");

    // At −21, the Peacekeepers and the plate are built at 30/51 ≈ 59% of full speed, and the plates pay at it.
    let pk = &runs[1];
    assert!(g.steady(pk.start, pk.done.unwrap()) && g.power_at(pk.start) == power);
    assert_eq!(pk.ticks(), 136, "a Peacekeeper at −21");
    let speed = f64::from(design(&FRAMES, PK)) / f64::from(pk.ticks());
    assert!((0.58..=0.60).contains(&speed), "~59%: {speed:.3}");
    let plate = &runs[2];
    assert!(g.steady(plate.start, plate.done.unwrap()) && g.power_at(plate.start) == power);
    assert_eq!(plate.ticks(), 680, "a plate at −21");
    let starting = g.payments()[g.plated.iter().find(|(_, t)| **t == 0).unwrap().0].clone();
    let slowed: Vec<Tick> = starting
        .windows(2)
        .filter(|w| g.steady(w[0].0 + 1, w[1].0) && g.power_at(w[1].0) == power)
        .map(|w| w[1].0 - w[0].0)
        .collect();
    assert!(slowed.len() >= 8, "{slowed:?}");
    assert!(slowed.iter().all(|&t| t == 81 || t == 82), "cycles at −21: {slowed:?}");

    // The Generators: never slowed, and once both are placed the power is positive and the cycles take 48 ticks.
    let generators = g.runs_of(Generator);
    assert!(generators.iter().all(|r| r.ticks() == 240));
    assert!(g.power_at(generators[0].start).is_negative());
    let caught_up = generators[1].placed.unwrap();
    assert_eq!(
        g.negative().last(),
        Some(&(caught_up - 1)),
        "until the Generators catch up"
    );
    let after: Vec<Tick> = starting
        .windows(2)
        .filter(|w| w[0].0 > caught_up)
        .map(|w| w[1].0 - w[0].0)
        .collect();
    assert!(!after.is_empty() && after.iter().all(|&t| t == CYCLE), "{after:?}");
}

/// Refunds (`factions/gdo/structures.md > DeploymentCenter > Cancellation`, `> Barracks`): a building cancelled
/// while it is constructed refunds its full cost, one cancelled once ready 75% of it rounded down, and a unit taken
/// off the BuildQueue its full cost. Commands by hand, through the gate; the books balance every tick, so the bank
/// gets exactly what the events report.
#[test]
fn cancelled_orders_refund_what_the_design_says() {
    let mut t = TestSim::with_config(reference_base(), SimConfig::default());
    let own = |sim: &Sim, kind| {
        sim.objects()
            .find(|o| o.owner == Some(P1) && o.kind == kind)
            .map(|o| o.id)
    };
    let dc = own(t.sim(), DC).unwrap();
    let construction = |command| Command::Construction {
        objects: vec![dc],
        command,
    };
    let spot = first_spot(t.sim(), P1, Barracks).unwrap();
    // (tick, command, what it pays): a Generator ready at the end of tick 240 and cancelled; a second cancelled
    // while constructed; a Barracks, placed, and a Peacekeeper taken off its queue.
    let mut plan: Vec<(Tick, Command, u32)> = vec![
        (1, construction(ConstructionCommand::Construct(Generator)), 100),
        (241, construction(ConstructionCommand::CancelReady), 0),
        (242, construction(ConstructionCommand::Construct(Generator)), 100),
        (260, construction(ConstructionCommand::CancelConstruction), 0),
        (337, construction(ConstructionCommand::Construct(Barracks)), 200),
        (
            497,
            construction(ConstructionCommand::Place {
                origin: spot,
                orientation: BuildingOrientation::default(),
            }),
            0,
        ),
    ];
    let mut refunds = Vec::new();
    while t.sim().tick() < 500 {
        let tick = t.sim().tick() + 1;
        if tick == 498 || tick == 499 {
            let barracks = own(t.sim(), Barracks).expect("placed in tick 497");
            let command = if tick == 498 {
                ProductionCommand::Train(PK)
            } else {
                ProductionCommand::CancelProduction
            };
            let cost = if tick == 498 { 50 } else { 0 };
            plan.push((
                tick,
                Command::Production {
                    objects: vec![barracks],
                    command,
                },
                cost,
            ));
        }
        let open = Books::open(t.sim(), P1);
        let mut spent = 0;
        for (_, command, cost) in plan.iter().filter(|(at, ..)| *at == tick) {
            t.sim()
                .validate(&sc_sim::PlayerCommand {
                    player: P1,
                    command: command.clone(),
                })
                .unwrap_or_else(|reject| panic!("tick {tick}: {reject:?}"));
            t.at(tick, P1, command.clone());
            spent += cost;
        }
        t.step();
        let entry = open.close(t.sim(), spent);
        if entry.refunded > 0 {
            refunds.push((tick, entry.refunded));
        }
    }
    println!("== Refunds: {refunds:?}");
    assert_eq!(refunds, [(241, 75), (260, 100), (499, 50)]);
}
