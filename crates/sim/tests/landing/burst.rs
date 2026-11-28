//! Design change 0458 (`factions/gdo/units/hornet.md > AttackAttributes`): "the missile **bursts at the spot** it flies
//! to (systems/combat.md > AoEDamage; friendly fire applies), hitting **ground (and underground) objects only**, as an
//! air-to-ground weapon. If nothing is in the circle, it bursts harmlessly"; "**Damage 45 total over an AoERadius of 32
//! su**". Its Verification, point by point, with (M6-GH2) a Hornet landed on its slot among the ground objects.

use sc_sim::math::PI;
use sc_sim::{Faction, math};

use super::*;

/// "**Damage 45 total over an AoERadius of 32 su**".
const DAMAGE: f32 = 45.0;
const RADIUS: f32 = 32.0;

/// Steps until `shooter`'s first missile has burst, at most `ticks` ticks: the tick, and where.
fn first_burst(t: &mut TestSim, shooter: u32, ticks: Tick) -> (Tick, Vec2) {
    let until = t.sim().tick() + ticks;
    loop {
        if let Some(&(tick, at, radius)) = blasts_of(t, shooter).first() {
            assert_eq!(radius, RADIUS, "32 su");
            return (tick, at);
        }
        assert!(t.sim().tick() < until, "#{shooter}'s burst: not by tick {until}");
        t.step();
    }
}

/// The hits of `shooter`'s burst in tick `tick`: (target, HP damage), in id order.
fn burst_hits(t: &TestSim, shooter: u32, tick: Tick) -> Vec<(u32, f32)> {
    (hits_by(t, shooter).into_iter())
        .filter(|hit| hit.0 == tick)
        .map(|hit| (hit.1.0, hit.2))
        .collect()
}

/// Verification 1: "Order a Hornet to attack a moving Raptor. If the Raptor has left the spot, the missile bursts there
/// and does at most partial damage". Player 2's Raptor (8) drives off east at its 10 su a frame as player 1's Hornet
/// (7) is ordered to attack it: the missile flies to the spot the Raptor stood on as it was launched (DoublyDisjointed,
/// "not homing: the missile flies to where the target was, so it can be dodged") and bursts there, the Raptor long
/// gone from the circle, unharmed.
#[test]
fn a_missile_at_a_moving_raptor_bursts_where_it_stood() {
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P1), at(1600.0, 1200.0))
            .spawn(ObjectKind::Raptor, Some(P2), at(1600.0, 1450.0))
    });
    let (hornet, raptor) = (7, 8);
    let away = Vec2::new(3400.0, 1450.0);
    t.at(2, P2, order(&[raptor], OrderKind::Move(Target::Point(away))));
    next(&mut t, P1, order(&[hornet], OrderKind::Attack(id(raptor))));
    let (tick, at) = first_burst(&mut t, hornet, 200);
    let (_, target, to, lands_at) = launches(&t, hornet)[0];
    assert_eq!(
        (target, to, lands_at),
        (Some(id(raptor)), at, tick),
        "it bursts on the spot it flew to"
    );
    let half = object(t.sim(), raptor).def().silhouette().expect("a unit").length() / 2.0;
    let off = t.pos(raptor).distance(at);
    assert!(off > RADIUS + half, "the Raptor has left the spot: {off} su off");
    assert_eq!(
        burst_hits(&t, hornet, tick),
        [],
        "a burst over an empty circle does nothing"
    );
    let max = object(t.sim(), raptor).def().max_hp.expect("HP");
    assert_eq!(object(t.sim(), raptor).hp, Some(max));
}

/// The area of a circle of radius `r` that a box of half extents `half` centred on it covers, for a box as wide as the
/// circle and less tall: the circle but for the two caps beyond the box's long sides.
fn squat_overlap(r: f32, half: Vec2) -> f32 {
    assert!(half.x >= r && half.y < r);
    let d = half.y;
    // acos, as `math` has it: a quarter turn less the arcsine.
    let cap = r * r * (PI / 2.0 - math::asin(d / r)) - d * math::sqrt(r * r - d * d);
    PI * r * r - 2.0 * cap
}

/// Verification 2: "Order it to attack a stationary tank square-on. The tank takes about 30 before armor"
/// (`factions/gdo/units/hornet.md > Placeholder stats`: "roughly 30 to a 64x40 vehicle hit square-on [...] before
/// armor"). Player 2's Hand of Heaven (2), 64 × 40 su, standing still and facing east, takes player 1's Hornet's (1)
/// burst on its centre: its share, 45 × the overlap ÷ the circle's area (`systems/combat.md > AoEDamage`), is 33.3
/// before armour, its effective FullArmor 2 × its overlapped fraction off that, and a blast on its centre counts as
/// from the front, × 0.75 (`combat::area::strike`). The 30 of the design's note are the share of a damage of 40; 45's
/// is 33.3.
#[test]
fn a_stationary_tank_square_on_takes_about_30_before_armour() {
    let builder = ScenarioBuilder::new(&flat(40, 30))
        .faction(P2, Faction::Cults)
        .spawn(HORNET, Some(P1), at(800.0, 900.0))
        .spawn(ObjectKind::HandOfHeaven, Some(P2), at(800.0, 1150.0));
    let mut t = started(builder);
    let (hornet, tank) = (1, 2);
    let o = object(t.sim(), tank);
    assert_eq!(o.facing, 0.0, "square-on, east");
    let half = o.def().silhouette().expect("a unit") / 2.0;
    let (full_armor, max) = (o.def().armor.full, o.def().max_hp.expect("HP"));
    next(&mut t, P1, order(&[hornet], OrderKind::Attack(id(tank))));
    let (tick, at) = first_burst(&mut t, hornet, 200);
    assert_eq!(at, Vec2::new(800.0, 1150.0), "on its centre");

    let circle = PI * RADIUS * RADIUS;
    let inside = squat_overlap(RADIUS, half);
    let share = DAMAGE * inside / circle;
    assert!((share - 33.3).abs() < 0.05, "about 30 before armour: {share}");
    let armour = full_armor * inside / (4.0 * half.x * half.y);
    let expected = (share - armour) * 0.75;
    let hits = burst_hits(&t, hornet, tick);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].0, tank);
    assert!((hits[0].1 - expected).abs() < 1e-3, "{} for {expected}", hits[0].1);
    let hp = object(t.sim(), tank).hp.expect("HP");
    assert!((hp - (max - expected)).abs() < 1e-3, "{hp}");
}

