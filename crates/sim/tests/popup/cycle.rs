//! The pop-up's cycle (`factions/syndicate/structures/laser_turret.md`'s placeholder stats: "Pop-up: rises in 8 frames
//! (already facing its target); retracts after 32 frames with no target, taking 8 frames. It can't fire while rising
//! or retracting"; `> Upright vs Pop-up`: "While it has no target it stays **underground** ... When an enemy comes into
//! range it rises to the surface **already facing the target** and fires; it retracts once it has no target").

use super::*;

/// A turret that a game starts with inside an own Tunnel Area stands raised as a pop-up, with nothing to shoot at: its
/// idle frames count from tick 1, it starts to retract in tick 32 and is down at the end of tick 40. From then on it
/// stays underground.
#[test]
fn with_no_target_it_goes_underground_and_stays_there() {
    let mut t = with_turret().start();
    assert_eq!(popup(&t, POPUP), None, "as the game starts");
    assert!(t.sim().is_popup(object(&t, POPUP)), "its form is derived from the area");
    t.run_until(31);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Raised { idle: frames(31, 32) }));
    t.step();
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(0, 8))));
    t.run_until(39);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(7, 8))));
    t.step();
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    t.run(200);
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new());
}

/// Plan §3.3 S2's acceptance: "It rises 8 ticks after an enemy enters range 5, and fires. It retracts 32 ticks after
/// its last target, over 8 ticks, firing at nothing meanwhile."
///
/// Player 2's Guard appears 4 grid units from the lowered turret in tick 41 (`g`). The turret, which sees 6 grid units
/// while lowered (design change 0296 point 2), picks it in that tick and starts to rise, facing it; it is raised at the
/// end of `g + 8`, aims in the four ticks from `g + 9` and fires in `g + 13`, then every 24 ticks. The Guard (80 HP)
/// dies of its fourth shot (24 − 1 each), in `g + 85`: the turret's last tick with a target was the one before. It
/// starts to retract 32 ticks later and is down after 8 more, `g + 124`, having fired at nothing since.
#[test]
fn it_rises_in_8_ticks_at_an_enemy_in_range_fires_and_retracts_32_ticks_after_its_last_target() {
    let mut t = lowered(with_turret());
    // Due south of it, 4 grid units from its tile, edge to edge.
    next_debug(&mut t, spawn(GUARD, P2, CENTRE + Vec2::new(0.0, 32.0 + 256.0 + 18.0)));
    let (g, guard) = (t.sim().tick(), newest(&t));
    assert_eq!(g, 41);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))), "it starts at once");
    let south = std::f32::consts::FRAC_PI_2;
    assert!(
        (object(&t, POPUP).facing - south).abs() < 1e-6,
        "facing its target from the first frame"
    );
    next(&mut t, P2, order(guard, OrderKind::Hold));
    for frame in 1..8 {
        assert_eq!(
            popup(&t, POPUP),
            Some(PopUp::Rising(frames(frame, 8))),
            "tick {}",
            g + frame
        );
        t.step();
    }
    assert_eq!(t.sim().tick(), g + 8);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Raised { idle: frames(0, 32) }));
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new(), "it can't fire while rising");
    t.run_until(g + 12);
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new(), "Aim 4");
    t.step();
    assert_eq!(shots(&t, POPUP), [g + 13]);
    let dies = g + 13 + 3 * 24;
    t.run_until(dies - 1);
    assert!(t.sim().object(guard).is_some());
    t.step();
    assert!(t.sim().object(guard).is_none(), "four shots kill a Guard");
    let fired = [g + 13, g + 37, g + 61, dies];
    assert_eq!(shots(&t, POPUP), fired);
    // The last tick it had a target at the end of was `dies - 1`: 32 idle ticks, the first of them `dies`.
    t.run_until(dies + 30);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Raised { idle: frames(31, 32) }));
    t.step();
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(0, 8))));
    t.run_until(dies + 38);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(7, 8))));
    t.step();
    assert_eq!(t.sim().tick(), dies - 1 + 40);
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    assert_eq!(shots(&t, POPUP), fired, "it fired at nothing meanwhile");
}

/// "It can't fire while rising or retracting", and each phase runs its full length (the simplest predictable rule,
/// design change 0223; the design doesn't say what a retracting turret does about a new target): an enemy that comes
/// in range while the turret retracts doesn't turn it back. It goes all the way down, starts to rise in the tick it is
/// down, never lowered in between, so its footprint stays blocked, and fires 13 ticks after that.
#[test]
fn a_target_seen_while_it_retracts_waits_for_it_to_go_down_and_come_up_again() {
    let mut t = with_turret().start();
    t.run_until(35);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(3, 8))));
    next_debug(&mut t, spawn(GUARD, P2, east(4.0)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run_until(39);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Retracting(frames(7, 8))));
    assert!(
        object(&t, POPUP)
            .attack
            .as_ref()
            .is_some_and(|a| a.target == Some(guard))
    );
    t.step();
    assert_eq!(
        popup(&t, POPUP),
        Some(PopUp::Rising(frames(0, 8))),
        "down and up again in tick 40"
    );
    assert!(blocked(&t), "its footprint blocks all along");
    t.run_until(52);
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new());
    t.step();
    assert_eq!(shots(&t, POPUP), [40 + 13]);
}
