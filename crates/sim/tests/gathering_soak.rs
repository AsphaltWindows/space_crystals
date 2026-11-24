//! The gathering soak (slice M3-H's acceptance; question Q0038): Drones on the reference field of
//! `factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Reference geometry` (design change 0107: 8
//! adjacent patches in a row, the Egress Node centred on the row's long side, 2 grid units from it edge to edge) —
//! under **hard collision with workers that never yield**, the rules this slice was told to build. Design change
//! 0106, the answer to Q0038 point 1, lets gathering Drones pass through each other; it arrived while the slice was
//! built and is not built yet (`orders::gather`, module docs).
//!
//! **What holds, and is asserted here**: the four starting Drones gather without a hitch; however crowded the field,
//! no Space Crystal is lost or made, no state names an object that is gone, and the movement invariants hold
//! (`TestSim` checks them every tick).
//!
//! **What doesn't hold**: the plan's acceptance for the soak, "24 Drones on 8 patches for 5 minutes. No deadlock:
//! every Drone delivers at least once every 30 s". Under hard collision the field **gridlocks**: a Drone that has
//! mined stands against its patch, the next Drone for that patch stands behind it, their neighbours stand left and
//! right, and nobody yields or swaps (`systems/units.md > UnitCollision > Ground Collision`: "Two moving units never
//! yield to each other or swap places", and a unit with a command never yields). That test is here and ignored
//! (`the_soak_of_24_drones_keeps_every_drone_delivering`), to be turned on when design change 0106 is built; the
//! smallest jam found is pinned by `three_drones_per_patch_gridlock_under_hard_collision`, and `saturation_curve`
//! prints the measurements:
//!
//! ```sh
//! cargo test -p sc_sim --release --test gathering_soak -- --ignored --nocapture --test-threads=1 saturation_curve
//! ```

use std::time::Duration;

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, ResourceState, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);

/// 5 minutes.
const SOAK_TICKS: Tick = 4800;
/// Income is measured once the field has settled: after the first 50 seconds.
const SETTLED: Tick = 800;
/// The plan's bound for the soak: "every Drone delivers at least once every 30 s".
const DELIVERY_GAP: Tick = 480;

/// A field: `patches` patches in a row at tiles (8 - patches / 2 .., 3), the Egress Node at tiles (7..9, 6..8), 2
/// grid units south of the row's middle, edge to edge; `drones` Drones in blocks west and east of the node, the
/// nearest places first, so that even a few Drones start in sight of the middle patches. Ids: the patches from 1,
/// from the west; the node after them; then the Drones.
struct Field {
    t: TestSim,
    patches: u32,
    drones: Vec<ObjectId>,
}

