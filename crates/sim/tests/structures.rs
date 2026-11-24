//! The GDO structures (slice M2-A; `factions/gdo/structures.md`, `factions/gdo/economy.md`): they come from
//! scenarios and debug spawns with their stat blocks, turned in any of the 8 orientations; they block ground units,
//! take hits minus their PointArmor, and leave the map at 0 HP. Their sight, memory, the plate on a patch and
//! targeting are in `structures_vision.rs`. `TestSim` checks the invariants after every tick.

use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Reject,
    Rotation, SideLetter, SimEvent, SpawnAt, SpawnProblem, Target, TilePos, Vec2, sides,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const STRUCTURES: [ObjectKind; 5] = [
    ObjectKind::DeploymentCenter,
    ObjectKind::Generator,
    ObjectKind::Barracks,
    ObjectKind::ExtractionFacility,
    ObjectKind::ExtractionPlate,
];
const ROTATIONS: [Rotation; 4] = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270];

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, at: SpawnAt, orientation: BuildingOrientation) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner,
        at,
        orientation,
    }
}

fn debug(command: DebugCommand) -> sc_sim::PlayerCommand {
    sc_sim::PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    }
}

/// Whether the Peacekeepers' grid (`(GroundRugged, 1)`, the light infantry's) blocks tile `(x, y)`.
fn blocked(t: &TestSim, x: i32, y: i32) -> bool {
    let grid = t
        .sim()
        .nav()
        .grid(NavClass::GroundRugged, 1)
        .expect("the Peacekeeper's grid");
    grid.is_blocked(TilePos::new(x, y))
}

/// Each structure type spawns with its stat block, from a scenario and by a debug spawn: its owner, its MaxHP, its
/// footprint at its centre, its sides; and nothing a unit has (no orders, movement or weapon). The scenario's plate
/// goes onto a patch (`factions/gdo/economy.md > ExtractionPlate`), the map's sixth object; a debug spawn needn't.
#[test]
fn each_structure_spawns_with_its_stat_block() {
    let mut builder = ScenarioBuilder::new(&flat(30, 12));
    for (i, kind) in STRUCTURES.into_iter().enumerate() {
        builder = builder.spawn(kind, Some(P1), tile(1 + 5 * i as i32, 1));
    }
    let plate = STRUCTURES
        .iter()
        .position(|&k| k == ObjectKind::ExtractionPlate)
        .unwrap();
    let mut t = builder
        .spawn(ObjectKind::CrystalPatch, None, tile(1 + 5 * plate as i32, 1))
        .start();
    for (i, kind) in STRUCTURES.into_iter().enumerate() {
        let at = tile(1 + 5 * i as i32, 6);
        t.debug_at(1, spawn(kind, Some(P2), at, BuildingOrientation::default()));
    }
    t.run_until(1);
    assert_eq!(t.sim().objects().count(), 11);
    for o in t.sim().objects().filter(|o| o.kind != ObjectKind::CrystalPatch) {
        let def = o.kind.def();
        let (owner, row, i) = if o.id.0 <= 5 {
            (P1, 1, o.id.0 as usize - 1)
        } else {
            (P2, 6, o.id.0 as usize - 7)
        };
        assert_eq!((o.kind, o.owner), (STRUCTURES[i], Some(owner)), "{o:?}");
        assert_eq!(o.hp, def.max_hp, "{:?}", o.kind);
        let placement = o.placement.expect("tile-based");
        assert_eq!(placement.origin, TilePos::new(1 + 5 * i as i32, row));
        assert_eq!(Some((placement.w, placement.h)), def.tile_size(), "{:?}", o.kind);
        assert_eq!(o.pos, placement.center());
        let letters = def.structure.as_ref().unwrap().symmetry.letters();
        assert_eq!(sides(o.kind, placement.orientation()), Some(letters), "{:?}", o.kind);
        assert!(
            o.orders.is_none() && o.mover.is_none() && o.attack.is_none(),
            "{:?}",
            o.kind
        );
    }
}

