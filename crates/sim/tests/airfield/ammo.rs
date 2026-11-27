//! The Hornet's missiles (`factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5 missiles**, then it must
//! reload at its assigned Airfield"; "**DoublyDisjointed** (not homing [...] it can Attack Ground)"), the automatic
//! return (`factions/gdo/structures/airfield.md > Returning and rearming`), the single-target hit by its placeholder
//! rule (Q0182), and its aim: "No turret (fires forward; it turns in place to aim)".

use sc_sim::math;
use sc_sim::ruleset::rules;

use super::*;

/// Where [`emptied`]'s Hornet waits, A's rally point 10 grid units east of it, and its target, 3 south of that.
const RALLY: Vec2 = Vec2::new(A_AT.x + 640.0, A_AT.y);
const TARGET_AT: Vec2 = Vec2::new(RALLY.x, RALLY.y + 192.0);
/// A Prospector of player 2's (300 HP), which stands still and fights nothing.
const TARGET: ObjectKind = ObjectKind::Prospector;
/// Its attack's cycle, a launch every 32 frames: "Aim 4, Firing 2, Cooldown 4, Reload 22 frames"
/// (`factions/gdo/units/hornet.md > Placeholder stats`).
const CYCLE: Tick = 32;

/// The Hornet's missiles so far.
fn missiles(t: &TestSim, hornet: u32) -> Vec<(Tick, ObjectId, Option<ObjectId>, Vec2, Tick)> {
    (launches(t).into_iter()).filter(|l| l.1 == id(hornet)).collect()
}

fn at_rest(sim: &Sim, n: u32) -> bool {
    sim.object(id(n))
        .and_then(|o| o.mover.as_ref())
        .is_some_and(|m| m.velocity == Vec2::ZERO)
}

/// [`field`] with player 2's [`TARGET`] (7) at [`TARGET_AT`]: A trains a Hornet (8) with its rally point at [`RALLY`],
/// where the Hornet flies and comes to rest; then it attacks the target until it has fired its fifth missile, the
/// ammo falling with each. Returns the game, in the tick of the fifth launch.
fn emptied() -> TestSim {
    let mut t = field(|builder| builder.spawn(TARGET, Some(P2), at(TARGET_AT.x, TARGET_AT.y)));
    let set_rally = Command::Production {
        objects: vec![id(A)],
        command: ProductionCommand::SetRally(Target::Point(RALLY)),
    };
    next(&mut t, P1, set_rally);
    next(&mut t, P1, train(A));
    step_until(&mut t, 2 * FRAMES, "the Hornet", |sim| sim.object(id(8)).is_some());
    assert_eq!(
        aircraft(t.sim(), 8),
        AircraftState {
            ammo: AMMO,
            base: Some(id(A))
        }
    );
    step_until(&mut t, 400, "at its rally point", |sim| {
        at_rest(sim, 8) && sim.object(id(8)).is_some_and(|o| o.pos.distance(RALLY) < 2.0)
    });
    next(&mut t, P1, order(&[8], OrderKind::Attack(id(7))));
    let mut fired = 0;
    while fired < AMMO as usize {
        assert!(t.sim().tick() < 2000, "five missiles");
        t.step();
        let now = missiles(&t, 8).len();
        if now > fired {
            fired = now;
            assert_eq!(aircraft(t.sim(), 8).ammo, AMMO - fired as u32, "a round a missile");
        }
    }
    t
}

