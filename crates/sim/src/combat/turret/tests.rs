//! (M5-U1) A unit's turret in phase 5e (`super`): its arc and its turn (`systems/units.md > TurretAttributes`), and
//! plan U1's acceptance of them, on the test turret units (`super::testing`): the time a turn takes before the first
//! shot, a cone that holds its fire until the base turns, and a turret that fires on the move. What locks its targets
//! is tested with its behaviour (`orders::turret`).

use glam::Vec2;

use super::testing::{
    GUN, TURRET_UNIT, current, facing_degrees, locked, ninety_at_ten, open, shots, toward, turret, turret_of,
    turret_unit,
};
use super::{TurretState, aim_facing, in_arc, turn};
use crate::combat::AttackPhase;
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::orders::{OrderKind, Target};
use crate::testing::{ScenarioBuilder, TestSim};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNIT: ObjectId = ObjectId(1);
const ENEMY: ObjectId = ObjectId(2);
/// Within float rounding of an angle, degrees.
const ROUNDING: f32 = 1e-3;

fn degrees(d: f32) -> f32 {
    math::degrees_to_radians(d)
}

/// The turn of a turret of `attrs` in `state`, on a base at the origin facing `facing` degrees, toward the point
/// `degrees` from it: its facing after, degrees.
fn turned(state: &mut TurretState, attrs: &crate::defs::TurretAttrs, facing: f32, toward_degrees: f32) -> f32 {
    turn(
        state,
        attrs,
        degrees(facing),
        Vec2::ZERO,
        toward(Vec2::ZERO, toward_degrees, 300.0),
    );
    state.facing.to_degrees()
}

/// `systems/units.md > TurretAttributes`: "The turret is centered on the unit's facing direction, with TurnAngle split
/// equally clockwise and counter-clockwise from center": a 90° arc reaches 45° either side of the base's facing, its
/// bounds included, whichever way the base faces; a full turn reaches everywhere, and a point on the base is in it.
#[test]
fn the_arc_is_split_equally_either_side_of_the_base_s_facing() {
    let ninety = turret(90.0, 10.0);
    let at = |d: f32| toward(Vec2::ZERO, d, 300.0);
    for d in [0.0, 30.0, -30.0, 44.0, -44.0] {
        assert!(in_arc(&ninety, 0.0, Vec2::ZERO, at(d)), "{d}°");
    }
    for d in [46.0, -46.0, 90.0, 180.0] {
        assert!(!in_arc(&ninety, 0.0, Vec2::ZERO, at(d)), "{d}°");
    }
    // The bounds: a point diagonally off is the game's own 45° (`tuning::orders::Orders::rotation_step`).
    let quarter = crate::defs::TurretAttrs {
        turn_angle: math::PI / 2.0,
        turn_rate: 1.0,
    };
    assert!(in_arc(&quarter, 0.0, Vec2::ZERO, Vec2::new(100.0, 100.0)));
    assert!(in_arc(&quarter, 0.0, Vec2::ZERO, Vec2::new(100.0, -100.0)));
    assert!(!in_arc(&quarter, 0.0, Vec2::ZERO, Vec2::new(100.0, 100.5)));
    // On a base facing south, 30° from it either way is in, east and west are out.
    assert!(in_arc(&ninety, degrees(90.0), Vec2::ZERO, at(120.0)));
    assert!(in_arc(&ninety, degrees(90.0), Vec2::ZERO, at(60.0)));
    assert!(!in_arc(&ninety, degrees(90.0), Vec2::ZERO, at(0.0)));
    assert!(!in_arc(&ninety, degrees(90.0), Vec2::ZERO, at(180.0)));
    let full = turret(360.0, 12.0);
    for d in [0.0, 90.0, 180.0, -135.0] {
        assert!(in_arc(&full, degrees(45.0), Vec2::ZERO, at(d)), "{d}°");
    }
    assert!(in_arc(&ninety, 0.0, Vec2::ZERO, Vec2::ZERO), "on the base");
}

