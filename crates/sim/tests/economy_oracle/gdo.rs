//! The GDO's openings (slice M2-J1) against `factions/gdo/economy.md > Opening Economy Baseline` (the module docs of
//! `main.rs`).

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, BuildOrder, Order, first_spot};
use sc_sim::{BuildingOrientation, Command, ConstructionCommand, ObjectKind, ProductionCommand, Sim, SimConfig, Tick};

use super::played::{CYCLE, FRAMES, LUMP, PLATE_RATE, Played, design, plates_first, reference_base};
use super::{P1, TABLE, at, clock, seconds};

use ObjectKind::{
    Barracks, DeploymentCenter as DC, ExtractionFacility as EF, ExtractionPlate as Plate, Generator, Peacekeeper as PK,
};

/// The reference line's income at `tick` by `> How the opening plays out`, as a trickle: the two starting plates and
/// the k-th plate built landing at 25·k seconds ("Fully plated at ~2:30 on the reference base (6 plates to build at 25
/// seconds each)"), each mining [`PLATE_RATE`] from then. Until design change 0236 this model was the GDO column of
/// `> Economy Targets (GDO vs Colonists)`'s table; since then the column is the line measured (`main.rs`'s `TABLE`,
/// checked in `colonists.rs`).
fn model(tick: Tick) -> f64 {
    PLATE_RATE * f64::from(2 + (tick / at(0, 25)).min(6))
}

/// The reference line against `> Opening Economy Baseline`: income at the times of `> Economy Targets (GDO vs
/// Colonists)`'s table by its plates ([`model`]); the Extraction Facility plating non-stop but for money, the plates
/// landing every 25 seconds, "fully plated at ~2:30"; "Power is not a constraint in the opening"; and every cost,
/// build time and lump exactly the design's.
#[test]
fn the_plates_first_line_earns_the_targets_income() {
    let g = plates_first(at(4, 0));
    g.print(&TABLE.map(|(t, ..)| t.max(1)));
    g.check_design_numbers();

    // The model lands the k-th built plate at 25·k seconds ("6 plates to build at 25 seconds each"). Here each
    // lands then or up to one cycle later: the Extraction Facility waits for its 75 Space Crystals once a Generator
    // took the bank.
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

    // Income at the table's times. A plate the model lands within the 15 seconds before (12 seconds for its first
    // lump and the window, 3 for the wait for money above) may not count yet: the measure may be that many plates
    // under the model's figure, never over it (to the float's rounding).
    for (tick, ..) in TABLE {
        let landing = |k: u32| at(0, 25 * k);
        let late = (1..=6)
            .filter(|&k| landing(k) <= tick && landing(k) + at(0, 15) > tick)
            .count();
        let (income, target) = (g.income(tick), model(tick));
        let low = target - PLATE_RATE * late as f64 - 1e-9;
        assert!(
            (low..=target + 1e-9).contains(&income),
            "income at {}: {income:.2}/s, the model's {target:.2} (down to {low:.2})",
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
