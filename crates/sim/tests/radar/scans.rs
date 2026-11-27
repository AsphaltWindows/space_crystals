//! What a Scan shows (`factions/gdo/structures/radar_station.md > Placeholder stats`: "**Scan** (hotkey **Q**,
//! targeted: left-click any point on the map): reveals a circle of radius **6 grid units** for **8 seconds** (128
//! frames)"; `sc_sim::vision::scans`), where it can be cast, and what it doesn't show: what is hidden (plan D39's
//! placeholder: "it shows hidden objects no more than other sight does").

use sc_sim::OrderKind;

use super::*;

/// The plan's acceptance, "at an unexplored point, its tiles are Visible for exactly 128 ticks, then fall to Explored
/// after the fog delay": station 3 scans the far tile's centre in tick 2. At the end of tick 1 player 1 had never seen
/// the disc; at the end of each tick from 2 to 129, the 128 of the scan, it sees every tile of the disc of 6 grid units
/// round the point and nothing more than it saw before, the scan listed in its vision; from tick 130 the scan is over
/// and the tiles linger for the fog delay (`core/vision.md > Visible > Fog delay`), then turn Explored at once, for
/// good.
#[test]
fn a_scan_shows_the_disc_round_its_point_for_128_ticks_then_the_fog_delay() {
    use VisibilityState::{Explored, Unexplored, Visible};
    let mut t = started(base());
    let tiles = disc(FAR_TILE);
    assert_eq!(tiles.len(), 113, "the lattice points of a circle of radius 6");
    assert!(all_seen(&t, P1, &tiles, Unexplored, false));
    let before = visible(&t, P1);
    assert!(before.iter().all(|tile| !tiles.contains(tile)));
    let cast = next(&mut t, scan(&[S1], FAR));
    assert_eq!(cast, 2);
    assert_eq!(casts(&t), [(cast, S1, FAR)]);
    let expected = Scan {
        at: FAR,
        radius: 6.0,
        until: cast + SCAN_TICKS,
    };
    let mut seen_with = before.clone();
    seen_with.extend(&tiles);
    seen_with.sort_by_key(|tile| (tile.y, tile.x));
    for tick in cast..cast + SCAN_TICKS {
        t.run_until(tick);
        assert!(all_seen(&t, P1, &tiles, Visible, false), "tick {tick}");
        assert_eq!(visible(&t, P1), seen_with, "tick {tick}: the disc, nothing more");
        assert_eq!(scans(&t, P1), [expected], "tick {tick}");
        assert_eq!(scans(&t, P2), []);
    }
    for tick in cast + SCAN_TICKS..cast + SCAN_TICKS + FOG_DELAY_TICKS {
        t.run_until(tick);
        assert!(all_seen(&t, P1, &tiles, Visible, true), "tick {tick}: lingering");
        assert_eq!(scans(&t, P1), [], "tick {tick}: over");
    }
    t.run_until(cast + SCAN_TICKS + FOG_DELAY_TICKS);
    assert!(all_seen(&t, P1, &tiles, Explored, false));
    assert_eq!(visible(&t, P1), before, "only the buildings' sight");
    t.run(100);
    assert!(all_seen(&t, P1, &tiles, Explored, false), "for good");
    assert!(
        all_seen(&t, P2, &tiles, Unexplored, false),
        "player 2 saw nothing of it"
    );
}

/// "targeted: left-click any point on the map" (`AbilityArea::AnyPoint`): a Scan is taken at a point player 1 has never
/// seen, and at one in sight; the gate refuses a point off the map or not finite (`Reject::OffMap`), a Deployment
/// Center, which casts nothing (`Reject::LacksAbility`), a Radar Station asked for the Shrine's ability, and another
/// player's station (`Reject::NotOwned`).
#[test]
fn a_scan_is_cast_anywhere_on_the_map_and_nowhere_off_it() {
    let mut t = started(base().spawn(RADAR, Some(P2), tile(40, 28)));
    let corner = Vec2::new(48.0 * 64.0 - 1.0, 32.0 * 64.0 - 1.0);
    assert_eq!(
        t.sim().visibility(P1, TilePos::new(47, 31)),
        VisibilityState::Unexplored
    );
    for at in [FAR, corner, Vec2::ZERO, Vec2::new(500.0, 600.0)] {
        assert_eq!(gate(&t, P1, scan(&[S1], at)), Ok(()), "{at}");
    }
    let off = [
        Vec2::new(-1.0, 600.0),
        Vec2::new(600.0, 32.0 * 64.0 + 1.0),
        Vec2::new(f32::NAN, 600.0),
        Vec2::new(f32::INFINITY, 600.0),
    ];
    for at in off {
        assert_eq!(gate(&t, P1, scan(&[S1], at)), Err(Reject::OffMap), "{at}");
    }
    assert_eq!(gate(&t, P1, scan(&[DC], FAR)), Err(Reject::LacksAbility));
    let heal = Command::Cast {
        casters: vec![id(S1)],
        ability: AreaAbility::HealthRegeneration,
        at: FAR,
    };
    assert_eq!(gate(&t, P1, heal), Err(Reject::LacksAbility));
    assert_eq!(gate(&t, P1, scan(&[5], FAR)), Err(Reject::NotOwned(id(5))));
    next(&mut t, scan(&[S1], corner));
    assert!(t.sim().visibility(P1, TilePos::new(47, 31)) == VisibilityState::Visible);
    assert_eq!(casts(&t), [(2, S1, corner)]);
}