/// Verification 3: "Fire into a clump of infantry. Several take a share, own units included". Four Peacekeepers stand
/// in a square, 28 su apart centre to centre, player 2's (8 and 9) on one diagonal and player 1's own (10 and 11) on
/// the other; player 1's Hornet (7) fires at the ground in their middle. The burst takes a share out of each of the
/// four, the same by symmetry: friendly fire (`systems/combat.md > AoEDamage`: "area damage hits **every** object in
/// the circle whose domain matches, **whoever owns it**").
#[test]
fn a_burst_into_a_clump_of_infantry_hits_several_own_units_included() {
    let c = Vec2::new(800.0, 1150.0);
    let corner = |x: f32, y: f32| at(c.x + 14.0 * x, c.y + 14.0 * y);
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P1), at(800.0, 900.0))
            .spawn(ObjectKind::Peacekeeper, Some(P2), corner(-1.0, -1.0))
            .spawn(ObjectKind::Peacekeeper, Some(P2), corner(1.0, 1.0))
            .spawn(ObjectKind::Peacekeeper, Some(P1), corner(1.0, -1.0))
            .spawn(ObjectKind::Peacekeeper, Some(P1), corner(-1.0, 1.0))
    });
    let hornet = 7;
    next(&mut t, P1, order(&[hornet], OrderKind::AttackGround(c)));
    let (tick, at) = first_burst(&mut t, hornet, 200);
    assert_eq!(at, c);
    let hits = burst_hits(&t, hornet, tick);
    let hit: Vec<u32> = hits.iter().map(|hit| hit.0).collect();
    assert_eq!(hit, [8, 9, 10, 11], "all four");
    assert!(
        hits.iter().all(|h| (h.1 - hits[0].1).abs() < 1e-4 && h.1 > 0.0),
        "{hits:?}"
    );
    let own = hit.iter().filter(|&&n| object(t.sim(), n).owner == Some(P1)).count();
    assert_eq!(own, 2, "own units included");
}

/// Verification 4: "Fire at the ground under an enemy aircraft. The aircraft takes nothing": player 1's Hornet (9)
/// fires at the ground under player 2's hovering Hornet (7), over player 2's Peacekeeper (8): the burst hits the
/// Peacekeeper, a ground object, and not the aircraft. (M6-GH2) An aircraft down on its slot is a ground target, for a
/// burst as for any weapon (design change 0428 point 1, `combat::object_domain`): player 2's Hornet's burst on the slot
/// of player 1's landed Hornet (10) hits it, and the Airfield under it.
#[test]
fn a_burst_under_an_aircraft_leaves_it_untouched_but_hits_one_landed() {
    let spot = Vec2::new(1600.0, 1200.0);
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P2), at(spot.x, spot.y))
            .spawn(ObjectKind::Peacekeeper, Some(P2), at(spot.x, spot.y))
            .spawn(HORNET, Some(P1), at(spot.x, spot.y - 250.0))
    });
    let (enemy, under, hornet) = (7, 8, 9);
    next(&mut t, P1, order(&[hornet], OrderKind::AttackGround(spot)));
    let (tick, at) = first_burst(&mut t, hornet, 200);
    assert_eq!(at, spot);
    assert_eq!(t.pos(enemy), spot, "hovering over the spot");
    let hit: Vec<u32> = burst_hits(&t, hornet, tick).iter().map(|hit| hit.0).collect();
    assert_eq!(hit, [under], "the ground object, not the aircraft");
    next(&mut t, P1, order(&[hornet], OrderKind::Stop));

    let landed = landed_hornets(&mut t, 10, &[Vec2::new(864.0, 760.0)], A)[0];
    let slot = t.pos(landed);
    next(&mut t, P2, order(&[enemy], OrderKind::AttackGround(slot)));
    let (tick, at) = first_burst(&mut t, enemy, 400);
    assert_eq!(at, slot);
    assert!(down(t.sim(), landed), "still down");
    let hit: Vec<u32> = burst_hits(&t, enemy, tick).iter().map(|hit| hit.0).collect();
    assert_eq!(hit, [A, landed], "the Airfield and the Hornet down on it");
}
