//! A soak of the Colonists against the GDO (slice M3-K2, `artifacts/director/m3-plan.md > K2: M3 QA`): the golden
//! game `colonists_vs_gdo`'s players (`sc_sim::testing::colonists::skirmish`, `Skirmish::versus_gdo`) played on for
//! [`MINUTES`] minutes of `dev_skirmish` from its real start, each through the gate with their own commands, as
//! players would. Player 1's Colonists gather, train 30 Drones, deploy Beacons as their Beacon Capacity needs them,
//! warp in a Bio Portal and train eight Sabre Legionnaires, each converted to a Rifle Legionnaire; player 2's GDO
//! plays the fastest clean Barracks and sends ten Peacekeepers to the Colonists' field in two waves of AttackMove.
//! After the fight the Colonists gather on until their field runs out, at about 8½ minutes with the Drone's numbers
//! of design change 0205 (its last patch at 8:36; at 9:02 before 0205).
//!
//! It checks, after every tick:
//! - the invariants: `TestSim`'s (the movement invariants and K1), and that nothing names a removed object
//!   (`testing::dangling_references`);
//! - that the gate takes every command the players give, and that both players' books balance
//!   (`testing::build_order::Books`, the Drones' deliveries included; the GDO's only while no patch under its plates
//!   could run out in the step, which the books can't follow);
//! - that nothing is stuck ([`Watch`]): every order that ends by itself ends within [`ORDER_TICKS`]; a Drone on a
//!   Gather goes from one step of its job to the next, or on to another patch, within [`STEP_TICKS`], bar waiting
//!   for a patch; a Drone waits only at a patch another Drone mines, or for at most [`RECHECK_TICKS`] (it mines its
//!   patch as soon as it frees, and one that waits short of a patch tries again at its next look); and while the
//!   field lasts, its Drones deliver at least every [`QUIET_TICKS`];
//!
//! at the end, that both players' lines ran to their end, that the field ran out with every crystal of it
//! delivered, and that every Gather ended with it; and over the whole game, that it is deterministic: a second run
//! gives the same hash at every checkpoint, and the first run's replay gives its last hash.
//!
//! A Drone may wait its turn long: of the Drones waiting at a patch the lowest id mines next (`orders::gather`; the
//! design leaves the rule to the implementation), and the line's 30 Drones oversaturate the field's eight patches.
//! So how long a Drone went without delivering is printed, not checked, with the longest each order and each step of
//! a Gather lasted.
//!
//! Ignored: two games of 9 600 ticks. `cargo test -p sc_sim --release --test skirmish_soak -- --ignored --nocapture`.

use std::collections::{BTreeMap, BTreeSet};
use std::mem::{Discriminant, discriminant};

use sc_sim::orders::state::GatherPhase;
use sc_sim::orders::{ActiveOrder, Behaviour, OrderGroup};
use sc_sim::testing::build_order::Books;
use sc_sim::testing::colonists::skirmish::{self, COLONISTS, GDO, Skirmish};
use sc_sim::testing::{self, TestSim};
use sc_sim::{
    Command, ConstructionCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerId, Reject, ResourceState, Sim,
    SimEvent, TICKS_PER_SECOND, Target, Tick,
};

/// How long the game runs.
const MINUTES: u32 = 10;
const END: Tick = MINUTES * 60 * TICKS_PER_SECOND;
/// A hash is noted at every multiple of this tick.
const CHECKPOINT: Tick = 64;
/// The longest an order that ends by itself may run: the time a Peacekeeper (4 su a tick) takes to walk twice
/// round `dev_skirmish`'s 64 × 48 tiles of 64 su, far more than the longest way across it and the fights on the way
/// (the GDO's waves walk about 60 tiles round the lake to the Colonists' field).
const ORDER_TICKS: Tick = 2 * (64 + 48) * 64 / 4;
/// The longest a Drone may take over one step of a Gather but waiting: a leg to a patch or a drop-off, its
/// MiningDuration (32 frames) or its DropOffDuration (2). The legs are short, the field lying within 4 tiles of the
/// Egress Node; the longest goes to a patch across the field, round it through the crowd of Drones (the field's arc,
/// the Egress Node and the Robo Portal close the arc's inside in two).
const STEP_TICKS: Tick = 30 * TICKS_PER_SECOND;
/// A waiting Drone looks round again every 16 ticks (`factions/colonists/units.md > Patch Occupancy > Waiting and
/// re-checking`, `tuning::gathering::RECHECK_TICKS`).
const RECHECK_TICKS: Tick = 16;
/// While the field lasts, its Drones deliver at least this often (the first delivery comes after the first Drones'
/// walk to the field and their first load).
const QUIET_TICKS: Tick = 15 * TICKS_PER_SECOND;

