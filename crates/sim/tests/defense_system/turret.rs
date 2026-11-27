//! The Missile Turret (`factions/gdo/structures/missile_turret.md > Placeholder stats`: "Attack: **HeadDisjointed**
//! (homing missiles, cannot miss; user, 2026-11-27), TargetDomain Universal, SingleTarget; Damage 18; Range 6 grid
//! units; MinRange 0; ProjectileSpeed 16 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 14 frames (one missile per 24
//! frames)"; "Targeting as the Laser Turret's (BaseAutoTargeting, closest, keeps its target)"), and low Power (design
//! change 0423: "**weapons** (the Missile Turret's whole attack cycle: aim, firing, cooldown and reload take longer by
//! the ratio)").

use sc_sim::economy::RATIO_ONE;

use super::*;

/// [`short`]'s Deployment Center, Defense System and Missile Turret.
const SHORT_DC: u32 = 3;
pub(super) const SHORT_DS: u32 = 7;
const SHORT_TURRET: u32 = 8;
/// Where [`short`]'s Deployment Center places its Generators.
const GENERATOR_SPOTS: [TilePos; 2] = [TilePos::new(7, 5), TilePos::new(10, 5)];

/// Player 1 at 30 generated for 51 consumed, "about 59%" (`economy::PowerSummary::ratio`: 38 551 of 65 536, rounded
/// up): crystal patches 1 and 2 at (7, 8) and (10, 8), the Deployment Center 3 (+30) at (2, 2), the Extraction
/// Facility 4 (−15) at (2, 7), plates 5 and 6 (−3 each) on the patches, the Defense System 7 (−20) at (7, 2) and the
/// Missile Turret 8 (−10) at (10, 2), on a flat map of 48 × 32 tiles.
pub(super) fn short() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::CrystalPatch, None, tile(7, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(10, 8))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(2, 7))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(7, 8))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(10, 8))
        .spawn(DEFENSE, Some(P1), tile(7, 2))
        .spawn(TURRET, Some(P1), tile(10, 2))
}

/// A flat map of 40 × 30 tiles with player 1's Missile Turret 1 on tile (10, 10), its centre at (672, 672), and its
/// Generator 2 in the north-west corner, which powers it (with none generated, the Power ratio is 0, and the turret
/// never fires: design change 0423); player 2 is the Cults', whose Sparrows they are.
fn field() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 30))
        .faction(P2, Faction::Cults)
        .spawn(TURRET, Some(P1), tile(10, 10))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
}

/// What the turret's missiles at `target` did, once it fell: every one launched at it hit it, but those still in
/// flight as it fell (`in flight`), which hit nothing else, and no other missile was launched. The hits, in order.
fn never_missed(t: &TestSim, turret: u32, target: u32, in_flight: usize) -> Vec<(Tick, u32, f32)> {
    let launched = launches(t, turret);
    assert!(
        launched.iter().all(|&(_, at, homing)| at == Some(target) && homing),
        "{launched:?}"
    );
    let hits = hits_by(t, turret);
    assert!(hits.iter().all(|&(_, hit, _)| hit == target), "{hits:?}");
    assert_eq!(hits.len() + in_flight, launched.len(), "{launched:?} {hits:?}");
    let missed =
        (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Missed { attacker, .. } if *attacker == id(turret)));
    assert!(!missed);
    hits
}

