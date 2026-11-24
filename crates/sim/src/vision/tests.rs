use super::*;
use crate::defs::ObjectKind;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn unit(id: u32, kind: ObjectKind, owner: Option<PlayerId>, pos: Vec2) -> Object {
    Object {
        id: ObjectId(id),
        kind,
        owner,
        pos,
        prev_pos: pos,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        placement: None,
        resource: None,
        orders: None,
        mover: None,
        attack: None,
    }
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn visible(vision: &Vision, player: PlayerId) -> usize {
    vision
        .player(player)
        .unwrap()
        .states()
        .filter(|&s| s == VisibilityState::Visible)
        .count()
}

/// A box covers the tiles it overlaps: touching an edge isn't overlapping, a box of no width covers the tile it
/// lies in, and the span is clamped to the map.
#[test]
fn tile_spans() {
    assert_eq!(tile_span(64.0, 128.0, 10), (1, 2));
    assert_eq!(tile_span(63.0, 128.0, 10), (0, 2));
    assert_eq!(tile_span(64.0, 129.0, 10), (1, 3));
    assert_eq!(tile_span(80.0, 80.0, 10), (1, 2));
    assert_eq!(tile_span(64.0, 64.0, 10), (1, 2));
    assert_eq!(tile_span(-10.0, 10.0, 10), (0, 1));
    assert_eq!(tile_span(-100.0, -50.0, 10), (0, 0));
    assert_eq!(tile_span(700.0, 800.0, 10), (10, 10));
}

/// A disc of radius 5 around a tile centre holds the 81 tiles whose centres are within 5 grid units: the lattice
/// points with dx² + dy² ≤ 25, the 3-4-5 ones on the boundary included.
#[test]
fn a_disc_of_tiles() {
    let map = Map::new(20, 20);
    let mut vision = Vision::new(&map, [P1]);
    vision.update(&map, [&unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10))]);
    assert_eq!(visible(&vision, P1), 81);
    let state = |x, y| vision.state(P1, TilePos::new(x, y));
    assert_eq!(state(13, 14), VisibilityState::Visible);
    assert_eq!(state(14, 14), VisibilityState::Unexplored);
    assert_eq!(state(15, 10), VisibilityState::Visible);
    assert_eq!(state(16, 10), VisibilityState::Unexplored);
}

/// Air units are exempt from the ElevationModifier (`core/vision.md > ElevationModifier`): no M1 unit flies, so
/// this stamps an exempt eye directly. From the same spot a ground eye on level 1 sees one grid unit further over
/// level 0 and one less up to level 2.
#[test]
fn an_exempt_eye_sees_its_sight_range_over_any_ground() {
    // Columns 0–9 at level 0, 10–19 at level 1, 20–29 at level 2.
    let levels: Vec<u8> = (0..30 * 3).map(|i| ((i % 30) / 10) as u8).collect();
    let row = |eye: &Eye| {
        let mut tiles = Vec::new();
        stamp(eye, 30, 3, &levels, &mut tiles);
        let row: Vec<u32> = tiles.into_iter().filter(|i| i / 30 == 1).map(|i| i % 30).collect();
        (row[0], row[row.len() - 1])
    };
    let mut eye = Eye {
        player: 0,
        at: centre(15, 1),
        sight: 5.0,
        elevation: Elevation::Exempt,
    };
    assert_eq!(row(&eye), (10, 20));
    eye.elevation = Elevation::Level(1);
    assert_eq!(row(&eye), (9, 19), "6 over level 0, 4 up to level 2");
}

/// The update changes what differs, and the revision only when a tile's state does.
#[test]
fn updates_change_only_what_moved() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let revision = |v: &Vision, p| v.player(p).unwrap().revision();
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    vision.update(&map, [&scout]);
    assert_eq!((revision(&vision, P1), revision(&vision, P2)), (1, 0));
    // Turning changes nothing.
    scout.facing = 1.0;
    vision.update(&map, [&scout]);
    assert_eq!(revision(&vision, P1), 1);
    // 10 su east: the tiles exactly on the boundary to the north, west and south drop out.
    scout.pos += Vec2::new(10.0, 0.0);
    vision.update(&map, [&scout]);
    assert_eq!((revision(&vision, P1), visible(&vision, P1)), (2, 76));
    // A step that sees the same tiles changes nothing.
    scout.pos += Vec2::new(1.0, 0.0);
    vision.update(&map, [&scout]);
    assert_eq!((revision(&vision, P1), visible(&vision, P1)), (2, 76));
    // On the next tile's centre: the west edge is Explored, the east edge Visible.
    scout.pos = centre(11, 10);
    vision.update(&map, [&scout]);
    assert_eq!((revision(&vision, P1), visible(&vision, P1)), (3, 81));
    assert_eq!(vision.state(P1, TilePos::new(5, 10)), VisibilityState::Explored);
    assert_eq!(vision.state(P1, TilePos::new(16, 10)), VisibilityState::Visible);
    vision.check(&map, [&scout]).unwrap();
    // Gone: everything it saw is Explored.
    vision.update(&map, []);
    assert_eq!(visible(&vision, P1), 0);
    assert_eq!(vision.state(P1, TilePos::new(11, 10)), VisibilityState::Explored);
    assert_eq!(revision(&vision, P1), 4);
    assert_eq!(revision(&vision, P2), 0, "the other player saw nothing change");
}

