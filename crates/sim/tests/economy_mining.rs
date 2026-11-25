//! Plate mining (slice M2-B; `factions/gdo/economy.md > ExtractionPlate`): plates pay their owner in lumps at the end
//! of each 48-tick cycle, slowed by the power ratio with the remainder carried over (design change 0098); they take
//! 10, then what is left, then the residual 1 (0099); and the removal path takes a depleted patch away with its plate,
//! opening its tile (`> On destruction`, 0038). `TestSim` checks the invariants after every tick.

use sc_sim::economy::RATIO_ONE;
use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    DebugCommand, Faction, LayoutObject, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, ResourceState,
    SimEvent, SpawnAt, SpawnSettings, Target, Tick, TilePos, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;
const DC: ObjectKind = ObjectKind::DeploymentCenter;
/// What a GDO player's bank holds when the game starts (`factions/gdo/economy.md > Starting Conditions`).
const START: u32 = 100;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn patch(starting_amount: u32) -> SpawnSettings {
    SpawnSettings::CrystalPatch { starting_amount }
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner,
        at: tile(x, y),
        orientation: Default::default(),
    }
}

/// Player 1's base as a game starts it (`factions/gdo/economy.md > Starting Conditions`): patches 1 and 2 (1500
/// each, at (1, 7) and (3, 7)), the Deployment Center 3, the Extraction Facility 4, and plates 5 and 6 on the
/// patches. Power: 30 generated, 21 consumed.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(16, 10))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 7))
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 7))
        .spawn(DC, Some(P1), tile(1, 1)) // (1, 1) to (4, 4)
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1)) // (6, 1) to (8, 3)
        .spawn(PLATE, Some(P1), tile(1, 7))
        .spawn(PLATE, Some(P1), tile(3, 7))
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

/// What patch `id` has left.
fn remaining(t: &TestSim, id: u32) -> u32 {
    match t.sim().object(ObjectId(id)).map(|o| &o.resource) {
        Some(Some(ResourceState::CrystalPatch { remaining })) => *remaining,
        other => panic!("#{id} is no patch: {other:?}"),
    }
}

/// Runs until `end` and lists the ticks in which `player`'s bank grew, with how much.
fn payouts(t: &mut TestSim, player: PlayerId, end: Tick) -> Vec<(Tick, u32)> {
    let mut list = Vec::new();
    while t.sim().tick() < end {
        let before = bank(t, player);
        t.step();
        let after = bank(t, player);
        if after != before {
            list.push((t.sim().tick(), after - before));
        }
    }
    list
}

/// Whether the Peacekeepers' grid blocks tile `(x, y)`.
fn blocked(t: &TestSim, x: i32, y: i32) -> bool {
    let grid = t.sim().nav().grid(NavClass::GroundRugged, 1).unwrap();
    grid.is_blocked(TilePos::new(x, y))
}

/// The ticks of the events that `matches` picks.
fn ticks_of(t: &TestSim, matches: impl Fn(&SimEvent) -> bool) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches(e))
        .map(|(tick, _)| *tick)
        .collect()
}

/// The plan's acceptance and design change 0098's verification 1: at full power two plates pay +20 at ticks 48, 96,
/// … in lumps, never in between, each taking 10 from its patch.
#[test]
fn plates_pay_in_lumps_every_48_ticks() {
    let mut t = base().start();
    assert_eq!(
        t.sim().object(ObjectId(5)).unwrap().extractor.unwrap().patch,
        ObjectId(1)
    );
    assert_eq!(
        t.sim().object(ObjectId(6)).unwrap().extractor.unwrap().patch,
        ObjectId(2)
    );
    assert_eq!(t.sim().power(P1).ratio(), RATIO_ONE);
    assert_eq!(payouts(&mut t, P1, 200), [(48, 20), (96, 20), (144, 20), (192, 20)]);
    assert_eq!((remaining(&t, 1), remaining(&t, 2)), (1460, 1460));
    assert_eq!(bank(&t, P2), START, "nothing for the other player");
}