/// `core/entities.md > Structure Type`: a Barracks (3 wide, 2 tall unturned) placed in each of the 16 ways to
/// rotate and flip it covers 3 × 2 tiles or, turned a quarter, 2 × 3; flips keep the size. Its sides go with it
/// (design change 0079's verification: unturned, B is the right end; turned 90° clockwise, the bottom end).
#[test]
fn the_barracks_footprint_in_every_orientation() {
    let mut t = ScenarioBuilder::new(&flat(32, 32)).start();
    let mut cases = Vec::new();
    for (r, rotation) in ROTATIONS.into_iter().enumerate() {
        for (f, (flip_h, flip_v)) in [(false, false), (true, false), (false, true), (true, true)]
            .into_iter()
            .enumerate()
        {
            let orientation = BuildingOrientation {
                rotation,
                flip_h,
                flip_v,
            };
            let origin = TilePos::new(1 + 8 * r as i32, 1 + 8 * f as i32);
            t.debug_at(
                1,
                spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(origin), orientation),
            );
            cases.push((origin, orientation));
        }
    }
    t.run_until(1);
    assert_eq!(t.sim().objects().count(), 16);
    for (o, (origin, orientation)) in t.sim().objects().zip(cases) {
        let placement = o.placement.unwrap();
        let size = if orientation.swaps_axes() { (2, 3) } else { (3, 2) };
        assert_eq!(
            (placement.origin, placement.w, placement.h),
            (origin, size.0, size.1),
            "{orientation:?}"
        );
        assert_eq!(placement.orientation(), orientation);
        assert_eq!(placement.tiles().count(), 6);
        for tile in placement.tiles() {
            assert!(blocked(&t, tile.x, tile.y), "{orientation:?}: {tile:?}");
        }
        // Where side B, the exit, faces: north, east, south or west.
        let placed = sides(ObjectKind::Barracks, orientation).unwrap();
        let exit = placed.iter().position(|&s| s == SideLetter::B).unwrap();
        // B starts on the right (east), or on the left (west) when flipped left and right (a top-and-bottom flip
        // leaves it where it is), and turns clockwise with the building.
        let start = if orientation.flip_h { 3 } else { 1 };
        let quarter_turns = ROTATIONS.iter().position(|&r| r == orientation.rotation).unwrap();
        assert_eq!(exit, (start + quarter_turns) % 4, "{orientation:?}");
    }
    let exit = |rotation| {
        let at = BuildingOrientation {
            rotation,
            ..Default::default()
        };
        sides(ObjectKind::Barracks, at).unwrap()
    };
    assert_eq!(exit(Rotation::R0)[1], SideLetter::B, "the right end");
    assert_eq!(exit(Rotation::R90)[2], SideLetter::B, "the bottom end");
}

/// `systems/units.md > UnitCollision > Ground Collision`: a structure's footprint blocks ground units, exactly its
/// tiles, and a Peacekeeper walks around a Generator in its way.
#[test]
fn a_peacekeeper_walks_around_a_generator() {
    let mut t = ScenarioBuilder::new(&flat(16, 9))
        .spawn(ObjectKind::Generator, Some(P2), tile(7, 3)) // 1: (7, 3) to (8, 4)
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 4))) // 2
        .start();
    for (x, y) in [(7, 3), (8, 3), (7, 4), (8, 4)] {
        assert!(blocked(&t, x, y), "({x}, {y})");
    }
    for (x, y) in [(6, 3), (9, 4), (7, 2), (8, 5)] {
        assert!(!blocked(&t, x, y), "({x}, {y})");
    }
    // Moving on, without stopping to attack the Generator on the way.
    let goal = centre(12, 4);
    t.order_at(1, P1, &[ObjectId(2)], OrderKind::Move(Target::Point(goal)), false);
    let (lo, hi) = (Vec2::new(7.0, 3.0) * 64.0, Vec2::new(9.0, 5.0) * 64.0);
    for _ in 0..200 {
        let p = t.step().pos(2);
        let overlaps = p.x + 12.0 > lo.x && p.x - 12.0 < hi.x && p.y + 12.0 > lo.y && p.y - 12.0 < hi.y;
        assert!(!overlaps, "over the Generator at {p} in tick {}", t.sim().tick());
    }
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(t.pos(2).distance(goal) < 1.0, "{}", t.pos(2));
}

