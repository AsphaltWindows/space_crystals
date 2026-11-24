use super::*;
use crate::map::Map;
use crate::vision::{VisibilityState, Vision};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn object(id: u32, kind: ObjectKind, owner: Option<PlayerId>, pos: Vec2) -> Object {
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

/// A Peacekeeper (SightRange 5) of `owner` on tile `(x, y)`.
fn scout(id: u32, owner: PlayerId, x: i32, y: i32) -> Object {
    object(id, ObjectKind::Peacekeeper, Some(owner), TilePos::new(x, y).center())
}

/// A resource of `kind` whose footprint starts at tile `(x, y)`.
fn resource(id: u32, kind: ObjectKind, x: i32, y: i32, state: ResourceState) -> Object {
    let placement = Placement::new(TilePos::new(x, y), kind.def().footprint);
    let mut o = object(id, kind, None, placement.center());
    o.placement = Some(placement);
    o.resource = Some(state);
    o
}

fn patch(id: u32, x: i32, y: i32, remaining: u32) -> Object {
    resource(
        id,
        ObjectKind::CrystalPatch,
        x,
        y,
        ResourceState::CrystalPatch { remaining },
    )
}

fn remaining(record: &LastKnown) -> u32 {
    match record.resource {
        Some(ResourceState::CrystalPatch { remaining }) => remaining,
        _ => panic!("not a patch: {record:?}"),
    }
}

/// The end of tick `tick` over `objects` (in id order): phase 5f, then the records; and V1.
fn end_tick(vision: &mut Vision, map: &Map, tick: Tick, objects: &[&Object]) {
    vision.update(map, objects.iter().copied());
    vision.remember(tick, |id| objects.iter().copied().find(|o| o.id == id));
    vision.check(map, objects.iter().copied(), tick).unwrap();
}

fn p1(vision: &Vision) -> &PlayerVision {
    vision.player(P1).unwrap()
}

/// A record is made when the player first sees a resource, takes its state every tick it stays in sight, and keeps
/// the last one once it is out of sight.
#[test]
fn records_follow_what_the_player_sees() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let mut eye = scout(1, P1, 10, 10);
    let mut crystals = patch(2, 13, 10, 100);
    end_tick(&mut vision, &map, 1, &[&eye, &crystals]);
    let record = p1(&vision).record(ObjectId(2)).unwrap();
    assert_eq!((remaining(record), record.seen, record.pos()), (100, 1, crystals.pos));
    assert_eq!(
        p1(&vision).remembered().count(),
        0,
        "in sight: the views show the object itself"
    );
    assert_eq!(vision.player(P2).unwrap().records().count(), 0);

    crystals.resource = Some(ResourceState::CrystalPatch { remaining: 90 });
    end_tick(&mut vision, &map, 2, &[&eye, &crystals]);
    let record = p1(&vision).record(ObjectId(2)).unwrap();
    assert_eq!((remaining(record), record.seen), (90, 2));

    // Out of sight: the record keeps tick 2, whatever happens to the patch.
    eye.pos = TilePos::new(30, 10).center();
    end_tick(&mut vision, &map, 3, &[&eye, &crystals]);
    crystals.resource = Some(ResourceState::CrystalPatch { remaining: 50 });
    end_tick(&mut vision, &map, 4, &[&eye, &crystals]);
    let record = p1(&vision).remembered_record(ObjectId(2)).unwrap();
    assert_eq!((remaining(record), record.seen), (90, 2));
    assert_eq!(p1(&vision).remembered().count(), 1);

    eye.pos = TilePos::new(10, 10).center();
    end_tick(&mut vision, &map, 5, &[&eye, &crystals]);
    let record = p1(&vision).record(ObjectId(2)).unwrap();
    assert_eq!((remaining(record), record.seen), (50, 5));
    assert!(p1(&vision).remembered_record(ObjectId(2)).is_none());
}

/// `core/vision.md > Explored`: a resource that disappears out of sight stays remembered, footprint and all, until
/// the player sees a tile of it again; one that disappears in sight is forgotten at once.
#[test]
fn a_record_lasts_until_its_place_is_seen_empty() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1]);
    let mut eye = scout(1, P1, 10, 10);
    let supplies = ResourceState::SupplyStation {
        delivery_size: 4,
        delivery_interval: 960,
        current_supplies: 0,
        next_delivery_in: Some(100),
    };
    let station = resource(2, ObjectKind::SupplyDeliveryStation, 14, 10, supplies);
    let crystals = patch(3, 8, 10, 100);
    end_tick(&mut vision, &map, 1, &[&eye, &station, &crystals]);
    assert_eq!(p1(&vision).records().count(), 2);
    let revision = p1(&vision).records_revision();

    // Out of sight, the station goes: still remembered.
    eye.pos = TilePos::new(30, 10).center();
    end_tick(&mut vision, &map, 2, &[&eye, &station, &crystals]);
    end_tick(&mut vision, &map, 3, &[&eye, &crystals]);
    assert!(p1(&vision).remembered_record(ObjectId(2)).is_some());
    assert_eq!(
        p1(&vision).records_revision(),
        revision,
        "nothing the player knows changed"
    );
    let tiles: Vec<TilePos> = p1(&vision).record_tiles().collect();
    let expected = [(8, 10), (14, 10), (15, 10), (14, 11), (15, 11)].map(|(x, y)| TilePos::new(x, y));
    assert_eq!(tiles, expected);
    assert!(p1(&vision).is_record_tile(TilePos::new(15, 11)));

    // One tile of the 2×2 footprint in sight is enough to see the station is gone: (20, 10) sees (15, 10) at
    // exactly 5 grid units, but none of the other three.
    eye.pos = TilePos::new(20, 10).center();
    end_tick(&mut vision, &map, 4, &[&eye, &crystals]);
    assert!(p1(&vision).record(ObjectId(2)).is_none());
    assert_ne!(p1(&vision).records_revision(), revision);
    assert!(!p1(&vision).is_record_tile(TilePos::new(15, 11)));
    assert!(p1(&vision).record(ObjectId(3)).is_some(), "the patch wasn't seen");

    // Gone while in sight: forgotten at once.
    eye.pos = TilePos::new(9, 10).center();
    end_tick(&mut vision, &map, 5, &[&eye, &crystals]);
    end_tick(&mut vision, &map, 6, &[&eye]);
    assert_eq!(p1(&vision).records().count(), 0);
    assert_eq!(p1(&vision).record_tiles().count(), 0);
}