/// `> Paid in lumps`: "A new plate's first payment comes 48 frames after it is placed": a plate placed in tick 10
/// pays first in tick 58.
#[test]
fn a_new_plate_pays_first_48_ticks_after_it_is_placed() {
    let mut t = base().start();
    t.debug_at(1, spawn(ObjectKind::CrystalPatch, None, 5, 7)) // 7
        .debug_at(10, spawn(PLATE, Some(P1), 5, 7)); // 8
    assert_eq!(payouts(&mut t, P1, 110), [(48, 20), (58, 10), (96, 20), (106, 10)]);
    assert_eq!(
        t.sim().object(ObjectId(8)).unwrap().extractor.unwrap().patch,
        ObjectId(7)
    );
}

/// The plan's acceptance under the penalty, design changes 0089 and 0098: at the all-in Barracks' power (30
/// generated, 51 consumed) the cycles advance at 30/51 with the remainder carried over, so the k-th load comes in
/// tick ⌈k × 48 × 51 / 30⌉, every 81 or 82 ticks (0098's verification 2: "every ≈81 frames"), five in exactly 408
/// ticks, and the long-run rate is 10 × 30/51 per 48 ticks. The loads keep their size.
#[test]
fn under_the_power_penalty_the_cycles_slow_down() {
    let mut t = base()
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 1)) // 7: (10, 1) to (12, 2)
        .start();
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 51));
    let pays = payouts(&mut t, P1, 2100);
    let expected: Vec<(Tick, u32)> = (1..=25u32).map(|k| ((k * 48 * 51).div_ceil(30), 20)).collect();
    assert_eq!(pays, expected);
    assert_eq!(
        &pays[..5].iter().map(|p| p.0).collect::<Vec<_>>(),
        &[82, 164, 245, 327, 408]
    );
    assert!(pays.windows(2).all(|w| (81..=82).contains(&(w[1].0 - w[0].0))));
}

/// "The cycle timer advances by the power ratio each frame" (design change 0098): when the ratio changes during a
/// cycle, the progress made so far counts. 50 ticks at 30/51 (38 551 a tick in 1/65 536 of a tick) leave 1 218 178
/// of the cycle's 3 145 728: 19 ticks at full power once the Barracks is gone, so the load comes in tick 69, and the
/// next ones 48 ticks apart.
#[test]
fn a_ratio_change_keeps_the_cycle_s_progress() {
    let mut t = base()
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 1)) // 7
        .start();
    t.debug_at(51, DebugCommand::Remove { id: ObjectId(7) });
    assert_eq!(t.run_until(50).sim().power(P1).ratio(), 38_551);
    assert_eq!(payouts(&mut t, P1, 170), [(69, 20), (117, 20), (165, 20)]);
    assert_eq!(t.sim().power(P1).ratio(), RATIO_ONE);
}

/// Design change 0099's verification 1 and the plan's depletion acceptance: the loads are 10 while the patch holds
/// 10, then what is left, then the residual 1 forever; the patch stays under its plate with 0 left, and
/// `PatchDepleted` comes once, with the load that emptied it.
#[test]
fn a_patch_pays_ten_then_the_rest_then_the_residual() {
    // The patch's starting amount, the six loads it pays, and the cycle whose load empties it.
    for (amount, loads, emptied) in [
        (25, [10, 10, 5, 1, 1, 1], 3),
        (15, [10, 5, 1, 1, 1, 1], 2),
        (20, [10, 10, 1, 1, 1, 1], 2),
        (10, [10, 1, 1, 1, 1, 1], 1),
        (1, [1, 1, 1, 1, 1, 1], 1),
    ] {
        let mut t = ScenarioBuilder::new(&flat(10, 8))
            .spawn_with(patch(amount), tile(1, 6)) // 1
            .spawn(DC, Some(P1), tile(1, 1)) // 2
            .spawn(PLATE, Some(P1), tile(1, 6)) // 3
            .start();
        let pays = payouts(&mut t, P1, 6 * 48);
        let expected: Vec<(Tick, u32)> = (1..=6).zip(loads).map(|(k, load)| (48 * k, load)).collect();
        assert_eq!(pays, expected, "a patch of {amount}");
        assert_eq!(remaining(&t, 1), 0, "a patch of {amount}: never below 0");
        let depleted: Vec<_> = t
            .events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::PatchDepleted { .. }))
            .cloned()
            .collect();
        let event = SimEvent::PatchDepleted {
            patch: ObjectId(1),
            plate: ObjectId(3),
        };
        assert_eq!(depleted, [(48 * emptied, event)], "a patch of {amount}");
        assert!(
            t.sim().object(ObjectId(1)).is_some() && blocked(&t, 1, 6),
            "it stays under its plate"
        );
    }
}