/// The real turrets' arcs (slice M5-G2's data): the Hand of Heaven's 20° refuse a target 30° off its facing and take
/// one 9° off, the Sparrow's 45° take 20° and refuse 25°, and the Seeker's turn all the way round.
#[test]
fn the_real_turrets_arcs_are_their_turn_angles() {
    let attrs = |kind: ObjectKind| kind.def().unit.as_ref().unwrap().turret.expect("a turret");
    let at = |d: f32| toward(Vec2::ZERO, d, 300.0);
    let hand = attrs(ObjectKind::HandOfHeaven);
    assert!(!in_arc(&hand, 0.0, Vec2::ZERO, at(30.0)));
    assert!(!in_arc(&hand, 0.0, Vec2::ZERO, at(-30.0)));
    assert!(in_arc(&hand, 0.0, Vec2::ZERO, at(9.0)));
    let sparrow = attrs(ObjectKind::Sparrow);
    assert!(in_arc(&sparrow, 0.0, Vec2::ZERO, at(-20.0)));
    assert!(!in_arc(&sparrow, 0.0, Vec2::ZERO, at(25.0)));
    let seeker = attrs(ObjectKind::Seeker);
    assert!(in_arc(&seeker, 0.0, Vec2::ZERO, at(180.0)));
    assert!(in_arc(&seeker, 0.0, Vec2::ZERO, at(-150.0)));
}

/// `systems/controls/action_channels.md > TurretTurning`: "applies the turret's turn rate each tick, constrained by
/// TurnAngle". At most its TurnRate a tick, landing exactly on the direction; within its arc, a turret turns the
/// only way the arc allows (through its centre, never round behind) and stops at the edge toward a point outside;
/// a full turn goes the shorter way round, across the back.
#[test]
fn a_turret_turns_at_its_rate_and_never_out_of_its_arc() {
    let ninety = turret(90.0, 10.0);
    let mut state = TurretState::default();
    let ticks: Vec<f32> = (0..3).map(|_| turned(&mut state, &ninety, 0.0, 25.0)).collect();
    assert!((ticks[0] - 10.0).abs() < ROUNDING && (ticks[1] - 20.0).abs() < ROUNDING);
    assert!((ticks[2] - 25.0).abs() < ROUNDING, "lands on it: {ticks:?}");
    // Toward a point 80° off: as far as the edge, and it stays there.
    let mut state = TurretState::default();
    let ticks: Vec<f32> = (0..6).map(|_| turned(&mut state, &ninety, 0.0, 80.0)).collect();
    assert!((ticks[3] - 40.0).abs() < ROUNDING && (ticks[4] - 45.0).abs() < ROUNDING);
    assert!((ticks[5] - 45.0).abs() < ROUNDING, "{ticks:?}");
    // From 40° to a point 40° the other side: back through its centre, 10° a tick.
    let mut state = TurretState {
        facing: degrees(40.0),
        locked: None,
    };
    let ticks: Vec<f32> = (0..8).map(|_| turned(&mut state, &ninety, 0.0, -40.0)).collect();
    assert!(
        (ticks[3] - 0.0).abs() < ROUNDING && (ticks[7] + 40.0).abs() < ROUNDING,
        "{ticks:?}"
    );
    // Relative to the base: on a base facing south, a point at 120° is 30° to its right.
    let mut state = TurretState::default();
    assert!((turned(&mut state, &ninety, 90.0, 120.0) - 10.0).abs() < ROUNDING);
    // A full turn from 170° to 190° (-170°): across the back, 12° then the last 8°.
    let full = turret(360.0, 12.0);
    let mut state = TurretState {
        facing: degrees(170.0),
        locked: None,
    };
    assert!((turned(&mut state, &full, 0.0, -170.0) + 178.0).abs() < ROUNDING);
    assert!((turned(&mut state, &full, 0.0, -170.0) + 170.0).abs() < ROUNDING);
}

/// The direction the attack faces is the base's for a turret with no state yet (it starts centred), else the
/// turret's on it.
#[test]
fn a_turret_aims_on_its_base() {
    assert_eq!(aim_facing(None, 1.0), 1.0);
    let turned = Some(TurretState {
        facing: 0.5,
        locked: None,
    });
    assert!((aim_facing(turned, 1.0) - 1.5).abs() < 1e-6);
    assert!((aim_facing(turned, 3.0) - (3.5 - math::TAU)).abs() < 1e-6, "wrapped");
}