/// Slot 1's field, which player 1's Drones gather: the map's first eight spawns, `skirmish::PATCH` among them.
fn field() -> [ObjectId; 8] {
    std::array::from_fn(|i| ObjectId(i as u32 + 1))
}

fn remaining(sim: &Sim, patch: ObjectId) -> Option<u32> {
    match sim.object(patch)?.resource {
        Some(ResourceState::CrystalPatch { remaining }) => Some(remaining),
        _ => None,
    }
}

/// A unit's current order.
fn current(o: &Object) -> Option<&ActiveOrder> {
    o.orders.as_ref()?.current.as_ref()
}

/// What a unit is doing, to see when it changes: its current order (its group and kind), and for a Gather the step
/// of its job. A Gather that goes on to another patch changes its kind (the order follows the Drone from patch to
/// patch, `orders::gather`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Doing {
    group: OrderGroup,
    kind: OrderKind,
    step: Option<Discriminant<GatherPhase>>,
}

impl Doing {
    fn of(current: &ActiveOrder) -> Self {
        let step = match &current.state {
            Behaviour::Gather(gather) => Some(discriminant(&gather.phase)),
            _ => None,
        };
        Self {
            group: current.order.group,
            kind: current.order.kind,
            step,
        }
    }
}

/// A readable name for what a unit does, and the longest it may do it.
fn label(sim: &Sim, current: &ActiveOrder) -> (String, Tick) {
    if let Behaviour::Gather(gather) = &current.state {
        let step = format!("{:?}", gather.phase);
        let step = step.split([' ', '{']).next().unwrap_or_default().to_string();
        let order = match current.order.kind {
            OrderKind::DropOff(_) => "Drop Off",
            _ => "Gather",
        };
        // Waiting lasts while the patches are busy ([`Watch::note`] checks that they are).
        let bound = match gather.phase {
            GatherPhase::Waiting { .. } => Tick::MAX,
            _ => STEP_TICKS,
        };
        return (format!("{order}, {step}"), bound);
    }
    let kind = current.order.kind;
    // Hold and Patrol last until the next order, and so does a Move to a unit, which follows it.
    let bound = match kind {
        OrderKind::Hold | OrderKind::Patrol(_) => Tick::MAX,
        OrderKind::Move(Target::Object(id)) if sim.object(id).is_some_and(|o| o.mover.is_some()) => Tick::MAX,
        _ => ORDER_TICKS,
    };
    let name = format!("{kind:?}");
    (name.split(['(', ' ']).next().unwrap_or_default().to_string(), bound)
}

/// What the soak watches, tick by tick (module docs).
#[derive(Default)]
struct Watch {
    /// What each unit is doing, and since when.
    doing: BTreeMap<ObjectId, (Doing, Tick)>,
    /// Each Drone that waits at a patch nobody mines (or that is gone), since when.
    idle: BTreeMap<ObjectId, Tick>,
    /// The longest each order and each step of a Gather lasted, by name: how long, the unit, and the tick.
    longest: BTreeMap<String, (Tick, ObjectId, Tick)>,
    /// Each Drone on a Gather or a Drop Off: the order's group, and its last delivery in it, or the order's start.
    undelivered: BTreeMap<ObjectId, (OrderGroup, Tick)>,
    /// The longest a Drone went without delivering: how long, the Drone, and the tick.
    gap: (Tick, Option<ObjectId>, Tick),
    /// The last delivery, and the longest the field's Drones went without one while the field lasted.
    last_delivery: Tick,
    quiet: Tick,
    /// What the Drones delivered.
    crystals: u32,
    /// The tick each patch of the field ran out in.
    ran_out: BTreeMap<ObjectId, Tick>,
}

