//! The Bike's mortar (`factions/syndicate/units/assault_bike.md`: "like the Grenadier's grenade (slow, dodgeable
//! explosive projectile with area damage) but with **slightly longer range**", "on a **turret with a narrow turn
//! angle**", "Ground only (assumed, like the Grenadier); can Attack Ground"; `> Placeholder stats`: "Damage 120 total
//! over the circle; AoERadius 48 su; Range **5 grid units**; ProjectileSpeed 8 su/frame", "Turret: TurnAngle **40°**
//! total; TurnRate 8°/frame"), and its wheels (design change 0445: it crushes nothing). On an open field, both players
//! the Syndicate's, no structure: units only, debug-spawned, auto-targeting off.

use std::f32::consts::PI;

use sc_sim::combat::{AttackPhase, overlap, turret_facing};
use sc_sim::math;

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
/// Guard 4 grid units off, in its range, 45°, 60° or 90° north of its facing, out of its arc. It drives on, turning as
/// its wheels let it, and stops only where the Guard will still be in its arc at rest: braking straight on
/// ("Stopping + Turning: invalid"), it keeps the Guard within 20° of its facing from its Aiming to its launch, and
/// fires at it, its turret onto it. (Before slice M6-SV, it braked as soon as the Guard came into its arc; coasting on
/// straight, the Guard slipped out of it again, its Aiming cancelled; and so it ended beside the Guard, as near as it
/// got, and gave the Attack up without firing.) The Guard lies inside the circle it turns on, or less than two grid
/// units beyond its rim (`orders::aim`'s margin of a grid unit, and its mortar's MinRange, a grid unit since design
/// change 0460): it can't aim from where it stands (design change 0454, `> Can't aim from where it stands`), so it
/// repositions first, straight on (a vehicle backs up only on the Reverse order) until the Guard lies that far beyond
/// the rim, and comes round onto it from there, to rest beyond its MinRange. Before 0454 it came round at once, and at
/// 60° and 90° brought the Guard into its arc only beside it, 48 su off, its own round's blast (AoERadius 48 su,
/// MinRange 0 then) hitting it too; now its blast never does.
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
        assert_eq!(hp(&t, bike), Some(100.0), "{degrees}°: its own blast misses it");
    }
}

/// Question Q0184's first case, as design change 0454 settled it ("**Can't aim from where it stands**",
/// `systems/controls/command_behaviors.md > AttackingObject`: "it keeps trying to get into a position from which it
/// can attack the target ... It doesn't wait in place, and doesn't give up for this reason"; slice M6-S6a,
/// `orders::aim`). Player 1's Bike, standing and facing east, attacks player 2's Guard 3 grid units off, 120° or 125°
/// north of its facing: in its range, out of its arc, inside the circle it turns on (its radius 128 su), near the
/// rim. Before 0454 it drove on round toward the Guard, never had it in its arc where it would come to rest, ended its
/// move against it and gave the Attack up without firing. Now it drives straight on, east, until the Guard lies two
/// grid units beyond the circle's rim (the margin, and its mortar's MinRange: design change 0460), more than 64 su,
/// comes round onto it, and fires at it from beyond its MinRange and its own round's blast, the Attack running all the
/// while.
#[test]
fn a_bike_repositions_from_a_target_inside_its_turning_circle_and_fires() {
    let bike = id(1);
    for degrees in [120.0, 125.0] {
        let (mut t, at) = bearing_game(degrees, 192.0);
        let mut east = 0.0_f32;
        let launch = loop {
            step_checked(&mut t);
            if let Some(&launch) = launches(&t).first() {
                break launch;
            }
            east = east.max(object(&t, bike).pos.x - 640.0);
            let current = (object(&t, bike).orders.as_ref()).and_then(|orders| orders.current.as_ref());
            assert!(current.is_some(), "{degrees}°: the Attack goes on");
            assert!(t.sim().tick() < 200, "{degrees}°: it fires");
        };
        assert_eq!((launch.shooter, launch.to), (bike, at), "{degrees}°");
        assert!(east > 64.0, "{degrees}°: it drove straight on first, {east} su");
        run_for(&mut t, 40);
        let hp = |t: &TestSim, of| t.sim().object(of).and_then(|o| o.hp);
        assert!(hp(&t, id(2)) < Some(100.0), "{degrees}°: the round hits the Guard");
        assert_eq!(hp(&t, bike), Some(100.0), "{degrees}°: its own blast misses it");
    }
}

