use sc_sim::{ObjectId, TileProblem, WholeProblem};

use super::*;
use crate::editor::orientation::Turn;

fn ghost(placing: Placing, check: PlacementCheck) -> Ghost {
    Ghost::of(placing, TilePos::new(3, 2), check).unwrap()
}

/// Design change 0093: each tile shows its own validity, and a ghost invalid as a whole is red throughout; the
/// outline gives the verdict.
#[test]
fn tiles_are_green_or_red_by_their_rules() {
    let generator = Placing::new(ObjectId(1), ObjectKind::Generator);
    let valid = ghost(generator, PlacementCheck::default());
    assert_eq!(valid.tiles().count(), 4);
    assert!(valid.tiles().all(|t| valid.tile_is_green(t)));
    assert_eq!(valid.edge_color(), VALID_EDGE);
    let blocked = TilePos::new(4, 3);
    let one_tile = ghost(
        generator,
        PlacementCheck {
            tiles: vec![(blocked, TileProblem::Unit)],
            whole: Vec::new(),
        },
    );
    let red: Vec<TilePos> = one_tile.tiles().filter(|&t| !one_tile.tile_is_green(t)).collect();
    assert_eq!(red, [blocked]);
    assert_eq!(one_tile.tile_color(blocked), INVALID_FILL);
    assert_eq!(one_tile.edge_color(), INVALID_EDGE);
    let outside = ghost(
        generator,
        PlacementCheck {
            tiles: Vec::new(),
            whole: vec![WholeProblem::OutsideBuildArea],
        },
    );
    assert!(outside.tiles().all(|t| !outside.tile_is_green(t)));
}

/// Design change 0079: the Barracks (3 wide, 2 tall, ABAC) shows A on top, B on the right, A at the bottom and C
/// on the left; turned clockwise it is 2 wide and 3 tall, with B at the bottom. A plate's single tile has no
/// letters.
#[test]
fn side_letters_follow_the_turns() {
    let barracks = Placing::new(ObjectId(1), ObjectKind::Barracks);
    let letters = |g: &Ghost| g.labels().map(|l| l.map(|(text, _)| text));
    let plain = ghost(barracks, PlacementCheck::default());
    assert_eq!(plain.size, (3, 2));
    let some = |s: &str| Some(s.to_string());
    assert_eq!(
        letters(&plain),
        [some("A"), some("B"), some("A"), some("C"), some("BR")]
    );
    let turned = ghost(barracks.turned(Turn::Clockwise), PlacementCheck::default());
    assert_eq!(turned.size, (2, 3));
    assert_eq!(
        letters(&turned),
        [some("C"), some("A"), some("B"), some("A"), some("BR")]
    );
    let flipped = ghost(barracks.turned(Turn::FlipHorizontal), PlacementCheck::default());
    assert_eq!(
        letters(&flipped),
        [some("A"), some("C"), some("A"), some("B"), some("BR")]
    );
    let plate = ghost(
        Placing::new(ObjectId(1), ObjectKind::ExtractionPlate),
        PlacementCheck::default(),
    );
    assert_eq!(letters(&plate), [None, None, None, None, some("EP")]);
    // The letters stand inside their edges: north on top, east on the right.
    let labels = plain.labels();
    let (north, east) = (labels[0].clone().unwrap().1, labels[1].clone().unwrap().1);
    assert!(north.y < plain.center().y && east.x > plain.center().x);
}

/// A flat tile is a quad, a sloped one four triangles; the outline is a quad per tile edge around the footprint.
#[test]
fn the_mesh_lies_on_the_terrain() {
    // Only corner (6, 4) is raised: tile (5, 3) slopes.
    let map = Map::from_ascii("......\n......\n......\n......\n\n0000000\n0000000\n0000000\n0000000\n0000001").unwrap();
    let generator = Placing::new(ObjectId(1), ObjectKind::Generator);
    let flat = ghost(generator, PlacementCheck::default());
    // 4 quads, then 8 tile edges.
    assert_eq!(ghost_mesh(&map, &flat).vertex_count(), 4 * 4 + 8 * 4);
    // At (4, 2) its south-east tile (5, 3) slopes.
    let sloped = Ghost::of(generator, TilePos::new(4, 2), PlacementCheck::default()).unwrap();
    assert_eq!(ghost_mesh(&map, &sloped).vertex_count(), 3 * 4 + 4 * 3 + 8 * 4);
}

/// Design change 0119: on the Warp In ghost, "a tile that isn't [Visible] is red", like any tile that breaks a rule.
/// The Beacon's own sight always covers its footprint (4 grid units; the farthest tile's centre is about 2.1 away),
/// so a game never shows it: the sim's `a_warp_in_needs_every_tile_visible` checks the rule itself.
#[test]
fn a_warp_in_ghost_shows_a_tile_out_of_sight_red() {
    let warp_in = Placing::warp_in(ObjectId(1), ObjectKind::RoboPortal);
    let hidden = TilePos::new(3, 2);
    let check = PlacementCheck {
        tiles: vec![(hidden, TileProblem::NotVisible)],
        whole: Vec::new(),
    };
    let shown = ghost(warp_in, check);
    assert_eq!(shown.size, (4, 4));
    let red: Vec<TilePos> = shown.tiles().filter(|&t| !shown.tile_is_green(t)).collect();
    assert_eq!(red, [hidden]);
    assert_eq!(shown.edge_color(), INVALID_EDGE);
}