impl Field {
    fn new(patches: u32, drones: u32) -> Self {
        let mut scenario = ScenarioBuilder::new(&vec![".".repeat(16); 14].join("\n"));
        let west = 8 - patches as i32 / 2;
        for x in 0..patches as i32 {
            scenario = scenario.spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(west + x, 3)));
        }
        scenario = scenario.spawn(ObjectKind::EgressNode, Some(P1), SpawnAt::Tile(TilePos::new(7, 6)));
        for i in 0..drones as i32 {
            // Beside the node, west and east in turn: three rows of four on each side.
            let (side, place) = (if i % 2 == 0 { -1.0 } else { 1.0 }, i / 2);
            let at = Vec2::new(
                512.0 + side * (92.0 + 44.0 * (place / 3) as f32),
                402.0 + 44.0 * (place % 3) as f32,
            );
            scenario = scenario.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(at));
        }
        Self {
            t: scenario.start(),
            patches,
            drones: (0..drones).map(|i| ObjectId(patches + 2 + i)).collect(),
        }
    }

    /// The reference field with its 8 patches (design change 0107), the Drones ordered as [`Field::order`] says.
    fn reference(drones: u32, spread: bool) -> Self {
        let mut field = Self::new(8, drones);
        field.order(spread);
        field
    }

    /// The orders of tick 1. `spread`: the Drones are dealt out over the patches, those west of the node over the
    /// western patches and those east of it over the eastern ones, the middle patches first. Else all go to a
    /// middle patch, as one right-click sends them, and the occupancy rule deals them out.
    fn order(&mut self, spread: bool) {
        let middle = self.patches.div_ceil(2);
        if !spread {
            self.t
                .order_at(1, P1, &self.drones, OrderKind::Gather(ObjectId(middle)), false);
            return;
        }
        let (western, eastern) = (middle, self.patches - middle);
        for (i, &drone) in self.drones.iter().enumerate() {
            let turn = i as u32 / 2;
            let patch = if i % 2 == 0 || eastern == 0 {
                middle - turn % western
            } else {
                middle + 1 + turn % eastern
            };
            self.t
                .order_at(1, P1, &[drone], OrderKind::Gather(ObjectId(patch)), false);
        }
    }

    fn phase(&self, id: ObjectId) -> Option<GatherPhase> {
        let orders = self.t.sim().object(id)?.orders.as_ref()?;
        match orders.current.as_ref()?.state {
            Behaviour::Gather(gather) => Some(gather.phase),
            _ => None,
        }
    }

    /// The Space Crystals in the game: in the patches, on the Drones and in player 1's bank.
    fn crystals(&self) -> u32 {
        let sim = self.t.sim();
        let held: u32 = sim
            .objects()
            .map(|o| match o.resource {
                Some(ResourceState::CrystalPatch { remaining }) => remaining,
                _ => o.worker.map_or(0, |worker| worker.load),
            })
            .sum();
        held + sim.player(P1).expect("in the game").crystals
    }

    /// What the run measured up to now, with the income from tick `from` on.
    fn measure(&self, from: Tick) -> Soak {
        let now = self.t.sim().tick();
        let index = |worker: ObjectId| self.drones.iter().position(|&id| id == worker).expect("a Drone");
        let mut last = vec![1; self.drones.len()];
        let mut count = vec![0u32; self.drones.len()];
        let (mut delivered, mut worst_gap) = (0, 0);
        for (tick, event) in self.t.events() {
            let SimEvent::ResourcesDelivered { worker, amount, .. } = event else {
                continue;
            };
            let i = index(*worker);
            worst_gap = worst_gap.max(tick - last[i]);
            last[i] = *tick;
            count[i] += 1;
            if *tick >= from {
                delivered += amount;
            }
        }
        Soak {
            drones: self.drones.len() as u32,
            per_second: delivered as f32 * 16.0 / (now - from) as f32,
            worst_gap: last.iter().fold(worst_gap, |worst, &at| worst.max(now - at)),
            fewest: count.iter().copied().min().unwrap_or(0),
            most: count.iter().copied().max().unwrap_or(0),
            idle: self.drones.iter().filter(|&&id| self.phase(id).is_none()).count() as u32,
        }
    }
}

/// What a run measured.
#[derive(Debug)]
struct Soak {
    drones: u32,
    /// Space Crystals per second, from the tick the measure was asked from.
    per_second: f32,
    /// The longest time a Drone went without delivering, ticks, from the first order to the end of the run.
    worst_gap: Tick,
    /// The deliveries of the Drone that delivered least, and of the one that delivered most.
    fewest: u32,
    most: u32,
    /// Drones without a job at the end.
    idle: u32,
}

impl Soak {
    /// One line of the measurements.
    fn line(&self) -> String {
        format!(
            "{:2} Drones: {:5.2} per second, worst gap {:4} ticks, {} to {} deliveries a Drone, {} idle",
            self.drones, self.per_second, self.worst_gap, self.fewest, self.most, self.idle
        )
    }
}

fn soak(patches: u32, drones: u32, spread: bool) -> Soak {
    let mut field = Field::new(patches, drones);
    field.order(spread);
    field.t.run_until(SOAK_TICKS);
    field.measure(SETTLED)
}

