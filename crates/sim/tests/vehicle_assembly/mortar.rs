//! The Bike's mortar (`factions/syndicate/units/assault_bike.md`: "like the Grenadier's grenade (slow, dodgeable
//! explosive projectile with area damage) but with **slightly longer range**", "on a **turret with a narrow turn
//! angle**", "Ground only (assumed, like the Grenadier); can Attack Ground"; `> Placeholder stats`: "Damage 120 total
//! over the circle; AoERadius 48 su; Range **5 grid units**; ProjectileSpeed 8 su/frame", "Turret: TurnAngle **40°**
//! total; TurnRate 8°/frame"), and its wheels (design change 0445: it crushes nothing). On an open field, both players
//! the Syndicate's, no structure: units only, debug-spawned, auto-targeting off.

use std::f32::consts::PI;

use sc_sim::combat::{AttackPhase, overlap, turret_facing};
use sc_sim::{MoveStatus, math};

use super::*;

/// The mortar's circle: "AoERadius 48 su", its area π × 48² su², and its "Damage 120 total over the circle".
const RADIUS: f32 = 48.0;
const CIRCLE: f32 = PI * RADIUS * RADIUS;
const DAMAGE: f32 = 120.0;
/// Half the turret's "TurnAngle **40°** total", radians: either side of the Bike's facing.
const HALF_ARC: f32 = 20.0 * PI / 180.0;
/// `tuning::combat::Combat::facing_tolerance`: 1°.
const TOLERANCE: f32 = PI / 180.0;
/// A Guard's (`factions/syndicate/units/guard.md`: "Silhouette - 36x36", "PointArmor - 1, FullArmor - 1") area.
const GUARD: ObjectKind = ObjectKind::Guard;
const GUARD_AREA: f32 = 36.0 * 36.0;

/// An open `w` × `h` field, both players the Syndicate's, auto-targeting off.
fn field(w: usize, h: usize) -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(w, h))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .start();
    t.without_auto_targeting();
    t
}

/// A projectile launched so far (`SimEvent::ProjectileLaunched`): none homes in.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Launch {
    tick: Tick,
    shooter: ObjectId,
    from: Vec2,
    to: Vec2,
    lands_at: Tick,
}

fn launches(t: &TestSim) -> Vec<Launch> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched {
                shooter,
                from,
                to,
                lands_at,
                homing,
                ..
            } => {
                assert!(!homing, "a mortar round flies to a spot");
                Some(Launch {
                    tick: *tick,
                    shooter,
                    from,
                    to,
                    lands_at,
                })
            }
            _ => None,
        })
        .collect()
}

/// The blasts so far: their tick and centre, each of the AoERadius.
fn blasts(t: &TestSim) -> Vec<(Tick, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Blast { at, radius, .. } => {
                assert_eq!(radius, RADIUS, "an AoERadius of 48 su");
                Some((*tick, at))
            }
            _ => None,
        })
        .collect()
}

/// The HP `target` took in each hit so far: (tick, damage).
fn hits_on(t: &TestSim, target: ObjectId) -> Vec<(Tick, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                target: hit, damage, ..
            } if hit == target => Some((*tick, damage)),
            _ => None,
        })
        .collect()
}

/// `systems/combat.md > AoEDamage` for an object `inside` su² of whose `area` su² lie inside the circle, of FullArmor
/// `full`, hit with the directional multiplier `directional`: its share of the 120, less its FullArmor's share, at
/// least its share of the MinimumDamage of 0.5.
fn expected(inside: f32, area: f32, full: f32, directional: f32) -> f32 {
    let fraction = inside / area;
    ((DAMAGE * inside / CIRCLE - full * fraction) * directional).max(0.5 * fraction)
}

/// The area of `id`'s shape inside the circle around `at`.
fn inside(t: &TestSim, id: ObjectId, at: Vec2) -> f32 {
    let (lo, hi) = shape(t, id).bounds();
    overlap(at, RADIUS, lo, hi)
}

