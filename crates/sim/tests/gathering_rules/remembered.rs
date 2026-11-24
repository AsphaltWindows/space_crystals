//! A Gather on a patch out of the player's sight, by what the player knows of it (design change 0139: remembered
//! objects can be targeted, Gather among the orders; "nothing about the real state leaks"), and the map's patches,
//! known from the start (design change 0159).

use sc_sim::orders::state::Behaviour;
use sc_sim::testing::dangling_references;
use sc_sim::{DebugCommand, MoveGoal, Reject, TargetView, VisibilityState};

use super::*;

/// How player 1 comes to know patch A, in [`far_field`].
#[derive(Copy, Clone, Debug, PartialEq)]
enum Known {
    /// Seen by a Drone that was there in tick 1 and removed in tick 2: a greyed patch, remembered since.
    Scouted,
    /// Never seen: the map's, known from the start (design change 0159).
    FromTheStart,
}

/// 30 × 10 open tiles. Patch A (1) on (24, 4); patch B (2) on (26, 4), two tiles east of it; player 1's node 3 on
/// (4, 6) and Drone 4 beside it, 19 tiles from A, with a Drone's sight of 5. With [`Known::Scouted`], a Drone of
/// player 1's, 5, stands by A in tick 1 and is removed in tick 2. Not started yet: at tick 0.
fn far_field(known: Known) -> TestSim {
    let mut t = flat(30, 10)
        .spawn(PATCH, None, tile(24, 4))
        .spawn(PATCH, None, tile(26, 4))
        .spawn(NODE, Some(P1), tile(4, 6))
        .spawn(DRONE, Some(P1), centre(4, 5))
        .start();
    if known == Known::Scouted {
        t.spawn_unit_at(1, DRONE, P1, TilePos::new(24, 6).center());
        t.debug_at(2, DebugCommand::Remove { id: id(5) });
    }
    t
}

/// Whether player 1 sees the tile of patch A.
fn sees_a(t: &TestSim) -> bool {
    t.sim().vision().state(P1, TilePos::new(24, 4)) == VisibilityState::Visible
}

/// The Gather of worker `n`: its spot, and whether its mover heads for an object.
fn gather_state(t: &TestSim, n: u32) -> (Option<Vec2>, bool) {
    let o = object(t, n);
    let spot = match o
        .orders
        .as_ref()
        .and_then(|orders| orders.current.as_ref())
        .map(|a| &a.state)
    {
        Some(Behaviour::Gather(gather)) => gather.spot,
        _ => None,
    };
    let to_object = matches!(o.mover.as_ref().map(|m| m.goal), Some(MoveGoal::Object { .. }));
    (spot, to_object)
}

/// The acceptance of slice M3-R for 0139 ("scout a field and leave, then right-click a greyed patch with Drones: they
/// walk there and mine"), and for 0159 a patch of the map the player has never seen, far off: the gate takes the
/// Gather, the Drone walks to where its owner knows the patch, without ever being told more (its move heads for a
/// point, never the patch itself, and the order notes the spot), and mines it once there.
#[test]
fn a_drone_walks_to_a_patch_out_of_sight_and_mines_it() {
    for known in [Known::Scouted, Known::FromTheStart] {
        let mut t = far_field(known);
        t.run_until(2);
        let record = t.sim().vision().player(P1).unwrap().record(id(1)).cloned();
        assert_eq!(
            record.map(|r| r.seen),
            Some(if known == Known::Scouted { 1 } else { 0 }),
            "{known:?}: the record"
        );
        assert!(
            matches!(t.sim().target_view(P1, id(1)), TargetView::Remembered { .. }),
            "{known:?}: greyed"
        );
        let cmd = order_of(P1, &[4], OrderKind::Gather(id(1)));
        assert_eq!(t.sim().validate(&cmd), Ok(()), "{known:?}");
        t.at(3, P1, cmd.command);
        t.run_until(3);
        let a = TilePos::new(24, 4).center();
        assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))), "{known:?}");
        assert_eq!(
            gather_state(&t, 4),
            (Some(a), false),
            "{known:?}: on its way, by the record"
        );
        let mut walked = false;
        while !sees_a(&t) {
            t.step();
            walked |= object(&t, 4).pos.x > TilePos::new(10, 0).center().x;
            assert_eq!(
                gather_state(&t, 4),
                (Some(a), false),
                "{known:?}: tick {}",
                t.sim().tick()
            );
            assert!(t.sim().tick() < 1000, "{known:?}: the Drone should see A by now");
        }
        assert!(walked, "{known:?}");
        run_to(&mut t, 200, "it mines A", |t| mines(t, 4, 1));
        assert_eq!(gather_state(&t, 4), (None, false), "{known:?}: in sight, by the patch");
        assert!(beside(&t, 4, 24, 4), "{known:?}");
    }
}