/// The damage of each hit on `target`.
fn hits_on(t: &TestSim, target: u32) -> Vec<f32> {
    t.events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::Hit {
                target: hit, damage, ..
            } if *hit == ObjectId(target) => Some(*damage),
            _ => None,
        })
        .collect()
}

/// `systems/combat.md > SingleTargetDamage`: a Peacekeeper's hit (Damage 10) takes a structure's PointArmor off: 9
/// against a Generator (PointArmor 1), 8 against a plate (2). At 0 HP the Generator leaves the map: its tiles open
/// for ground units again, and the grids' version moves on. The plate's patch stays.
#[test]
fn hits_take_the_point_armor_off_and_a_destroyed_generator_frees_its_tiles() {
    let mut t = ScenarioBuilder::new(&flat(16, 9))
        .spawn(ObjectKind::Generator, Some(P2), tile(6, 2)) // 1: (6, 2) to (7, 3)
        .spawn(ObjectKind::CrystalPatch, None, tile(6, 6)) // 2
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(6, 6)) // 3, on the patch
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 2))) // 4
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 3))) // 5
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 6))) // 6
        .start();
    t.order_at(
        1,
        P1,
        &[ObjectId(4), ObjectId(5)],
        OrderKind::Attack(ObjectId(1)),
        false,
    )
    .order_at(1, P1, &[ObjectId(6)], OrderKind::Attack(ObjectId(3)), false);
    let version = t.sim().nav_version();
    t.run_until(400);
    let (generator, plate) = (hits_on(&t, 1), hits_on(&t, 3));
    // 350 HP at 9 a hit: 39 hits, or 40 if both Peacekeepers hit in its last tick (a tick's hits land together).
    assert!(generator.iter().all(|&d| d == 9.0), "{generator:?}");
    assert!((39..=40).contains(&generator.len()), "{}", generator.len());
    // 85 HP at 8 a hit: 11 hits.
    assert_eq!(plate, [8.0; 11]);
    let died: Vec<(ObjectKind, Option<PlayerId>)> = t
        .events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::Died { kind, owner, .. } => Some((*kind, *owner)),
            _ => None,
        })
        .collect();
    assert_eq!(died.len(), 2);
    assert!(
        died.iter()
            .all(|&(kind, owner)| owner == Some(P2) && kind != ObjectKind::CrystalPatch)
    );
    assert!(t.sim().object(ObjectId(1)).is_none() && t.sim().object(ObjectId(3)).is_none());
    assert!(t.sim().nav_version() > version);
    for (x, y) in [(6, 2), (7, 2), (6, 3), (7, 3)] {
        assert!(!blocked(&t, x, y), "({x}, {y}) is open again");
    }
    assert!(t.sim().object(ObjectId(2)).is_some(), "the patch stays");
    assert!(blocked(&t, 6, 6), "and blocks");
}

