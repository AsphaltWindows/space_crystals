//! Banks, power and Unit Control (slice M2-B; `factions/gdo/overview.md > GDOResources`): the banks start at the
//! Starting Conditions (slice M2-F1) and `DebugCommand::SetResources` sets them; power and Unit Control follow the objects, counting a building from the
//! tick it is placed until the tick it is destroyed (design change 0089); and a game with mining is deterministic:
//! hashed, serialized and replayed.

use sc_sim::economy::RATIO_ONE;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Reject, Replay, Sim, SimConfig, SpawnAt,
    SpawnSettings, Tick, TilePos, UNIT_CONTROL_CAP,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner,
        at,
        orientation: Default::default(),
    }
}

/// Player 1's base as a game starts it (`factions/gdo/economy.md > Starting Conditions`): patches 1 and 2, the
/// Deployment Center 3, the Extraction Facility 4, and plates 5 and 6 on the patches.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(16, 10))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 7))
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 7))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1)) // (1, 1) to (4, 4)
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1)) // (6, 1) to (8, 3)
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 7))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 7))
}

fn banks(sim: &Sim, player: PlayerId) -> (u32, u32) {
    let p = sim.player(player).unwrap();
    (p.crystals, p.supplies)
}

fn set_resources(player: PlayerId, crystals: u32, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// Runs until `end` and lists the ticks in which `player`'s Space Crystals grew, with how much.
fn payouts(t: &mut TestSim, player: PlayerId, end: Tick) -> Vec<(Tick, u32)> {
    let mut list = Vec::new();
    while t.sim().tick() < end {
        let before = banks(t.sim(), player).0;
        t.step();
        let after = banks(t.sim(), player).0;
        if after != before {
            list.push((t.sim().tick(), after - before));
        }
    }
    list
}

/// The banks start at the GDO Starting Conditions, 100 Space Crystals and 0 Supplies (`factions/gdo/economy.md >
/// Starting Conditions`; Supplies: design change 0077), and `SetResources` sets any player's, whoever issues it.
/// Income into a full bank saturates instead of panicking.
#[test]
fn the_banks_start_at_the_starting_conditions_and_set_resources_sets_them() {
    let mut t = base().start();
    assert_eq!((banks(t.sim(), P1), banks(t.sim(), P2)), ((100, 0), (100, 0)));
    t.debug_at(2, set_resources(P2, 100, 7))
        .debug_at(3, set_resources(P1, u32::MAX - 15, 1));
    t.run_until(3);
    assert_eq!((banks(t.sim(), P1), banks(t.sim(), P2)), ((u32::MAX - 15, 1), (100, 7)));
    t.run_until(48);
    assert_eq!(banks(t.sim(), P1), (u32::MAX, 1), "+20 into a bank 15 short of full");
    t.run_until(96);
    assert_eq!(banks(t.sim(), P1), (u32::MAX, 1));
}

/// The gate checks the player a `SetResources` names (`Reject::UnknownPlayer`), and that debug commands are on.
#[test]
fn set_resources_needs_a_player_and_debug_commands() {
    let command = |player| PlayerCommand {
        player: P1,
        command: Command::Debug(set_resources(player, 5, 5)),
    };
    let sim = base().start().sim().clone();
    assert_eq!(sim.validate(&command(P2)), Ok(()));
    for player in [PlayerId(0), PlayerId(3), PlayerId(255)] {
        assert_eq!(sim.validate(&command(player)), Err(Reject::UnknownPlayer(player)));
    }
    let sim = Sim::new(&base().config(SimConfig::default()).build());
    assert_eq!(sim.validate(&command(P2)), Err(Reject::DebugDisabled));
}

/// Design changes 0089 and 0090, and the plan's power acceptance: the starting base is +9, 21 consumed out of 30
/// generated (DC +30; EF −15 and two plates −3 each), at full speed; an all-in Barracks makes it −21, 51 out of 30, at
/// the ratio 30/51 (≈ 59%); a Generator adds 20. A building counts at the end of the tick it is placed in, and no
/// longer at the end of the tick it leaves. A player without structures has nothing, at full speed.
#[test]
fn power_follows_the_structures() {
    let mut t = base().start();
    let power = |t: &TestSim| {
        let p = t.sim().power(P1);
        (p.generated, p.consumed, p.ratio(), p.is_negative())
    };
    assert_eq!(power(&t), (30, 21, RATIO_ONE, false));
    t.debug_at(5, spawn(ObjectKind::Barracks, Some(P1), tile(10, 1))) // 7
        .debug_at(10, spawn(ObjectKind::Generator, Some(P1), tile(10, 5))) // 8
        .debug_at(15, DebugCommand::Remove { id: ObjectId(7) });
    t.run_until(5);
    assert_eq!(power(&t), (30, 51, 38_551, true), "the all-in Barracks");
    t.run_until(10);
    assert_eq!(power(&t), (50, 51, 64_251, true));
    t.run_until(15);
    assert_eq!(power(&t), (50, 21, RATIO_ONE, false));
    let nobody = t.sim().power(P2);
    assert_eq!((nobody.generated, nobody.consumed, nobody.ratio()), (0, 0, RATIO_ONE));
    assert_eq!(t.sim().power(PlayerId(9)), Default::default());
}

/// Design change 0089, point 4, in the plates' speed: two plates with no generator mine nothing; a Generator placed
/// in tick 10 counts in tick 10, so the first load comes 48 ticks of progress later, in tick 57; removed in tick 105,
/// it no longer counts in tick 105, so the cycle that would end in it never ends.
#[test]
fn a_building_powers_the_plates_from_the_tick_it_is_placed_until_the_tick_it_goes() {
    let mut t = ScenarioBuilder::new(&flat(12, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 6)) // 1
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 6)) // 2
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 6)) // 3
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 6)) // 4
        .start();
    assert_eq!(t.sim().power(P1).ratio(), 0, "6 consumed, nothing generated");
    t.debug_at(10, spawn(ObjectKind::Generator, Some(P1), tile(6, 1))) // 5
        .debug_at(105, DebugCommand::Remove { id: ObjectId(5) });
    assert_eq!(payouts(&mut t, P1, 400), [(57, 20)]);
}

