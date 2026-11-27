//! The Seeker in a fight (`factions/cults/units/seeker.md`: "a **machine-gun turret** that fires **low damage at a high
//! rate**, **cannot miss**, and **rotates 360°**"): its turret fires on the move (`systems/controls/behaviors.md`: on a
//! Move, "Turret units' TurretAutonomousScanning still operates independently"); hits on it count by the side they
//! come from (`systems/combat.md > DirectionalArmor`, the WheeledVehicle's); and its ground is never Rugged
//! (`systems/units.md > WheeledVehicle`: RuggedTerrain false). Its hits on a Sparrow are `tests/airstrip/combat.rs`'.

use sc_sim::defs::{TargetDomain, UnitBase};
use sc_sim::math;
use sc_sim::mover::MoveStatus;
use sc_sim::tuning::combat::{DIRECTIONAL_FRONT, DIRECTIONAL_REAR, DIRECTIONAL_SIDE};

use super::*;

/// The plan's acceptance (slice M5-N6): "it kills a Militant while driving past". Player 1's Seeker is sent east along
/// row 6 of an open map, past player 2's Militant, which stands 86 su off its way, both with auto-targeting on. The
/// Seeker never stops for it: its turret fires on the move (every one of its hits lands while it drives) and kills the
/// Militant, a 3-damage shot every 4 ticks on 40 HP and no armour, and the Seeker drives on to its goal. The Militant
/// fires back: its hits take (4 − PointArmor 1) times the multiplier of the Seeker's side it stands on.
#[test]
fn it_kills_a_militant_while_driving_past() {
    let row = 6.5 * 64.0;
    let mut t = cults(&flat(40, 14))
        .spawn(SEEKER, Some(P1), SpawnAt::Point(Vec2::new(200.0, row)))
        .spawn(
            ObjectKind::Militant,
            Some(P2),
            SpawnAt::Point(Vec2::new(1300.0, row + 86.0)),
        )
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 1))
        .spawn(ObjectKind::Storage, Some(P2), tile(35, 11))
        .start();
    let (seeker, militant) = (id(1), id(2));
    let goal = Vec2::new(2300.0, row);
    t.order_at(1, P1, &[seeker], OrderKind::Move(Target::Point(goal)), false);
    let mut seen = 0;
    let mut fired = 0;
    let mut killed = None;
    while t.status(seeker.0) != MoveStatus::Arrived {
        assert!(t.sim().tick() < 600, "{:?}", t.mover(seeker.0));
        step_checked(&mut t);
        let tick = t.sim().tick();
        let all = hits(&t);
        for &(_, by, target, damage) in &all[seen..] {
            if (by, target) == (seeker, militant) {
                fired += 1;
                assert_eq!(damage, 3.0, "no armour");
                assert!(t.mover(seeker.0).momentum.length() > 1.0, "tick {tick}: on the move");
            } else if (by, target) == (militant, seeker) {
                let multiplier = damage / 3.0;
                let sides = [DIRECTIONAL_FRONT, DIRECTIONAL_SIDE, DIRECTIONAL_REAR];
                assert!(sides.iter().any(|&m| near(m, multiplier, 1e-6)), "{damage}");
            }
        }
        seen = all.len();
        if killed.is_none() && t.sim().object(militant).is_none() {
            killed = Some(tick);
        }
        assert_eq!(
            current(&t, seeker),
            Some(OrderKind::Move(Target::Point(goal))),
            "tick {tick}"
        );
    }
    let killed = killed.expect("the Militant killed on the way");
    assert!(fired >= 14, "{fired} hits of 3 on its 40 HP");
    assert!(pos(&t, seeker).distance(goal) < 32.0);
    println!(
        "the Militant killed in tick {killed}, the Seeker at its goal in {}",
        t.sim().tick()
    );
}

