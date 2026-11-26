//! Design change 0360 (`factions/syndicate/structures/laser_turret.md`): "it is always a hidden **pop-up turret**, and
//! is **only active inside its Tunnel's area**"; `> When its Tunnel is lost`: "The turret **behaves exactly like the
//! Tunnel's other expansions** ...: it becomes **neutral**, **visible to everyone**, keeps its **high underground
//! armor** (a raised turret retracts at once; it never fires again while neutral), and **slowly decays** (1% of MaxHP
//! per second, placeholder). A new Tunnel whose area covers it **reclaims** it like an expansion (Tier 1 suffices), and
//! it is active again." Active is derived, never stored (`Sim::is_active_turret`): a turret with a parent Tunnel, the
//! expansions' rule (`tunnel::area::parent`: an own Tunnel in operation whose current area fully contains it).

use sc_sim::defs::Armor;

use super::*;

/// The decay of a neutral turret from full HP, `ticks` ticks of it: its 300 HP over 1600 ticks, 1% of its MaxHP a
/// second (`sc_sim::tuning::tunnel::DECAY_PERCENT_PER_SECOND`), 3 HP a second.
fn decayed(ticks: u32) -> f32 {
    300.0 - 300.0 * ticks as f32 / 1600.0
}

/// Whether `hp` is `expected`, to a thousandth of an HP.
fn about(hp: f32, expected: f32) -> bool {
    (hp - expected).abs() < 1e-3
}

/// The turret's HP.
fn hp(t: &TestSim) -> f32 {
    object(t, POPUP).hp.expect("destructible")
}

