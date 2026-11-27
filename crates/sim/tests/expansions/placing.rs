//! Where an expansion may go (`factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`): its ghost
//! (`Sim::expansion_check`, by what the owner knows) and the command (by the truth), in a game.

use sc_sim::WholeProblem;

use super::*;

use TileProblem::{NotDrillable, Resource, Structure};

/// "The expansion must fit entirely within the Tunnel Area": inside it, at its corners, the ghost is green and the
/// command taken; half outside or outside, both refuse it as a whole. "Own and neutral underground structures block
/// it; so does the Tunnel's own footprint" (design change 0258: a neutral expansion "still **blocks its cells**").
#[test]
fn inside_the_area_clear_of_every_structure() {
    let mut t = started(base().spawn(HIDEOUT, None, tile(2, 10)), 1000); // 4: neutral
    for (x, y) in [(2, 6), (8, 6), (2, 12), (6, 12)] {
        assert!(ghost(&t, x, y).is_valid(), "({x}, {y})");
        assert_eq!(gate(&t, P1, expand(x, y)), Ok(()), "({x}, {y})");
    }
    let outside = Err(Reject::Unplaceable(PlacementProblem::Whole(
        WholeProblem::OutsideTunnelArea,
    )));
    for (x, y) in [(9, 6), (1, 6), (6, 13), (2, 5), (20, 10)] {
        assert_eq!(ghost(&t, x, y).whole, [WholeProblem::OutsideTunnelArea], "({x}, {y})");
        assert_eq!(gate(&t, P1, expand(x, y)), outside, "({x}, {y})");
    }
    assert_eq!(
        gate(&t, P1, expand(3, 7)),
        tile_problem(4, 8, Structure),
        "the Tunnel's own footprint"
    );
    assert_eq!(
        gate(&t, P1, expand(8, 11)),
        tile_problem(8, 12, Structure),
        "its own Hideout"
    );
    assert_eq!(
        gate(&t, P1, expand(2, 9)),
        tile_problem(2, 10, Structure),
        "a neutral Hideout"
    );
    assert_eq!(ghost(&t, 2, 9).tiles, [(at(2, 10), Structure), (at(3, 10), Structure)]);
    next(&mut t, P1, expand(2, 6));
    assert!(t.sim().object(building(&t)).is_some_and(|o| o.kind == HIDEOUT));
}

/// Design change 0241: "a Space Crystals patch or Supply Delivery Station above blocks an expansion".
#[test]
fn resources_block_it() {
    let scenario = base()
        .spawn(ObjectKind::CrystalPatch, None, tile(2, 6)) // 4
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(8, 6)); // 5
    let t = started(scenario, 1000);
    assert_eq!(gate(&t, P1, expand(2, 6)), tile_problem(2, 6, Resource), "a patch");
    assert_eq!(ghost(&t, 2, 6).tiles, [(at(2, 6), Resource)]);
    assert_eq!(gate(&t, P1, expand(8, 7)), tile_problem(8, 7, Resource), "a station");
    assert_eq!(gate(&t, P1, expand(6, 12)), Ok(()));
}

/// Design change 0290's verification, "Place a Hideout under a standing Guard: it is accepted" (`core/entities.md >
/// Direct Placement`'s exception: "ground units **don't** block a Syndicate expansion's placement, since it is
/// underground and units walk over expansions anyway"), and under another player's unit alike: the ghost is green,
/// the click builds it, and the units stand where they stood.
#[test]
fn ground_units_dont_block_it() {
    let scenario = base()
        .spawn(ObjectKind::Guard, Some(P1), point(3, 7)) // 4
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(7, 13)); // 5
    let mut t = started_peaceful(scenario, 1000);
    let stand = |t: &mut TestSim, player: PlayerId, unit: u32| {
        let hold = Command::Order {
            units: vec![ObjectId(unit)],
            kind: sc_sim::OrderKind::Hold,
            queue: false,
        };
        next(t, player, hold);
    };
    stand(&mut t, P1, 4);
    stand(&mut t, P2, 5);
    let (guard, peacekeeper) = (object(&t, ObjectId(4)).pos, object(&t, ObjectId(5)).pos);
    for (x, y) in [(2, 6), (6, 12)] {
        assert!(ghost(&t, x, y).is_valid(), "({x}, {y})");
        assert_eq!(gate(&t, P1, expand(x, y)), Ok(()), "({x}, {y})");
    }
    next(&mut t, P1, expand(2, 6));
    let site = building(&t);
    assert_eq!(object(&t, site).placement.map(|p| p.origin), Some(at(2, 6)));
    t.run(10);
    assert_eq!(
        (object(&t, ObjectId(4)).pos, object(&t, ObjectId(5)).pos),
        (guard, peacekeeper)
    );
}

