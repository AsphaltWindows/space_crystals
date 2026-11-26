use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, ObjectId, PlayerCommand, SimConfig, SpawnAt, TickInputs, TilePos,
};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

#[test]
fn seconds_round_up() {
    assert_eq!(seconds(960), "60 s");
    assert_eq!(seconds(1), "1 s");
    assert_eq!(seconds(0), "0 s");
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// Ids 1–3 patches; player 1's Barracks 4 and plate 5 (on patch 1); player 2's Generator 6 and plate 7 (on patch
/// 2); a neutral plate 8 on patch 3; a plate of player 1's on bare ground, 9, which only a debug spawn puts there
/// (map validation refuses it), in tick 1. All in plain sight of both players: the panel is about who owns what,
/// the fog's part is tested with the client.
fn sim() -> Sim {
    use ObjectKind::{Barracks, CrystalPatch, ExtractionPlate, Generator};
    let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .config(SimConfig::DEBUG)
        .spawn(CrystalPatch, None, tile(1, 5))
        .spawn(CrystalPatch, None, tile(3, 5))
        .spawn(CrystalPatch, None, tile(5, 5))
        .spawn(Barracks, Some(P1), tile(1, 1))
        .spawn(ExtractionPlate, Some(P1), tile(1, 5))
        .spawn(Generator, Some(P2), tile(6, 1))
        .spawn(ExtractionPlate, Some(P2), tile(3, 5))
        .spawn(ExtractionPlate, None, tile(5, 5))
        .build();
    let mut sim = Sim::new(&scenario);
    let spawn = DebugCommand::Spawn {
        kind: ExtractionPlate,
        owner: Some(P1),
        at: tile(9, 5),
        orientation: Default::default(),
    };
    let command = PlayerCommand {
        player: P1,
        command: Command::Debug(spawn),
    };
    sim.step(&TickInputs {
        commands: vec![command],
    });
    assert_eq!(sim.object(ObjectId(9)).map(|o| o.kind), Some(ExtractionPlate));
    sim
}

/// The panel of object `id` alone, as `viewer` sees it without the fog.
fn panel(sim: &Sim, viewer: PlayerId, id: u32) -> InfoContent {
    let kind = sim.object(ObjectId(id)).unwrap().kind;
    let selection = Selection::new([(ObjectId(id), kind)]);
    info_text(sim, &Sight::new(sim, viewer, false), &selection).unwrap()
}

fn content(title: &str, body: &[&str]) -> InfoContent {
    InfoContent {
        title: title.into(),
        body: body.iter().map(|line| line.to_string()).collect(),
        activity: Vec::new(),
    }
}

/// Design change 0105: an own structure shows its name, HP and activity (none here: the Barracks is idle and has
/// no rally point; a busy one's is `activity`'s); another player's its name, owner and HP only, whichever player
/// looks.
#[test]
fn structures_show_their_owner_only_to_other_players() {
    let sim = sim();
    assert_eq!(panel(&sim, P1, 4), content("Barracks", &["HP 300 / 300"]));
    assert_eq!(panel(&sim, P2, 4), content("Barracks", &["Player 1", "HP 300 / 300"]));
    assert_eq!(panel(&sim, P2, 6), content("Generator", &["HP 350 / 350"]));
    assert_eq!(panel(&sim, P1, 6), content("Generator", &["Player 2", "HP 350 / 350"]));
}

/// Design change 0099 (`factions/gdo/economy.md > ExtractionPlate > InfoPanel`) as 0155 limits it: a plate shows
/// the remaining crystals of the patch under it to its owner only; anyone else gets the reduced panel of another
/// player's structure, name, owner and HP (0105), and so does everyone for a neutral plate, which nobody owns. One on
/// bare ground has none to show. An uncovered patch (10, a debug spawn) shows its count to everyone, as before
/// (`core/map.md > SpaceCrystalsPatch > InfoPanel`).
#[test]
fn only_its_owner_sees_the_crystals_left_under_a_plate() {
    let mut sim = sim();
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: tile(12, 5),
        orientation: Default::default(),
    };
    sim.step(&TickInputs {
        commands: vec![PlayerCommand {
            player: P1,
            command: Command::Debug(spawn),
        }],
    });
    let plate = |body: &[&str]| content("Extraction Plate", body);
    assert_eq!(panel(&sim, P1, 5), plate(&["HP 85 / 85", "Space Crystals: 1500"]));
    assert_eq!(panel(&sim, P2, 5), plate(&["Player 1", "HP 85 / 85"]));
    assert_eq!(panel(&sim, P2, 7), plate(&["HP 85 / 85", "Space Crystals: 1500"]));
    assert_eq!(panel(&sim, P1, 7), plate(&["Player 2", "HP 85 / 85"]));
    assert_eq!(panel(&sim, P1, 8), plate(&["Neutral", "HP 85 / 85"]), "a neutral plate");
    assert_eq!(panel(&sim, P1, 9), plate(&["HP 85 / 85"]), "on bare ground");
    for viewer in [P1, P2] {
        assert_eq!(
            panel(&sim, viewer, 10),
            content("Space Crystals", &["Neutral", "Space Crystals: 1500"]),
            "uncovered"
        );
    }
}