/// Design change 0360's verification: "Destroy a Tunnel with a raised turret: it drops, appears to everyone as
/// neutral, stops firing, and loses about 3 HP/s. A new Tunnel covering it makes it yours and active again."
///
/// Player 1's turret, raised, has shot once at player 2's Guard when [`T1`] is destroyed (a debug removal; player 1's
/// second Tunnel, far east, keeps it in the game). In that tick the turret is down, at its underground armour, neutral
/// and walked over, hidden from nobody and seen by both players, who remember it neutral. It never fires again though
/// the Guard stays in its range, and loses 3 HP a second, the first tick of it in that tick. Player 1's Agent builds a
/// Tunnel where the lost one stood: as the site completes the turret is player 1's again, at the HP the decay left it,
/// hidden from player 2 again, and decays no more; it rises at the Guard and fires.
#[test]
fn its_tunnel_lost_it_drops_neutral_for_all_never_fires_and_decays_until_a_new_tunnel_takes_it_back() {
    let mut t = with_turret()
        .spawn(TUNNEL, Some(P1), tile(30, 8)) // 4
        .spawn(AGENT, Some(P1), SpawnAt::Point(at(6, 16).center())) // 5
        .start();
    let supplies = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    next_debug(&mut t, supplies);
    next_debug(&mut t, spawn(GUARD, P2, east(4.0)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run_until(20);
    assert_eq!(shots(&t, POPUP).len(), 1);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    assert!(blocked(&t));

    next_debug(&mut t, DebugCommand::Remove { id: T1 });
    let fell = t.sim().tick();
    let o = object(&t, POPUP);
    assert_eq!(o.owner, None, "neutral");
    assert_eq!(o.popup, Some(PopUp::LOWERED), "down at once");
    assert_eq!(
        o.armor(),
        Armor { point: 6.0, full: 6.0 },
        "its high underground armour"
    );
    assert!(
        o.attack.as_ref().is_some_and(|a| a.target.is_none()),
        "its target dropped"
    );
    assert!(!blocked(&t), "walked over");
    for player in [P1, P2] {
        assert!(!t.sim().hidden_from(o, player), "{player:?}: no longer hidden");
        assert!(t.sim().is_visible(player, o), "{player:?} sees it");
        let record = t.sim().vision().player(player).and_then(|v| v.record(POPUP));
        assert_eq!(record.map(|r| (r.kind, r.owner)), Some((TURRET, None)), "{player:?}");
    }
    assert!(about(hp(&t), decayed(1)), "{}", hp(&t));
    t.run_until(fell + 16);
    let a_second_in = hp(&t);
    assert!(about(a_second_in, decayed(17)), "{a_second_in}");
    t.run_until(fell + 32);
    assert!(about(a_second_in - hp(&t), 3.0), "3 HP a second");
    t.run_until(fell + 160);
    assert!(about(hp(&t), decayed(161)), "{}", hp(&t));
    assert_eq!(shots(&t, POPUP).len(), 1, "it never fired again");
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    assert!(t.sim().object(guard).is_some_and(|o| o.owner == Some(P2)));

    let orientation = BuildingOrientation::default();
    let build = OrderKind::Build {
        kind: TUNNEL,
        origin: at(4, 8),
        orientation,
    };
    next(&mut t, P1, order(ObjectId(5), build));
    let completed =
        |t: &TestSim| (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::SiteCompleted { site, .. } if site.0 > 6));
    while !completed(&t) {
        assert!(t.sim().tick() < fell + 1500, "the new Tunnel completes");
        assert_eq!(object(&t, POPUP).owner, None, "neutral while the Tunnel is a site");
        t.step();
    }
    let back = t.sim().tick();
    let o = object(&t, POPUP);
    assert_eq!(
        (o.owner, o.decay),
        (Some(P1), None),
        "player 1's again: it decays no more"
    );
    assert!(t.sim().is_active_turret(o));
    assert!(t.sim().hidden_from(o, P2), "hidden again, lowered");
    let left = hp(&t);
    assert!(
        about(left, decayed(back - fell)),
        "{left}: no decay in the tick it is taken back"
    );
    t.run(30);
    assert!(shots(&t, POPUP).iter().any(|&tick| tick > back), "it fires again");
    assert_eq!(hp(&t), left, "no more decay");
}

/// "**Only active inside its Tunnel's area**": player 2's turret inside player 1's area, and player 1's in no area
/// (debug placements: no game builds either), each with an enemy Drone in its range, stay down and never fire; neither
/// takes a cell of player 1's Tunnel Space: another player's turret never does, and one outside every area has no cell
/// to take.
#[test]
fn a_turret_outside_its_owners_tunnel_areas_stays_down_and_never_fires() {
    let scenario = base()
        .spawn(TURRET, Some(P2), SpawnAt::Tile(SPOT)) // 3: player 2's, in player 1's area
        .spawn(TURRET, Some(P1), tile(20, 10)) // 4: in no area
        .spawn(DRONE, Some(P1), SpawnAt::Point(east(2.0))) // 5: by turret 3
        .spawn(DRONE, Some(P2), SpawnAt::Point(Vec2::new(23.0 * 64.0, CENTRE.y))); // 6: by turret 4
    let mut t = scenario.start();
    let turrets = [ObjectId(3), ObjectId(4)];
    for _ in 0..100 {
        for id in turrets {
            let o = object(&t, id);
            assert!(!t.sim().is_active_turret(o), "{id:?}");
            assert_eq!(o.popup, Some(PopUp::LOWERED), "{id:?}");
        }
        t.step();
    }
    for id in turrets {
        assert_eq!(shots(&t, id), Vec::<Tick>::new(), "{id:?}");
    }
    assert_eq!(tunnel_space(&t, P1), (0, 11), "48 cells less the Optics Lab's 4");
}

/// An own Tunnel's area that comes to cover a turret makes it active, here a higher tier's (a debug one: an upgrade's
/// area is the new tier's once it completes, design change 0249). Player 1's turret on (11, 10) is outside [`T1`]'s
/// area at Tiers 1 and 2 (x up to 9 and 10) and inside it at Tier 3 (x up to 11), with player 2's Drone in its range:
/// inactive, it stays down and takes no cell; at Tier 3 it rises at the Drone in that tick and fires 13 ticks later,
/// and takes a cell of the region (128 less the Optics Lab's 4 and its own: 123 cells, 30).
#[test]
fn an_area_that_comes_to_cover_it_makes_it_active() {
    let mut t = base()
        .spawn(TURRET, Some(P1), tile(11, 10)) // 3
        .spawn(DRONE, Some(P2), SpawnAt::Point(at(14, 10).center())) // 4
        .start();
    t.run(100);
    assert!(!t.sim().is_active_turret(object(&t, POPUP)));
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    assert_eq!(tunnel_space(&t, P1), (0, 11));
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 2 });
    t.run(50);
    assert_eq!(
        popup(&t, POPUP),
        Some(PopUp::LOWERED),
        "Tier 2's area ends a tile short of it"
    );
    assert_eq!(tunnel_space(&t, P1), (0, 20), "84 cells less the Optics Lab's 4");
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new());
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 3 });
    let covered = t.sim().tick();
    assert!(t.sim().is_active_turret(object(&t, POPUP)));
    assert_eq!(
        popup(&t, POPUP),
        Some(PopUp::Rising(frames(0, 8))),
        "it rises at the Drone at once"
    );
    assert_eq!(tunnel_space(&t, P1), (0, 30), "123 cells: it occupies one");
    t.run_until(covered + 13);
    assert_eq!(shots(&t, POPUP), [covered + 13]);
}