/// `> On destruction`: "If the underlying Space Crystal Patch still has resources: the patch becomes uncovered and
/// accessible": it stays with what it has left and blocks, and a new plate placed on it mines it.
#[test]
fn a_plate_gone_from_a_patch_with_crystals_uncovers_it() {
    let mut t = base().start();
    t.debug_at(50, DebugCommand::Remove { id: ObjectId(5) })
        .debug_at(60, spawn(PLATE, Some(P1), 1, 7)); // 7
    assert_eq!(payouts(&mut t, P1, 150), [(48, 20), (96, 10), (108, 10), (144, 10)]);
    assert_eq!(remaining(&t, 1), 1480, "10 for plate 5, 10 for plate 7");
    assert_eq!(
        t.sim().object(ObjectId(7)).unwrap().extractor.unwrap().patch,
        ObjectId(1)
    );
    assert!(blocked(&t, 1, 7));
    let removed = ticks_of(&t, |e| matches!(e, SimEvent::Removed(_)));
    assert_eq!(removed, [50], "the plate only");
}

/// The plan's acceptance for a plate destroyed on a depleted patch, and design change 0038's verification ("Mine a
/// patch out: units can now walk through its tile"): a patch of 10 in the one gap of a mountain wall is mined out by
/// player 2's plate in tick 48 and stays under it; player 1's Peacekeepers destroy the plate, and the patch leaves
/// the game in the same tick, its tile open at once (the grids rebuilt); a third Peacekeeper then walks through it.
#[test]
fn a_plate_destroyed_on_a_depleted_patch_takes_it_along() {
    let wall = ".........M..........\n";
    let gap = "....................\n";
    let map = [wall, wall, wall, wall, wall, gap, wall, wall, wall, wall, wall].concat();
    let pk = ObjectKind::Peacekeeper;
    let mut t = ScenarioBuilder::new(&map)
        .spawn_with(patch(10), tile(9, 5)) // 1, in the gap
        .spawn(DC, Some(P2), tile(14, 0)) // 2: (14, 0) to (17, 3)
        .spawn(PLATE, Some(P2), tile(9, 5)) // 3
        .spawn(pk, Some(P1), SpawnAt::Point(TilePos::new(6, 4).center())) // 4
        .spawn(pk, Some(P1), SpawnAt::Point(TilePos::new(6, 6).center())) // 5
        .spawn(pk, Some(P1), SpawnAt::Point(TilePos::new(3, 5).center())) // 6
        .start();
    t.without_auto_targeting();
    t.run_until(48);
    assert_eq!(
        (bank(&t, P2), remaining(&t, 1)),
        (START + 10, 0),
        "mined out in tick 48"
    );
    t.order_at(
        49,
        P1,
        &[ObjectId(4), ObjectId(5)],
        OrderKind::Attack(ObjectId(3)),
        false,
    );
    let version = t.sim().nav_version();
    while t.sim().object(ObjectId(3)).is_some() {
        assert!(t.sim().tick() < 400, "the plate should be destroyed by now");
        assert!(blocked(&t, 9, 5), "the depleted patch blocks under its plate");
        t.step();
    }
    let death = t.sim().tick();
    let events: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == death)
        .map(|(_, e)| e)
        .collect();
    assert!(events.contains(&&SimEvent::Removed(ObjectId(1))), "{events:?}");
    assert!(events.contains(&&SimEvent::Removed(ObjectId(3))), "{events:?}");
    assert!(t.sim().object(ObjectId(1)).is_none(), "the patch left with its plate");
    assert!(
        !blocked(&t, 9, 5) && t.sim().nav_version() > version,
        "its tile is open"
    );
    assert_eq!(dangling_references(t.sim()), []);

    let beyond = TilePos::new(15, 5).center();
    t.order_at(
        death + 1,
        P1,
        &[ObjectId(6)],
        OrderKind::Move(Target::Point(beyond)),
        false,
    );
    let mut through = false;
    for _ in 0..200 {
        through |= TilePos::containing(t.step().pos(6)) == TilePos::new(9, 5);
    }
    assert!(through, "it walked through the patch's tile");
    assert_eq!(t.status(6), MoveStatus::Arrived);
    assert!(t.pos(6).distance(beyond) < 1.0, "{}", t.pos(6));
}

