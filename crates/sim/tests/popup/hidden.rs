//! What the other player sees of a pop-up turret, and its armour (design change 0296 points 1 and 2;
//! `factions/syndicate/structures/laser_turret.md > Pop-up rules`: "**Hidden only while fully lowered**: that is when
//! it is invisible to enemies without detection and has its underground armor. While rising, raised and retracting it
//! is visible and has its raised armor").

use sc_sim::defs::Armor;
use sc_sim::vision::VisibilityState;
use sc_sim::{PlacementProblem, TileProblem};

use super::*;

/// Plan §3.3 S2's acceptance: "Lowered, enemies don't see it, can't target it, and their placement over it fails as
/// over an expansion."
///
/// Player 2's Guard stands by the lowered turret, the tile under it Visible to player 2, with auto-targeting off, so
/// that the turret has no target. Player 2 doesn't see it or record it, and can't name it in an order; a placement
/// over it, judged by the truth, is refused as over an expansion (design change 0240; no worker of player 2's builds
/// anything a ghost could show there: only a Tunnel builds a turret, design change 0359, and inside its own area,
/// which never overlaps another's). With auto-targeting on again the turret rises at the Guard: player 2 sees it from
/// that tick, records it, and may attack it; lowered again, it is hidden again and the record goes.
///
/// (A lowered turret's death and the blasts over it are events the client must keep from the other players: slice
/// M4-V's per-viewer filter.)
#[test]
fn lowered_the_enemy_doesnt_see_it_and_rising_it_does() {
    let mut t = with_turret().start();
    t.without_auto_targeting();
    next_debug(&mut t, spawn(GUARD, P2, east(0.5)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run(sc_sim::tuning::vision::FOG_DELAY_TICKS + 1);
    let attack = || order(guard, OrderKind::Attack(POPUP));
    let record = |t: &TestSim| {
        t.sim()
            .vision()
            .player(P2)
            .expect("in the game")
            .record(POPUP)
            .is_some()
    };
    for _ in 0..20 {
        let o = object(&t, POPUP);
        assert_eq!(o.popup, Some(PopUp::LOWERED));
        assert!(t.sim().hidden_from(o, P2) && !t.sim().hidden_from(o, P1));
        assert_eq!(t.sim().visibility(P2, SPOT), VisibilityState::Visible);
        assert!(!t.sim().is_visible(P2, o) && t.sim().is_visible(P1, o));
        assert!(!record(&t));
        assert_eq!(gate(&t, P2, attack()), Err(Reject::NotVisible(POPUP)));
        t.step();
    }
    // A structure of player 2's over it, by the truth.
    let orientation = BuildingOrientation::default();
    let truth = t.sim().placement_check(P2, TURRET, SPOT, orientation);
    assert_eq!(
        truth.first_problem(),
        Some(PlacementProblem::Tile(SPOT, TileProblem::Structure)),
        "as over an expansion"
    );
    // It rises at the Guard: seen from the tick it starts.
    next_debug(&mut t, DebugCommand::SetAutoTargeting { on: true });
    next(&mut t, P2, order(guard, OrderKind::Hold));
    let o = object(&t, POPUP);
    assert!(matches!(o.popup, Some(PopUp::Rising(_))), "{:?}", o.popup);
    assert!(!t.sim().hidden_from(o, P2) && t.sim().is_visible(P2, o));
    assert!(record(&t), "remembered like any structure seen");
    assert_eq!(gate(&t, P2, attack()), Ok(()));
    // With auto-targeting off again it lets its target go, stands its 32 frames and goes down: hidden again, and
    // player 2, which still sees the tile, drops its record.
    next_debug(&mut t, DebugCommand::SetAutoTargeting { on: false });
    t.run(8 + 32 + 8);
    let o = object(&t, POPUP);
    assert_eq!(o.popup, Some(PopUp::LOWERED));
    assert_eq!(t.sim().visibility(P2, SPOT), VisibilityState::Visible);
    assert!(t.sim().hidden_from(o, P2) && !t.sim().is_visible(P2, o));
    assert!(!record(&t), "a hidden object is never remembered");
}

/// `factions/syndicate/structures/laser_turret.md`: "Armor (building armor rule, 1x1): raised PointArmor 1, FullArmor
/// 1; underground (pop-up) PointArmor 6, FullArmor 6", the underground one "only while fully lowered" (design change
/// 0296 point 1): the turret's armour through a whole cycle, from the underground one it starts with. Player 2's Guard
/// brings it up, and is taken off the map (a debug removal) once it stands: it is raised, then retracts, at its raised
/// armour, until it is down again. (What a blast takes of each is `combat/area/tests.rs`'s.)
#[test]
fn its_armour_is_the_underground_one_only_while_fully_lowered() {
    let (raised, underground) = (Armor { point: 1.0, full: 1.0 }, Armor { point: 6.0, full: 6.0 });
    let mut t = with_turret().start();
    for tick in 0..10 {
        assert_eq!(
            object(&t, POPUP).armor(),
            underground,
            "tick {tick}: lowered from the start"
        );
        t.step();
    }
    next_debug(&mut t, spawn(GUARD, P2, east(4.0)));
    let guard = newest(&t);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Rising(_))));
    assert_eq!(object(&t, POPUP).armor(), raised, "from the first rising frame");
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run(7);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    next_debug(&mut t, DebugCommand::Remove { id: guard });
    let mut retracted = false;
    while popup(&t, POPUP) != Some(PopUp::LOWERED) {
        let o = object(&t, POPUP);
        assert_eq!(o.armor(), raised, "tick {}: {:?}", t.sim().tick(), o.popup);
        retracted |= matches!(o.popup, Some(PopUp::Retracting(_)));
        t.step();
    }
    assert!(retracted);
    assert_eq!(object(&t, POPUP).armor(), underground, "down again");
}

/// Design change 0296 point 2: "**Sight 6 grid units in both forms**: a lowered turret notices enemies by itself and
/// rises": lowered, it sees a tile 6 grid units away and none 7 away, with nothing else of its owner's near.
#[test]
fn lowered_it_sees_six_grid_units() {
    let t = lowered(with_turret());
    let see = |x: i32| t.sim().visibility(P1, at(x, SPOT.y));
    assert_eq!(see(SPOT.x + 6), VisibilityState::Visible);
    assert_ne!(see(SPOT.x + 7), VisibilityState::Visible);
}
