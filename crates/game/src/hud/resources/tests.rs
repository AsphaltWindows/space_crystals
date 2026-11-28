use sc_sim::testing::ScenarioBuilder;
use sc_sim::{ObjectKind, SpawnAt, TilePos};

use super::*;

const P1: PlayerId = PlayerId(1);

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// The texts of `player`'s bar, a warning marked with `!`.
fn texts(sim: &Sim, player: PlayerId) -> Vec<String> {
    resource_bar(sim, player)
        .into_iter()
        .map(|(_, r)| if r.alert { format!("{}!", r.text) } else { r.text })
        .collect()
}

/// `factions/gdo/overview.md > DisplayHud` on the starting base of `factions/gdo/economy.md > Starting Conditions`
/// (design change 0090's first verification: "Power 21/30"; Supplies 1),
/// then with a Barracks before any Generator (its second: "51/30", in red). The banks start at the Starting
/// Conditions' 100 Space Crystals (slice M2-F1).
#[test]
fn the_gdo_bar_reads_the_banks_power_and_unit_control() {
    let base = || {
        ScenarioBuilder::new(&vec![".".repeat(16); 10].join("\n"))
            .spawn(ObjectKind::CrystalPatch, None, tile(1, 7))
            .spawn(ObjectKind::CrystalPatch, None, tile(3, 7))
            .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
            .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1))
            .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 7))
            .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 7))
    };
    let sim = Sim::new(&base().build());
    let figures: Vec<ResourceFigure> = resource_bar(&sim, P1).into_iter().map(|(f, _)| f).collect();
    use ResourceFigure::{Power, SpaceCrystals, Supplies, UnitControl};
    assert_eq!(figures, [SpaceCrystals, Supplies, Power, UnitControl]);
    assert_eq!(
        texts(&sim, P1),
        ["Space Crystals 100", "Supplies 1", "Power 21/30", "Unit Control 0/200"]
    );
    let all_in = base().spawn(ObjectKind::Barracks, Some(P1), tile(10, 1)).spawn(
        ObjectKind::Peacekeeper,
        Some(P1),
        SpawnAt::Point(Vec2::new(800.0, 500.0)),
    );
    let sim = Sim::new(&all_in.build());
    assert_eq!(
        texts(&sim, P1),
        ["Space Crystals 100", "Supplies 1", "Power 51/30!", "Unit Control 1/200"]
    );
}

/// Power exactly used up isn't in the red (design change 0090: red "when used exceeds available"), and a player
/// without structures has nothing: 0/0.
#[test]
fn power_is_red_only_when_consumption_exceeds_generation() {
    let sim = |kinds: &[(ObjectKind, i32)]| {
        let mut builder = ScenarioBuilder::new(&vec![".".repeat(20); 8].join("\n"));
        for &(kind, x) in kinds {
            builder = builder.spawn(kind, Some(P1), tile(x, 1));
        }
        Sim::new(&builder.build())
    };
    let power = |sim: &Sim| texts(sim, P1)[2].clone();
    assert_eq!(power(&sim(&[])), "Power 0/0");
    assert_eq!(power(&sim(&[(ObjectKind::Generator, 1)])), "Power 0/20");
    // A Barracks (-30) on a Deployment Center (+30): all of it used, not more.
    let even = sim(&[(ObjectKind::DeploymentCenter, 1), (ObjectKind::Barracks, 6)]);
    assert_eq!(power(&even), "Power 30/30");
    let short = sim(&[(ObjectKind::Generator, 1), (ObjectKind::Barracks, 6)]);
    assert_eq!(power(&short), "Power 30/20!");
}