/// A Warp In's ghost covers its Beacon, whose tag shows at the Beacon's centre: the ghost's own tag stands a little
/// south of the middle, clear of it, and north of the south side's letter; any other ghost's in the middle.
#[test]
fn a_warp_in_ghosts_tag_clears_the_beacons() {
    let tag = |g: &Ghost| g.labels()[4].clone().expect("a tag").1;
    let unchecked = PlacementCheck::default;
    let node = ghost(Placing::warp_in(ObjectId(1), ObjectKind::EgressNode), unchecked());
    let middle = TilePos::new(3, 2).corner() + Vec2::splat(SU_PER_GU);
    assert_eq!(tag(&node), middle + Vec2::new(0.0, TAG_BELOW_SU));
    let (tag_top, tag_bottom) = (tag(&node).y - TAG_SU / 2.0, tag(&node).y + TAG_SU / 2.0);
    let south = node.labels()[2].clone().expect("a side letter").1.y;
    assert!(tag_bottom <= south - SIDE_SU / 2.0, "clear of the south letter");
    assert!(tag_top >= middle.y + TAG_SU / 2.0, "clear of the Beacon's tag");
    let generator = ghost(Placing::new(ObjectId(1), ObjectKind::Generator), unchecked());
    assert_eq!(tag(&generator), middle);
}

/// [`check_now`] asks each builder's own check: a Warp In the Beacon's (`Sim::warp_in_check`); the Drones' the one of
/// the Drone the click would go to (`Sim::deploy_check`, design changes 0120 and 0125). Drone 3 is ordered to deploy
/// at `site`: a plain click there would go to it again and replace its order, green; a Shift-click to Drone 4, the
/// next without a Deploy, red over Drone 3's site; with Drone 4's Deploy sent but not yet applied (`unsent`), to none
/// of them, the ghost asking for the first one, its site reserved. Nothing for a selection whose ActiveGroup isn't
/// Drones.
#[test]
fn each_builder_asks_its_own_check() {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{Command, Faction, OrderKind, PlayerCommand, PlayerId, SpawnAt};

    use crate::selection::Selection;
    let p1 = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(p1, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(p1), SpawnAt::Tile(TilePos::new(1, 1)))
        .spawn(ObjectKind::Beacon, Some(p1), SpawnAt::Tile(TilePos::new(10, 6)))
        .spawn(
            ObjectKind::Drone,
            Some(p1),
            SpawnAt::Point(TilePos::new(4, 12).center()),
        )
        .spawn(
            ObjectKind::Drone,
            Some(p1),
            SpawnAt::Point(TilePos::new(6, 12).center()),
        )
        .start();
    let (beacon, drones) = (ObjectId(2), [ObjectId(3), ObjectId(4)]);
    let site = TilePos::new(16, 3);
    let deploy = |drone: ObjectId, queue: bool| Command::Order {
        units: vec![drone],
        kind: OrderKind::Deploy { origin: site },
        queue,
    };
    t.at(1, p1, deploy(drones[0], false));
    t.run_until(1);
    let sim = t.sim();
    assert_eq!(sim.pending_builds(p1).len(), 1);

    let robo = Placing::warp_in(beacon, ObjectKind::RoboPortal);
    let on_beacon = Selection::new([(beacon, ObjectKind::Beacon)]);
    let origin = TilePos::new(9, 5);
    let check = check_now(sim, p1, &on_beacon, robo, origin, false, &[]);
    assert_eq!(
        check,
        sim.warp_in_check(beacon, ObjectKind::RoboPortal, robo.orientation)
    );
    assert!(check.is_some_and(|c| c.is_valid()));

    let group = Selection::new(drones.map(|d| (d, ObjectKind::Drone)));
    let placing = Placing::deploy();
    let reserved = |check: Option<PlacementCheck>| {
        let check = check.expect("a check");
        check.tiles.iter().filter(|(_, p)| *p == TileProblem::Reserved).count()
    };
    assert_eq!(reserved(check_now(sim, p1, &group, placing, site, false, &[])), 0);
    assert_eq!(reserved(check_now(sim, p1, &group, placing, site, true, &[])), 4);
    let unsent = [PlayerCommand {
        player: p1,
        command: deploy(drones[1], true),
    }];
    assert_eq!(reserved(check_now(sim, p1, &group, placing, site, true, &unsent)), 4);
    assert_eq!(check_now(sim, p1, &on_beacon, placing, site, false, &[]), None);
}