/// Plan D39's placeholder, "it shows hidden objects no more than other sight does" (`core/vision.md`; `crate::vision::
/// hidden`, `factions/syndicate/structures/README.md > Tunnel Expansions`: "Expansions are invisible to enemies without
/// detection"): player 2, the Syndicate, has a Tunnel, a Hideout in its area and an Agent, all in the circle of a Scan
/// at the far point. Before it player 1 sees none of them; under it player 1 sees the Tunnel and the Agent, and the
/// Hideout's tiles but not the Hideout, which it doesn't remember after either, and can't order an attack on. After the
/// scan player 1 remembers the Tunnel.
#[test]
fn in_the_circle_the_enemy_is_seen_and_an_underground_expansion_stays_hidden() {
    let scenario = base()
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P2), tile(34, 16)) // 5: (34, 16) to (37, 19), its area (32, 14) to (39, 21)
        .spawn(ObjectKind::Hideout, Some(P2), tile(38, 20)) // 6: (38, 20) to (39, 21)
        .spawn(
            ObjectKind::Agent,
            Some(P2),
            SpawnAt::Point(Vec2::new(33.0 * 64.0, 21.0 * 64.0)),
        ) // 7
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            SpawnAt::Point(Vec2::new(600.0, 900.0)),
        ); // 8
    let mut t = started(scenario);
    let (tunnel, hideout, agent) = (id(5), id(6), id(7));
    let object = |t: &TestSim, id: ObjectId| t.sim().object(id).cloned().expect("on the map");
    for n in [tunnel, hideout, agent] {
        assert!(!t.sim().is_visible(P1, &object(&t, n)), "{n:?} unseen at first");
    }
    let cast = next(&mut t, scan(&[S1], FAR));
    for tick in [cast, cast + 60, cast + SCAN_TICKS - 1] {
        t.run_until(tick);
        let vision = t.sim().vision().player(P1).unwrap();
        assert!(t.sim().is_visible(P1, &object(&t, tunnel)), "tick {tick}");
        assert!(t.sim().is_visible(P1, &object(&t, agent)), "tick {tick}");
        let hidden = object(&t, hideout);
        let footprint = [TilePos::new(38, 20), TilePos::new(39, 21)];
        assert!(
            footprint
                .iter()
                .all(|&tile| vision.state(tile) == VisibilityState::Visible)
        );
        assert!(
            t.sim().hidden_from(&hidden, P1) && !t.sim().is_visible(P1, &hidden),
            "tick {tick}"
        );
        assert!(vision.record(hideout).is_none(), "tick {tick}: unremembered");
        let attack = Command::Order {
            units: vec![id(8)],
            kind: OrderKind::Attack(hideout),
            queue: false,
        };
        assert_eq!(gate(&t, P1, attack), Err(Reject::NotVisible(hideout)));
    }
    t.run_until(cast + SCAN_TICKS + FOG_DELAY_TICKS);
    let vision = t.sim().vision().player(P1).unwrap();
    assert!(!t.sim().is_visible(P1, &object(&t, tunnel)));
    assert!(vision.record(tunnel).is_some(), "the Tunnel remembered");
    assert!(vision.record(hideout).is_none(), "never the Hideout");
}

/// Two scans whose discs overlap, cast 50 ticks apart by the two stations, each show their own disc for their own 128
/// ticks: the tiles of both stay Visible, not lingering, until the later one ends, and those of the first alone
/// linger from its end. The scans are listed in the order they end.
#[test]
fn two_overlapping_scans_each_end_on_their_own() {
    let mut t = started(base());
    let second_tile = TilePos::new(FAR_TILE.x + 4, FAR_TILE.y);
    let second = second_tile.center();
    let (first_disc, second_disc) = (disc(FAR_TILE), disc(second_tile));
    let both: Vec<TilePos> = first_disc
        .iter()
        .copied()
        .filter(|tile| second_disc.contains(tile))
        .collect();
    let first_only: Vec<TilePos> = first_disc
        .iter()
        .copied()
        .filter(|tile| !second_disc.contains(tile))
        .collect();
    assert!(!both.is_empty() && !first_only.is_empty());
    let a = next(&mut t, scan(&[S1], FAR));
    t.run_until(a + 49);
    let b = next(&mut t, scan(&[S2], second));
    assert_eq!(b, a + 50);
    let ends = |t: &TestSim| scans(t, P1).iter().map(|scan| scan.until).collect::<Vec<_>>();
    assert_eq!(ends(&t), [a + SCAN_TICKS, b + SCAN_TICKS]);
    t.run_until(a + SCAN_TICKS);
    assert_eq!(ends(&t), [b + SCAN_TICKS]);
    assert!(
        all_seen(&t, P1, &first_only, VisibilityState::Visible, true),
        "lingering"
    );
    assert!(all_seen(&t, P1, &both, VisibilityState::Visible, false), "still seen");
    assert!(all_seen(&t, P1, &second_disc, VisibilityState::Visible, false));
    t.run_until(a + SCAN_TICKS + FOG_DELAY_TICKS);
    assert!(all_seen(&t, P1, &first_only, VisibilityState::Explored, false));
    assert!(all_seen(&t, P1, &both, VisibilityState::Visible, false));
    t.run_until(b + SCAN_TICKS);
    assert_eq!(ends(&t), Vec::<Tick>::new());
    assert!(all_seen(&t, P1, &second_disc, VisibilityState::Visible, true));
    t.run_until(b + SCAN_TICKS + FOG_DELAY_TICKS);
    let mut all = first_disc.clone();
    all.extend(second_disc.iter().filter(|tile| !first_disc.contains(tile)));
    assert!(all_seen(&t, P1, &all, VisibilityState::Explored, false));
}