/// The plan's acceptance, "hits a circling Sparrow [...] and never misses": player 2's Sparrow 3, on Hold 320 su east
/// of the turret's centre, circles there (`factions/cults/units/sparrow.md > Flight rules`: "**Hold Position**: it
/// circles there and fires only at what comes into range", its Range 3 grid units, which never reaches the turret),
/// always within the turret's Range and sight. The turret, complete from the start, aims at it from tick 1 and fires
/// from tick 6, at the end of its Aim 4 and Firing 2, then every 24 ticks; each missile homes in on the Sparrow and hits
/// it for 18 (its armour 0), and the third brings it down (MaxHP 50).
#[test]
fn every_missile_hits_a_circling_sparrow() {
    let at = Vec2::new(672.0 + 320.0, 672.0);
    let mut t = field().spawn(ObjectKind::Sparrow, Some(P2), point(at.x, at.y)).start();
    order(&mut t, 1, P2, &[3], OrderKind::Hold);
    let mut flying = 0;
    while died(&t, 3).is_none() {
        assert!(t.sim().tick() < 16 * 20, "down in 20 seconds");
        t.step();
        if let Some(sparrow) = t.sim().object(id(3)) {
            assert!(sparrow.pos.distance(at) < 72.0, "circling: {}", sparrow.pos);
            assert!(sparrow.pos != sparrow.prev_pos, "always moving");
            let orders = sparrow.orders.as_ref().unwrap();
            assert_eq!(orders.current.as_ref().map(|a| a.order.kind), Some(OrderKind::Hold));
        } else {
            flying = in_flight(&t, 1);
        }
    }
    let launched: Vec<Tick> = launches(&t, 1).iter().map(|&(tick, ..)| tick).collect();
    assert_eq!(&launched[..3], [6, 30, 54], "from its first tick, every 24");
    let hits = never_missed(&t, 1, 3, flying);
    assert_eq!(hits.len(), 3, "{hits:?}");
    assert!(hits.iter().all(|&(.., damage)| damage == 18.0));
    assert!(hits_by(&t, 3).is_empty(), "the Sparrow never reached the turret");
}

/// The plan's acceptance, "hits [...] a running Peacekeeper and never misses": player 2's Peacekeeper 3 runs east,
/// 200 su south of the turret's centre, across its Range at its full 4 su a frame, on a Move, which no attack stops. The
/// turret fires at it once it sees it in Range, every 24 ticks, and each missile hits it, running, for 17 (its
/// PointArmor 1): the third brings it down (MaxHP 50).
#[test]
fn every_missile_hits_a_running_peacekeeper() {
    let mut t = field()
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(72.0, 872.0))
        .start();
    order(
        &mut t,
        1,
        P2,
        &[3],
        OrderKind::Move(Target::Point(Vec2::new(2400.0, 872.0))),
    );
    let mut flying = 0;
    let mut running_at_hits = Vec::new();
    while died(&t, 3).is_none() {
        assert!(t.sim().tick() < 16 * 30, "down in 30 seconds");
        let before = hits_by(&t, 1).len();
        t.step();
        match t.sim().object(id(3)) {
            Some(pk) => {
                if hits_by(&t, 1).len() > before {
                    running_at_hits.push(pk.pos.distance(pk.prev_pos));
                }
            }
            None => flying = in_flight(&t, 1),
        }
    }
    let hits = never_missed(&t, 1, 3, flying);
    assert_eq!(hits.len(), 3, "{hits:?}");
    assert!(hits.iter().all(|&(.., damage)| damage == 17.0), "{hits:?}");
    assert_eq!(running_at_hits.len(), 2, "the third brought it down");
    assert!(
        running_at_hits.iter().all(|&step| (step - 4.0).abs() < 1e-3),
        "{running_at_hits:?}"
    );
    let launched: Vec<Tick> = launches(&t, 1).iter().map(|&(tick, ..)| tick).collect();
    assert!(launched.windows(2).all(|w| w[1] - w[0] == 24), "{launched:?}");
}

/// The plan's acceptance, "it fires from its first tick (GDO buildings are placed complete)": player 2's Peacekeeper
/// 7 holds 320 su east of where the Defense System's turret goes, which player 1 doesn't see, within the turret's Range
/// and sight. The turret placed in tick `p` sees it at the end of tick `p`, aims from tick `p + 1`, its first, and
/// fires at the end of tick `p + 6`.
#[test]
fn a_placed_turret_fires_from_its_first_tick() {
    let spot = TilePos::new(14, 9); // its centre (928, 608)
    let scenario = base().spawn(ObjectKind::Peacekeeper, Some(P2), point(928.0 + 320.0, 608.0)); // 7
    let mut t = started(scenario);
    order(&mut t, 2, P2, &[7], OrderKind::Hold);
    next(&mut t, construction(DS, Construct(TURRET)));
    until_ready(&mut t, DS);
    let pk = t.sim().object(id(7)).cloned().unwrap();
    assert!(!t.sim().is_visible(P1, &pk), "unseen before the turret");
    let p = next(&mut t, construction(DS, place(spot)));
    let turret = 8;
    assert_eq!(t.sim().object(id(turret)).map(|o| o.kind), Some(TURRET));
    assert!(
        t.sim().is_visible(P1, t.sim().object(id(7)).unwrap()),
        "the turret's sight"
    );
    assert_eq!(t.sim().object(id(turret)).unwrap().attack.as_ref().unwrap().phase, None);
    t.run_until(p + 1);
    let phase = t.sim().object(id(turret)).unwrap().attack.as_ref().unwrap().phase;
    assert_eq!(
        phase,
        Some(AttackPhase::Aiming { target: id(7) }),
        "aiming in its first tick"
    );
    t.run_until(p + 20);
    assert_eq!(launches(&t, turret).first(), Some(&(p + 6, Some(7), true)));
}