/// Steps until the first launch, at most `ticks` ticks.
fn until_launched(t: &mut TestSim, ticks: Tick) -> Launch {
    step_until(t, ticks, "the launch", |t| !launches(t).is_empty());
    launches(t)[0]
}

/// The plan's acceptance, "The mortar hits a group by the area rule" (`systems/combat.md > AoEDamage`: "A unit's damage
/// share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit overlap area / unit
/// total area)"), from a DoublyDisjointed round (`> AttackAttributes`: a "slow projectile to the target's position at
/// the end of Aiming"). Player 1's Bike, facing east, attacks the middle one of player 2's three Guards in a row 4 grid
/// units east of it, 40 su apart; a fourth stands 80 su south of the middle one. The round leaves from the Bike for the
/// middle Guard's spot and lands there 32 ticks later (256 su at 8 su a tick). The middle Guard, wholly inside the 48
/// su circle, takes 120 × 1296 / 7238.2 less its FullArmor of 1, about 20.5; each of the two beside it, partly inside,
/// its share of the 120 less its share of the armour; the fourth, out of the circle, nothing.
#[test]
fn a_round_hits_a_group_by_the_area_rule() {
    let mut t = field(16, 10);
    let from = Vec2::new(200.0, 320.0);
    let at = from + Vec2::new(256.0, 0.0);
    let (bike, middle) = (id(1), id(3));
    t.spawn_unit_at(1, BIKE, P1, from);
    for spot in [
        at - Vec2::new(40.0, 0.0),
        at,
        at + Vec2::new(40.0, 0.0),
        at + Vec2::new(0.0, 80.0),
    ] {
        t.spawn_unit_at(1, GUARD, P2, spot);
    }
    t.at(2, P1, order(&[bike], OrderKind::Attack(middle)));
    let launch = until_launched(&mut t, 40);
    let lands_at = launch.tick + 32;
    assert_eq!(
        (launch.shooter, launch.from, launch.to, launch.lands_at),
        (bike, from, at, lands_at)
    );
    run_checked(&mut t, lands_at - 1);
    assert_eq!(blasts(&t), [], "in the air");
    step_checked(&mut t);
    assert_eq!(blasts(&t), [(lands_at, at)]);
    let whole = expected(GUARD_AREA, GUARD_AREA, 1.0, 1.0);
    assert!((whole - 20.486).abs() < 1e-3, "{whole}");
    for guard in [id(2), middle, id(4)] {
        let inside = inside(&t, guard, at);
        assert!(inside > 0.0);
        let [(tick, taken)] = hits_on(&t, guard)[..] else {
            panic!("{guard:?} hit once: {:?}", hits_on(&t, guard));
        };
        assert_eq!(tick, lands_at);
        let share = expected(inside, GUARD_AREA, 1.0, 1.0);
        assert!((taken - share).abs() < 1e-4, "{guard:?}: {taken}, not {share}");
        let full = GUARD.def().max_hp.unwrap();
        assert!((object(&t, guard).hp.unwrap() - (full - taken)).abs() < 1e-4);
    }
    let partly = inside(&t, id(2), at);
    assert!(partly < GUARD_AREA - 100.0, "the two beside it partly: {partly}");
    assert_eq!(hits_on(&t, id(5)), [], "out of the circle");
    assert_eq!(hits_on(&t, bike), []);
}

