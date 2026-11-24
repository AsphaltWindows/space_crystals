//! (G) One turn budget per tick (`movement::Turns`): the movement step (5d) and the aim (5e) share a unit's
//! TurnRate, with the defs of M1, whose units turn 180° per tick.

use super::*;
use crate::combat::FACING_TOLERANCE;
use crate::math::{self, PI};
use crate::mover::MoveGoal;

/// A Peacekeeper that walks off to the south-west turns 135° in phase 5d, and in 5e of the same tick begins to aim
/// at an enemy to its north-west, about 90° further. The aim has only the 45° left of the tick's 180°: the unit ends
/// the tick facing west, not its target. In the next tick its attack holds its base, 5d doesn't turn it, and the aim
/// turns it onto the target.
#[test]
fn moving_and_aiming_share_the_turn_rate_of_a_tick() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.setup.config = SimConfig::DEBUG;
    let mut sim = Sim::new(&scenario);
    let start = Vec2::new(500.0, 300.0);
    let walker = insert_unit(&mut sim, start);
    let enemy = insert_unit(&mut sim, start + Vec2::new(-100.0, -100.0));
    for id in [walker, enemy] {
        let o = obj(&mut sim, id.0);
        (o.facing, o.prev_facing, o.orders) = (0.0, 0.0, None);
    }
    obj(&mut sim, enemy.0).owner = Some(PlayerId(2));
    let end = start + Vec2::new(-200.0, 200.0);
    mover(&mut sim, walker.0).go_to(MoveGoal::Point(end), None, 0);
    mover(&mut sim, walker.0).follow(vec![start, end], end, 0);
    attack(&mut sim, walker.0).engage(Some(enemy));
    sim.vision.update(&sim.map, sim.objects.values(), sim.tick);

    sim.step(&TickInputs::default());
    let moved = sim.nav.turns().angle(walker);
    assert!((moved - 0.75 * PI).abs() < 1e-6, "5d turned it south-west: {moved}");
    let o = &sim.objects[&walker];
    assert_ne!(o.pos, start, "and it walked");
    assert_eq!(
        o.attack.as_ref().unwrap().phase,
        Some(AttackPhase::Aiming { target: enemy })
    );
    // The aim turned it by the rest of the budget, the same way: 180° in all.
    assert_eq!(o.facing, math::wrap_angle(moved + (PI - moved)));
    assert!((o.facing - PI).abs() < 1e-6, "it faces west: {}", o.facing);
    let bearing = |sim: &Sim| {
        let (o, e) = (&sim.objects[&walker], &sim.objects[&enemy]);
        math::wrap_angle(math::vector_angle(e.pos - o.pos))
    };
    let off = math::wrap_angle(bearing(&sim) - o.facing).abs();
    assert!(off > FACING_TOLERANCE, "not yet at its target: {off} rad off");

    sim.step(&TickInputs::default());
    assert_eq!(sim.nav.turns().angle(walker), 0.0, "held by its attack");
    assert_eq!(sim.objects[&walker].facing, bearing(&sim));
}
