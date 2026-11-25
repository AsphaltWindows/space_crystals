//! `factions/syndicate/structures/laser_turret.md > Conversion`: "The form updates automatically whenever Tunnel Areas
//! change": the form is derived from the areas (plan decision T50), so an upgrade, a new Tunnel and a Tunnel's loss
//! convert a turret with no event.

use super::*;

/// A turret on (11, 10): outside [`T1`]'s area at Tiers 1 and 2 (x up to 9 and 10), inside it at Tier 3 (x up to 11).
fn outside() -> ScenarioBuilder {
    base().spawn(TURRET, Some(P1), tile(11, 10))
}

/// "A new Tunnel, or a Tunnel upgrade, whose Tunnel Area now covers an upright turret converts it into a pop-up turret
/// (it starts occupying that Tunnel's tunnel-region)". Upright outside the area, the turret has no pop-up state, blocks
/// and costs no Tunnel Space. At Tier 3 the area covers it: it is a pop-up from that tick, standing raised, goes down
/// 40 ticks later, and takes a cell of the region (128 less the Optics Lab's 4 and its own: 123 cells, 30).
#[test]
fn an_upgrade_that_covers_an_upright_turret_makes_it_a_popup() {
    let mut t = outside().start();
    let grid_blocked = |t: &TestSim| t.sim().nav().grids()[0].is_blocked(at(11, 10));
    t.run(100);
    assert_eq!(popup(&t, POPUP), None);
    assert!(!t.sim().is_popup(object(&t, POPUP)) && grid_blocked(&t));
    assert_eq!(tunnel_space(&t, P1), (0, 11));
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 2 });
    t.run(50);
    assert_eq!(popup(&t, POPUP), None, "Tier 2's area ends a tile short of it");
    assert_eq!(tunnel_space(&t, P1), (0, 20), "84 cells less the Optics Lab's 4");
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 3 });
    let covered = t.sim().tick();
    assert!(t.sim().is_popup(object(&t, POPUP)));
    assert_eq!(popup(&t, POPUP), Some(PopUp::Raised { idle: frames(1, 32) }));
    assert_eq!(tunnel_space(&t, P1), (0, 30), "123 cells: it occupies one");
    t.run_until(covered + 39);
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    assert!(!grid_blocked(&t));
}

/// "When a Tunnel is destroyed, the pop-up turrets in its Tunnel Area become upright turrets". A raised one is upright
/// at once. A lowered one comes up as a pop-up does, in 8 frames, with no enemy near: it is upright by its form from
/// the tick the Tunnel goes, and by its state once it stands. Its owner still owns it, and isn't defeated.
#[test]
fn a_popup_whose_tunnel_is_lost_stands_upright() {
    // Lowered.
    let mut t = lowered(with_turret());
    next_debug(&mut t, DebugCommand::Remove { id: T1 });
    assert!(!t.sim().is_popup(object(&t, POPUP)));
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))));
    assert!(blocked(&t));
    t.run(8);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    t.step();
    assert_eq!(popup(&t, POPUP), None, "an upright turret");
    t.run(100);
    assert_eq!(popup(&t, POPUP), None, "and it stays up");
    assert!(blocked(&t));
    assert_eq!(object(&t, POPUP).owner, Some(P1));
    assert!(t.sim().outcome().is_none(), "a turret is a structure: not defeated");
    // Raised.
    let mut t = with_turret().start();
    t.run(10);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    next_debug(&mut t, DebugCommand::Remove { id: T1 });
    assert_eq!(popup(&t, POPUP), None);
    assert!(blocked(&t));
}

/// The same with a unit over the lowered turret (a consequence of design change 0296 point 3, "it can't rise while any
/// unit stands over it"): it stays down, hidden and armoured as lowered, until the unit leaves; nothing ever stands on
/// a blocked tile.
#[test]
fn a_lowered_turret_whose_tunnel_is_lost_waits_for_a_clear_footprint() {
    let mut t = with_turret().start();
    t.without_auto_targeting();
    t.run_until(DOWN);
    next_debug(&mut t, spawn(GUARD, P2, CENTRE));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    next_debug(&mut t, DebugCommand::Remove { id: T1 });
    for _ in 0..50 {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert!(!blocked(&t) && !t.sim().is_visible(P2, object(&t, POPUP)));
        t.step();
    }
    next(
        &mut t,
        P2,
        order(guard, OrderKind::Move(Target::Point(CENTRE + Vec2::new(160.0, 0.0)))),
    );
    while over_the_turret(&t, guard) {
        t.step();
    }
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))));
    t.run(9);
    assert_eq!(popup(&t, POPUP), None);
}

/// "Only an operational Tunnel's area makes a pop-up" (a reading, as for reclaiming, design change 0258): a turret
/// inside the area of another player's Tunnel, or of none, is upright.
#[test]
fn only_an_own_tunnels_area_makes_a_popup() {
    let scenario = base()
        .spawn(TURRET, Some(P2), SpawnAt::Tile(SPOT)) // 3: player 2's, in player 1's area
        .spawn(TURRET, Some(P1), tile(20, 10)); // 4: in no area
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.run(100);
    for id in [ObjectId(3), ObjectId(4)] {
        assert_eq!(popup(&t, id), None);
        assert!(!t.sim().is_popup(object(&t, id)));
    }
    assert_eq!(tunnel_space(&t, P1), (0, 11), "another player's turret takes no cell");
}
