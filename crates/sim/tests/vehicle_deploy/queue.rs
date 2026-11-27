//! A queued deploy (`factions/gdo/units/artillery_rig.md > Modes`: "**Queueable**: Shift-Deploy waits until the
//! earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at
//! once.").

use super::*;

/// The brief's "move, then Shift-Deploy: it deploys where the move ends": the Rig drives 6 grid units east, an
/// Artillery Rig all the way; its deploy starts as the Move completes, where the Rig stopped, and 48 frames later it
/// is deployed there.
#[test]
fn a_shift_deploy_after_a_move_deploys_where_the_move_ends() {
    let mut t = field(RIG, |b| b);
    let goal = from_home(gu(6.0), 0.0);
    give(&mut t, &[UNIT], move_to(goal), false);
    give(&mut t, &[UNIT], deploy(), true);
    assert_eq!(queued(&t, UNIT), [deploy()]);
    let started = loop {
        assert!(t.sim().tick() < 400, "it never started its deploy");
        assert_eq!(kind_of(&t, UNIT), RIG);
        t.step();
        if current(&t, UNIT) == Some(deploy()) {
            break t.sim().tick();
        }
    };
    let stood = place(&t, UNIT);
    assert!(stood.0.distance(goal) < 8.0, "at the Move's end: {stood:?}");
    t.run_until(started + FRAMES - 1);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (RIG, stood));
    t.step();
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (DEPLOYED, stood));
    assert_eq!(kind_changes(&t), [(started + FRAMES, UNIT, DEPLOYED)]);
}

/// "Without Shift it replaces the current orders and starts at once": a Deploy given to a driving Rig, unshifted,
/// ends its Move.
#[test]
fn a_deploy_without_shift_replaces_the_move() {
    let mut t = field(RIG, |b| b);
    give(&mut t, &[UNIT], move_to(from_home(gu(10.0), 0.0)), false);
    t.run(20);
    let given = give(&mut t, &[UNIT], deploy(), false);
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (Some(deploy()), vec![]));
    t.run_until(given + FRAMES);
    assert_eq!(kind_of(&t, UNIT), DEPLOYED);
    assert!(t.pos(UNIT.0).x < from_home(gu(4.0), 0.0).x, "it stopped well short");
}