/// "slow, dodgeable" (`> AttackAttributes`: "can be dodged"; plan acceptance: "can be dodged"): player 1's Bike attacks
/// player 2's Bike, facing east 4 grid units east of it, which stands, then, in the second game, drives off east as the
/// round's Firing begins, at the end of the Aiming. Standing, it takes the whole of its 56 × 32 box's share of the 120
/// (its box wholly inside the circle) less its FullArmor of 1, times its front's 0.75 (`systems/combat.md >
/// DirectionalArmor`, a vehicle's, from the circle's centre: its own); driving off, it is out of the circle when the
/// round lands where it stood, and takes nothing.
#[test]
fn a_bike_that_drives_off_as_the_round_is_fired_dodges_it() {
    for drives in [false, true] {
        let mut t = field(30, 10);
        let from = Vec2::new(200.0, 320.0);
        let stood = from + Vec2::new(256.0, 0.0);
        let (bike, target) = (id(1), id(2));
        t.spawn_unit_at(1, BIKE, P1, from).spawn_unit_at(1, BIKE, P2, stood);
        t.at(2, P1, order(&[bike], OrderKind::Attack(target)));
        run_checked(&mut t, 1);
        let firing = |t: &TestSim| {
            let phase = object(t, bike).attack.as_ref().and_then(|attack| attack.phase);
            matches!(phase, Some(AttackPhase::FiringAt { .. }))
        };
        step_until(&mut t, 40, "the Firing", firing);
        let Some(AttackPhase::FiringAt { at, .. }) = object(&t, bike).attack.as_ref().and_then(|a| a.phase) else {
            unreachable!()
        };
        assert_eq!(at, stood, "where it stood at the end of the Aiming");
        if drives {
            let away = Target::Point(Vec2::new(1800.0, 320.0));
            t.order_at(t.sim().tick() + 1, P2, &[target], OrderKind::Move(away), false);
        }
        let launch = until_launched(&mut t, 10);
        assert_eq!(launch.to, stood);
        run_checked(&mut t, launch.lands_at);
        assert_eq!(blasts(&t), [(launch.lands_at, stood)]);
        let o = object(&t, target);
        if drives {
            assert!(!shape(&t, target).overlaps(&Shape::silhouette(BIKE, stood, 0.0)));
            assert!(inside(&t, target, stood) == 0.0, "clear of the circle: at {}", o.pos);
            assert_eq!((hits_on(&t, target), o.hp), (vec![], Some(100.0)), "untouched");
        } else {
            let whole = BIKE_LENGTH * BIKE_WIDTH;
            assert_eq!(inside(&t, target, stood), whole, "wholly inside");
            let share = expected(whole, whole, 1.0, 0.75);
            assert!((share - 21.53).abs() < 0.01, "{share}");
            let [(_, taken)] = hits_on(&t, target)[..] else {
                panic!("one hit: {:?}", hits_on(&t, target));
            };
            assert!((taken - share).abs() < 1e-4, "{taken}, not {share}");
        }
    }
}

/// "can Attack Ground" (`systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has
/// CanTargetGround"), and "Ground only" (design change 0445: the blast's domain is the attack's TargetDomain, Ground,
/// which "hits Ground and surfaced Underground units", not Air ones; "**Friendly fire**", whoever owns them): player
/// 1's Bike fires at a spot of the open field 4 grid units east, where player 1's own Guard stands under player 2's
/// Phantom (a HoverCraft, of the Air domain), and keeps firing there, one round every 48 ticks. Each round's blast
/// hits the Guard, its own player's, wholly inside the circle, for its 20.5; the Phantom above it, never. In another
/// game, an Attack on the Phantom itself finds no target (`systems/combat.md > ValidTarget`: its TargetDomain, Ground,
/// doesn't reach the air): the Bike never fires at it.
#[test]
fn it_attacks_the_ground_and_its_blast_hits_ground_units_only() {
    let mut t = field(16, 10);
    let from = Vec2::new(200.0, 320.0);
    let at = from + Vec2::new(256.0, 0.0);
    let (bike, guard, phantom) = (id(1), id(2), id(3));
    t.spawn_unit_at(1, BIKE, P1, from)
        .spawn_unit_at(1, GUARD, P1, at)
        .spawn_unit_at(1, ObjectKind::Phantom, P2, at);
    run_checked(&mut t, 1);
    assert_eq!(gate(&t, P1, order(&[bike], OrderKind::AttackGround(at))), Ok(()));
    assert_eq!(object(&t, phantom).pos, at, "over the Guard");
    t.at(2, P1, order(&[bike], OrderKind::AttackGround(at)));
    let first = until_launched(&mut t, 40);
    run_checked(&mut t, first.tick + 48 + 32);
    let thrown: Vec<(Tick, Vec2)> = launches(&t).iter().map(|launch| (launch.tick, launch.to)).collect();
    assert_eq!(
        thrown,
        [(first.tick, at), (first.tick + 48, at)],
        "one round per 48 frames"
    );
    let landed: Vec<Tick> = blasts(&t).iter().map(|&(tick, _)| tick).collect();
    assert_eq!(landed, [first.lands_at, first.lands_at + 48]);
    let taken: Vec<f32> = hits_on(&t, guard).iter().map(|&(_, damage)| damage).collect();
    let whole = expected(GUARD_AREA, GUARD_AREA, 1.0, 1.0);
    assert!(
        taken.len() == 2 && taken.iter().all(|&d| (d - whole).abs() < 1e-4),
        "{taken:?}"
    );
    let full = ObjectKind::Phantom.def().max_hp;
    assert_eq!(
        (hits_on(&t, phantom), object(&t, phantom).hp),
        (vec![], full),
        "the air unit, never"
    );
    assert!(object(&t, phantom).pos.distance(at) < 1.0, "it stayed over the blasts");
    let mut t = field(16, 10);
    t.spawn_unit_at(1, BIKE, P1, from)
        .spawn_unit_at(1, ObjectKind::Phantom, P2, at);
    t.at(2, P1, order(&[id(1)], OrderKind::Attack(id(2))));
    run_checked(&mut t, 100);
    assert_eq!(launches(&t), [], "never fired at the air unit");
}