/// Two sources on one tile count twice: it stays Visible while either sees it.
#[test]
fn overlapping_sources_count_each() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1]);
    let a = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    let b = unit(2, ObjectKind::Guard, Some(P1), centre(14, 10));
    vision.update(&map, [&a, &b]);
    vision.update(&map, [&b]);
    assert_eq!(vision.state(P1, TilePos::new(12, 10)), VisibilityState::Visible);
    assert_eq!(vision.state(P1, TilePos::new(8, 10)), VisibilityState::Explored);
    vision.check(&map, [&b]).unwrap();
}

/// `core/entities.md > Object Type > SightRange`: vision goes to the owner. A neutral unit and a resource (sight 0)
/// see nothing; a unit that changes hands takes its sight to the new owner.
#[test]
fn only_owned_objects_with_sight_give_vision() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let neutral = unit(1, ObjectKind::Guard, None, centre(5, 5));
    let patch = unit(2, ObjectKind::CrystalPatch, Some(P1), centre(20, 5));
    let mut turncoat = unit(3, ObjectKind::Peacekeeper, Some(P1), centre(15, 12));
    vision.update(&map, [&neutral, &patch, &turncoat]);
    assert_eq!(visible(&vision, P1), 81);
    turncoat.owner = Some(P2);
    vision.update(&map, [&neutral, &patch, &turncoat]);
    assert_eq!((visible(&vision, P1), visible(&vision, P2)), (0, 81));
    vision.check(&map, [&neutral, &patch, &turncoat]).unwrap();
}

/// V1 catches a count that drifted from the objects, and a Visible tile that isn't Explored.
#[test]
fn the_check_finds_drift() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1]);
    let scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    vision.update(&map, [&scout]);
    let mut drifted = vision.clone();
    drifted.players[0].seers[0] = 1;
    let (at, message) = drifted.check(&map, [&scout]).unwrap_err();
    assert_eq!(at, centre(0, 0));
    assert!(message.contains("fresh count"), "{message}");
    let mut forgotten = vision.clone();
    forgotten.players[0].explored = TileBits::new(30 * 20);
    let (_, message) = forgotten.check(&map, [&scout]).unwrap_err();
    assert!(message.contains("not Explored"), "{message}");
}

/// A save keeps each player's Explored tiles as rows of symbols, and loading rebuilds the rest from the objects.
#[test]
fn explored_rows_round_trip() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(6, 6));
    vision.update(&map, [&scout]);
    scout.pos = centre(20, 12);
    vision.update(&map, [&scout]);
    let rows = vision.explored_rows();
    assert_eq!(rows[0].explored.len(), 20);
    assert_eq!(&rows[0].explored[6][..13], ".###########.");
    assert!(rows[1].explored.iter().all(|r| !r.contains(EXPLORED_SYMBOL)));
    let loaded = load(&map, &[P1, P2], rows.clone(), [&scout]).unwrap();
    assert_eq!(loaded, vision);
    assert_eq!(loaded.state(P1, TilePos::new(6, 6)), VisibilityState::Explored);
    assert_eq!(loaded.state(P1, TilePos::new(20, 12)), VisibilityState::Visible);
    loaded.check(&map, [&scout]).unwrap();

    // Tiles Visible now are Explored, even if the save forgot them.
    let mut forgetful = rows.clone();
    forgetful[0].explored = vec![UNEXPLORED_SYMBOL.to_string().repeat(30); 20];
    let loaded = load(&map, &[P1, P2], forgetful, [&scout]).unwrap();
    assert_eq!(loaded.state(P1, TilePos::new(20, 12)), VisibilityState::Visible);
    loaded.check(&map, [&scout]).unwrap();

    assert!(
        load(&map, &[P2, P1], rows.clone(), [&scout]).is_err(),
        "players out of order"
    );
    assert!(load(&map, &[P1], rows.clone(), [&scout]).is_err(), "a player too many");
    let mut short = rows.clone();
    short[1].explored.pop();
    assert!(load(&map, &[P1, P2], short, [&scout]).is_err());
    let mut odd = rows;
    odd[1].explored[0].replace_range(0..1, "?");
    assert!(load(&map, &[P1, P2], odd, [&scout]).is_err());
}