/// A test turret unit (`super::testing`, of `attrs`) at `at` facing east, holding position from tick 2 (its base
/// engages nothing, `orders::engage::hold`), and a Drone of player 2's at `enemy` (unarmed: it never fights back). The
/// sim is at tick 1, after the spawns.
fn holding(at: Vec2, enemy: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(&open(16, 10)).start();
    t.spawn_unit_at(1, TURRET_UNIT, P1, at)
        .spawn_unit_at(1, ObjectKind::Drone, P2, enemy)
        .order_at(2, P1, &[UNIT], OrderKind::Hold, false)
        .run_until(1);
    assert_eq!(facing_degrees(&t, UNIT), 0.0, "it starts facing east");
    t
}

/// Plan U1's acceptance: "Turn rate: 90° at 12°/tick takes 8 ticks before the first shot." A 360° turret turning 12°
/// a tick, its gun with no Aim and one frame of Firing, holds position with an enemy due south. It locks the enemy in
/// tick 2 and turns 12° a tick from then (`systems/combat.md > AttackPhase`: Aiming's TurretSourceAction is Turning);
/// the eighth tick's turn lands on 90°, and the shot lands at the end of that tick. The base never turns.
#[test]
fn turning_ninety_degrees_at_twelve_a_tick_takes_eight_ticks_before_the_first_shot() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(300.0, 300.0);
    let mut t = holding(at, at + Vec2::new(0.0, 120.0));
    assert_eq!(turret_of(&t, UNIT), None, "no state before it locks or turns");
    for tick in 2..=9 {
        t.step();
        assert_eq!(locked(&t, UNIT), Some(ENEMY), "tick {tick}");
        let facing = turret_of(&t, UNIT).unwrap().facing.to_degrees();
        let wanted = (12.0 * (tick - 1) as f32).min(90.0);
        assert!((facing - wanted).abs() < ROUNDING, "tick {tick}: {facing}°");
        assert_eq!(
            (facing_degrees(&t, UNIT), t.pos(1)),
            (0.0, at),
            "tick {tick}: the base stays"
        );
        let fired = shots(&t, UNIT);
        assert_eq!(fired.is_empty(), tick < 9, "tick {tick}: {fired:?}");
    }
    assert_eq!(shots(&t, UNIT), vec![(9, ENEMY)], "the 8th tick of the turn");
}

/// The addendum's test unit, "a turret unit on TurnRate movement (a 90° arc at 10° a tick)", holding position: an enemy
/// 40° to its left, it locks at once, turns to in 4 ticks and hits at the end of the 4th; one 50° to its left, outside
/// its arc, it never locks nor fires at.
#[test]
fn a_90_degree_turret_at_10_a_tick_turns_to_what_its_arc_takes() {
    let _def = ninety_at_ten();
    let at = Vec2::new(300.0, 300.0);
    let mut t = holding(at, toward(at, -40.0, 120.0));
    t.run_until(5);
    assert_eq!(shots(&t, UNIT), vec![(5, ENEMY)], "turned in ticks 2 to 5");
    let facing = turret_of(&t, UNIT).unwrap().facing.to_degrees();
    assert!((facing + 40.0).abs() < ROUNDING, "{facing}°");
    let mut t = holding(at, toward(at, -50.0, 120.0));
    t.run_until(40);
    assert_eq!((shots(&t, UNIT), locked(&t, UNIT)), (vec![], None));
}

/// Plan U1's acceptance: "Arc: a 45° cone doesn't fire at a target 30° off until the base turns." A test turret unit
/// with a 45° arc holds position with an enemy 30° to its right, in range: its turret never locks it, nor turns, nor
/// fires (`systems/controls/behaviors.md > TurretAutonomousScanning`: "within Range and within TurnAngle arc"). Ordered
/// to attack it, its base turns toward it (`systems/controls/command_behaviors.md > AttackingObject` 4: "If target
/// moves out of range or arc: resume MovingToObject"), 5° a tick; once the enemy is within 22.5° of its facing the
/// base stops, the turret locks it, and the shots follow.
#[test]
fn a_45_degree_cone_holds_its_fire_at_a_target_30_degrees_off_until_the_base_turns() {
    let _def = turret_unit(turret(45.0, 10.0), GUN);
    let at = Vec2::new(300.0, 300.0);
    let enemy = toward(at, 30.0, 130.0);
    let mut t = holding(at, enemy);
    t.run_until(60);
    assert_eq!(shots(&t, UNIT), vec![]);
    assert_eq!((locked(&t, UNIT), facing_degrees(&t, UNIT)), (None, 0.0));
    assert!(
        turret_of(&t, UNIT).is_none_or(|turret| turret.facing == 0.0),
        "it never turned"
    );
    t.order_at(61, P1, &[UNIT], OrderKind::Attack(ENEMY), false);
    while shots(&t, UNIT).is_empty() {
        assert!(t.sim().tick() < 120, "it never fired\n{}", t.ascii(None));
        t.step();
    }
    let base = facing_degrees(&t, UNIT);
    let bearing = math::vector_angle(enemy - t.pos(1)).to_degrees();
    assert!(base >= 7.5 - ROUNDING, "the base turned toward it: {base}°");
    assert!(
        (bearing - base).abs() <= 22.5 + ROUNDING,
        "within the cone: {bearing}° on {base}°"
    );
    assert_eq!(locked(&t, UNIT), Some(ENEMY));
}