/// "Five shots of 30" (`m6-plan.md > GH1 > Acceptance`): five missiles, one every 32 frames, each landing on the spot
/// its target stood on and hitting it for its 30 less the target's armour. Then the attack is unavailable
/// (`Reject::OutOfAmmo`, for an Attack and an Attack Ground alike; its right-click on an enemy no longer attacks), and
/// "when an aircraft runs out of ammo, it automatically flies back to its assigned Airfield", where it hovers until
/// slice M6-GH2 lands it. Back home with auto-targeting on, it fires at nothing.
#[test]
fn five_missiles_of_30_then_it_flies_home() {
    let mut t = emptied();
    let shots = missiles(&t, 8);
    assert_eq!(shots.len(), AMMO as usize);
    let attack = HORNET.def().unit.as_ref().and_then(|u| u.attack).expect("its missiles");
    assert_eq!(attack.damage, DAMAGE);
    for pair in shots.windows(2) {
        assert_eq!(pair[1].0 - pair[0].0, CYCLE, "{shots:?}");
    }
    assert!(
        shots.iter().all(|s| s.2 == Some(id(7)) && s.3 == TARGET_AT),
        "aimed at its spot: {shots:?}"
    );
    assert_eq!(
        aircraft(t.sim(), 8),
        AircraftState {
            ammo: 0,
            base: Some(id(A))
        }
    );
    assert_eq!(
        gate(t.sim(), P1, &order(&[8], OrderKind::Attack(id(7)))),
        Err(Reject::OutOfAmmo)
    );
    let ground = order(&[8], OrderKind::AttackGround(TARGET_AT));
    assert_eq!(gate(t.sim(), P1, &ground), Err(Reject::OutOfAmmo));
    let click = t.sim().default_order(id(8), Target::Object(id(7)));
    assert!(!matches!(click, Some(OrderKind::Attack(_))), "{click:?}");
    assert_eq!(doing(t.sim(), 8), Some(OrderKind::ReturnToBase), "home by itself");

    let last_lands = shots.last().expect("five").4;
    t.run_until(last_lands);
    let hits = hits_of(&t, 8);
    assert_eq!(hits.len(), AMMO as usize, "every one hit: {hits:?}");
    let lands: Vec<Tick> = shots.iter().map(|s| s.4).collect();
    assert_eq!(hits.iter().map(|h| h.0).collect::<Vec<_>>(), lands);
    let each = hits[0].2;
    let point = TARGET.def().armor.point;
    assert!(each > 0.0 && each <= DAMAGE - point + 1e-4, "{each}");
    assert!(hits.iter().all(|h| h.1 == id(7) && h.2 == each), "{hits:?}");
    let max = TARGET.def().max_hp.expect("HP");
    let hp = object(t.sim(), 7).hp.expect("HP");
    assert!((hp - (max - 5.0 * each)).abs() < 1e-3, "{hp}");
    assert!(blasts_of(&t, 8).iter().all(|b| b.2 == 0.0), "a single-target missile");

    step_until(&mut t, 400, "home", |sim| at_rest(sim, 8) && doing(sim, 8).is_none());
    assert!(t.pos(8).distance(A_AT) < 2.0, "over its Airfield: {}", t.pos(8));
    assert_eq!(
        aircraft(t.sim(), 8),
        AircraftState {
            ammo: 0,
            base: Some(id(A))
        }
    );
    debug(&mut t, DebugCommand::SetAutoTargeting { on: true });
    let near = A_AT + Vec2::new(0.0, 160.0);
    t.spawn_unit_at(t.sim().tick() + 1, TARGET, P2, near);
    t.run(3 * CYCLE);
    assert_eq!(missiles(&t, 8).len(), AMMO as usize, "out of ammo, it fires at nothing");
    assert!(t.pos(8).distance(A_AT) < 2.0, "and stays home: {}", t.pos(8));
}

/// "The automatic return **can be interrupted** by giving the aircraft any other order": a Move takes it elsewhere,
/// where it stays, with no ammo and its base; F (Return to Base) then flies it home.
#[test]
fn a_move_interrupts_the_return_and_the_ammo_stays_at_0() {
    let mut t = emptied();
    let away = RALLY + Vec2::new(320.0, 320.0);
    next(&mut t, P1, order(&[8], OrderKind::Move(Target::Point(away))));
    step_until(&mut t, 400, "away", |sim| at_rest(sim, 8) && doing(sim, 8).is_none());
    assert!(t.pos(8).distance(away) < 2.0, "{}", t.pos(8));
    t.run(64);
    assert!(t.pos(8).distance(away) < 2.0, "it stays: {}", t.pos(8));
    assert_eq!(doing(t.sim(), 8), None);
    assert_eq!(
        aircraft(t.sim(), 8),
        AircraftState {
            ammo: 0,
            base: Some(id(A))
        }
    );

    let home = order(&[8], OrderKind::ReturnToBase);
    assert_eq!(gate(t.sim(), P1, &home), Ok(()));
    next(&mut t, P1, home);
    step_until(&mut t, 400, "home", |sim| at_rest(sim, 8) && doing(sim, 8).is_none());
    assert!(t.pos(8).distance(A_AT) < 2.0, "{}", t.pos(8));
}