/// Design change 0358's verification, its point 1 replaced by 0360's (a turret that lost its Tunnel no longer rises,
/// upright: it stays down, neutral): "Kill a Tunnel while an enemy tank sits on its lowered turret: the turret appears
/// to all, takes little damage, and rises once the tank leaves." Player 2's Guard holds over player 1's lowered turret,
/// which picks it and can't rise, hidden from player 2. [`T1`] destroyed (player 1's second Tunnel, far east, keeps it
/// in the game), the turret is seen by both players in that tick; the Guard's explicit Attack takes 12 HP of it a hit,
/// its 18 less the underground PointArmor 6 (raised, its PointArmor 1: 17); and once the Guard walks off, the turret,
/// neutral, stays down and never fires at it.
#[test]
fn its_tunnel_lost_under_an_enemy_it_shows_takes_little_damage_and_stays_down_once_clear() {
    let mut t = lowered(with_turret().spawn(TUNNEL, Some(P1), tile(30, 8))); // 4
    next_debug(&mut t, spawn(GUARD, P2, CENTRE));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run(10);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
    assert!(!t.sim().is_visible(P2, object(&t, POPUP)));

    next_debug(&mut t, DebugCommand::Remove { id: T1 });
    let o = object(&t, POPUP);
    assert_eq!((o.owner, o.popup), (None, Some(PopUp::LOWERED)));
    for player in [P1, P2] {
        assert!(t.sim().is_visible(player, o), "{player:?} sees it at once");
    }
    let attack = order(guard, OrderKind::Attack(POPUP));
    assert_eq!(gate(&t, P2, attack.clone()), Ok(()));
    next(&mut t, P2, attack);
    let hits = |t: &TestSim| -> Vec<f32> {
        (t.events().iter())
            .filter_map(|(_, e)| match *e {
                SimEvent::Hit {
                    attacker,
                    target,
                    damage,
                    ..
                } if attacker == guard && target == POPUP => Some(damage),
                _ => None,
            })
            .collect()
    };
    let start = t.sim().tick();
    while hits(&t).is_empty() {
        assert!(t.sim().tick() < start + 64, "the Guard hits it");
        t.step();
    }
    assert_eq!(hits(&t), [18.0 - 6.0], "little damage: its underground armour");
    assert_eq!(object(&t, POPUP).armor(), Armor { point: 6.0, full: 6.0 });

    let off = OrderKind::Move(Target::Point(CENTRE + Vec2::new(160.0, 0.0)));
    next(&mut t, P2, order(guard, off));
    while over_the_turret(&t, guard) {
        t.step();
    }
    let clear = t.sim().tick();
    t.run(100);
    assert!(t.sim().tick() >= clear + 100);
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED), "neutral, it stays down");
    assert_eq!(shots(&t, POPUP), Vec::<Tick>::new(), "and never fires");
    assert!(
        object(&t, guard).pos.distance(CENTRE) < 5.0 * 64.0,
        "the Guard in its range"
    );
}