/// `factions/colonists/overview.md > DisplayHud` on the Colonists' starting base
/// (`factions/colonists/economy.md > Starting Conditions`; the plan's acceptance for slice M3-C: "Space
/// Crystals 100 ... Beacon Capacity 4/10", in ASCII): the four banks, the refined ones at 0, and Beacon Capacity
/// as used / available, with no Power. The refined banks show what `SetResources` sets, the capacity follows the
/// Drones in production and not those only queued (design change 0143, answering Q0041 point 1), and over
/// capacity it shows in red.
#[test]
fn the_colonists_bar_reads_their_banks_and_beacon_capacity() {
    use ResourceFigure::{Alloys, BeaconCapacity, Essence, Flux, SpaceCrystals};
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(16); 10].join("\n"))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 1))
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(6, 1));
    for x in 1..=4 {
        let at = TilePos::new(x, 8).center();
        builder = builder.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    let figures: Vec<ResourceFigure> = resource_bar(t.sim(), P1).into_iter().map(|(f, _)| f).collect();
    assert_eq!(figures, [SpaceCrystals, Alloys, Essence, Flux, BeaconCapacity]);
    let start = texts(t.sim(), P1);
    assert_eq!(
        start,
        [
            "Space Crystals 100",
            "Alloys 0",
            "Essence 0",
            "Flux 0",
            "Beacon Capacity 4/10"
        ]
    );
    assert!(start.iter().all(|text| text.is_ascii()));
    t.debug_at(
        1,
        sc_sim::DebugCommand::SetResources {
            player: P1,
            crystals: 500,
            supplies: 0,
            alloys: 3,
            essence: 4,
            flux: 5,
        },
    );
    let train = sc_sim::ProductionCommand::Train(ObjectKind::Drone);
    let portal = sc_sim::ObjectId(2);
    t.at(
        2,
        P1,
        sc_sim::Command::Production {
            objects: vec![portal],
            command: train,
        },
    );
    t.run(2);
    assert_eq!(
        texts(t.sim(), P1),
        [
            "Space Crystals 450",
            "Alloys 3",
            "Essence 4",
            "Flux 5",
            "Beacon Capacity 5/10"
        ]
    );
    // Design change 0143: the Drone in production counts, a second one only queued doesn't; with two more Drones,
    // 7 of 10. Over capacity (`factions/colonists/overview.md > Beacon Capacity > Over capacity`, design change
    // 0112): the Egress Node lost leaves 7 used of 5, in red.
    t.at(
        3,
        P1,
        sc_sim::Command::Production {
            objects: vec![portal],
            command: train,
        },
    );
    for x in [5, 6] {
        t.spawn_unit_at(3, ObjectKind::Drone, P1, TilePos::new(x, 8).center());
    }
    t.run(1);
    assert_eq!(texts(t.sim(), P1)[4], "Beacon Capacity 7/10");
    t.debug_at(
        4,
        sc_sim::DebugCommand::Remove {
            id: sc_sim::ObjectId(1),
        },
    );
    t.run(1);
    assert_eq!(texts(t.sim(), P1)[4], "Beacon Capacity 7/5!");
}

/// `factions/syndicate/overview.md > DisplayHud` on the Syndicate's starting objects (slice M4-B,
/// `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`): Space Crystals 100, Supplies 1, and Tunnel
/// Space 1/11, the Tunnel's 48 tunnel-region cells less the Hideout's 4 over 4, of which the Agent uses 1. Over
/// capacity (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: "usage is simply displayed over capacity
/// (e.g. 12/10)"): with a second Hideout and 11 more Agents, 12/10, in red.
#[test]
fn the_syndicate_bar_reads_their_banks_and_tunnel_space() {
    use ResourceFigure::{SpaceCrystals, Supplies, TunnelSpace};
    let agent = |x: i32| SpawnAt::Point(TilePos::new(x, 12).center());
    let base = || {
        ScenarioBuilder::new(&vec![".".repeat(16); 14].join("\n"))
            .faction(P1, Faction::Syndicate)
            .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 4))
            .spawn(ObjectKind::Hideout, Some(P1), tile(8, 8))
            .spawn(ObjectKind::Agent, Some(P1), agent(1))
    };
    let sim = Sim::new(&base().build());
    let figures: Vec<ResourceFigure> = resource_bar(&sim, P1).into_iter().map(|(f, _)| f).collect();
    assert_eq!(figures, [SpaceCrystals, Supplies, TunnelSpace]);
    assert_eq!(
        texts(&sim, P1),
        ["Space Crystals 100", "Supplies 1", "Tunnel Space 1/11"]
    );
    let mut full = base().spawn(ObjectKind::Hideout, Some(P1), tile(2, 2));
    for x in 2..13 {
        full = full.spawn(ObjectKind::Agent, Some(P1), agent(x));
    }
    let sim = Sim::new(&full.build());
    assert_eq!(texts(&sim, P1)[2], "Tunnel Space 12/10!");
}

