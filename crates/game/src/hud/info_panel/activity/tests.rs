mod armory;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, ObjectId, ObjectKind, PlayerId,
    ProductionCommand, SpawnAt, Target, TilePos, TunnelCommand, Vec2,
};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const BARRACKS: ObjectId = ObjectId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn production(command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![BARRACKS],
        command,
    }
}

/// The activity lines of player 1's structure `id`, as player 1 sees the world.
fn lines_of(sim: &Sim, id: ObjectId) -> Vec<String> {
    lines(sim, &Sight::new(sim, P1, true), sim.object(id).unwrap())
}

/// The Barracks' activity lines.
fn shown(sim: &Sim) -> Vec<String> {
    lines_of(sim, BARRACKS)
}

/// `core/entities.md > Object Type > InfoPanel` (design change 0105's first verification: "Select your own
/// Barracks while it's producing. It shows its name, HP, progress, queue length and rally point"). Player 1's
/// Deployment Center (1) and Barracks (2, at full power) with a Generator (3) flush against its exit side, so the
/// first Peacekeeper stays in the doorway and the second waits inside at 100% (design change 0085); player 2's
/// Peacekeeper (4) in sight.
#[test]
fn a_barracks_shows_its_unit_queue_and_rally_point() {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4))
        .spawn(ObjectKind::Generator, Some(P1), tile(11, 4))
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(600.0, 520.0)))
        .start();
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "idle, without a rally point");
    t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    for _ in 0..3 {
        t.at(2, P1, production(ProductionCommand::Train(PK)));
    }
    t.run_until(41);
    assert_eq!(shown(t.sim()), ["Training Peacekeeper: 50%", "Queue: 3 / 20"]);
    t.at(
        42,
        P1,
        production(ProductionCommand::SetRally(Target::Point(Vec2::new(900.0, 320.0)))),
    );
    t.run_until(42);
    assert_eq!(
        shown(t.sim()),
        ["Training Peacekeeper: 51%", "Queue: 3 / 20", "Rally point: ground"]
    );
    t.at(
        43,
        P1,
        production(ProductionCommand::SetRally(Target::Object(ObjectId(4)))),
    );
    t.run_until(43);
    assert_eq!(shown(t.sim())[2], "Rally point: Peacekeeper (enemy)");
    t.at(
        44,
        P1,
        production(ProductionCommand::SetRally(Target::Object(ObjectId(1)))),
    );
    t.run_until(200);
    let status = t.sim().production_status(BARRACKS).unwrap();
    assert!(status.waiting, "the first one blocks the doorway");
    assert_eq!(
        shown(t.sim()),
        [
            "Training Peacekeeper: 100%",
            "Queue: 2 / 20",
            "Rally point: Deployment Center"
        ]
    );
    // The building itself clears the rally point (design change 0086).
    t.at(
        201,
        P1,
        production(ProductionCommand::SetRally(Target::Object(BARRACKS))),
    );
    t.run_until(201);
    assert_eq!(shown(t.sim()), ["Training Peacekeeper: 100%", "Queue: 2 / 20"]);
    let generator = ObjectId(3);
    assert_eq!(
        lines_of(t.sim(), generator),
        Vec::<String>::new(),
        "it produces nothing"
    );
}

/// Design change 0143: a Barracks whose Peacekeeper waits at 0% for Unit Control says so under the unit, until
/// it starts. Player 1's Deployment Center (1) and Barracks (2) with 200 Peacekeepers (3 to 202), the cap, far
/// off; one of them removed gives the room.
#[test]
fn a_waiting_barracks_says_what_it_waits_for() {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4));
    for i in 0..200 {
        let at = Vec2::new(1000.0 + 30.0 * (i % 20) as f32, 80.0 + 30.0 * (i / 20) as f32);
        builder = builder.spawn(PK, Some(P1), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.at(2, P1, production(ProductionCommand::Train(PK)));
    t.at(2, P1, production(ProductionCommand::Train(PK)));
    t.run_until(10);
    assert_eq!(
        shown(t.sim()),
        ["Training Peacekeeper: 0%", "Waiting for Unit Control", "Queue: 2 / 20"]
    );
    t.debug_at(11, DebugCommand::Remove { id: ObjectId(3) });
    t.run_until(11);
    assert_eq!(shown(t.sim()), ["Training Peacekeeper: 1%", "Queue: 2 / 20"]);
}

/// Design change 0105: an own constructor shows its construction and its progress, then the building ready to
/// place. Player 1's Deployment Center (1) constructs a Generator (240 frames at full power: its own +30 power
/// covers it), which is ready after 240 frames.
#[test]
fn a_constructor_shows_its_construction_then_its_ready_building() {
    let dc = ObjectId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .start();
    let shown = |sim: &Sim| lines_of(sim, dc);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "idle");
    t.at(
        1,
        P1,
        Command::Construction {
            objects: vec![dc],
            command: ConstructionCommand::Construct(ObjectKind::Generator),
        },
    );
    t.run_until(1);
    assert_eq!(shown(t.sim()), ["Constructing Generator: 0%"]);
    t.run_until(121);
    assert_eq!(shown(t.sim()), ["Constructing Generator: 50%"]);
    t.run_until(400);
    assert_eq!(shown(t.sim()), ["Generator ready to place"]);
}