/// `core/entities.md > Placement Validation`: structures keep apart from every other footprint (design change
/// 0096: no buildings on resources); only a plate shares its tile, with the uncovered patch it goes onto. The
/// debug spawn refuses the rest (`Reject::Overlaps`), as scenario validation does (`structures_vision.rs`).
#[test]
fn a_debug_spawn_keeps_footprints_apart_but_for_a_plate_on_its_patch() {
    let t = ScenarioBuilder::new(&flat(12, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(2, 2)) // 1
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(6, 2)) // 2: (6, 2) to (7, 3)
        .spawn(ObjectKind::Generator, Some(P2), tile(2, 5)) // 3: (2, 5) to (3, 6)
        .start();
    let at = |kind: ObjectKind, x, y| {
        // Map resources have no owner.
        let owner = (kind.def().category != sc_sim::Category::Resource).then_some(P1);
        debug(spawn(kind, owner, tile(x, y), BuildingOrientation::default()))
    };
    let check = |kind, x, y| t.sim().validate(&at(kind, x, y));
    assert_eq!(
        check(ObjectKind::Generator, 1, 1),
        Err(Reject::Overlaps(ObjectId(1))),
        "on a patch"
    );
    assert_eq!(
        check(ObjectKind::Generator, 7, 3),
        Err(Reject::Overlaps(ObjectId(2))),
        "on a station"
    );
    assert_eq!(
        check(ObjectKind::Barracks, 3, 6),
        Err(Reject::Overlaps(ObjectId(3))),
        "on a Generator"
    );
    assert_eq!(
        check(ObjectKind::ExtractionPlate, 6, 2),
        Err(Reject::Overlaps(ObjectId(2)))
    );
    assert_eq!(
        check(ObjectKind::CrystalPatch, 3, 5),
        Err(Reject::Overlaps(ObjectId(3)))
    );
    assert_eq!(check(ObjectKind::ExtractionPlate, 2, 2), Ok(()), "a plate on the patch");
    assert_eq!(check(ObjectKind::Generator, 4, 5), Ok(()), "next to the Generator");
    // Once the patch has its plate (4), it takes no other.
    let mut t = t;
    t.debug_at(
        1,
        spawn(
            ObjectKind::ExtractionPlate,
            Some(P1),
            tile(2, 2),
            BuildingOrientation::default(),
        ),
    );
    t.run_until(1);
    assert_eq!(t.sim().objects().count(), 4);
    let again = at(ObjectKind::ExtractionPlate, 2, 2);
    assert_eq!(t.sim().validate(&again), Err(Reject::Overlaps(ObjectId(4))));
}

/// `core/map.md > Elevation > Structures only on flat terrain`: a structure on a sloped tile is refused, by a
/// scenario and by a debug spawn.
#[test]
fn structures_need_flat_ground() {
    // A rise at corner (4, 2): the four tiles around it, (3, 1) to (4, 2), slope.
    let map = format!(
        "{}\n\n{}",
        flat(8, 5),
        [
            "000000000",
            "000000000",
            "000010000",
            "000000000",
            "000000000",
            "000000000"
        ]
        .join("\n")
    );
    let mut scenario = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::Barracks, Some(P1), tile(4, 2)) // (4, 2) to (6, 3)
        .build();
    assert_eq!(
        scenario.validate(),
        [sc_sim::ScenarioProblem::InvalidSpawn {
            spawn: 0,
            kind: ObjectKind::Barracks,
            problem: SpawnProblem::NotFlat(TilePos::new(4, 2)),
        }]
    );
    scenario.map.spawns.clear();
    let t = TestSim::new(scenario);
    let turned = BuildingOrientation {
        rotation: Rotation::R90,
        ..Default::default()
    };
    // Turned, the Barracks at (4, 0) covers (4, 0) to (5, 2), sloped from (4, 1) on; at (5, 0) it clears the rise.
    let barracks = |x| debug(spawn(ObjectKind::Barracks, Some(P1), tile(x, 0), turned));
    assert_eq!(
        t.sim().validate(&barracks(4)),
        Err(Reject::Spawn(SpawnProblem::NotFlat(TilePos::new(4, 1))))
    );
    assert_eq!(t.sim().validate(&barracks(5)), Ok(()));
}

/// The debug spawn's orientation is optional in files: replays recorded before it existed hold spawns without
/// one, which load unturned, and an unturned spawn is still written without it.
#[test]
fn a_debug_spawn_without_an_orientation_loads_unturned() {
    let unturned = spawn(
        ObjectKind::Peacekeeper,
        Some(P1),
        SpawnAt::Point(centre(1, 1)),
        Default::default(),
    );
    let json = serde_json::to_string(&unturned).unwrap();
    assert!(!json.contains("orientation"), "{json}");
    assert_eq!(serde_json::from_str::<DebugCommand>(&json).unwrap(), unturned);
    let turned = spawn(
        ObjectKind::Barracks,
        Some(P1),
        tile(1, 1),
        BuildingOrientation {
            rotation: Rotation::R270,
            flip_h: false,
            flip_v: true,
        },
    );
    let json = serde_json::to_string(&turned).unwrap();
    assert!(json.contains("orientation"), "{json}");
    assert_eq!(serde_json::from_str::<DebugCommand>(&json).unwrap(), turned);
}
