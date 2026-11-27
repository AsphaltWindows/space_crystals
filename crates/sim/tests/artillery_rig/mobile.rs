//! The mobile Rig (`factions/gdo/units/artillery_rig.md > Modes`: "**Mobile:** moves normally; **has no attack**").

use super::*;

/// Acceptance, "The mobile Rig never fires": with auto-targeting on and player 2's Peacekeeper 4 grid units off firing
/// at it, it attacks nothing for 200 frames; the gate refuses it an Attack (`Reject::CannotAttack`) and an Attack
/// Ground (`Reject::CannotTargetGround`). It drives.
#[test]
fn the_mobile_rig_never_fires() {
    let pk = id(2);
    let mut t = field(RIG, |b| {
        b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0, half(PK), 0.0)))
    });
    auto_targeting(&mut t);
    t.run(200);
    assert!(!ever_attacked(&t, UNIT), "it never fired");
    assert!(ever_attacked(&t, pk), "the Peacekeeper fired at it");
    assert_eq!(
        gate(&t, P1, order(&[UNIT], OrderKind::Attack(pk))),
        Err(Reject::CannotAttack)
    );
    let ground = OrderKind::AttackGround(from_home(gu(5.0), 0.0));
    assert_eq!(gate(&t, P1, order(&[UNIT], ground)), Err(Reject::CannotTargetGround));
    let goal = from_home(0.0, gu(5.0));
    give(&mut t, P1, order(&[UNIT], OrderKind::Move(Target::Point(goal))));
    t.run(200);
    assert!(object(&t, UNIT).pos.distance(goal) < 8.0, "it drives");
}
