//! The GDO lines (slice M2-J1): a build order played for player 1 on the reference base, with its books, power and
//! plates step by step, and what can be read off them.

use std::collections::BTreeMap;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, BuildOrder, Entry, Order};
use sc_sim::{MapDef, ObjectId, ObjectKind, PowerSummary, Scenario, SimConfig, SimEvent, TICKS_PER_SECOND, Tick};

use super::{P1, clock, seconds};

use ObjectKind::{
    Barracks, DeploymentCenter as DC, ExtractionFacility as EF, ExtractionPlate as Plate, Generator, Peacekeeper as PK,
};

/// A plate's cycle at full power, ticks: "10 Space Crystals per 48 frames" (`factions/gdo/economy.md >
/// ExtractionPlate > MiningRate`), paid at its end (0098).
pub(crate) const CYCLE: Tick = 48;
/// The income window: three cycles, 9 seconds.
pub(crate) const WINDOW: Tick = 3 * CYCLE;
/// What a plate pays per cycle (`> MiningRate`).
pub(crate) const LUMP: u32 = 10;
/// A plate's income at full power, Space Crystals per second.
pub(crate) const PLATE_RATE: f64 = LUMP as f64 * TICKS_PER_SECOND as f64 / CYCLE as f64;
/// The design's costs: `> ExtractionFacility > Constructs`, `factions/gdo/structures/deployment_center.md >
/// Constructs` and `factions/gdo/structures/barracks.md > Produces`.
pub(crate) const COSTS: [(ObjectKind, u32); 4] = [(Plate, 75), (Generator, 100), (Barracks, 200), (PK, 50)];
/// The design's build times, frames: the same sections.
pub(crate) const FRAMES: [(ObjectKind, u32); 4] = [(Plate, 400), (Generator, 240), (Barracks, 160), (PK, 80)];

pub(crate) fn design(table: &[(ObjectKind, u32)], kind: ObjectKind) -> u32 {
    table.iter().find(|(k, _)| *k == kind).unwrap().1
}

/// The power ratio by the design's formula, available / used, at most 1 (`> ExtractionPlate > PowerPenalty`, design
/// change 0089): independent of the sim's fixed point.
pub(crate) fn ratio(power: PowerSummary) -> f64 {
    if power.consumed == 0 {
        return 1.0;
    }
    (f64::from(power.generated) / f64::from(power.consumed)).min(1.0)
}

/// The reference base: `maps/dev_skirmish.json` as a game, from its real start.
pub(crate) fn reference_base() -> Scenario {
    let map: MapDef = serde_json::from_str(include_str!("../../../../maps/dev_skirmish.json")).unwrap();
    Scenario::new(map)
}

/// One order of a line as it went: the tick it started in (its command applied; the recipe's first frame), the
/// tick at whose end it finished (ready to place, or the unit released) and, for a building, the tick it was placed
/// in.
#[derive(Clone, Debug)]
pub(crate) struct Run {
    pub(crate) kind: ObjectKind,
    pub(crate) cost: u32,
    pub(crate) start: Tick,
    pub(crate) done: Option<Tick>,
    pub(crate) placed: Option<Tick>,
}

impl Run {
    /// How many ticks it took: from its first frame to the end of its last.
    pub(crate) fn ticks(&self) -> Tick {
        self.done.expect("finished") - self.start + 1
    }
}

/// A line played: the game, its build order, and player 1's accounts.
pub(crate) struct Played {
    pub(crate) name: &'static str,
    pub(crate) t: TestSim,
    pub(crate) line: BuildOrder,
    /// Player 1's books: `books[i]` is the step to tick `i + 1`'s.
    pub(crate) books: Vec<Entry>,
    /// Player 1's power as each step left it, and so as it worked in that step: `power[i]` is tick `i + 1`'s.
    pub(crate) power: Vec<PowerSummary>,
    /// Each patch under a plate of player 1's, with the tick the plate was placed in (0 for the starting plates).
    pub(crate) plated: BTreeMap<ObjectId, Tick>,
}

impl Played {
    /// Plays `line` for player 1 on the reference base to tick `until`.
    pub(crate) fn play(name: &'static str, mut line: BuildOrder, until: Tick) -> Self {
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
    pub(crate) fn bank(&self, tick: Tick) -> u32 {
        self.books[tick as usize - 1].bank
    }

    /// What player 1's plates mined from tick `from + 1` to tick `to`.
    pub(crate) fn mined(&self, from: Tick, to: Tick) -> u32 {
        self.books[from as usize..to as usize].iter().map(|e| e.mined).sum()
    }

    /// Income at `tick`, Space Crystals per second: the average over the [`WINDOW`] ending then (over the first
    /// window for a time before its end).
    pub(crate) fn income(&self, tick: Tick) -> f64 {
        let end = tick.max(WINDOW);
        f64::from(self.mined(end - WINDOW, end)) / seconds(WINDOW)
    }

    pub(crate) fn power_at(&self, tick: Tick) -> PowerSummary {
        self.power[tick as usize - 1]
    }

    /// Each plate's payments, by its patch: the ticks it paid in, and how much.
    pub(crate) fn payments(&self) -> BTreeMap<ObjectId, Vec<(Tick, u32)>> {
        let mut payments: BTreeMap<ObjectId, Vec<(Tick, u32)>> = BTreeMap::new();
        for (i, e) in self.books.iter().enumerate() {
            for &(patch, amount) in &e.paid {
                payments.entry(patch).or_default().push((i as Tick + 1, amount));
            }
        }
        payments
    }

    /// The line's orders as they went, in the order they were issued.
    pub(crate) fn runs(&self) -> Vec<Run> {
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
    pub(crate) fn runs_of(&self, kind: ObjectKind) -> Vec<Run> {
        self.runs().into_iter().filter(|r| r.kind == kind).collect()
    }

    /// Whether player 1's power ratio was the same through ticks `from` to `to`: full power throughout, or the same
    /// penalty.
    pub(crate) fn steady(&self, from: Tick, to: Tick) -> bool {
        let speed = ratio(self.power_at(from));
        (from..=to).all(|t| ratio(self.power_at(t)) == speed)
    }

    /// The ticks in which player 1's power was negative.
    pub(crate) fn negative(&self) -> Vec<Tick> {
        (1..=self.power.len() as Tick)
            .filter(|&t| self.power_at(t).is_negative())
            .collect()
    }

    /// Exact checks of the design's numbers that hold on every line: each order paid the design's cost (the books
    /// saw the bank pay it), and, where the power ratio stayed the same throughout, each recipe and each plate's
    /// cycle took its frames at that ratio, to the whole frame: `frames / ratio` ticks, the last frame's tick
    /// counting whole. The Deployment Center is never slowed (0089); the Extraction Facility's plates, the Barracks'
    /// units and the plates' cycles are (`> ExtractionPlate > PowerPenalty`, 0098). Every lump is 10.
    pub(crate) fn check_design_numbers(&self) {
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
    pub(crate) fn print(&self, checkpoints: &[Tick]) {
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
pub(crate) fn plates_first(until: Tick) -> Played {
    let line = BuildOrder::new(P1)
        .track(vec![Order::new(EF, Plate); 6])
        .track(vec![Order::new(DC, Generator); 20]);
    Played::play("Plates first (the reference line)", line, until)
}