/// `> Unit Control`: the units' UnitControlCost (Peacekeeper 1), counted while they are in the game, so a unit that
/// dies gives it back ("for as long as the unit lives", design change 0143; 0082 before it); structures and the
/// units of factions without Unit Control take none. The cap is 200.
#[test]
fn unit_control_counts_the_units_in_the_game() {
    let mut t = base().start();
    assert_eq!((t.sim().unit_control(P1), UNIT_CONTROL_CAP), (0, 200));
    let at = |x: i32, y: i32| SpawnAt::Point(TilePos::new(x, y).center());
    t.debug_at(1, spawn(PK, Some(P1), at(12, 5))) // 7
        .debug_at(1, spawn(PK, Some(P1), at(13, 5))) // 8
        .debug_at(1, spawn(PK, Some(P1), at(14, 5))) // 9
        .debug_at(1, spawn(PK, Some(P2), at(12, 8))) // 10
        .debug_at(1, spawn(ObjectKind::Guard, Some(P1), at(14, 8))) // 11
        .debug_at(1, DebugCommand::SetAutoTargeting { on: false })
        .debug_at(2, DebugCommand::Remove { id: ObjectId(8) });
    t.run_until(1);
    assert_eq!((t.sim().unit_control(P1), t.sim().unit_control(P2)), (3, 1));
    t.run_until(2);
    assert_eq!(t.sim().unit_control(P1), 2);
    assert_eq!(t.sim().unit_control(PlayerId(9)), 0);
}

/// The plan's determinism acceptance: a game with mining under the penalty, a ratio that changes, a bank set by
/// command, a patch mined out and its plate removed with it, replays to the same state, also through JSON, and the
/// state serializes and loads as it was. (The state hash covers the banks and the extractors: `Sim::state_hash`
/// destructures both, and `sim::tests::state_hash_covers_every_field` changes each field.)
#[test]
fn a_mining_game_replays_and_round_trips() {
    let mut t = ScenarioBuilder::new(&flat(16, 10))
        .spawn_with(SpawnSettings::CrystalPatch { starting_amount: 25 }, tile(1, 7)) // 1
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 7)) // 2
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1)) // 3
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1)) // 4
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 7)) // 5
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 7)) // 6
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 1)) // 7
        .start();
    t.debug_at(3, set_resources(P1, 1000, 4))
        .debug_at(100, spawn(ObjectKind::Generator, Some(P1), tile(10, 5))) // 8
        .debug_at(150, spawn(ObjectKind::Generator, Some(P1), tile(13, 5))) // 9
        .debug_at(400, DebugCommand::Remove { id: ObjectId(5) });
    t.run_until(600);
    let sim = t.sim();
    assert!(
        sim.object(ObjectId(1)).is_none(),
        "the patch of 25 was mined out and went with its plate"
    );
    assert!(banks(sim, P1).0 > 1000 + 25, "{:?}", banks(sim, P1));

    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), sim.state_hash());
    assert_eq!(&replayed, sim);
    let json = serde_json::to_string(t.replay()).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.replay());
    assert_eq!(loaded.run().state_hash(), sim.state_hash());
    let state: Sim = serde_json::from_str(&serde_json::to_string(sim).unwrap()).unwrap();
    assert_eq!(state.state_hash(), sim.state_hash());
    assert_eq!(&state, sim);
}