/// Design change 0139's Verification for a Gather ("Gather: the redirect rules"), and its rule that nothing about the
/// real state leaks: patch A goes in tick 2 out of player 1's sight, in one of two games. Until the Drone sees its
/// place, the Drone is the same in both games, tick by tick. Then, where A stands, it mines A; where it doesn't, it
/// goes on to where A was and looks round there, as at a patch that ran out (design change 0134): on to B.
#[test]
fn a_patch_gone_out_of_sight_changes_nothing_until_its_place_is_seen() {
    let mut standing = far_field(Known::FromTheStart);
    let mut gone = far_field(Known::FromTheStart);
    gone.debug_at(2, DebugCommand::Remove { id: id(1) });
    for t in [&mut standing, &mut gone] {
        t.run_until(2);
        assert!(matches!(t.sim().target_view(P1, id(1)), TargetView::Remembered { .. }));
        t.order_at(3, P1, &[id(4)], OrderKind::Gather(id(1)), false);
    }
    loop {
        standing.step();
        gone.step();
        let tick = gone.sim().tick();
        assert_eq!(object(&gone, 4), object(&standing, 4), "tick {tick}");
        assert!(tick < 1000, "the Drone should see A's place by now");
        if sees_a(&gone) {
            break;
        }
    }
    assert!(!exists(&gone, 1) && sees_a(&standing));
    assert_eq!(gone.sim().vision().player(P1).unwrap().record(id(1)), None, "seen gone");
    // The Gather goes on, to the spot (`orders::Order::outlives_target`).
    assert_eq!(order(&gone, 4), Some(OrderKind::Gather(id(1))));
    run_to(&mut standing, 200, "it mines A where A stands", |t| mines(t, 4, 1));
    run_to(&mut gone, 200, "it goes on to B where A is gone", |t| {
        order(t, 4) == Some(OrderKind::Gather(id(2)))
    });
    assert!(beside(&gone, 4, 24, 4), "it looked round where A was");
    run_to(&mut gone, 200, "it mines B", |t| mines(t, 4, 2));
    assert_eq!(dangling_references(gone.sim()), []);
}

/// Design change 0159 against 0131 ("A patch within 6 GU that the player has never seen is never chosen"): a patch
/// placed after the start out of the player's sight is unknown to it: the gate refuses a Gather on it as for any
/// object the player doesn't know (`Reject::NotVisible`), and a Drone looking round never picks it
/// (`knowledge::only_a_known_uncovered_patch_is_one_to_go_on_to`).
#[test]
fn a_patch_placed_later_out_of_sight_is_unknown() {
    let mut t = far_field(Known::FromTheStart);
    let at = tile(20, 8);
    t.debug_at(
        3,
        DebugCommand::Spawn {
            kind: PATCH,
            owner: None,
            at,
            orientation: Default::default(),
        },
    )
    .run_until(3);
    let placed = id(5);
    assert!(exists(&t, 5));
    assert_eq!(t.sim().vision().player(P1).unwrap().record(placed), None);
    assert_eq!(t.sim().target_view(P1, placed), TargetView::Gone);
    let cmd = order_of(P1, &[4], OrderKind::Gather(placed));
    assert_eq!(t.sim().validate(&cmd), Err(Reject::NotVisible(placed)));
}
