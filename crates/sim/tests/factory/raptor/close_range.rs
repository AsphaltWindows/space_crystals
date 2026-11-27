//! Question Q0184 for the Raptor, the Raptor as built (slice M6-GF): its cases at close range, which the Assault Bike
//! raised (`tests/vehicle_assembly/mortar.rs`), pinned here together, readings for the Director. Design change 0454
//! (answering Q0179) changes the first: "Can't aim from where it stands" (`systems/controls/command_behaviors.md >
//! AttackingObject`): a unit in range that can't bring its weapon to bear keeps trying to reach a spot it can attack
//! from, rather than giving the Attack up. Slice M6-S6a builds it, and changes these pins.

use sc_sim::math;
use sc_sim::mover::MoveStatus;

use super::*;

/// Question Q0184's first case for the Raptor, a reading for the Director (as `tests/vehicle_assembly/mortar.rs`
/// pins it for the Bike): player 1's Raptor, standing and facing east at (640, 1100), attacks player 2's Peacekeeper 2
/// grid units off, 120° or 125° north of its facing: in its Range, out of its ±30° arc, inside the circle it turns on
/// (its radius 80 su), near its rim. It drives round toward the Peacekeeper, but never has it in its arc where it
/// would come to rest (`rests_in_arc`), and its move ends against it (`MoveGoal::Object`, reach 0), the Peacekeeper
/// more than 30° off its facing; so it gives the Attack up (`systems/controls/command_behaviors.md > AttackingObject >
/// Failure`) without firing, and stands idle. It doesn't loop. Deeper inside the circle, 100 su off at 120°, or 2 grid
/// units off at 90°, it brings the Peacekeeper into its arc and kills it.
#[test]
fn a_raptor_gives_up_a_target_near_the_rim_of_its_turning_circle_that_it_meets_out_of_its_arc() {
    let from = Vec2::new(640.0, 1100.0);
    let game = |degrees: f32, far: f32| {
        let pk = from + math::facing_vector(-degrees.to_radians()) * far;
        let mut t = field(from, pk);
        t.at(2, P1, order(&[RAPTOR_ID], OrderKind::Attack(PK_ID)));
        (t, pk)
    };
    for degrees in [120.0, 125.0] {
        let (mut t, pk) = game(degrees, 128.0);
        let centre = from + Vec2::new(0.0, -80.0);
        assert!(pk.distance(centre) < 80.0, "{degrees}°: inside the circle it turns on");
        run_for(&mut t, 200);
        assert_eq!(launches(&t), [], "{degrees}°: never fired");
        assert_eq!(current(&t, RAPTOR_ID), None, "{degrees}°: the Attack given up");
        assert_eq!(t.status(RAPTOR_ID.0), MoveStatus::Arrived, "{degrees}°");
        let (raptor, target) = (object(&t, RAPTOR_ID), object(&t, PK_ID));
        let off = math::wrap_angle(math::vector_angle(target.pos - raptor.pos) - raptor.facing);
        assert!(off.abs() > HALF_ARC + DEGREE, "{degrees}°: {}° off", off.to_degrees());
        assert!(
            raptor.pos.distance(target.pos) < 64.0,
            "{degrees}°: against it, at {}",
            raptor.pos
        );
        assert_eq!(target.hp, PK.def().max_hp, "{degrees}°");
    }
    for (degrees, far) in [(120.0, 100.0), (90.0, 128.0)] {
        let (mut t, _) = game(degrees, far);
        run_for(&mut t, 200);
        assert!(died(&t, PK_ID).is_some(), "{degrees}°, {far} su: killed");
        assert_eq!(launches(&t).len(), 4, "{degrees}°, {far} su");
    }
}

/// Question Q0184's second case, the Bike's Attack Ground at a spot out of its arc, which it circles forever: not the
/// Raptor's, whose HeadDisjointed missile can't target the ground (`systems/controls/commands.md > AttackGround`:
/// "Only available to units whose AttackType has CanTargetGround"; `combat::can_target_ground`). The gate refuses its
/// Attack Ground (`Reject::CannotTargetGround`), and one given anyway changes nothing.
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