impl Watch {
    /// Notes what the units do at the end of the tick, and checks that none is stuck.
    fn note(&mut self, sim: &Sim) {
        let tick = sim.tick();
        let stuck = |o: &Object, what: &str| {
            format!(
                "tick {tick}: #{} ({:?} of player {:?}) is stuck: {what}: {o:?}",
                o.id.0,
                o.kind,
                o.owner.map(|p| p.0)
            )
        };
        // The patches a Drone mines.
        let mined: BTreeSet<ObjectId> = (sim.objects().filter_map(current))
            .filter_map(|c| match (c.order.kind, &c.state) {
                (OrderKind::Gather(patch), Behaviour::Gather(g)) if matches!(g.phase, GatherPhase::Mining { .. }) => {
                    Some(patch)
                }
                _ => None,
            })
            .collect();
        let (mut doing, mut idle, mut gathering) = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
        for o in sim.objects() {
            let Some(c) = current(o) else { continue };
            let now = Doing::of(c);
            let since = match self.doing.get(&o.id) {
                Some(&(was, since)) if was == now => since,
                _ => tick,
            };
            doing.insert(o.id, (now, since));
            let (name, bound) = label(sim, c);
            let age = tick - since;
            let longest = self.longest.entry(name.clone()).or_insert((0, o.id, tick));
            if age > longest.0 {
                *longest = (age, o.id, tick);
            }
            assert!(age <= bound, "{}", stuck(o, &format!("{name} for {age} ticks")));
            let Behaviour::Gather(gather) = &c.state else { continue };
            gathering.insert(o.id, c.order.group);
            // It waits at a patch nobody mines: it mines it in the next tick, or it waits short of it and tries again
            // at its next look (or looks round, if the patch is gone).
            if let (OrderKind::Gather(patch), GatherPhase::Waiting { since: waiting }) = (c.order.kind, gather.phase)
                && !mined.contains(&patch)
            {
                let since = (self.idle.get(&o.id).copied().unwrap_or(tick)).max(waiting);
                idle.insert(o.id, since);
                let age = tick - since;
                assert!(
                    age <= RECHECK_TICKS,
                    "{}",
                    stuck(
                        o,
                        &format!("waiting at patch #{} for {age} ticks, which nobody mines", patch.0)
                    )
                );
            }
        }
        (self.doing, self.idle) = (doing, idle);

        // Deliveries.
        let delivered: BTreeSet<ObjectId> = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::ResourcesDelivered { worker, amount, .. } => {
                    self.crystals += amount;
                    Some(worker)
                }
                _ => None,
            })
            .collect();
        if !delivered.is_empty() {
            self.last_delivery = tick;
        }
        let mut undelivered = BTreeMap::new();
        for (&drone, &group) in &gathering {
            let since = match self.undelivered.get(&drone) {
                Some(&(was, since)) if was == group && !delivered.contains(&drone) => since,
                _ => tick,
            };
            undelivered.insert(drone, (group, since));
            if tick - since > self.gap.0 {
                self.gap = (tick - since, Some(drone), tick);
            }
        }
        self.undelivered = undelivered;
        for patch in field() {
            if sim.object(patch).is_none() {
                self.ran_out.entry(patch).or_insert(tick);
            }
        }
        if self.ran_out.len() < field().len() && !gathering.is_empty() {
            let quiet = tick - self.last_delivery;
            self.quiet = self.quiet.max(quiet);
            assert!(
                quiet <= QUIET_TICKS,
                "tick {tick}: the field's Drones delivered nothing for {quiet} ticks"
            );
        }
    }
}

/// Whether `player`'s books can follow the step: no patch under one of its plates holds 10 or less, the most a
/// plate takes in a step, which the step could empty (`Books::close`).
fn followable(sim: &Sim, player: PlayerId) -> bool {
    (sim.objects().filter(|o| o.owner == Some(player)))
        .filter_map(|o| remaining(sim, o.extractor?.patch))
        .all(|left| left > 10)
}

/// One game, and what it came to.
struct Run {
    t: TestSim,
    players: Skirmish,
    hashes: Vec<(Tick, u64)>,
    watch: Watch,
    /// The crystals the field held at the start.
    field: u32,
    /// The tick from which the GDO's books were no longer kept, if they stopped.
    books_stopped: Option<Tick>,
}