/// Q0182's placeholder for a single-target projectile that can miss (`sc_sim::combat::projectiles`): "whatever object
/// covers the locked point is hit". The Hornet's missile lands on the spot its target stood on as it was launched: a
/// target standing still is hit; one that has moved off the spot is not, and the missile's damage is lost.
#[test]
fn a_missile_hits_a_target_standing_still_and_misses_one_that_moved() {
    let hornet_at = Vec2::new(1600.0, 1600.0);
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P1), at(hornet_at.x, hornet_at.y)) // 7
            .spawn(TARGET, Some(P2), at(hornet_at.x + 224.0, hornet_at.y)) // 8
            .spawn(TARGET, Some(P2), at(hornet_at.x, hornet_at.y + 224.0)) // 9
    });
    next(&mut t, P1, order(&[7], OrderKind::Attack(id(8))));
    let mut shot = None;
    for _ in 0..200 {
        if let Some(&s) = missiles(&t, 7).first() {
            shot = Some(s);
            break;
        }
        t.step();
    }
    let (_, _, target, to, lands_at) = shot.expect("a missile at #8");
    assert_eq!((target, to), (Some(id(8)), t.pos(8)), "aimed at the spot #8 stands on");
    t.run_until(lands_at);
    assert_eq!(
        hits_of(&t, 7).iter().map(|h| (h.0, h.1)).collect::<Vec<_>>(),
        [(lands_at, id(8))]
    );

    next(&mut t, P1, order(&[7], OrderKind::Attack(id(9))));
    let spot = t.pos(9);
    for _ in 0..200 {
        if missiles(&t, 7).len() == 2 {
            break;
        }
        t.step();
    }
    let (_, _, target, to, lands_at) = missiles(&t, 7)[1];
    assert_eq!((target, to), (Some(id(9)), spot));
    assert!(lands_at > t.sim().tick() + 1, "still in flight");
    let hp = object(t.sim(), 9).hp;
    debug(
        &mut t,
        DebugCommand::SetPosition {
            id: id(9),
            pos: spot + Vec2::new(-160.0, 0.0),
        },
    );
    t.run_until(lands_at);
    let blasts = blasts_of(&t, 7);
    assert_eq!(
        blasts.last().map(|b| (b.0, b.1)),
        Some((lands_at, spot)),
        "it lands on the spot"
    );
    assert!(
        hits_of(&t, 7).iter().all(|h| h.1 != id(9)),
        "and hits nothing: {:?}",
        hits_of(&t, 7)
    );
    assert_eq!(object(t.sim(), 9).hp, hp);
}

/// "No turret (fires forward; it turns in place to aim)": ordered to attack a target in range behind it, the Hornet
/// hovers where it is, turns toward it by at most its TurnRate a frame (8°), and launches only once it faces it
/// (`Combat::facing_tolerance`); it holds its place through all five missiles.
#[test]
fn it_hovers_and_turns_in_place_to_aim() {
    let hornet_at = Vec2::new(1600.0, 1600.0);
    let target_at = hornet_at + Vec2::new(-192.0, 64.0);
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P1), at(hornet_at.x, hornet_at.y)) // 7
            .spawn(TARGET, Some(P2), at(target_at.x, target_at.y)) // 8
    });
    let bearing = math::vector_angle(target_at - hornet_at);
    let off = |facing: f32| math::wrap_angle(bearing - facing).abs();
    let start = object(t.sim(), 7).facing;
    assert!(off(start) > math::PI / 2.0, "behind it: {start}");
    let turn_rate = 8.0 * math::PI / 180.0;
    let tolerance = rules().tuning.combat.facing_tolerance;
    next(&mut t, P1, order(&[7], OrderKind::Attack(id(8))));
    let ordered = t.sim().tick();
    let first = object(t.sim(), 7).facing;
    assert!(
        math::wrap_angle(first - start).abs() <= turn_rate + 1e-4,
        "at most 8° a frame"
    );
    let (mut facing, mut faced) = (first, None);
    while missiles(&t, 7).len() < AMMO as usize {
        assert!(t.sim().tick() < 1000, "five missiles");
        t.step();
        let o = object(t.sim(), 7);
        assert_eq!(o.pos, hornet_at, "it hovers in place");
        assert!(off(o.facing) <= off(facing) + 1e-4, "it turns toward the target");
        assert!(
            math::wrap_angle(o.facing - facing).abs() <= turn_rate + 1e-4,
            "at most 8° a frame"
        );
        facing = o.facing;
        if off(facing) <= tolerance && faced.is_none() {
            faced = Some(t.sim().tick());
        }
        for &(tick, ..) in &missiles(&t, 7) {
            if tick == t.sim().tick() {
                assert!(off(facing) <= tolerance, "it fires only facing its target: {facing}");
            }
        }
    }
    let faced = faced.expect("it faced its target");
    let turns = ((off(start) - tolerance) / turn_rate).ceil() as Tick;
    assert!(faced + 1 - ordered >= turns, "{faced}: {turns} turns");
    assert!(missiles(&t, 7)[0].0 > faced, "after its aim");
}
