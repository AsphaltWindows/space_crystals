//! `Command::Debug`: valid commands take effect, invalid ones are ignored without a panic
//! (`ARCHITECTURE.md > Commands, events and replays`).

use sc_sim::sim::DEFAULT_PATCH_AMOUNT;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, ObjectId, ObjectKind, PlayerId, ResourceState, SimEvent, SpawnAt, TilePos, Vec2};

/// 6 × 4 tiles, 384 × 256 su.
const MAP: &str = "
    ......
    ...~~.
    ......
    ......
";

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn { kind, owner, at }
}

fn ids(t: &TestSim) -> Vec<u32> {
    t.sim().objects().map(|o| o.id.0).collect()
}

#[test]
fn spawn_creates_objects_with_ascending_ids_and_events() {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.debug_at(1, spawn(ObjectKind::CrystalPatch, None, tile(0, 3)));
    t.debug_at(1, spawn(ObjectKind::SupplyDeliveryStation, None, tile(4, 2)));
    t.step();
    let sim = t.sim();
    assert_eq!(
        sim.events(),
        [SimEvent::Spawned(ObjectId(1)), SimEvent::Spawned(ObjectId(2))]
    );

    let patch = sim.object(ObjectId(1)).unwrap();
    assert_eq!(
        (patch.kind, patch.owner, patch.hp),
        (ObjectKind::CrystalPatch, None, None)
    );
    assert_eq!(
        (patch.pos, patch.prev_pos),
        (Vec2::new(32.0, 224.0), Vec2::new(32.0, 224.0))
    );
    let placement = patch.placement.unwrap();
    assert_eq!((placement.origin, placement.w, placement.h), (TilePos::new(0, 3), 1, 1));
    let remaining = DEFAULT_PATCH_AMOUNT;
    assert_eq!(patch.resource, Some(ResourceState::CrystalPatch { remaining }));

    let station = sim.object(ObjectId(2)).unwrap();
    assert_eq!(station.pos, Vec2::new(320.0, 192.0));
    let placement = station.placement.unwrap();
    assert_eq!((placement.origin, placement.w, placement.h), (TilePos::new(4, 2), 2, 2));
}

#[test]
fn invalid_spawns_are_ignored() {
    let mut t = ScenarioBuilder::new(MAP).start();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    let invalid = [
        // `core/map.md > Resource`: resources are never owned, by a player or anyone else.
        spawn(patch, Some(PlayerId(1)), tile(0, 0)),
        spawn(patch, Some(PlayerId(9)), tile(0, 0)),
        // Tile-based types need a tile location.
        spawn(patch, None, SpawnAt::Point(Vec2::new(32.0, 32.0))),
        // Footprints must lie entirely on the map.
        spawn(patch, None, tile(-1, 0)),
        spawn(patch, None, tile(0, -1)),
        spawn(patch, None, tile(6, 0)),
        spawn(patch, None, tile(0, 4)),
        spawn(station, None, tile(5, 0)),
        spawn(station, None, tile(0, 3)),
        spawn(station, None, tile(i32::MAX, i32::MAX)),
        spawn(station, None, tile(i32::MIN, i32::MIN)),
        spawn(station, None, tile(i32::MAX, 0)),
    ];
    for (i, command) in invalid.into_iter().enumerate() {
        t.debug_at(i as u32 + 1, command);
    }
    let end = t.sim().tick() + 12;
    t.run_until(end);
    assert!(t.events().is_empty(), "{:?}", t.events());
    assert!(ids(&t).is_empty());
    // Rejected spawns use up no ids.
    t.debug_at(13, spawn(patch, None, tile(5, 3)));
    t.step();
    assert_eq!(ids(&t), [1]);
}

#[test]
fn remove_deletes_once_and_ignores_unknown_ids() {
    let mut t = ScenarioBuilder::new(MAP)
        .spawn(ObjectKind::CrystalPatch, None, tile(0, 0))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 0))
        .start();
    t.debug_at(1, DebugCommand::Remove { id: ObjectId(1) });
    t.step();
    assert_eq!(t.sim().events(), [SimEvent::Removed(ObjectId(1))]);
    assert_eq!(ids(&t), [2]);

    for id in [1, 0, 3, 99, u32::MAX] {
        t.debug_at(2, DebugCommand::Remove { id: ObjectId(id) });
    }
    t.step();
    assert!(t.sim().events().is_empty());
    assert_eq!(ids(&t), [2]);

    // Ids are never reused.
    t.debug_at(3, spawn(ObjectKind::CrystalPatch, None, tile(0, 0)));
    t.step();
    assert_eq!(ids(&t), [2, 3]);
}

#[test]
fn set_position_ignores_tile_objects_unknown_ids_and_bad_points() {
    let mut t = ScenarioBuilder::new(MAP)
        .spawn(ObjectKind::CrystalPatch, None, tile(2, 2))
        .start();
    let before: Vec<_> = t.sim().objects().cloned().collect();
    let commands = [
        // Tile-based objects keep their placement (units arrive in M1).
        (1, Vec2::new(100.0, 100.0)),
        (1, Vec2::new(f32::NAN, 100.0)),
        // Unknown ids.
        (0, Vec2::new(100.0, 100.0)),
        (2, Vec2::new(100.0, 100.0)),
        (u32::MAX, Vec2::new(f32::INFINITY, f32::NAN)),
    ];
    for (id, pos) in commands {
        t.debug_at(1, DebugCommand::SetPosition { id: ObjectId(id), pos });
    }
    t.step();
    let after: Vec<_> = t.sim().objects().cloned().collect();
    assert_eq!(after, before);
    assert!(t.sim().events().is_empty());
}

#[test]
fn commands_apply_in_order_within_a_tick() {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.debug_at(1, spawn(ObjectKind::CrystalPatch, None, tile(0, 0)));
    t.debug_at(1, DebugCommand::Remove { id: ObjectId(1) });
    t.debug_at(1, spawn(ObjectKind::CrystalPatch, None, tile(0, 0)));
    t.step();
    let events = [
        SimEvent::Spawned(ObjectId(1)),
        SimEvent::Removed(ObjectId(1)),
        SimEvent::Spawned(ObjectId(2)),
    ];
    assert_eq!(t.sim().events(), events);
    assert_eq!(ids(&t), [2]);
}