fn play() -> Run {
    let scenario = skirmish::scenario();
    let config = scenario.setup.config;
    let mut t = TestSim::with_config(scenario, config);
    let mut players = Skirmish::versus_gdo();
    let mut watch = Watch::default();
    let mut hashes = vec![(0, t.sim().state_hash())];
    let field: Vec<u32> = field().iter().filter_map(|&patch| remaining(t.sim(), patch)).collect();
    assert_eq!(field.len(), 8, "dev_skirmish's first eight spawns are slot 1's patches");
    let mut books_stopped = None;
    while t.sim().tick() < END {
        let tick = t.sim().tick() + 1;
        let commands = players.commands(t.sim());
        // The gate takes every command, as the ones before it in the step leave the game; this check sees the game
        // before the step, so a constructor placing its ready building may take its next Construct in the same step
        // (as in the golden games).
        let placing: Vec<ObjectId> = (commands.iter())
            .filter_map(|c| match &c.command {
                Command::Construction {
                    objects,
                    command: ConstructionCommand::Place { .. },
                } => Some(objects.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        for c in &commands {
            match t.sim().validate(c) {
                Ok(()) => {}
                Err(Reject::ConstructorBusy(id)) if placing.contains(&id) => {}
                Err(reject) => panic!("tick {tick}: the gate refuses {c:?}: {reject:?}"),
            }
        }
        let mut books = Vec::new();
        for player in [COLONISTS, GDO] {
            if followable(t.sim(), player) {
                books.push((Books::open(t.sim(), player), players.spent_in(player, tick)));
            } else {
                books_stopped.get_or_insert(tick);
            }
        }
        for c in commands {
            t.at(tick, c.player, c.command);
        }
        t.step();
        for (books, spent) in books {
            books.close(t.sim(), spent);
        }
        assert_eq!(testing::dangling_references(t.sim()), [], "tick {tick}");
        watch.note(t.sim());
        if tick.is_multiple_of(CHECKPOINT) {
            hashes.push((tick, t.sim().state_hash()));
        }
    }
    Run {
        t,
        players,
        hashes,
        watch,
        field: field.iter().sum(),
        books_stopped,
    }
}

/// The units and structures each player has, by kind.
fn census(sim: &Sim) -> BTreeMap<(u8, ObjectKind), usize> {
    let mut counts = BTreeMap::new();
    for o in sim.objects() {
        if let Some(owner) = o.owner {
            *counts.entry((owner.0, o.kind)).or_default() += 1;
        }
    }
    counts
}

#[test]
#[ignore = "a soak of two 10-minute games: run it in release"]
fn ten_minutes_of_the_colonists_against_the_gdo() {
    let run = play();
    let (sim, w) = (run.t.sim(), &run.watch);
    let died = |player| {
        (run.t.events().iter())
            .filter(|(_, e)| matches!(e, SimEvent::Died { owner: Some(p), .. } if *p == player))
            .count()
    };
    println!("at tick {}: {:?}", sim.tick(), census(sim));
    println!(
        "lost: player 1 {}, player 2 {}; banks {:?}; the GDO's books kept to tick {:?}",
        died(COLONISTS),
        died(GDO),
        [COLONISTS, GDO].map(|p| sim.player(p).map_or(0, |p| p.crystals)),
        run.books_stopped
    );
    println!(
        "player 1's Drones delivered {} crystals, the last at tick {}; the field ran out at ticks {:?}",
        w.crystals,
        w.last_delivery,
        w.ran_out.values().collect::<Vec<_>>()
    );
    println!(
        "the longest without a delivery: the field's Drones {} ticks, a Drone {} ticks (#{} at tick {})",
        w.quiet,
        w.gap.0,
        w.gap.1.map_or(0, |id| id.0),
        w.gap.2
    );
    println!("the longest each order or step of a Gather lasted (ticks, the unit, at tick):");
    for (name, (ticks, id, at)) in &w.longest {
        println!("  {name:<20} {ticks:>5} (#{} at tick {at})", id.0);
    }

    // The game went on all along: both lines ran to their end, nobody was defeated, and units died on either side.
    assert!(
        run.players.colonists().line().is_done(),
        "player 1's line ran to its end"
    );
    assert!(
        run.players.gdo().is_some_and(|line| line.is_done()),
        "player 2's line ran to its end"
    );
    assert!(sim.outcome().is_none(), "the game ended: {:?}", sim.outcome());
    assert!(died(COLONISTS) > 0 && died(GDO) > 0, "units died on either side");
    // The field ran out, every crystal of it delivered, and every Gather ended with it.
    assert_eq!(w.ran_out.len(), 8, "the field ran out");
    assert_eq!(w.crystals, run.field, "the field's crystals, delivered");
    let gathering: Vec<ObjectId> = (sim.objects())
        .filter(|o| o.worker.is_some() && o.orders.as_ref().is_some_and(|orders| !orders.is_idle()))
        .map(|o| o.id)
        .collect();
    assert_eq!(gathering, [], "Drones with orders left");

    // Deterministic: the same game again, and the replay of the first.
    let again = play();
    assert_eq!(again.hashes, run.hashes, "a second run");
    assert_eq!(run.t.replay().run().state_hash(), sim.state_hash(), "the replay");
}