/// Design change 0042's second verification with the mining itself ("A patch mined out in the fog is still routed
/// around until a unit sees its tile"; `path_knowledge/` has it with a debug removal): player 2's plate mines out a
/// map patch of 10 in tick 48 and is removed in tick 50, taking the patch along, which opens the true grid at once.
/// Player 1, who never saw either, still knows the patch there by its record from the start (design change 0159),
/// its planning grid blocked, until its Peacekeeper comes in sight of the tile.
#[test]
fn a_patch_mined_out_in_the_fog_is_known_until_its_tile_is_seen() {
    let mut t = ScenarioBuilder::new(&flat(40, 10))
        .spawn_with(patch(10), tile(30, 5)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(36, 1)) // 2
        .spawn(PLATE, Some(P2), tile(30, 5)) // 3
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            SpawnAt::Point(TilePos::new(2, 5).center()),
        ) // 4
        .start();
    t.without_auto_targeting()
        .debug_at(50, DebugCommand::Remove { id: ObjectId(3) });
    let known = |t: &TestSim| -> Vec<ObjectId> {
        let sight = t.sim().vision().player(P1).unwrap();
        sight.records().map(|r| r.id).collect()
    };
    let known_blocked = |t: &TestSim| {
        let grid = t.sim().known_nav(P1).unwrap().grid(NavClass::GroundRugged, 1).unwrap();
        grid.is_blocked(TilePos::new(30, 5))
    };
    assert_eq!(known(&t), [ObjectId(1)], "the patch, not player 2's starting layout");
    t.run_until(50);
    assert_eq!(bank(&t, P2), START + 10, "mined out in tick 48");
    assert!(
        t.sim().object(ObjectId(1)).is_none() && !blocked(&t, 30, 5),
        "gone with its plate"
    );
    assert!(
        known_blocked(&t) && known(&t) == [ObjectId(1)],
        "player 1 didn't see it go"
    );

    // Towards the tile, until it comes into sight (a Peacekeeper sees 5 tiles): the record goes in that tick.
    let sees = |t: &TestSim| t.sim().vision().state(P1, TilePos::new(30, 5)) == VisibilityState::Visible;
    let lookout = TilePos::new(26, 5).center();
    t.order_at(51, P1, &[ObjectId(4)], OrderKind::Move(Target::Point(lookout)), false);
    while !sees(&t) {
        assert!(
            known_blocked(&t) && known(&t) == [ObjectId(1)],
            "tick {}",
            t.sim().tick()
        );
        assert!(t.sim().tick() < 1000, "the Peacekeeper should see the tile by now");
        t.step();
    }
    assert!(!known_blocked(&t) && known(&t).is_empty());
}