/// Unit `bike`'s angle off its facing to `at`, radians in `(-PI, PI]`, and its turret's.
fn off_facing(t: &TestSim, bike: ObjectId, at: Vec2) -> (f32, f32) {
    let o = object(t, bike);
    let towards = math::vector_angle(at - o.pos);
    let turret = turret_facing(o).expect("a turret");
    (math::wrap_angle(towards - o.facing), math::wrap_angle(towards - turret))
}

/// `bike`'s attack phase now.
fn phase(t: &TestSim, bike: ObjectId) -> Option<AttackPhase> {
    object(t, bike).attack.as_ref().and_then(|attack| attack.phase)
}

/// Player 1's Bike, standing and facing east at (640, 1100) of an open 30 × 24 field, ordered in tick 2 to attack
/// player 2's Guard, `far` su off on a bearing `degrees` north of east; tick 1 has run.
fn bearing_game(degrees: f32, far: f32) -> (TestSim, Vec2) {
    let mut t = field(30, 24);
    let from = Vec2::new(640.0, 1100.0);
    let at = from + math::facing_vector(-degrees.to_radians()) * far;
    t.spawn_unit_at(1, BIKE, P1, from).spawn_unit_at(1, GUARD, P2, at);
    t.at(2, P1, order(&[id(1)], OrderKind::Attack(id(2))));
    run_checked(&mut t, 1);
    (t, at)
}

