//! The Recruitment Area display (slice M5-J2; `factions/cults/structures/recruitment_hall.md > Recruitment Area
//! display`): the own pending Halls in their order of placement, and the mesh. What the player knows of the other
//! players' Halls, and the overlay in a game, are the client tests' (`crates/game/tests/commands/recruitment_areas.rs`).

use bevy::math::Vec2;
use sc_sim::defs::Footprint;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{BuildingOrientation, Command, DebugCommand, Faction, ObjectKind, OrderKind, SpawnAt};

use super::*;

const P1: PlayerId = PlayerId(1);
const HALL: ObjectKind = ObjectKind::RecruitmentHall;

fn build(recruit: u32, x: i32, y: i32) -> Command {
    Command::Order {
        units: vec![ObjectId(recruit)],
        kind: OrderKind::Build {
            kind: HALL,
            origin: TilePos::new(x, y),
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    }
}

/// A flat 64 × 48 map: player 1's Recruits 1 and 2 in the west, and 3 beside the site of a Hall at (30, 20), which it
/// is ordered to build first, in tick 2; then Recruit 2 a Hall at (10, 30), in tick 3, and Recruit 1 one at (40, 30),
/// in tick 4. Plenty of Space Crystals.
fn field() -> TestSim {
    let map = vec![".".repeat(64); 48].join("\n");
    let point = |x: f32, y: f32| SpawnAt::Point(Vec2::new(x, y) * 64.0);
    let mut t = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Recruit, Some(P1), point(4.5, 4.5))
        .spawn(ObjectKind::Recruit, Some(P1), point(5.5, 4.5))
        .spawn(ObjectKind::Recruit, Some(P1), point(29.5, 21.0))
        .start();
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 5000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, crystals)
        .at(2, P1, build(3, 30, 20))
        .at(3, P1, build(2, 10, 30))
        .at(4, P1, build(1, 40, 30));
    t
}

/// Design change 0351: "The preview treats own pending Halls as claimants, in their order of placement": the sites
/// first, in the order they started, then the ghosts in the order they were placed (`sc_sim::PendingBuild::group`),
/// not their workers' (the sim lists the ghosts by worker); those of the Recruits the click replaces don't count.
#[test]
fn the_pending_halls_claim_in_their_order_of_placement() {
    let mut t = field();
    for _ in 0..200 {
        if t.sim().objects().any(|o| o.is_site()) {
            break;
        }
        t.step();
    }
    let site = t.sim().objects().find(|o| o.is_site()).expect("Recruit 3's site").id;
    let sight = Sight::new(t.sim(), P1, true);
    let pending = |replaced: &[ObjectId]| {
        let claimants = known_claimants(t.sim(), &sight, replaced);
        let ranked = claimants.iter().map(|c| (c.id, c.placement.origin, c.claimer, c.rank));
        ranked.collect::<Vec<_>>()
    };
    let at = TilePos::new;
    let ghosts = [at(10, 30), at(40, 30)];
    let in_order = |sim: &Sim| sim.pending_builds(P1).iter().map(|p| p.site.origin).collect::<Vec<_>>();
    assert_eq!(in_order(t.sim()), [ghosts[1], ghosts[0]], "by worker");
    assert_eq!(
        pending(&[]),
        [
            (Some(site), at(30, 20), Claimer::OwnPending, Rank::Pending(0)),
            (None, ghosts[0], Claimer::OwnPending, Rank::Pending(1)),
            (None, ghosts[1], Claimer::OwnPending, Rank::Pending(2)),
        ]
    );
    assert_eq!(
        pending(&[ObjectId(1)]),
        [
            (Some(site), at(30, 20), Claimer::OwnPending, Rank::Pending(0)),
            (None, ghosts[0], Claimer::OwnPending, Rank::Pending(1)),
        ],
        "Recruit 1 placing without Shift"
    );
}

/// The mesh: each tile of the previewed area tinted, and the area's outline, on the map's tiles drawn (a 10 × 10 area
/// is 100 tiles and 40 tile edges, 4 vertices each); a selected Hall's claimed tiles the same way. In the corner, the
/// 49 tiles on the map and 14 edges, to the east and the south; none on the tiles not drawn.
#[test]
fn the_mesh_tints_each_tile_within_the_outline() {
    let map = Map::new(40, 40);
    let shown = |x: i32, y: i32| {
        let placement = Placement::new(TilePos::new(x, y), Footprint::Tiles { w: 4, h: 4 });
        let preview = Preview::of(HALL, placement, &[], &map, |_| true).expect("a Hall");
        ShownPreview { placement, preview }
    };
    let all = |_: TilePos| true;
    let one = (100 + 40) * 4;
    assert_eq!(overlay_mesh(&map, &[], Some(&shown(10, 10)), all).vertex_count(), one);
    assert_eq!(
        overlay_mesh(&map, &[], Some(&shown(0, 0)), all).vertex_count(),
        (49 + 14) * 4
    );
    let claims = SelectedClaims {
        hall: ObjectId(1),
        tiles: shown(20, 20).preview.tiles.iter().map(|&(t, _)| t).collect(),
    };
    let both = overlay_mesh(&map, std::slice::from_ref(&claims), Some(&shown(10, 10)), all);
    assert_eq!(both.vertex_count(), 2 * one);
    assert_eq!(
        overlay_mesh(&map, &[claims], Some(&shown(10, 10)), |_| false).vertex_count(),
        0
    );
    assert_eq!(overlay_mesh(&map, &[], None, all).vertex_count(), 0);
    let label = shown(0, 0).label();
    assert_eq!(label, "Effectiveness: 49%");
}