/// Plan U1's acceptance: "Firing on the move: driving east with a 360° turret, it fires at an enemy to its north and
/// never stops" (`systems/combat.md > TurretSource`: "the unit base is free to move and turn during all attack
/// phases"). On a Move, the turret scans by itself (`systems/controls/behaviors.md > BaseAutoTargeting > Not active
/// during`: "Turret units' TurretAutonomousScanning still operates independently"), and fires as the unit passes a
/// Drone 140 su north of its road; the unit drives on at full speed every tick, straight, until it arrives. In
/// Firing and Cooldown the turret holds its facing (their TurretSourceActions are None), though its target moves
/// round it; in the other ticks it tracks it.
#[test]
fn a_turret_fires_on_the_move_at_an_enemy_beside_the_road() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let (start, goal) = (Vec2::new(200.0, 400.0), Vec2::new(1600.0, 400.0));
    let mut t = ScenarioBuilder::new(&open(32, 10)).start();
    let order = OrderKind::Move(Target::Point(goal));
    t.spawn_unit_at(1, TURRET_UNIT, P1, start)
        .spawn_unit_at(1, ObjectKind::Drone, P2, Vec2::new(700.0, 260.0))
        .order_at(2, P1, &[UNIT], order, false)
        .run_until(2);
    let (mut moving, mut held, mut tracked) = (Vec::new(), 0, 0);
    let phase_of = |t: &TestSim| t.sim().object(UNIT).unwrap().attack.as_ref().unwrap().phase;
    loop {
        let (pos, turret, phase) = (t.pos(1), turret_of(&t, UNIT), phase_of(&t));
        t.step();
        let tick = t.sim().tick();
        if current(&t, UNIT) != Some(order) {
            break;
        }
        assert!(tick < 400, "it never arrived\n{}", t.ascii(None));
        let now = t.pos(1);
        if now.x > pos.x {
            moving.push(tick);
        } else {
            assert!(moving.is_empty(), "tick {tick}: it stopped at {now}");
        }
        assert_eq!(
            (now.y, facing_degrees(&t, UNIT)),
            (start.y, 0.0),
            "tick {tick}: straight on"
        );
        let (Some(before), Some(after)) = (turret, turret_of(&t, UNIT)) else {
            continue;
        };
        match phase {
            Some(AttackPhase::Firing { .. } | AttackPhase::Cooldown) => {
                assert_eq!(after.facing, before.facing, "tick {tick}: held in {phase:?}");
                held += 1;
            }
            _ if after.locked.is_some() && after.facing != before.facing => tracked += 1,
            _ => {}
        }
    }
    assert!(t.pos(1).distance(goal) < 1.0, "it arrived: {}", t.pos(1));
    let fired = shots(&t, UNIT);
    assert!(fired.len() >= 10, "{fired:?}");
    for (tick, target) in &fired {
        assert_eq!(*target, ENEMY);
        assert!(moving.contains(tick), "tick {tick}: it fired standing");
    }
    // The ticks it moved in follow one another (it never stopped), from soon after the order.
    assert_eq!(moving.windows(2).find(|w| w[1] != w[0] + 1), None);
    assert!(moving[0] <= 4, "it started in tick {}", moving[0]);
    assert!(held > 0 && tracked > 0, "held {held}, tracked {tracked}");
}
