//! Low Power slows the cooldown (design change 0423, `factions/gdo/overview.md > GDOResources > Power`: "**Everything
//! a consumer does is slowed** [...] **ability cooldowns** (the Radar Station's Scan) [...] Each frame, these timers
//! advance by the speed ratio (generated ÷ consumed, capped at 100%), as production does"; `sc_sim::cooldown`).

use sc_sim::economy::RATIO_ONE;

use super::*;

/// [`short`]'s Deployment Center and Radar Station.
const SHORT_DC: u32 = 3;
const STATION: u32 = 7;
/// Where [`short`]'s Deployment Center places its Generators.
const GENERATOR_SPOTS: [TilePos; 2] = [TilePos::new(7, 5), TilePos::new(10, 5)];

/// Player 1 at 30 generated for 51 consumed, "about 59%" (`economy::PowerSummary::ratio`: 38 551 of 65 536, rounded
/// up): crystal patches 1 and 2 at (7, 8) and (10, 8), the Deployment Center 3 (+30) at (2, 2), the Extraction
/// Facility 4 (−15) at (2, 7), plates 5 and 6 (−3 each) on the patches, the Radar Station 7 (−20) at (7, 2) and a
/// Missile Turret 8 (−10) at (10, 2).
fn short() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::CrystalPatch, None, tile(7, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(10, 8))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(2, 7))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(7, 8))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(10, 8))
        .spawn(RADAR, Some(P1), tile(7, 2))
        .spawn(ObjectKind::MissileTurret, Some(P1), tile(10, 2))
}

/// The station's cooldown's progress now, in `RATIO_ONE`s of a tick.
fn elapsed(t: &TestSim) -> u32 {
    cooldown(t, STATION).expect("cooling down").elapsed
}

/// 0423's Verification, "Run Power to about 59% (e.g. 30 generated, 51 consumed): Scan's cooldown takes about 102
/// s": the station scans in tick 2, and its cooldown takes 38 551 a tick, `⌈960 × 65536 / 38551⌉` = 1632 ticks
/// (102 s) from its cast's: its next Scan is refused up to tick 1633 and taken in tick 1634. The Deployment Center, a
/// producer, builds a Generator meanwhile at full speed, in 240 ticks ("Producers (Deployment Center, Generators) are
/// still never slowed").
#[test]
fn at_30_generated_for_51_consumed_the_cooldown_takes_1632_ticks() {
    let mut t = started(short());
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed, power.ratio()), (30, 51, 38_551));
    assert_eq!(power.speed(RADAR), 38_551);
    let c = t.sim().tick() + 1;
    t.at(c, P1, scan(&[STATION], FAR))
        .at(c, P1, construction(SHORT_DC, ConstructionCommand::Construct(GENERATOR)))
        .run_until(c);
    assert_eq!(elapsed(&t), 38_551, "the cast's tick at the ratio");
    let ticks = (u64::from(COOLDOWN_TICKS) * u64::from(RATIO_ONE)).div_ceil(38_551) as Tick;
    assert_eq!(ticks, 1632);
    assert_eq!(ticks * 1000 / 16, 102_000, "102 s");
    t.run_until(c + ticks - 2);
    assert_eq!(elapsed(&t), (ticks - 1) * 38_551);
    assert_eq!(gate(&t, P1, scan(&[STATION], FAR)), Err(Reject::CoolingDown));
    t.run_until(c + ticks - 1);
    assert_eq!(cooldown(&t, STATION), None, "over at the end of tick {}", c + ticks - 1);
    assert_eq!(next(&mut t, scan(&[STATION], FAR)), c + ticks);
    assert_eq!(casts(&t), [(c, STATION, FAR), (c + ticks, STATION, FAR)]);
    let generator = (t.events().iter()).find_map(|(tick, e)| match *e {
        SimEvent::ConstructionCompleted { kind: GENERATOR, .. } => Some(*tick),
        _ => None,
    });
    assert_eq!(generator, Some(c + 239), "the Deployment Center at full speed");
}

/// 0423's Verification, "Build a Generator back to positive: all return to full speed at once", and "Durations already
/// running stretch from that moment on (they progress at the ratio each frame)": the station scans in tick 2, at
/// 30/51, while the Deployment Center builds a Generator. Its cooldown takes 38 551 a tick until the Generator is
/// placed, then 64 251 a tick at 50/51, still short, until the second Generator is placed; from that tick on, at 70/51,
/// a whole tick each, and it is over a whole number of ticks later, what was left of it at full speed.
#[test]
fn generators_that_bring_the_power_back_give_the_cooldown_full_speed_at_once() {
    let mut t = started(short());
    let c = t.sim().tick() + 1;
    t.at(c, P1, scan(&[STATION], FAR))
        .at(c, P1, construction(SHORT_DC, ConstructionCommand::Construct(GENERATOR)))
        .run_until(c);
    let mut speeds: Vec<(Tick, u32)> = Vec::new();
    let mut last = elapsed(&t);
    let mut step = |t: &mut TestSim, speeds: &mut Vec<(Tick, u32)>| {
        t.step();
        let now = elapsed(t);
        speeds.push((t.sim().tick(), now - last));
        last = now;
    };
    let mut placed = Vec::new();
    for spot in GENERATOR_SPOTS {
        while t.sim().object(id(SHORT_DC)).unwrap().constructor.unwrap().ready() != Some(GENERATOR) {
            step(&mut t, &mut speeds);
        }
        let tick = t.sim().tick() + 1;
        t.at(tick, P1, construction(SHORT_DC, place(spot)));
        if placed.is_empty() {
            t.at(
                tick,
                P1,
                construction(SHORT_DC, ConstructionCommand::Construct(GENERATOR)),
            );
        }
        step(&mut t, &mut speeds);
        placed.push(tick);
    }
    let [first, second] = placed[..] else { unreachable!() };
    assert_eq!(
        first,
        c + 240,
        "the first Generator ready at the end of tick {}",
        c + 239
    );
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed, power.ratio()), (70, 51, RATIO_ONE));
    let expected = |tick: Tick| match tick {
        _ if tick < first => 38_551,
        _ if tick < second => 64_251,
        _ => RATIO_ONE,
    };
    assert_eq!((50 * u64::from(RATIO_ONE)).div_ceil(51), 64_251, "50/51, rounded up");
    for &(tick, speed) in &speeds {
        assert_eq!(speed, expected(tick), "tick {tick}");
    }
    let left = u64::from(COOLDOWN_TICKS * RATIO_ONE - elapsed(&t));
    let over = second + left.div_ceil(u64::from(RATIO_ONE)) as Tick;
    t.run_until(over - 1);
    assert!(cooldown(&t, STATION).is_some());
    t.run_until(over);
    assert_eq!(cooldown(&t, STATION), None, "over at the end of tick {over}");
}