/// "a **turret with a narrow turn angle**: it can aim a little to either side of the bike's facing, but the bike must
/// still roughly point toward its target" (`> Placeholder stats`: "TurnAngle **40°** total; TurnRate 8°/frame";
/// `systems/controls/command_behaviors.md > AttackingObject`, step 2: "within turret arc (turret units)"; step 4: "If
/// target moves out of range or arc: resume MovingToObject"). Player 1's Bike, standing and facing east, attacks
/// player 2's Guard. 4 grid units off and 15° north of its facing, within the arc and in range: it fires from where it
/// stands, its hull still, its turret turned the 15° onto the Guard in two ticks of its 8° a tick. 400 su off (out of
/// its range) and 60°, 90° and 120° off, out of the arc: it drives on, turning toward the Guard as its wheels let it
/// (it can't turn on the spot: the wheeled invariant W1 checks every tick), and its Aiming ends, the round's spot
/// locked, only with the Guard within 20° of its facing and its turret onto it.
#[test]
fn it_fires_from_its_40_degree_turret_once_its_hull_points_the_target_into_the_arc() {
    let (mut t, at) = bearing_game(15.0, 256.0);
    let bike = id(1);
    step_until(&mut t, 4, "the turret turns", |t| phase(t, bike).is_some());
    let launch = until_launched(&mut t, 20);
    let o = object(&t, bike);
    assert_eq!(
        (o.pos, o.facing, launch.from, launch.to),
        (Vec2::new(640.0, 1100.0), 0.0, o.pos, at)
    );
    let (off, turret) = off_facing(&t, bike, at);
    assert!(
        (off + 15.0_f32.to_radians()).abs() < 1e-4 && turret.abs() < 1e-4,
        "{off}, {turret}"
    );
    for degrees in [60.0, 90.0, 120.0] {
        let (mut t, at) = bearing_game(degrees, 400.0);
        let aimed = |t: &TestSim| matches!(phase(t, bike), Some(AttackPhase::FiringAt { .. }));
        step_until(&mut t, 200, "the Aiming ends", aimed);
        let (off, turret) = off_facing(&t, bike, at);
        assert!(
            off.abs() <= HALF_ARC + TOLERANCE,
            "{degrees}°: {}° off its facing",
            off.to_degrees()
        );
        assert!(
            turret.abs() <= 2.0 * TOLERANCE,
            "{degrees}°: its turret {}° off",
            turret.to_degrees()
        );
        let o = object(&t, bike);
        assert!(
            o.pos.distance(Vec2::new(640.0, 1100.0)) > 100.0,
            "{degrees}°: it drove on"
        );
        assert!(
            math::wrap_angle(o.facing).abs() > 40.0_f32.to_radians(),
            "{degrees}°: its hull turned"
        );
        let launch = until_launched(&mut t, 4);
        assert_eq!(launch.to, at, "{degrees}°");
    }
}

/// `systems/controls/command_behaviors.md > AttackingObject` step 2's "Locomotion = Stopping" for a unit that can't
/// turn in place (`orders::engage`'s `rests_in_arc`): player 1's Bike, standing and facing east, attacks player 2's
/// Guard 4 grid units off, in its range, 45°, 60° or 90° north of its facing, out of its arc. It drives on toward the
/// Guard, turning as its wheels let it, and stops only where the Guard will still be in its arc at rest: braking
/// straight on ("Stopping + Turning: invalid"), it keeps the Guard within 20° of its facing from its Aiming to its
/// launch, and fires at it, its turret onto it. (Before, it braked as soon as the Guard came into its arc; coasting on
/// straight, the Guard slipped out of it again, its Aiming cancelled; and so it ended beside the Guard, as near as it
/// got, and gave the Attack up without firing.) At 45° it fires about 106 su from the Guard. At 60° and 90° the Guard
/// lies near or on the circle it turns on, so it points it into its arc only close up: it fires from beside the Guard,
/// 48 su off, and its own round's blast (AoERadius 48 su, MinRange 0) hits it too, a reading for the Director.
#[test]
fn a_bike_drives_on_until_a_target_in_range_far_out_of_its_arc_stays_in_it_at_rest_and_fires() {
    let bike = id(1);
    for degrees in [45.0, 60.0, 90.0] {
        let (mut t, at) = bearing_game(degrees, 256.0);
        let mut aiming = false;
        let launch = loop {
            step_checked(&mut t);
            aiming |= phase(&t, bike).is_some();
            if aiming {
                let (off, _) = off_facing(&t, bike, at);
                assert!(
                    off.abs() <= HALF_ARC + TOLERANCE,
                    "{degrees}°: {}° off its facing in tick {}",
                    off.to_degrees(),
                    t.sim().tick()
                );
            }
            if let Some(&launch) = launches(&t).first() {
                break launch;
            }
            let current = (object(&t, bike).orders.as_ref()).and_then(|orders| orders.current.as_ref());
            assert!(current.is_some(), "{degrees}°: the Attack goes on");
            assert!(t.sim().tick() < 200, "{degrees}°: it fires");
        };
        assert_eq!((launch.shooter, launch.to), (bike, at), "{degrees}°");
        let (_, turret) = off_facing(&t, bike, at);
        assert!(
            turret.abs() <= 2.0 * TOLERANCE,
            "{degrees}°: its turret {}° off",
            turret.to_degrees()
        );
        let o = object(&t, bike);
        assert!(
            o.pos.distance(Vec2::new(640.0, 1100.0)) > 100.0,
            "{degrees}°: it drove on"
        );
        assert!(
            math::wrap_angle(o.facing).abs() > HALF_ARC,
            "{degrees}°: its hull turned"
        );
        run_for(&mut t, 30);
        let hp = |t: &TestSim, of| t.sim().object(of).and_then(|o| o.hp);
        assert!(hp(&t, id(2)) < Some(100.0), "{degrees}°: the round hits the Guard");
        assert_eq!(
            hp(&t, bike) < Some(100.0),
            degrees > 45.0,
            "{degrees}°: its own blast hits it only from beside the Guard"
        );
    }
}