/// Unit `unit`'s current order.
fn current(t: &TestSim, unit: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(unit)?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The HP the first hit of player 2's Militant takes from player 1's idle Seeker, the Militant ordered to attack it
/// from 150 su off in direction `side` (radians) from the way the Seeker faces; auto-targeting off, so that neither
/// moves but to attack.
fn first_hit_from(side: f32) -> f32 {
    let at = Vec2::new(900.0, 700.0);
    let mut t = cults(&flat(30, 22))
        .spawn(SEEKER, Some(P1), SpawnAt::Point(at))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Storage, Some(P2), tile(25, 18))
        .start();
    t.without_auto_targeting();
    let seeker = id(1);
    let facing = t.sim().object(seeker).expect("the Seeker").facing;
    let militant = id(next_id(&t));
    t.spawn_unit_at(
        1,
        ObjectKind::Militant,
        P2,
        at + math::facing_vector(facing + side) * 150.0,
    );
    t.order_at(2, P2, &[militant], OrderKind::Attack(seeker), false);
    while !hits(&t).iter().any(|&(_, by, _, _)| by == militant) {
        assert!(t.sim().tick() < 100, "a hit");
        step_checked(&mut t);
    }
    let [(_, _, target, damage)] = hits(&t)[..] else {
        panic!("one hit: {:?}", hits(&t));
    };
    assert_eq!(target, seeker);
    assert_eq!(t.sim().object(seeker).map(|o| o.facing), Some(facing), "it stood still");
    damage
}

/// The plan's acceptance: "rear hits do more than frontal ones". `systems/units.md > WheeledVehicle`: DirectionalArmor
/// true, so a hit from within 45° of its facing takes ×0.75, one from within 45° of behind it ×1.25, one from its side
/// ×1 (`systems/combat.md > DirectionalArmor`), after its PointArmor of 1: of a Militant's 4 damage, 2.25 from the
/// front, 3 from the side and 3.75 from behind.
#[test]
fn rear_hits_do_more_than_frontal_ones() {
    let unit = SEEKER.def().unit.as_ref().expect("a unit");
    assert_eq!(unit.base, UnitBase::WheeledVehicle);
    assert!(unit.base_def().directional_armor);
    let attack = ObjectKind::Militant
        .def()
        .unit
        .as_ref()
        .and_then(|unit| unit.attack.as_ref());
    assert_eq!(
        attack.map(|attack| (attack.damage, attack.target_domain)),
        Some((4.0, TargetDomain::Universal))
    );
    let (front, side, rear) = (
        first_hit_from(0.0),
        first_hit_from(math::PI / 2.0),
        first_hit_from(math::PI),
    );
    assert_eq!((front, side, rear), (2.25, 3.0, 3.75));
    assert!(front < side && side < rear);
}

/// The plan's acceptance: "it can't enter Rugged tiles". `systems/units.md > WheeledVehicle`: RuggedTerrain false, so
/// its ground is `NavClass::Ground`, whose Rugged tiles are blocked (`tests/vehicles/walls.rs` drives one round a band of
/// them). A band of Rugged tiles two wide runs the height of the map between the Chop Shop's exit and its rally point:
/// the Seeker drives up to it and stays east of it, never with any part of its box on it, while a Recruit released
/// after it, LightInfantry (RuggedTerrain true), walks over it to the rally point.
#[test]
fn it_never_drives_onto_rugged_tiles_where_a_recruit_walks() {
    let map: Vec<String> = (0..22)
        .map(|_| (0..30).map(|x| if (6..8).contains(&x) { 'r' } else { '.' }).collect())
        .collect();
    let (mut t, order) = filled_on(on(&map.join("\n")), 3);
    let rally_point = Vec2::new(200.0, 736.0);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], rally_point));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    t.at(start, P1, release(&[CS], RecruitKind::Plain));
    let (out, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!(seeker, order[0]);
    let band_east = 8.0 * 64.0;
    while t.sim().tick() < out + 400 {
        step_checked(&mut t);
        let (lo, _) = shape(&t, seeker).bounds();
        assert!(
            lo.x >= band_east - 1e-3,
            "tick {}: on the band at {}",
            t.sim().tick(),
            pos(&t, seeker)
        );
    }
    assert!(
        pos(&t, seeker).x < 12.0 * 64.0,
        "it drove out toward the band: {}",
        pos(&t, seeker)
    );
    let recruit = order[2];
    assert!(
        pos(&t, recruit).distance(rally_point) < 48.0,
        "the Recruit crossed it: {}",
        pos(&t, recruit)
    );
}