/// Slice M3-J2: one of the local player's own workers shows what it carries and its AssignedDropOff under its HP
/// and Shields (`factions/colonists/units/drone.md > Assigned Drop-off`); another player sees its owner, HP and
/// Shields only, as for a structure (placeholder reading of design change 0105). Player 1's Drone (2), with nothing
/// to carry and no drop-off, then carrying 8 Space Crystals for its Egress Node (1) (a saved state edited).
#[test]
fn an_own_worker_shows_what_it_carries_and_where_it_delivers() {
    let t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .faction(P1, sc_sim::Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(Vec2::new(544.0, 160.0)))
        .start();
    let idle = panel(t.sim(), P1, 2);
    assert_eq!(idle.body, ["Player 1", "HP 20 / 20", "Shields 10 / 10"]);
    let mut state = serde_json::to_value(t.sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let drone = objects.iter_mut().find(|o| o["id"] == 2).unwrap();
    drone["worker"] = serde_json::json!({ "load": 8, "drop_off": 1 });
    let sim: Sim = serde_json::from_value(state).unwrap();
    let loaded = [
        "Player 1",
        "HP 20 / 20",
        "Shields 10 / 10",
        "Carrying 8 Space Crystals",
        "Drop-off: Egress Node",
    ];
    assert_eq!(panel(&sim, P1, 2), content("Drone", &loaded));
    assert_eq!(panel(&sim, P2, 2).body, idle.body, "another player's");
}

/// Slice M4-I: an Agent carrying a Supply says so (`factions/syndicate/units/agent.md > Gathering > Supplies`:
/// "CarryCapacity: 1 Supply per trip"). Placeholder wording, as for crystals.
#[test]
fn an_own_agent_shows_the_supply_it_carries() {
    let t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .faction(P1, sc_sim::Faction::Syndicate)
        .spawn(ObjectKind::Agent, Some(P1), SpawnAt::Point(Vec2::new(544.0, 160.0)))
        .start();
    let mut state = serde_json::to_value(t.sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let agent = objects.iter_mut().find(|o| o["id"] == 1).unwrap();
    agent["worker"] = serde_json::json!({ "load": 1, "resource": "Supplies" });
    let sim: Sim = serde_json::from_value(state).unwrap();
    let body = panel(&sim, P1, 1).body;
    assert_eq!(body.last().map(String::as_str), Some("Carrying 1 Supply"), "{body:?}");
}

/// Slice M5-J1: a Recruit carrying Space Crystals says so, as a Drone does (`factions/cults/units/recruit.md >
/// Gathering`: "CarryCapacity: 2 Space Crystals per load"), to its owner only. Placeholder wording.
#[test]
fn an_own_recruit_shows_the_crystals_it_carries() {
    let t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .faction(P1, sc_sim::Faction::Cults)
        .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(544.0, 160.0)))
        .start();
    let idle = panel(t.sim(), P1, 1).body;
    let mut state = serde_json::to_value(t.sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let recruit = objects.iter_mut().find(|o| o["id"] == 1).unwrap();
    recruit["worker"] = serde_json::json!({ "load": 2 });
    let sim: Sim = serde_json::from_value(state).unwrap();
    let body = panel(&sim, P1, 1).body;
    assert_eq!(
        body.last().map(String::as_str),
        Some("Carrying 2 Space Crystals"),
        "{body:?}"
    );
    assert_eq!(body.len(), idle.len() + 1, "{idle:?}");
    let seen = panel(&sim, P2, 1).body;
    assert!(!seen.iter().any(|line| line.starts_with("Carrying")), "{seen:?}");
}

/// Slice M3-E: an object being warped in shows its new type, and its HP out of its MaxHP now, interpolated
/// (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`): player 1's Beacon (1) warping in a Robo
/// Portal, 288 of its 640 frames done, has 120 + (600 - 120) × 0.45 = 336 of 336, and Shields 10 + (20 - 10) ×
/// 0.45 = 14.5 of 14.5, shown rounded up (slice M3-B). Its owner sees the Warp In in the activity area
/// ([`activity`]); player 2 sees its owner instead.
#[test]
fn a_warp_in_shows_its_new_type_and_its_maximum_so_far() {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .spawn(ObjectKind::Beacon, Some(P1), tile(8, 6))
        .start();
    let funds = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    let warp_in = ConstructionCommand::WarpIn {
        kind: ObjectKind::RoboPortal,
        orientation: Default::default(),
    };
    t.debug_at(1, funds);
    t.at(
        1,
        P1,
        Command::Construction {
            objects: vec![ObjectId(1)],
            command: warp_in,
        },
    );
    t.run_until(288);
    let mut own = content("Robo Portal", &["HP 336 / 336", "Shields 15 / 15"]);
    own.activity = vec!["Warping in: Robo Portal 45%".to_string()];
    assert_eq!(panel(t.sim(), P1, 1), own);
    assert_eq!(
        panel(t.sim(), P2, 1),
        content("Robo Portal", &["Player 1", "HP 336 / 336", "Shields 15 / 15"])
    );
}

/// `systems/combat.md > Shields > Display` (design change 0148): "Shields current/max" next to the HP line, for an
/// object that has Shields, rounded up as the HP are; one without (a Peacekeeper) shows none. Player 1's Sabre
/// Legionnaire (1), its Shields at 2.25 of 10 (a saved state edited), and player 2's Peacekeeper (2).
#[test]
fn an_object_with_shields_shows_them_under_its_hp() {
    let t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .spawn(
            ObjectKind::SabreLegionnaire,
            Some(P1),
            SpawnAt::Point(Vec2::new(160.0, 160.0)),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(544.0, 160.0)),
        )
        .start();
    let mut state = serde_json::to_value(t.sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let sabre = objects.iter_mut().find(|o| o["id"] == 1).unwrap();
    sabre["shields"] = serde_json::json!(2.25);
    let sim: Sim = serde_json::from_value(state).unwrap();
    assert_eq!(panel(&sim, P1, 1).body, ["Player 1", "HP 70 / 70", "Shields 3 / 10"]);
    assert_eq!(panel(&sim, P1, 2).body, ["Player 2", "HP 50 / 50"]);
}

/// (M5-N4a) `factions/cults/units/disciple.md > Placeholder stats`: "Energy - MaxEnergy 200; starts at 50; regenerates
/// 0.5 per second". A unit with an energy pool shows it on the line after its HP, "Energy 50 / 200", rounded down, to
/// every player who sees it (Placeholder, module docs); one without (a Militant) shows none. Player 1's Disciple (1)
/// comes into the game with 50, and after 31 ticks has 50.97, still "50"; the 32nd makes it 51.
#[test]
fn a_unit_with_an_energy_pool_shows_its_energy_under_its_hp() {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(160.0, 160.0)))
        .spawn(ObjectKind::Militant, Some(P2), SpawnAt::Point(Vec2::new(544.0, 160.0)))
        .start();
    let energy = |sim: &Sim| (sim.object(ObjectId(1))).and_then(sc_sim::energy::of);
    assert_eq!(energy(t.sim()), Some((50.0, 200.0)));
    let disciple = ["Player 1", "HP 40 / 40", "Energy 50 / 200"];
    assert_eq!(panel(t.sim(), P1, 1).body, disciple);
    assert_eq!(panel(t.sim(), P2, 1).body, disciple);
    assert_eq!(panel(t.sim(), P1, 2).body, ["Player 2", "HP 40 / 40"]);
    t.run(31);
    assert_eq!(energy(t.sim()), Some((50.0 + 31.0 / 32.0, 200.0)));
    assert_eq!(panel(t.sim(), P1, 1).body[2], "Energy 50 / 200");
    t.run(1);
    assert_eq!(panel(t.sim(), P1, 1).body[2], "Energy 51 / 200");
}