/// Design change 0239: "**Every tile under it must be Drillable** (... Plane, Rugged and Cliff yes; Mountain and Water
/// no ...)", whose verification "a ghost over Water inside the area is red"; and `core/map.md > Elevation` (design
/// change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be placed on sloped tiles".
#[test]
fn drillable_ground_sloped_or_not() {
    let map = "
        ........................
        ........................
        ........................
        ........................
        ........................
        ........................
        ........................
        ..~~rr..................
        ..~~rr..................
        ........................
        ........................
        ........................
        ........................
        ........................
        ........................
        ........................

        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111111111111111111111
        1111111112222222222222222
        1111111112222222222222222
        1111111112222222222222222
        1111111112222222222222222";
    let mut t = started(
        syndicate(map)
            .spawn(TUNNEL, Some(P1), tile(4, 8))
            .spawn(HIDEOUT, Some(P1), tile(2, 10)),
        1000,
    );
    let water = ghost(&t, 2, 7);
    let wet = [at(2, 7), at(3, 7), at(2, 8), at(3, 8)];
    assert_eq!(water.tiles, wet.map(|tile| (tile, NotDrillable)));
    assert_eq!(
        gate(&t, P1, expand(2, 7)),
        tile_problem(2, 7, NotDrillable),
        "Water: red"
    );
    assert!(ghost(&t, 4, 6).is_valid(), "Rugged");
    let sloped = at(8, 12);
    assert!(
        !t.sim().map().is_flat(sloped),
        "a slope from level 1 to 2 at x = 8 and 9 below y = 12"
    );
    assert!(ghost(&t, 8, 12).is_valid());
    next(&mut t, P1, expand(8, 12));
    assert_eq!(
        t.sim().object(building(&t)).and_then(|o| o.placement).map(|p| p.origin),
        Some(sloped)
    );
}

/// Design change 0240: "**Surface structures block it too** ...: no expansion can be placed under any surface
/// building, any player's (judged by the truth; the placement is refused)", whose verification "A Syndicate player
/// can't place an expansion under an enemy's building". A GDO Generator in a corner of player 1's Tier 3 area that
/// player 1 has never seen: the ghost, which judges by what the player knows, is green, and the command is refused
/// as it applies, with every problem the truth shows (`SimEvent::PlacementRefused`, the Tunnel its constructor);
/// nothing is paid. Seen, the ghost is red too.
#[test]
fn an_enemy_building_refuses_it_by_the_truth() {
    let scenario = base()
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Generator, Some(P2), tile(0, 4)); // 4: (0, 4) to (1, 5)
    let mut t = started(scenario, 1000);
    next_debug(&mut t, DebugCommand::SetTier { id: T1, tier: 3 });
    // The Tier 3 area: (0, 4) to (11, 15).
    assert!(ghost(&t, 0, 4).is_valid(), "unknown to player 1: {:?}", ghost(&t, 0, 4));
    assert_eq!(gate(&t, P1, expand(0, 4)), tile_problem(0, 4, Structure));
    next(&mut t, P1, expand(0, 4));
    let refused = events(&t, |e| match e {
        SimEvent::PlacementRefused {
            constructor,
            kind,
            origin,
            problems,
            ..
        } => Some((*constructor, *kind, *origin, problems.tiles.len())),
        _ => None,
    });
    assert_eq!(refused, [(T1, HIDEOUT, at(0, 4), 4)]);
    assert_eq!((operation(&t), crystals(&t, P1)), (None, 1000));
    // In sight, its tiles are red on the ghost too.
    next_debug(
        &mut t,
        DebugCommand::Spawn {
            kind: AGENT,
            owner: Some(P1),
            at: point(2, 6),
            orientation: BuildingOrientation::default(),
        },
    );
    t.run(sc_sim::tuning::vision::BUILTIN.fog_delay_ticks + 2);
    assert_eq!(ghost(&t, 0, 4).tiles.len(), 4, "{:?}", ghost(&t, 0, 4));
}

/// "Click an expansion type: begins construction if the Tunnel is not already performing an operation"
/// (`factions/syndicate/tunnel_menus.md > ExpandMenu`; "A Tunnel can only perform one operation at a time"): while it
/// builds, another Expand is refused, wherever it goes, and the menu's check says so; once done, it builds again.
/// (Refused while it upgrades: slice M4-G's upgrade takes the same slot.)
#[test]
fn refused_while_the_tunnel_builds() {
    let mut t = started(base(), 1000);
    next(&mut t, P1, expand(2, 6));
    let busy = Err(Reject::OperationUnderWay(T1));
    assert_eq!(gate(&t, P1, expand(6, 12)), busy);
    assert_eq!(t.sim().can_expand(P1, T1, HIDEOUT), busy);
    t.run(400);
    assert_eq!(operation(&t), None, "done");
    assert_eq!(gate(&t, P1, expand(6, 12)), Ok(()));
}