/// Slice M6-SV's grid of close targets, every one in the Bike's range and out of its arc ([`bearing_game`]: the Guard
/// 168 to 224 su off, every 8, and 50° to 130° north of its facing, every 5°): before design change 0454, 34 of its
/// 136 games ended with the Attack given up unfired, the Bike against the Guard, and the other 102 fired from beside
/// the Guard, its own blast hitting it. Now it fires in every one, within 120 ticks, its Attack never given up, from
/// about 2 grid units off the Guard, edge to edge (`orders::aim`: inside the circle it turns on or near its rim, it
/// repositions first, and comes round onto the Guard to rest beyond its mortar's MinRange of a grid unit, design change
/// 0460), and its own blast never hits it.
#[test]
fn it_fires_at_every_close_target_of_the_grid_beyond_its_own_blast() {
    let bike = id(1);
    for far in (168..=224).step_by(8) {
        for degrees in (50..=130).step_by(5) {
            let (mut t, at) = bearing_game(degrees as f32, far as f32);
            let launch = loop {
                step_checked(&mut t);
                if let Some(&launch) = launches(&t).first() {
                    break launch;
                }
                let current = (object(&t, bike).orders.as_ref()).and_then(|orders| orders.current.as_ref());
                assert!(current.is_some(), "{far} su, {degrees}°: the Attack goes on");
                assert!(t.sim().tick() < 120, "{far} su, {degrees}°: it fires");
            };
            assert_eq!(launch.to, at, "{far} su, {degrees}°");
            let gap = shape(&t, bike).gap(&shape(&t, id(2)));
            assert!(
                ABOUT_2_GU.contains(&gap),
                "{far} su, {degrees}°: fired from {gap} su off"
            );
            run_for(&mut t, 40);
            assert_eq!(
                object(&t, bike).hp,
                Some(100.0),
                "{far} su, {degrees}°: its own blast misses it"
            );
        }
    }
}

/// About 2 grid units, space units: where a Bike that comes round onto a mark beyond its mortar's MinRange of a grid
/// unit, or backs off to MinRange + 1 grid unit from one inside it, fires from (design change 0460: "fires from about 2
/// GU").
const ABOUT_2_GU: std::ops::RangeInclusive<f32> = 120.0..=192.0;