/// 0423's Verification, "Run Power to about 59% (e.g. 30 generated, 51 consumed): [...] a Missile Turret fires about
/// every 41 frames", and "Build a Generator back to positive: all return to full speed at once". The turret of
/// [`short`] fires at player 2's Deployment Center 9, 4 grid units east of it: at 38 551 of 65 536 a tick its cycle of
/// 24 ticks takes `⌈24 × 65536 / 38551⌉` = 41, its first missile at the 11th tick of each, `⌈6 × 65536 / 38551⌉`, from
/// tick 11 on. The Deployment Center places a Generator in tick 242, 50/51, and the cycle takes 25, `⌈24 × 65536 /
/// 64251⌉`; then a second in tick 482, 70/51, and from that tick every tick is a whole one for the turret, nothing banked
/// (`AttackState::power_carry`), so that its missiles come every 24 ticks.
#[test]
fn at_30_generated_for_51_consumed_the_turret_fires_every_41_ticks_and_a_generator_restores_it_at_once() {
    let mut t = started(short().spawn(ObjectKind::DeploymentCenter, Some(P2), tile(15, 1))); // 9
    assert_eq!(t.sim().power(P1).ratio(), 38_551);
    let c = t.sim().tick() + 1;
    t.at(c, P1, construction(SHORT_DC, Construct(ObjectKind::Generator)));
    let carry = |t: &TestSim| {
        t.sim()
            .object(id(SHORT_TURRET))
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .power_carry
    };
    let mut placed = Vec::new();
    let mut carries = Vec::new();
    for spot in GENERATOR_SPOTS {
        t.step();
        until_ready(&mut t, SHORT_DC);
        let tick = t.sim().tick() + 1;
        t.at(tick, P1, construction(SHORT_DC, place(spot)));
        if placed.is_empty() {
            t.at(tick, P1, construction(SHORT_DC, Construct(ObjectKind::Generator)));
        }
        placed.push(tick);
    }
    let [first, second] = placed[..] else { unreachable!() };
    assert_eq!((first, second), (242, 482));
    while t.sim().tick() < second + 24 * 6 {
        t.step();
        if t.sim().tick() >= second {
            carries.push(carry(&t));
        }
    }
    assert_eq!(t.sim().power(P1).ratio(), RATIO_ONE);
    assert!(
        carries.iter().all(|&carry| carry == 0),
        "nothing banked at full speed: {carries:?}"
    );
    let launched: Vec<Tick> = launches(&t, SHORT_TURRET).iter().map(|&(tick, ..)| tick).collect();
    assert!(
        launches(&t, SHORT_TURRET)
            .iter()
            .all(|&(_, at, homing)| at == Some(9) && homing)
    );
    let slow: Vec<Tick> = launched.iter().copied().filter(|&tick| tick < first).collect();
    assert_eq!(slow, [11, 52, 93, 134, 175, 216], "every 41 ticks");
    let gaps = |from: Tick, to: Tick| -> Vec<Tick> {
        let within: Vec<Tick> = launched
            .iter()
            .copied()
            .filter(|&tick| tick >= from && tick < to)
            .collect();
        within.windows(2).map(|w| w[1] - w[0]).collect()
    };
    // The cycles that start and end at 50/51: those after the first launch from tick 242 on.
    let middle = gaps(first, second);
    assert!(
        middle.len() >= 5 && middle[1..].iter().all(|&gap| gap == 25),
        "{middle:?}"
    );
    let after: Vec<Tick> = launched.iter().copied().filter(|&tick| tick >= second).collect();
    assert!(
        after[0] <= second + 24,
        "the cycle under way at full speed at once: {after:?}"
    );
    assert!(gaps(second, Tick::MAX).iter().all(|&gap| gap == 24), "{after:?}");
}