/// An object placed out of sight, on Explored tiles, stays unknown until the player sees it.
#[test]
fn objects_placed_in_the_fog_stay_unknown() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1]);
    let mut eye = scout(1, P1, 10, 10);
    end_tick(&mut vision, &map, 1, &[&eye]);
    eye.pos = TilePos::new(30, 10).center();
    let crystals = patch(2, 12, 10, 100);
    end_tick(&mut vision, &map, 2, &[&eye, &crystals]);
    assert_eq!(p1(&vision).state(TilePos::new(12, 10)), VisibilityState::Explored);
    assert_eq!(p1(&vision).records().count(), 0);
    eye.pos = TilePos::new(10, 10).center();
    end_tick(&mut vision, &map, 3, &[&eye, &crystals]);
    assert_eq!(p1(&vision).record(ObjectId(2)).map(|r| r.seen), Some(3));
}

/// A record whose object moved goes once its old place is seen, and the object is remembered where it is seen next.
#[test]
fn a_moved_footprint_is_seen_gone_from_its_old_place() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1]);
    let mut eye = scout(1, P1, 10, 10);
    let mut crystals = patch(2, 13, 10, 100);
    end_tick(&mut vision, &map, 1, &[&eye, &crystals]);
    eye.pos = TilePos::new(30, 10).center();
    crystals = patch(2, 16, 10, 100);
    end_tick(&mut vision, &map, 2, &[&eye, &crystals]);
    assert_eq!(
        p1(&vision).record(ObjectId(2)).unwrap().placement.origin,
        TilePos::new(13, 10)
    );
    // (10, 10) sees (13, 10) but not (16, 10), 6 grid units away.
    eye.pos = TilePos::new(10, 10).center();
    end_tick(&mut vision, &map, 3, &[&eye, &crystals]);
    assert!(p1(&vision).record(ObjectId(2)).is_none());
    eye.pos = TilePos::new(12, 10).center();
    end_tick(&mut vision, &map, 4, &[&eye, &crystals]);
    assert_eq!(
        p1(&vision).record(ObjectId(2)).unwrap().placement.origin,
        TilePos::new(16, 10)
    );
}

/// Each player remembers what it saw; nobody keeps a record of its own objects.
#[test]
fn players_remember_apart_and_never_their_own() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let a = scout(1, P1, 5, 5);
    let b = scout(2, P2, 30, 10);
    let near_a = patch(3, 7, 5, 10);
    let near_b = patch(4, 32, 10, 20);
    // No M1 structure exists yet; a patch with an owner stands in for one.
    let mut tower = patch(5, 28, 12, 30);
    tower.owner = Some(P1);
    end_tick(&mut vision, &map, 1, &[&a, &b, &near_a, &near_b, &tower]);
    let ids = |v: &Vision, p: PlayerId| -> Vec<u32> { v.player(p).unwrap().records().map(|r| r.id.0).collect() };
    assert_eq!((ids(&vision, P1), ids(&vision, P2)), (vec![3], vec![4, 5]));
    assert_eq!(vision.player(P2).unwrap().record(ObjectId(5)).unwrap().owner, Some(P1));

    // Taken over by P2: its own now, so no longer a record.
    tower.owner = Some(P2);
    end_tick(&mut vision, &map, 2, &[&a, &b, &near_a, &near_b, &tower]);
    assert_eq!(ids(&vision, P2), vec![4]);
}

/// V1 catches a record that should be gone, one that wasn't refreshed, and one of the player's own object.
#[test]
fn the_check_finds_wrong_records() {
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1]);
    let eye = scout(1, P1, 10, 10);
    let crystals = patch(2, 13, 10, 100);
    end_tick(&mut vision, &map, 1, &[&eye, &crystals]);

    let (_, message) = vision.check(&map, [&eye], 1).unwrap_err();
    assert!(message.contains("the record of #2"), "{message}");
    let (_, message) = vision.check(&map, [&eye, &crystals], 2).unwrap_err();
    assert!(message.contains("the record of #2"), "{message}");
    let mut own = vision.clone();
    let mut record = own.players[0].record(ObjectId(2)).unwrap().clone();
    record.owner = Some(P1);
    own.players[0].store(record);
    let (_, message) = own.check(&map, [&eye, &crystals], 1).unwrap_err();
    assert!(message.contains("its own object"), "{message}");
}
