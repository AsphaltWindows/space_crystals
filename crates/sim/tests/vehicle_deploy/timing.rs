//! How long a deploy takes, and what the unit does meanwhile (`factions/gdo/units/artillery_rig.md > Modes`: "the unit
//! can neither move nor attack while switching"; "it stays the mobile type until the process completes").

use super::*;

/// Acceptance, "It takes 48 ticks, during which the Rig neither moves nor attacks [...] It stays an Artillery Rig
/// until the last tick, then becomes the Deployed kind": a Deploy given in tick `t` starts at once, none of its 48
/// frames done, and takes one a tick: the Rig is an Artillery Rig through tick `t + 47`, 47 frames done; in tick
/// `t + 48` it is an Artillery Rig (Deployed), with its `SimEvent::KindChanged`, idle, its deploy done. It stands on its
/// spot, facing as it did, throughout.
#[test]
fn a_deploy_takes_48_frames_and_the_rig_stays_mobile_until_the_last() {
    let mut t = field(RIG, |b| b);
    let at = place(&t, UNIT);
    let given = give(&mut t, &[UNIT], deploy(), false);
    for done in 0..FRAMES {
        let tick = t.sim().tick();
        assert_eq!(tick, given + done);
        assert_eq!(kind_of(&t, UNIT), RIG, "tick {tick}");
        assert_eq!(current(&t, UNIT), Some(deploy()), "tick {tick}");
        assert_eq!(progress(&t, UNIT), Some((DEPLOYED, done, FRAMES)), "tick {tick}");
        assert_eq!(place(&t, UNIT), at, "tick {tick}: it stands");
        t.step();
    }
    assert_eq!(t.sim().tick(), given + FRAMES);
    assert_eq!(kind_of(&t, UNIT), DEPLOYED);
    assert_eq!(kind_changes(&t), [(given + FRAMES, UNIT, DEPLOYED)]);
    assert_eq!((progress(&t, UNIT), current(&t, UNIT)), (None, None));
    assert!(queued(&t, UNIT).is_empty() && dropped(&t).is_empty());
    assert_eq!(place(&t, UNIT), at);
    t.run(100);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (DEPLOYED, at), "and stays so");
}

/// A Rig driving when its Deploy comes brakes at once, as at any order that stops it (`Mover::stop`: its Deceleration
/// of 1 su a frame, from 4 su a frame), and then stands while its frames run; it deploys where it stopped, on time.
#[test]
fn a_driving_rig_stops_to_deploy() {
    let mut t = field(RIG, |b| b);
    give(&mut t, &[UNIT], move_to(from_home(gu(10.0), 0.0)), false);
    t.run(40);
    assert!(t.mover(UNIT.0).momentum.length() > 3.9, "at full speed");
    let given = give(&mut t, &[UNIT], deploy(), false);
    t.run(4);
    let stopped = place(&t, UNIT);
    assert!(t.mover(UNIT.0).momentum.length() < 1e-3, "stopped in four frames");
    t.run_until(given + FRAMES - 1);
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (RIG, stopped));
    t.step();
    assert_eq!((kind_of(&t, UNIT), place(&t, UNIT)), (DEPLOYED, stopped));
}

/// Acceptance, "Stop doesn't interrupt it" (design change 0213: "once deploying has started it always runs to
/// completion"): a Stop halfway is held like any order, and the deploy completes on time; the Stop then leaves the
/// deployed Rig idle, nothing dropped.
#[test]
fn stop_doesnt_interrupt_a_deploy() {
    let mut t = field(RIG, |b| b);
    let given = give(&mut t, &[UNIT], deploy(), false);
    t.run(20);
    give(&mut t, &[UNIT], OrderKind::Stop, false);
    assert_eq!(current(&t, UNIT), Some(deploy()));
    assert_eq!(queued(&t, UNIT), [OrderKind::Stop]);
    assert_eq!(progress(&t, UNIT), Some((DEPLOYED, 21, FRAMES)), "it goes on");
    t.run_until(given + FRAMES);
    assert_eq!(kind_changes(&t), [(given + FRAMES, UNIT, DEPLOYED)]);
    assert_eq!((current(&t, UNIT), queued(&t, UNIT)), (None, vec![]));
    assert!(dropped(&t).is_empty());
}

/// Acceptance, "Undeploying is symmetric": the deployed Rig's Undeploy takes 48 frames, a deployed Rig until the last,
/// and Stop doesn't interrupt it either; then it is an Artillery Rig, which drives again.
#[test]
fn undeploying_takes_48_frames_too() {
    let mut t = deployed(|b| b);
    let at = place(&t, UNIT);
    let given = give(&mut t, &[UNIT], undeploy(), false);
    give(&mut t, &[UNIT], OrderKind::Stop, false);
    t.run_until(given + FRAMES - 1);
    assert_eq!(kind_of(&t, UNIT), DEPLOYED);
    assert_eq!(progress(&t, UNIT), Some((RIG, FRAMES - 1, FRAMES)));
    assert_eq!(place(&t, UNIT), at);
    t.step();
    assert_eq!(kind_of(&t, UNIT), RIG);
    assert_eq!(kind_changes(&t).last(), Some(&(given + FRAMES, UNIT, RIG)));
    let goal = from_home(gu(4.0), 0.0);
    give(&mut t, &[UNIT], move_to(goal), false);
    t.run(200);
    assert!(t.pos(UNIT.0).distance(goal) < 8.0, "it drives: {}", t.pos(UNIT.0));
}

/// "The unit can neither move nor attack while switching": a deployed Rig that shells a Peacekeeper by itself
/// (auto-targeting on once it has deployed), told to undeploy halfway through the aim of its next shell, fires no
/// more: its aim is given up, and its turret, with an order that names no target, scans for nothing while it undeploys,
/// the Peacekeeper still in range and firing at it. Undeployed, it has no attack.
#[test]
fn a_rig_that_undeploys_fires_no_new_shell() {
    let mut t = deployed(|b| b.spawn(PK, Some(P2), SpawnAt::Point(east_of_rig(4.0))));
    let on = t.sim().tick() + 1;
    t.debug_at(on, DebugCommand::SetAutoTargeting { on: true });
    t.run(60);
    let first = blasts_by(&t, UNIT);
    assert_eq!(first.len(), 1, "one shell so far: {first:?}");
    // Its next aim, of 16 frames, starts after its Cooldown of 8 and its Reload of 64.
    t.run_until(first[0] + 8 + 64 + 8);
    let given = give(&mut t, &[UNIT], undeploy(), false);
    t.run_until(given + FRAMES);
    assert_eq!(kind_of(&t, UNIT), RIG);
    assert!(object(&t, UNIT).attack.is_none(), "mobile, no attack");
    t.run(100);
    assert_eq!(blasts_by(&t, UNIT), first, "no shell after the first");
    assert!(t.sim().object(id(2)).is_some(), "the Peacekeeper lives, in range");
}