/// Slice M3-E: a Beacon warping in a building shows the building and the Warp In's progress, "frames elapsed /
/// WarpInTime" (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`) in whole percent rounded
/// down, and nothing else (design change 0116: functions start at completion); cancelled, a Beacon again, it
/// shows nothing. Player 1's Beacon (1) warps in a Robo Portal (4 × 4 tiles centred on it, WarpInTime 640 frames)
/// ordered in tick 1: at the end of tick 288, 288 frames are done, 45%.
#[test]
fn a_warp_in_shows_its_building_and_its_progress() {
    let beacon = ObjectId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .spawn(ObjectKind::Beacon, Some(P1), tile(8, 6))
        .start();
    let shown = |sim: &Sim| lines_of(sim, beacon);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "a Beacon has nothing to show");
    let construction = |command| Command::Construction {
        objects: vec![beacon],
        command,
    };
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let warp_in = ConstructionCommand::WarpIn {
        kind: ObjectKind::RoboPortal,
        orientation: Default::default(),
    };
    t.at(1, P1, construction(warp_in));
    t.run_until(1);
    assert_eq!(shown(t.sim()), ["Warping in: Robo Portal 0%"]);
    t.run_until(288);
    assert_eq!(shown(t.sim()), ["Warping in: Robo Portal 45%"]);
    t.at(289, P1, construction(ConstructionCommand::CancelTransform));
    t.run_until(289);
    assert_eq!(t.sim().object(beacon).unwrap().kind, ObjectKind::Beacon);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "a Beacon again");
}

/// Slice M4-F, `factions/syndicate/tunnel_menus.md > Expansions under construction`: "**The Tunnel meanwhile** shows
/// the progress". Player 1's Tunnel (1) builds a Hideout (400 frames) in its area from tick 1: its line names the
/// expansion and its site's progress, a quarter after 100 frames, until the Tunnel's X cancels it.
#[test]
fn a_tunnel_shows_the_expansion_it_builds_and_its_progress() {
    let tunnel = ObjectId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(8, 6))
        .start();
    let shown = |sim: &Sim| lines_of(sim, tunnel);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "no operation");
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let command = |command| Command::Tunnel { tunnel, command };
    let expand = TunnelCommand::Expand {
        kind: ObjectKind::Hideout,
        origin: TilePos::new(6, 4),
        orientation: BuildingOrientation::default(),
    };
    t.at(1, P1, command(expand));
    t.run_until(1);
    assert_eq!(shown(t.sim()), ["Building Hideout: 0%"]);
    t.run_until(101);
    assert_eq!(shown(t.sim()), ["Building Hideout: 25%"]);
    t.at(102, P1, command(TunnelCommand::Cancel));
    t.run_until(102);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "cancelled");
}

/// Slice M4-G (placeholder wording): Player 1's Tunnel (1) upgrades to Tier 2 (480 frames) from tick 1; its line
/// names the tier and the progress, a quarter after 120 frames, until the Tunnel's X cancels it.
#[test]
fn a_tunnel_shows_the_tier_it_upgrades_to_and_its_progress() {
    let tunnel = ObjectId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(10, 6))
        .start();
    let shown = |sim: &Sim| lines_of(sim, tunnel);
    let supplies = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, supplies);
    let command = |command| Command::Tunnel { tunnel, command };
    t.at(1, P1, command(TunnelCommand::Upgrade));
    t.run_until(1);
    assert_eq!(shown(t.sim()), ["Upgrading to Tier 2: 0%"]);
    t.run_until(121);
    assert_eq!(shown(t.sim()), ["Upgrading to Tier 2: 25%"]);
    t.at(122, P1, command(TunnelCommand::Cancel));
    t.run_until(122);
    assert_eq!(shown(t.sim()), Vec::<String>::new(), "cancelled");
}

/// Slice M5-C (placeholder wording; `factions/cults/structures/recruitment_hall.md > RecruitmentHallInstanceState`):
/// player 1's Recruitment Hall in the map's top right corner, its side B open, claims the 49 tiles of its area on the
/// map (**Placeholder (Q0099 point 1)**), so 49% and a capacity of 9.8, 9 (design change 0341: "Capacity is rounded
/// down everywhere"). From its first step its Recruit is in production at 49 tiles a frame, and counts (design change
/// 0342); out at 392 frames (192 × 100 / 49, rounded up), when its next Recruit starts at once: the Hall's usage is 2.
/// No queue line: a Hall has none to fill.
#[test]
fn a_recruitment_hall_shows_its_effectiveness_unit_control_and_recruit() {
    let hall = ObjectId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(20, 0))
        .start();
    let shown = |sim: &Sim| lines_of(sim, hall);
    assert_eq!(
        shown(t.sim()),
        ["Effectiveness: 49% (49/100 tiles)", "Unit Control: 0 / 9"]
    );
    t.run_until(196);
    assert_eq!(
        shown(t.sim()),
        [
            "Effectiveness: 49% (49/100 tiles)",
            "Unit Control: 1 / 9",
            "Recruiting Recruit: 50%"
        ]
    );
    let rally = ProductionCommand::SetRally(Target::Point(Vec2::new(900.0, 900.0)));
    t.at(
        197,
        P1,
        Command::Production {
            objects: vec![hall],
            command: rally,
        },
    );
    t.run_until(392);
    assert_eq!(
        shown(t.sim()),
        [
            "Effectiveness: 49% (49/100 tiles)",
            "Unit Control: 2 / 9",
            "Recruiting Recruit: 0%",
            "Rally point: ground"
        ]
    );
}