/// The same through a debug removal of the plate, in the command phase: the depleted patch goes in the same tick,
/// its `Removed` event with the plate's, and the grids follow (`Sim::step` checks them in debug builds).
#[test]
fn a_depleted_patch_goes_with_its_plate_on_a_debug_removal() {
    let mut t = ScenarioBuilder::new(&flat(10, 8))
        .spawn_with(patch(10), tile(1, 6)) // 1
        .spawn(DC, Some(P1), tile(1, 1)) // 2
        .spawn(PLATE, Some(P1), tile(1, 6)) // 3
        .start();
    t.debug_at(50, DebugCommand::Remove { id: ObjectId(3) });
    t.run_until(50);
    let removed: Vec<_> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == 50)
        .map(|(_, e)| e.clone())
        .collect();
    assert_eq!(
        removed,
        [SimEvent::Removed(ObjectId(1)), SimEvent::Removed(ObjectId(3))]
    );
    assert!(!blocked(&t, 1, 6));
}

/// When the patch under a plate is removed (a debug command; nothing else removes a covered patch), the plate stays
/// and mines nothing from then on, and nothing refers to the patch any more.
#[test]
fn a_patch_removed_from_under_its_plate_stops_the_plate() {
    let mut t = base().start();
    t.debug_at(30, DebugCommand::Remove { id: ObjectId(1) });
    assert_eq!(payouts(&mut t, P1, 150), [(48, 10), (96, 10), (144, 10)]);
    let plate = t.sim().object(ObjectId(5)).unwrap();
    assert_eq!(plate.extractor, None);
    assert_eq!(dangling_references(t.sim()), []);
}

/// The link between a plate and its patch comes when both are in the game, in whatever order (module docs of
/// `economy::mining`): a map that lists the plate first, a Starting Layout's plate on a map patch (listed before the
/// layout's Deployment Center, whose build area it needs: design change 0097), a patch spawned under a plate that
/// stood on bare ground (which mined nothing), and its first load 48 ticks after. A plate without an owner mines for
/// nobody.
#[test]
fn plates_link_to_their_patch_in_any_order() {
    let mut scenario = ScenarioBuilder::new(&flat(12, 8))
        .spawn(DC, Some(P1), tile(1, 1)) // 1
        .spawn(PLATE, Some(P1), tile(1, 6)) // 2, before its patch
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 6)) // 3
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 6)) // 4, for the layout's plate
        .spawn(ObjectKind::CrystalPatch, None, tile(9, 6)) // 5, for a neutral plate
        .spawn(PLATE, None, tile(9, 6)) // 6
        .build();
    let laid = |kind, x, y| LayoutObject {
        kind,
        at: tile(x, y),
        orientation: Default::default(),
    };
    *scenario.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![
        laid(PLATE, 3, 6), // 7
        laid(DC, 6, 1),    // 8: (6, 1) to (9, 4)
    ];
    let mut t = TestSim::new(scenario);
    let patch_of = |t: &TestSim, id: u32| t.sim().object(ObjectId(id)).unwrap().extractor.map(|e| e.patch);
    assert_eq!(patch_of(&t, 2), Some(ObjectId(3)));
    assert_eq!(patch_of(&t, 7), Some(ObjectId(4)));
    assert_eq!(patch_of(&t, 6), Some(ObjectId(5)));
    t.debug_at(5, spawn(PLATE, Some(P1), 5, 6)) // 9, on bare ground
        .debug_at(20, spawn(ObjectKind::CrystalPatch, None, 5, 6)); // 10, under it
    t.run_until(19);
    assert_eq!(patch_of(&t, 9), None);
    assert_eq!(payouts(&mut t, P1, 100), [(48, 20), (68, 10), (96, 20)]);
    assert_eq!(patch_of(&t, 9), Some(ObjectId(10)));
    assert_eq!(remaining(&t, 5), 1500, "the neutral plate mines nothing");
    assert_eq!(bank(&t, P2), START);
}