/// `factions/cults/overview.md > DisplayHud` (slice M5-C): Space Crystals, and Unit Control as used / available
/// (`factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`). A Recruitment Hall on open Plane
/// provides 20; two map Recruits, no Hall's, are orphaned points (design change 0344: "Recruits no Hall produced
/// (custom maps, debug) are orphans from the start"), and the Hall's Recruits count from the start of their production
/// (design change 0342). The Hall lost (a Storage keeps the player in the game), its capacity goes, and so does its
/// Recruit in production, while the one out stays, over capacity and not in red. A player not in the game shows
/// nothing.
#[test]
fn the_cults_bar_reads_their_bank_and_unit_control() {
    use ResourceFigure::{CultsUnitControl, SpaceCrystals};
    let recruit = |x: i32| SpawnAt::Point(TilePos::new(x, 20).center());
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(PlayerId(2), Faction::Cults)
        .spawn(ObjectKind::RecruitmentHall, Some(PlayerId(2)), tile(10, 8))
        .spawn(ObjectKind::Recruit, Some(PlayerId(2)), recruit(2))
        .spawn(ObjectKind::Recruit, Some(PlayerId(2)), recruit(4))
        // A structure left when the Hall is lost: the player stays in the game.
        .spawn(ObjectKind::Storage, Some(PlayerId(2)), tile(1, 1))
        .start();
    let figures: Vec<ResourceFigure> = resource_bar(t.sim(), PlayerId(2)).into_iter().map(|(f, _)| f).collect();
    assert_eq!(figures, [SpaceCrystals, CultsUnitControl]);
    assert_eq!(texts(t.sim(), PlayerId(2)), ["Space Crystals 0", "Unit Control 2/20"]);
    t.run_until(191);
    assert_eq!(
        texts(t.sim(), PlayerId(2))[1],
        "Unit Control 3/20",
        "the Recruit in production counts"
    );
    t.run(1);
    assert_eq!(
        texts(t.sim(), PlayerId(2))[1],
        "Unit Control 4/20",
        "one out, the next in production"
    );
    t.debug_at(
        193,
        sc_sim::DebugCommand::Remove {
            id: sc_sim::ObjectId(1),
        },
    );
    t.run(1);
    assert_eq!(texts(t.sim(), PlayerId(2))[1], "Unit Control 3/0");
    assert!(texts(t.sim(), PlayerId(3)).is_empty());
}

/// Design change 0342's Verification: "At 19/20 with a Recruit in production, the HUD shows 20/20". A Hall on open
/// Plane has made 19 Recruits by tick 3648, when its 20th starts: the bar reads 20/20, as it does once that one is out.
#[test]
fn at_19_with_a_recruit_in_production_the_bar_reads_20_of_20() {
    let recruits = |t: &sc_sim::testing::TestSim| t.sim().objects().filter(|o| o.kind == ObjectKind::Recruit).count();
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(PlayerId(2), Faction::Cults)
        .spawn(ObjectKind::RecruitmentHall, Some(PlayerId(2)), tile(10, 8))
        .start();
    t.run_until(19 * 192 - 1);
    assert_eq!(
        (recruits(&t), texts(t.sim(), PlayerId(2))[1].as_str()),
        (18, "Unit Control 19/20")
    );
    t.run(2);
    assert_eq!(
        (recruits(&t), texts(t.sim(), PlayerId(2))[1].as_str()),
        (19, "Unit Control 20/20")
    );
    t.run_until(20 * 192);
    assert_eq!(
        (recruits(&t), texts(t.sim(), PlayerId(2))[1].as_str()),
        (20, "Unit Control 20/20")
    );
}