/// Design change 0460's Verification 1, Q0184's second case ("at 60° and 90° off, it reaches the target's side before
/// the target is in its arc. It then fires from 48 su, and its blast takes 13–15 of its own 100 HP each round"): "Order
/// a Bike to attack a target 48 su to its side. It drives off, comes round, and fires from about 2 GU. It takes no
/// damage from its own rounds." Its mortar's MinRange of a grid unit (`factions/syndicate/units/assault_bike.md >
/// Placeholder stats`) has player 2's Guard, 48 su north of player 1's Bike, edge to edge, inside it: the Bike backs
/// off to MinRange + 1 grid unit (`systems/controls/command_behaviors.md > Target inside MinRange`), driving off (a
/// vehicle backs up only on the Reverse order, `systems/units.md > UnitCollision`), comes round onto the Guard, its
/// hull pointing it into its arc, and fires at it from about 2 grid units, round after round until it falls, its own HP
/// whole.
#[test]
fn ordered_onto_a_target_48_su_beside_it_it_drives_off_comes_round_and_fires_from_about_2_grid_units() {
    let mut t = field(30, 24);
    let (bike, guard) = (id(1), id(2));
    let from = Vec2::new(640.0, 1100.0);
    // Its half width 16 su, the Guard's 18.
    let at = from + Vec2::new(0.0, -(16.0 + 48.0 + 18.0));
    t.spawn_unit_at(1, BIKE, P1, from).spawn_unit_at(1, GUARD, P2, at);
    t.at(2, P1, order(&[bike], OrderKind::Attack(guard)));
    run_checked(&mut t, 1);
    assert!((shape(&t, bike).gap(&shape(&t, guard)) - 48.0).abs() < 1e-3);
    let mut farthest = 0.0_f32;
    let first = loop {
        step_checked(&mut t);
        farthest = farthest.max(shape(&t, bike).gap(&shape(&t, guard)));
        if let Some(&launch) = launches(&t).first() {
            break launch;
        }
        assert!(t.sim().tick() < 150, "it fires");
    };
    assert_eq!((first.shooter, first.to), (bike, at));
    let gap = shape(&t, bike).gap(&shape(&t, guard));
    assert!(ABOUT_2_GU.contains(&gap), "it fires from {gap} su off");
    assert!(
        farthest >= 127.0,
        "it drove off to MinRange + 1 grid unit: {farthest} su"
    );
    let (off, turret) = off_facing(&t, bike, at);
    assert!(
        off.abs() <= HALF_ARC + TOLERANCE,
        "it came round: {}° off",
        off.to_degrees()
    );
    assert!(
        turret.abs() <= 2.0 * TOLERANCE,
        "its turret {}° off",
        turret.to_degrees()
    );
    step_until(&mut t, 600, "the Guard falls", |t| t.sim().object(guard).is_none());
    run_for(&mut t, 40);
    assert!(launches(&t).len() > 1, "round after round");
    assert_eq!(object(&t, bike).hp, Some(100.0), "its own rounds never hit it");
}

/// Player 1's Bike, standing and facing east at (640, 1100) of an open 30 × 24 field, ordered in tick 2 to attack the
/// ground `far` su off on a bearing `degrees` north of east; tick 1 has run.
fn ground_game(degrees: f32, far: f32) -> (TestSim, Vec2) {
    let mut t = field(30, 24);
    let from = Vec2::new(640.0, 1100.0);
    let at = from + math::facing_vector(-degrees.to_radians()) * far;
    t.spawn_unit_at(1, BIKE, P1, from);
    t.at(2, P1, order(&[id(1)], OrderKind::AttackGround(at)));
    run_checked(&mut t, 1);
    (t, at)
}

/// Q0184's third case ("Attack Ground at a spot in range but out of its arc, 96–256 su away and 30–150° off its
/// facing. As built (step 4 as written): it circles the spot forever, stopping once a lap, and never fires"), as design
/// change 0460 settled it (`systems/controls/command_behaviors.md > AttackingLocation > Algorithm`: "**Can't aim from
/// where it stands** applies [...] it doesn't circle the spot forever"; Verification 2: "Order Attack Ground at a spot
/// in range but 90° off its facing. It repositions and keeps shelling the spot, and doesn't circle forever"; slice
/// M6-S6a, `orders::attack_ground` and `orders::aim`). Player 1's Bike, standing and facing east, attacks the ground
/// in its range, out of its arc: 96 to 256 su off, every 32, and 30° to 150° north of its facing, every 15°. In every
/// one it repositions, fires at the spot within 200 ticks from beyond its MinRange of a grid unit, and goes on, from
/// where it stopped, one round every 48 frames, the order running and its own blast never hitting it.
#[test]
fn attacking_the_ground_out_of_its_arc_it_repositions_and_keeps_shelling_the_spot() {
    let bike = id(1);
    for far in (96..=256).step_by(32) {
        for degrees in (30..=150).step_by(15) {
            let (mut t, at) = ground_game(degrees as f32, far as f32);
            let first = loop {
                step_checked(&mut t);
                if let Some(&launch) = launches(&t).first() {
                    break launch;
                }
                assert!(t.sim().tick() < 200, "{far} su, {degrees}°: it fires");
            };
            let off = shape(&t, bike).distance_to(at);
            assert!(
                off >= 64.0,
                "{far} su, {degrees}°: it fires from {off} su off, beyond its MinRange"
            );
            run_checked(&mut t, first.tick + 4 * 48 + 8);
            let thrown: Vec<(Tick, Vec2)> = launches(&t).iter().map(|launch| (launch.tick, launch.to)).collect();
            let every: Vec<(Tick, Vec2)> = (0..5).map(|n| (first.tick + 48 * n, at)).collect();
            assert_eq!(thrown, every, "{far} su, {degrees}°: one round per 48 frames");
            run_for(&mut t, 40);
            let current = (object(&t, bike).orders.as_ref()).and_then(|orders| orders.current.as_ref());
            assert!(current.is_some(), "{far} su, {degrees}°: the order goes on");
            assert_eq!(
                object(&t, bike).hp,
                Some(100.0),
                "{far} su, {degrees}°: its own blast misses it"
            );
        }
    }
}