/// A reading for the Director: a target the rule above still doesn't bring into the Bike's arc. Player 1's Bike,
/// standing and facing east, attacks player 2's Guard 3 grid units off, 120° or 125° north of its facing: in its
/// range, out of its arc, inside the circle it turns on (its radius 128 su), near the rim. It drives on round toward
/// the Guard, but never has the Guard in its arc where it would come to rest, and its move ends against the Guard
/// (`MoveGoal::Object`, reach 0), the Guard more than 20° off its facing; so it gives the Attack up
/// (`systems/controls/command_behaviors.md > AttackingObject > Failure`) without firing, and stands idle. It doesn't
/// loop.
#[test]
fn a_bike_gives_up_a_target_inside_its_turning_circle_that_it_meets_out_of_its_arc() {
    let bike = id(1);
    for degrees in [120.0, 125.0] {
        let (mut t, at) = bearing_game(degrees, 192.0);
        run_for(&mut t, 200);
        assert_eq!(launches(&t), [], "{degrees}°: never fired");
        let o = object(&t, bike);
        let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
        assert!(current.is_none(), "{degrees}°: the Attack given up");
        assert_eq!(
            (t.status(1), phase(&t, bike)),
            (MoveStatus::Arrived, None),
            "{degrees}°"
        );
        let (off, _) = off_facing(&t, bike, at);
        assert!(
            off.abs() > HALF_ARC + TOLERANCE,
            "{degrees}°: {}° off",
            off.to_degrees()
        );
        assert!(o.pos.distance(at) < 64.0, "{degrees}°: against the Guard, at {}", o.pos);
        assert_eq!(object(&t, id(2)).hp, GUARD.def().max_hp);
    }
}

/// Design change 0445: "The Bike doesn't crush" (`systems/units.md > UnitCollision > Crushing`: "Units whose UnitBase
/// can crush (TrackedVehicle, ...)"; a WheeledVehicle can't). Player 1's Bike, sent east through a line of five of
/// player 2's Peacekeepers (GDO Light Infantry, crushable by a crusher) standing across its way, crushes none: it
/// gets past them, and all five stand, untouched, at the end.
#[test]
fn it_crushes_nothing() {
    let mut t = ScenarioBuilder::new(&flat(30, 12))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .start();
    t.without_auto_targeting();
    let (bike, line) = (id(1), [id(2), id(3), id(4), id(5), id(6)]);
    t.spawn_unit_at(1, BIKE, P1, Vec2::new(200.0, 384.0));
    for n in 0..5 {
        t.spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            Vec2::new(640.0 + 48.0 * n as f32, 384.0),
        );
    }
    let goal = Vec2::new(1600.0, 384.0);
    t.at(2, P1, order(&[bike], OrderKind::Move(Target::Point(goal))));
    run_checked(&mut t, 1);
    step_until(&mut t, 400, "the Bike gets there", |t| {
        object(t, bike).pos.distance(goal) < 64.0
    });
    let crushed = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Crushed { .. }));
    assert!(!crushed, "nobody crushed");
    for unit in line {
        assert_eq!(
            object(&t, unit).hp,
            ObjectKind::Peacekeeper.def().max_hp,
            "{unit:?} untouched"
        );
    }
    assert!(!(ObjectKind::AssaultBike.def().unit.as_ref().unwrap().base_def().crushes));
}
