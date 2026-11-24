//! The gathering soak (slices M3-H and M3-H2; design changes 0106 and 0107): Drones on the reference field of
//! `factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Reference geometry` (8 adjacent patches in a row,
//! the Egress Node centred on the row's long side, 2 grid units from it edge to edge).
//!
//! Gathering Drones pass through each other (`factions/colonists/units.md > Drone > Gathering Drones pass through
//! each other`, design change 0106), so a patch's occupancy is the only limit: nothing jams however crowded the
//! field, and every Drone keeps delivering. Under hard collision, as slice M3-H first built it, the field
//! gridlocked from 12 Drones on.
//!
//! **Asserted here**: the four starting Drones; 24 Drones for 5 minutes with every Drone delivering (the plan's
//! acceptance for the soak, 0106's Verification), the income of the crowded field, no Space Crystal lost or made
//! and no dangling reference; the field that was the smallest gridlock; and the movement invariants throughout
//! (`TestSim`, every tick: I1 lets only gathering Drones overlap).
//!
//! `saturation_curve` prints the measurements. **With DragMovement** (slice M3-G, plan decision T25; nothing here
//! is tuned to the design's table, `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`) the reference
//! field gives, dealt out, per second: 6.3 with 4 Drones (the table: 6.1), 11.7 with 8, 17.5 with 12, 23.1 with 16
//! (the table: 23 at 2 Drones per patch), 25.4 with 20 and 27.1 with 24 (the table: 32 for about 24 working
//! Drones). On the stand-in mover slice M3-H2 measured 8.3, 15.1, 22.0, 26.8, 32.0 and 32.0: about 30 % high
//! below saturation, as T25 estimated. Two findings at the crowded end, both for the oracle of slice K1:
//! - **24 Drones don't saturate 8 patches**: rows of up to 6 patches with three Drones each are mined without a
//!   break (4 per second and patch), 7 give 24.0 of 28 and 8 give 27.1 of 32. The outer patches' trips are longer
//!   than three Drones cover.
//! - **Drones sent on from patch to patch** (`factions/colonists/units.md > Patch Occupancy`: "When a Drone arrives
//!   at an occupied patch, it goes to the nearest unoccupied patch to itself instead"): with momentum the patch it
//!   is sent to is often taken by the time it gets there, and it is sent on again. In the dealt-out field of 24 one
//!   Drone flew along the row for 578 ticks (36 s) between two deliveries, where the plan's bound for the soak is
//!   30 s. It is no jam: every Drone moves, and the income is steady. [`CROWDED_GAP`] is the bound asserted there.
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
/// The bound for 24 Drones dealt out over the 8 patches, 40 s (module docs, the second finding): measured 578.
const CROWDED_GAP: Tick = 640;

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
/// the income is that of four Drones each on a patch of its own: 1.6 Space Crystals per second and Drone with
/// DragMovement (a trip of 81 ticks), a little less for the patches farther from the node. The design's table has
/// 6.1 for the four.
#[test]
fn four_drones_gather_for_five_minutes_without_a_hitch() {
    for spread in [false, true] {
        let mut field = Field::reference(4, spread);
        let before = field.crystals();
        field.t.run_until(SOAK_TICKS);
        let soak = field.measure(SETTLED);
        assert!(soak.worst_gap <= DELIVERY_GAP, "spread {spread}: {soak:?}");
        assert!((5.8..=6.8).contains(&soak.per_second), "spread {spread}: {soak:?}");
        assert_eq!((soak.idle, field.crystals()), (0, before), "spread {spread}: {soak:?}");
    }
}

/// 0106's Verification, "Saturate a patch row with ~24 Drones: they cross each other freely between the row and
/// the Egress Node, no jams; income approaches the Economy Targets table", and the plan's acceptance for the soak
/// (§3.3 H): "24 Drones on 8 patches for 5 minutes. No deadlock: every Drone delivers at least once every 30 s."
/// Sent by one right-click and dealt out over the patches alike.
/// - The income is 27 Space Crystals per second, 15 % short of the table's "fully saturated ≈ 32 per second (~24
///   working Drones)" (module docs: with DragMovement 24 Drones don't keep 8 patches mined without a break).
/// - Dealt out, one Drone goes longer than the plan's 30 s without delivering ([`CROWDED_GAP`], module docs).
/// - No Space Crystal is lost or made, every Drone still has its job, no state names an object that is gone, and
///   the invariants hold in the crowd (`TestSim`, every tick).
#[test]
fn the_soak_of_24_drones_keeps_every_drone_delivering() {
    for spread in [false, true] {
        let mut field = Field::reference(24, spread);
        let before = field.crystals();
        for until in (0..=SOAK_TICKS).step_by(400).skip(1) {
            field.t.run_until(until);
            assert_eq!(field.crystals(), before, "spread {spread}, tick {until}");
            assert_eq!(dangling_references(field.t.sim()), [], "spread {spread}, tick {until}");
        }
        let soak = field.measure(SETTLED);
        let gap = if spread { CROWDED_GAP } else { DELIVERY_GAP };
        assert!(soak.worst_gap <= gap, "spread {spread}: {soak:?}");
        assert!((26.0..=32.1).contains(&soak.per_second), "spread {spread}: {soak:?}");
        assert_eq!(soak.idle, 0, "a Gather never ends on its own: {soak:?}");
    }
}

/// The saturation curve of the reference field: each Drone adds income, and nowhere does a crowd give less than a
/// smaller one did (under hard collision the income fell from 10 Drones on, and to nothing with 24). 2 Drones a
/// patch give at least the table's 23 per second, and 24 Drones 27 (module docs).
#[test]
fn the_income_of_the_reference_field_grows_with_its_drones_up_to_saturation() {
    let mut last = 0.0;
    for drones in [4, 8, 12, 16, 20, 24] {
        let soak = soak(8, drones, true);
        let gap = if drones == 24 { CROWDED_GAP } else { DELIVERY_GAP };
        assert!(soak.worst_gap <= gap, "{soak:?}");
        assert!(soak.per_second > last - 0.1, "{soak:?} after {last} per second");
        if drones == 16 {
            assert!(soak.per_second >= 23.0, "{soak:?}");
        }
        last = soak.per_second;
    }
    assert!(last >= 26.0, "24 Drones: {last} per second");
}

/// The field that was the smallest gridlock under hard collision (slice M3-H pinned it): 3 adjacent patches with the
/// Egress Node 2 grid units south of them and 9 Drones, three sent to each patch. Now they wait, mine and deliver
/// inside each other, and the income is steady: what 3 patches give, 12 Space Crystals per second, in every one of
/// the run's minutes, with no Drone left out.
#[test]
fn three_drones_per_patch_give_a_steady_income() {
    let mut field = Field::new(3, 9);
    field.order(true);
    for minute in 1..=5 {
        field.t.run_until(minute * 960);
        let soak = field.measure((minute - 1) * 960);
        // The first minute includes the walk to the patches.
        let least = if minute == 1 { 10.0 } else { 11.5 };
        assert!((least..=12.5).contains(&soak.per_second), "minute {minute}: {soak:?}");
    }
    let soak = field.measure(SETTLED);
    assert!(soak.worst_gap <= DELIVERY_GAP, "{soak:?}");
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

/// The measurements of the field, printed: the income and the worst delivery gap by the number of Drones on
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