/// Design change 0460's Verification 3: "Order Attack Ground right next to the Bike. It backs off before firing"
/// (`systems/controls/command_behaviors.md > AttackingLocation > Algorithm`: "**Target inside MinRange** applies to the
/// location too (it backs off to MinRange + 1 GU)"). Player 1's Bike, standing and facing east, attacks the ground 48
/// su north of it, from its edge, inside its mortar's MinRange of a grid unit: it drives off, comes round, and fires
/// at the spot from about 2 grid units, then round after round, its own blast never hitting it.
#[test]
fn attacking_the_ground_right_beside_it_it_backs_off_before_firing() {
    let bike = id(1);
    // Its half width 16 su.
    let (mut t, at) = ground_game(90.0, 16.0 + 48.0);
    assert!((shape(&t, bike).distance_to(at) - 48.0).abs() < 1e-3);
    let first = until_launched(&mut t, 150);
    assert_eq!(first.to, at);
    let off = shape(&t, bike).distance_to(at);
    assert!(ABOUT_2_GU.contains(&off), "it backed off, and fires from {off} su off");
    run_checked(&mut t, first.tick + 2 * 48 + 40);
    assert_eq!(launches(&t).len(), 3, "round after round");
    assert_eq!(object(&t, bike).hp, Some(100.0), "its own blast misses it");
}

/// Design change 0460: "Auto-targeting skips enemies inside MinRange, as for every weapon" (`systems/combat.md >
/// ValidTarget`; `factions/syndicate/units/assault_bike.md > Placeholder stats`: the mortar's MinRange of a grid unit).
/// Player 1's idle Bike, auto-targeting on, faces east; player 2's Shadebringer, unarmed, stands straight ahead of it,
/// in its arc and its range. 48 su off, edge to edge, inside its MinRange, the Bike never takes it for a target and
/// stands where it is; 96 su off, beyond it, the Bike fires at it.
#[test]
fn an_idle_bike_never_takes_an_enemy_inside_its_min_range_for_a_target() {
    for (gap, fires) in [(48.0, false), (96.0, true)] {
        let mut t = ScenarioBuilder::new(&flat(30, 24))
            .faction(P1, Faction::Syndicate)
            .faction(P2, Faction::Syndicate)
            .start();
        let (bike, from) = (id(1), Vec2::new(640.0, 1100.0));
        // Their half lengths: 28 su each.
        let at = from + Vec2::new(28.0 + gap + 28.0, 0.0);
        t.spawn_unit_at(1, BIKE, P1, from)
            .spawn_unit_at(1, ObjectKind::Shadebringer, P2, at);
        run_checked(&mut t, 1);
        assert!((shape(&t, bike).gap(&shape(&t, id(2))) - gap).abs() < 1e-3);
        run_checked(&mut t, 200);
        assert_eq!(!launches(&t).is_empty(), fires, "{gap} su off");
        if !fires {
            let o = object(&t, bike);
            let target = o.attack.as_ref().and_then(|attack| attack.target);
            assert_eq!(
                (o.pos, target),
                (from, None),
                "{gap} su off: it stands, without a target"
            );
        }
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
