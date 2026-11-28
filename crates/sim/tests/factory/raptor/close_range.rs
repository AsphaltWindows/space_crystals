//! Question Q0184 for the Raptor (slice M6-GF): its cases at close range, which the Assault Bike raised
//! (`tests/vehicle_assembly/mortar.rs`), pinned here together. Design change 0454 (answering Q0179) settled the first:
//! "**Can't aim from where it stands**" (`systems/controls/command_behaviors.md > AttackingObject`): a unit in range
//! that can't bring its weapon to bear "keeps trying to get into a position from which it can attack", rather than
//! giving the Attack up (slice M6-S6a, `orders::aim`); and design change 0460 (answering Q0184) the second, for a unit
//! whose weapon can target the ground, which the Raptor's can't.

use sc_sim::math;

use super::*;

/// Question Q0184's first case for the Raptor, as design change 0454 settled it (and as `tests/vehicle_assembly/
/// mortar.rs` pins it for the Bike): player 1's Raptor, standing and facing east at (640, 1100), attacks player 2's
/// Peacekeeper 2 grid units off, 120° or 125° north of its facing: in its Range, out of its ±30° arc, inside the circle
/// it turns on (its radius 80 su), near its rim. As built before, it drove round toward the Peacekeeper, never had it
/// in its arc where it would come to rest (`rests_in_arc`), ended its move against it and gave the Attack up without
/// firing. Now it can't aim from where it stands, and repositions (`orders::aim`): it drives straight on, east, more
/// than its turning radius before its first missile, until the Peacekeeper lies a grid unit beyond the circles it
/// turns on, then comes round onto it and kills it with four missiles, the Attack running until the Peacekeeper dies.
/// Deeper inside the circle, 100 su off at 120°, or 2 grid units off at 90° (which it brought into its arc and killed
/// before, closing in), it does the same.
#[test]
fn a_raptor_repositions_from_a_target_inside_its_turning_circle_and_kills_it() {
    let from = Vec2::new(640.0, 1100.0);
    let game = |degrees: f32, far: f32| {
        let pk = from + math::facing_vector(-degrees.to_radians()) * far;
        let mut t = field(from, pk);
        t.at(2, P1, order(&[RAPTOR_ID], OrderKind::Attack(PK_ID)));
        (t, pk)
    };
    for (degrees, far) in [(120.0, 128.0), (125.0, 128.0), (120.0, 100.0), (90.0, 128.0)] {
        let (mut t, pk) = game(degrees, far);
        let centre = from + Vec2::new(0.0, -80.0);
        assert!(
            pk.distance(centre) < 80.0,
            "{degrees}°, {far} su: inside the circle it turns on"
        );
        t.run_until(2);
        let mut east = 0.0_f32;
        while died(&t, PK_ID).is_none() {
            assert!(t.sim().tick() < 300, "{degrees}°, {far} su: it kills the Peacekeeper");
            assert_eq!(
                current(&t, RAPTOR_ID),
                Some(OrderKind::Attack(PK_ID)),
                "{degrees}°, {far} su, tick {}",
                t.sim().tick()
            );
            if launches(&t).is_empty() {
                east = east.max(object(&t, RAPTOR_ID).pos.x - from.x);
            }
            t.step();
        }
        assert_eq!(launches(&t).len(), 4, "{degrees}°, {far} su");
        assert!(east > 80.0, "{degrees}°, {far} su: it drove on first, {east} su");
    }
}

/// Question Q0184's second case, the Bike's Attack Ground at a spot out of its arc, which it circled forever (design
/// change 0460 settled it for the Bike: `tests/vehicle_assembly/mortar.rs`): not the Raptor's, whose HeadDisjointed
/// missile can't target the ground (`systems/controls/commands.md > AttackGround`: "Only available to units whose
/// AttackType has CanTargetGround"; `combat::can_target_ground`). The gate refuses its Attack Ground
/// (`Reject::CannotTargetGround`), and one given anyway changes nothing.
#[test]
fn its_attack_ground_is_refused() {
    let from = Vec2::new(640.0, 1100.0);
    let mut t = field(from, Vec2::new(1400.0, 1100.0));
    let spot = from + math::facing_vector(-120f32.to_radians()) * 128.0;
    let attack_ground = order(&[RAPTOR_ID], OrderKind::AttackGround(spot));
    assert_eq!(gate(&t, P1, attack_ground.clone()), Err(Reject::CannotTargetGround));
    t.at(2, P1, attack_ground);
    run_for(&mut t, 40);
    assert_eq!(current(&t, RAPTOR_ID), None);
    assert_eq!(object(&t, RAPTOR_ID).pos, from, "it never moved");
}