/// The Colonists' four starting Drones (`factions/colonists/economy.md > Starting Conditions`) on the reference
/// field, for 5 minutes: nobody is in anybody's way for long. Every Drone delivers at least once every 30 s, and
/// the income is that of four Drones each on a patch of its own: 2 Space Crystals per second and Drone on the
/// stand-in mover, a little less for the patches farther from the node.
#[test]
fn four_drones_gather_for_five_minutes_without_a_hitch() {
    for spread in [false, true] {
        let mut field = Field::reference(4, spread);
        let before = field.crystals();
        field.t.run_until(SOAK_TICKS);
        let soak = field.measure(SETTLED);
        assert!(soak.worst_gap <= DELIVERY_GAP, "spread {spread}: {soak:?}");
        assert!((7.0..=8.6).contains(&soak.per_second), "spread {spread}: {soak:?}");
        assert_eq!((soak.idle, field.crystals()), (0, before), "spread {spread}: {soak:?}");
    }
}

/// The soak's safety half, which holds however Drones collide: 24 Drones on the 8 patches for 5 minutes, sent by
/// one right-click. No Space Crystal is lost or made, every Drone still has its job, no state names an object that
/// is gone, and the invariants hold in the crowd (`TestSim`, every tick).
#[test]
fn the_soak_of_24_drones_loses_nothing_and_keeps_the_invariants() {
    let mut field = Field::reference(24, false);
    let before = field.crystals();
    for until in (0..=SOAK_TICKS).step_by(400).skip(1) {
        field.t.run_until(until);
        assert_eq!(field.crystals(), before, "tick {until}");
        assert_eq!(dangling_references(field.t.sim()), [], "tick {until}");
    }
    assert_eq!(field.measure(SETTLED).idle, 0, "a Gather never ends on its own");
}

/// Slice M3-H's acceptance for the soak (plan §3.3 H): "24 Drones on 8 patches for 5 minutes. No deadlock: every
/// Drone delivers at least once every 30 s." **It fails under hard collision** (module docs): the field gridlocks
/// in the first minute. To be turned on with design change 0106.
#[test]
#[ignore = "fails under hard collision with workers that never yield: design change 0106 isn't built yet"]
fn the_soak_of_24_drones_keeps_every_drone_delivering() {
    for spread in [false, true] {
        let soak = soak(8, 24, spread);
        assert!(soak.worst_gap <= DELIVERY_GAP, "spread {spread}: {soak:?}");
    }
}

/// The smallest jam found, pinned: 3 adjacent patches with the Egress Node 2 grid units south of
/// them and 9 Drones, three sent to each patch. They mine a load or two, then stand in each other's way for good: a
/// loaded Drone against its patch, the Drones for that patch behind and beside it. From 50 s to 5 minutes nearly
/// nothing is delivered, where 3 patches give Drones 12 Space Crystals per second.
///
/// This pins the game as it is, not as it should be: it is to go, with the ignored acceptance test above turned on
/// in its place, when design change 0106 is built and gathering Drones pass through each other.
#[test]
fn three_drones_per_patch_gridlock_under_hard_collision() {
    let soak = soak(3, 9, true);
    assert!(soak.per_second < 1.0, "{soak:?}");
    assert!(
        soak.worst_gap > 2400,
        "a Drone went half the run without a delivery: {soak:?}"
    );
    assert_eq!(soak.idle, 0, "they all still have their Gather: {soak:?}");
}

/// Wall-clock time of `work`. Only this measurement reads a clock: it measures the sim from outside, and nothing in
/// the sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

/// The measurements of the field under hard collision, printed: the income and the worst delivery gap by the number of Drones on
/// the reference field, for both ways of ordering them; rows of 1 to 8 patches with two and three Drones a patch;
/// and the tick cost of the 24-Drone soak.
#[test]
#[ignore = "a measurement: prints the saturation curve and the soak's tick cost; run it in release"]
fn saturation_curve() {
    for spread in [false, true] {
        println!("8 patches, {}:", if spread { "dealt out" } else { "one right-click" });
        for drones in [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24] {
            println!("  {}", soak(8, drones, spread).line());
        }
    }
    for patches in 1..=8 {
        for per_patch in [2, 3] {
            println!("{patches} patches, {}", soak(patches, patches * per_patch, true).line());
        }
    }
    let mut field = Field::reference(24, false);
    field.t.check_invariants(false);
    let (_, took) = timed(|| field.t.run_until(SOAK_TICKS));
    println!(
        "24 Drones, {SOAK_TICKS} ticks: {:.1} µs a tick",
        took.as_secs_f64() * 1e6 / f64::from(SOAK_TICKS)
    );
}
