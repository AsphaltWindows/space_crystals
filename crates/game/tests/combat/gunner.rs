//! The Gunner's shots (slice M5-O3; `factions/cults/units/gunner.md > AttackAttributes`, design change 0304: "Visually
//! still an instant gunshot"): tracers, each drawn to the spot its TailDisjointed attack locked (`SimEvent::Attacked`,
//! `SimEvent::Missed`), so that a miss shows where the shot went.

use sc_sim::AttackPhase;

use super::*;

/// Player 1's Gunner attacks player 2's Peacekeeper, 3 grid units east of its edge. In the first frame of its Firing the
/// Peacekeeper is taken two tiles south (a debug teleport: no unit walks off the spot in two frames). The shot misses:
/// a tracer in the Gunner's style from the Gunner to the spot it locked, where the Peacekeeper stood at the end of
/// Aiming, not to where it is, naming no object hit. Its next shot hits the Peacekeeper where it stands now: a tracer
/// to it there.
#[test]
fn a_gunners_miss_is_drawn_to_the_spot_it_locked() {
    let (gunner, target) = (ObjectId(1), ObjectId(2));
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    let from = Vec2::new(300.0, 300.0);
    let at = from + Vec2::new(24.0 + 3.0 * SU_PER_GU, 0.0);
    spawn(&mut app, ObjectKind::Gunner, P1, from);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, at);
    app.update();
    attack(&mut app, P1, gunner, target);
    let ordered = tick(&app);
    let locked = |app: &App| {
        let o = app.world().resource::<SimRes>().object(gunner).unwrap();
        matches!(o.attack.as_ref().unwrap().phase, Some(AttackPhase::FiringAt { .. }))
    };
    while !locked(&app) {
        assert!(tick(&app) < ordered + 20, "it never locked a spot");
        app.update();
    }
    let away = at + Vec2::new(0.0, 2.0 * SU_PER_GU);
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetPosition { id: target, pos: away }),
    );
    while effects(&app).tracers.is_empty() {
        assert!(tick(&app) < ordered + 20, "it never fired");
        app.update();
    }
    let shot = effects(&app).tracers[0].clone();
    assert_eq!(
        (shot.attacker, shot.weapon, shot.target, shot.from, shot.to),
        (gunner, Some(ObjectKind::Gunner), None, from, at),
        "a miss, to the spot"
    );
    assert_eq!(ShotStyle::of(shot.weapon), ShotStyle::Tracer);
    let pos = app.world().resource::<SimRes>().object(target).unwrap().pos;
    assert_eq!(pos, away, "not where the Peacekeeper is");

    while effects(&app).tracers.iter().all(|t| t.target.is_none()) {
        assert!(tick(&app) < shot.tick + 30, "it never fired again");
        app.update();
    }
    let hit = effects(&app).tracers.iter().find(|t| t.target.is_some()).unwrap();
    assert_eq!((hit.target, hit.from, hit.to), (Some(target), from, away));
}
