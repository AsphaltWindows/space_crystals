//! The Colonists' kinds in the game (slice M3-A; `factions/colonists/units/README.md`,
//! `factions/colonists/economy.md`): they spawn for any player with their stat blocks, their structures block the
//! ground and see with their SightRange, the Legionnaires fight (the Sabre at contact, without the ElevationModifier;
//! the Rifle at 4 grid units, with it), and the Drone moves on the stand-in (plan decision T25). Their stat blocks
//! against the design are `defs/colonists/tests.rs`; `Sim::change_kind` is `sim/kinds/tests.rs`. `TestSim` checks the
//! invariants after every tick.

use sc_sim::combat;
use sc_sim::defs::MELEE_CONTACT_GU;
use sc_sim::elevation::Elevation;
use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Rotation, Shape,
    SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, VisibilityState, gu, sides,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const RIFLE: ObjectKind = ObjectKind::RifleLegionnaire;
const COLONISTS: [ObjectKind; 7] = [
    ObjectKind::Drone,
    ObjectKind::Beacon,
    ObjectKind::EgressNode,
    ObjectKind::RoboPortal,
    ObjectKind::BioPortal,
    SABRE,
    RIFLE,
];

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn object(t: &TestSim, id: u32) -> &sc_sim::Object {
    t.sim().object(ObjectId(id)).expect("exists")
}

/// Where a kind spawns with its top-left tile, or its centre for a unit, at tile (`x`, `y`).
fn at(kind: ObjectKind, x: i32, y: i32) -> SpawnAt {
    if kind.def().unit.is_some() {
        SpawnAt::Point(centre(x, y))
    } else {
        tile(x, y)
    }
}

/// The tiles `player` sees, row by row.
fn seen(t: &TestSim, player: PlayerId) -> Vec<TilePos> {
    let map = t.sim().map();
    let tiles = (0..map.height() as i32).flat_map(|y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)));
    tiles
        .filter(|&tile| t.sim().visibility(player, tile) == VisibilityState::Visible)
        .collect()
}

/// Each of the seven kinds spawns for player 1 from the scenario and for player 2 by a debug spawn, turned a quarter
/// clockwise: its owner, its MaxHP, a structure's footprint at its centre and its sides, and the capabilities its
/// kind has. A unit takes orders and moves; a Legionnaire has its weapon, the Drone none; a portal produces. Every
/// structure blocks its tiles on every ground grid. There is one: the Drone, a HoverVehicle, crosses Rugged terrain
/// (design change 0039) and so shares `(GroundRugged, 1)` with the infantry, as in M2 (slice M3-G; on the stand-in
/// it had a grid of its own, `(Ground, 1)`, which doubled every set of grids); the Cults' wheeled and tracked vehicles
/// (slice M5-G2), which don't cross it (`systems/units.md > WheeledVehicle`, `systems/units.md > TrackedVehicle`), walk
/// on `(Ground, 1)`, which a game builds only once one of them is in it (slice M5-Z2, `paths.rs`).
#[test]
fn each_kind_spawns_for_two_owners() {
    let mut builder = ScenarioBuilder::new(&flat(44, 14));
    for (i, kind) in COLONISTS.into_iter().enumerate() {
        builder = builder.spawn(kind, Some(P1), at(kind, 1 + 6 * i as i32, 1));
    }
    let mut t = builder.start();
    let turned = BuildingOrientation {
        rotation: Rotation::R90,
        ..Default::default()
    };
    for (i, kind) in COLONISTS.into_iter().enumerate() {
        let spawn = DebugCommand::Spawn {
            kind,
            owner: Some(P2),
            at: at(kind, 1 + 6 * i as i32, 8),
            orientation: turned,
        };
        t.debug_at(1, spawn);
    }
    t.run_until(1);
    assert_eq!(t.sim().objects().count(), 14);
    let grids = t.sim().nav().grids();
    let keys: Vec<(NavClass, u8)> = grids.iter().map(|g| (g.class(), g.size())).collect();
    assert_eq!(keys, [(NavClass::GroundRugged, 1)]);
    for o in t.sim().objects() {
        let (owner, i) = if o.id.0 <= 7 {
            (P1, o.id.0 as usize - 1)
        } else {
            (P2, o.id.0 as usize - 8)
        };
        let def = o.kind.def();
        assert_eq!(
            (o.kind, o.owner, o.hp),
            (COLONISTS[i], Some(owner), def.max_hp),
            "{o:?}"
        );
        assert!(o.extractor.is_none() && o.constructor.is_none(), "{:?}", o.kind);
        let unit = def.unit.as_ref();
        assert_eq!(o.orders.is_some(), unit.is_some(), "{:?}", o.kind);
        assert_eq!(o.mover.is_some(), unit.is_some(), "{:?}", o.kind);
        assert_eq!(o.attack.is_some(), matches!(o.kind, SABRE | RIFLE), "{:?}", o.kind);
        let portal = matches!(o.kind, ObjectKind::RoboPortal | ObjectKind::BioPortal);
        assert_eq!(o.production.is_some(), portal, "{:?}", o.kind);
        let row = if owner == P1 { 1 } else { 8 };
        let Some(placement) = o.placement else {
            assert_eq!(o.pos, centre(1 + 6 * i as i32, row), "{:?}", o.kind);
            continue;
        };
        assert_eq!(placement.origin, TilePos::new(1 + 6 * i as i32, row), "{:?}", o.kind);
        assert_eq!(Some((placement.w, placement.h)), def.tile_size(), "{:?}", o.kind);
        assert_eq!(o.pos, placement.center());
        // Turned a quarter clockwise, each side's letter moves on to the next side.
        let mut letters = def.structure.as_ref().unwrap().symmetry.letters();
        let mut orientation = BuildingOrientation::default();
        if owner == P2 {
            letters.rotate_right(1);
            orientation = turned;
        }
        assert_eq!(placement.orientation(), orientation);
        assert_eq!(sides(o.kind, orientation), Some(letters), "{:?}", o.kind);
        for grid in grids {
            for tile in placement.tiles() {
                assert!(
                    grid.is_blocked(tile),
                    "{:?} on {:?}: {tile:?}",
                    o.kind,
                    (grid.class(), grid.size())
                );
            }
        }
    }
}

/// `core/vision.md > Vision`: each kind gives its owner sight of the tiles whose centres lie within its SightRange
/// of its centre: 4 grid units for a Beacon and a Robo Portal, 3 for an Egress Node and a Bio Portal, 5 for the
/// Drone and the Legionnaires.
#[test]
fn each_kind_sees_with_its_sight_range() {
    for (kind, sight) in [
        (ObjectKind::Beacon, 4.0),
        (ObjectKind::EgressNode, 3.0),
        (ObjectKind::RoboPortal, 4.0),
        (ObjectKind::BioPortal, 3.0),
        (ObjectKind::Drone, 5.0),
        (SABRE, 5.0),
        (RIFLE, 5.0),
    ] {
        let mut t = ScenarioBuilder::new(&flat(24, 20))
            .spawn(kind, Some(P1), at(kind, 9, 8))
            .start();
        t.run_until(1);
        let from = object(&t, 1).pos;
        let expected: Vec<TilePos> = (0..20)
            .flat_map(|y| (0..24).map(move |x| TilePos::new(x, y)))
            .filter(|tile| tile.center().distance(from) <= gu(sight))
            .collect();
        assert_eq!(seen(&t, P1), expected, "{kind:?}");
        assert_eq!(kind.def().sight_range, sight);
    }
}

/// `factions/colonists/units/sabre_legionnaire.md`: a melee attack (`systems/combat.md > FullyConnected > Melee`),
/// at contact: the reach of 8 space units of the placeholder (design change 0149). Ordered to attack a Peacekeeper
/// 200 su away, a Sabre Legionnaire closes in and strikes only once the gap between their edges is at most 8 su;
/// four hits of 14 (Damage 15 less the Peacekeeper's PointArmor 1) destroy its 50 HP. The Peacekeeper doesn't shoot
/// back: auto-targeting is off.
#[test]
fn a_sabre_legionnaire_kills_a_peacekeeper_at_contact() {
    let mut t = ScenarioBuilder::new(&flat(16, 6))
        .spawn(SABRE, Some(P1), SpawnAt::Point(Vec2::new(200.0, 192.0))) // 1
        .spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(430.0, 192.0))) // 2: a gap of 200 su
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Attack(ObjectId(2)), false)
        .run_until(150);
    let mut strikes = 0;
    for (tick, e) in t.events() {
        if let SimEvent::Attacked { attacker, from, to, .. } = e {
            assert_eq!(*attacker, ObjectId(1));
            let gap = Shape::rect(*from, Vec2::splat(18.0)).gap(&Shape::rect(*to, Vec2::splat(12.0)));
            assert!((0.0..=8.0).contains(&gap), "a strike across {gap} su in tick {tick}");
            strikes += 1;
        }
    }
    let hits: Vec<f32> = t
        .events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::Hit { target, damage, .. } if *target == ObjectId(2) => Some(*damage),
            _ => None,
        })
        .collect();
    assert_eq!((strikes, hits), (4, vec![14.0; 4]));
    assert!(t.sim().object(ObjectId(2)).is_none(), "destroyed");
    assert!(
        t.events()
            .iter()
            .any(|(_, e)| matches!(e, SimEvent::Died { id, kind: PK, .. } if *id == ObjectId(2)))
    );
    assert_eq!(object(&t, 1).hp, Some(70.0), "untouched");
}

/// 12 × 3 tiles. Level 0 up to x = 448 su; tile 7 ramps from 0 to 1 eastwards, so the height reaches level 1
/// exactly at x = 512, one level per 64 su; level 1 from there on (as in `combat_range.rs`).
const RAMP: &str = "
    ............
    ............
    ............

    0000000011111
    0000000011111
    0000000011111
    0000000011111
";

/// `attacker` (player 1, id 1) at `attacker_x` and a Peacekeeper (player 2, id 2) at `target_x`, both on row y = 96
/// of `RAMP`, spawned in tick 1 and ordered to hold their position in tick 2; the game runs to tick 12. The ticks of
/// the attacker's attacks.
fn attacks(attacker: ObjectKind, attacker_x: f32, target_x: f32) -> Vec<Tick> {
    let mut t = ScenarioBuilder::new(RAMP).start();
    t.spawn_unit_at(1, attacker, P1, Vec2::new(attacker_x, 96.0))
        .spawn_unit_at(1, PK, P2, Vec2::new(target_x, 96.0))
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Hold, false)
        .order_at(2, P2, &[ObjectId(2)], OrderKind::Hold, false)
        .run_until(12);
    assert!(t.sim().object(ObjectId(2)).is_some(), "both spawned");
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == ObjectId(1)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// `systems/combat.md > FullyConnected > Melee`: melee attacks don't take the ElevationModifier. A Sabre Legionnaire
/// at the top of the ramp (level 1) strikes a Peacekeeper on level 0 at contact (a gap of 8 su), and not across a
/// gap of 20 su, which a modifier of +1 grid unit would reach.
#[test]
fn a_sabre_legionnaire_takes_no_elevation_bonus() {
    let sabre = SABRE.def().unit.as_ref().unwrap().attack.unwrap();
    let (high, low) = (Elevation::Level(1), Elevation::Level(0));
    assert_eq!(combat::max_range(&sabre, high, low), gu(MELEE_CONTACT_GU));
    assert_eq!(combat::max_range(&sabre, low, high), 8.0);
    // The Sabre's west edge at 494 su.
    let gap = |gap: f32| 512.0 - 18.0 - gap - 12.0;
    assert!(!attacks(SABRE, 512.0, gap(8.0)).is_empty(), "at contact");
    assert_eq!(
        attacks(SABRE, 512.0, gap(20.0)),
        [] as [Tick; 0],
        "20 su from higher ground"
    );
}

/// `factions/colonists/units/rifle_legionnaire.md`: Range 4 grid units (256 su) edge to edge, with the
/// ElevationModifier (`core/vision.md > ElevationModifier`). On level ground it reaches a Peacekeeper across a gap of
/// exactly 256 su and not 257; from the top of the ramp (level 1) across 275 su, which it doesn't from 1 su lower
/// (level 0); and uphill not across 200 su (3 grid units).
#[test]
fn a_rifle_legionnaire_fires_at_four_grid_units_with_the_elevation_modifier() {
    assert!(
        !attacks(RIFLE, 400.0, 400.0 - 18.0 - 256.0 - 12.0).is_empty(),
        "level, 256 su"
    );
    assert_eq!(
        attacks(RIFLE, 400.0, 400.0 - 18.0 - 257.0 - 12.0),
        [] as [Tick; 0],
        "level, 257 su"
    );
    let below = 512.0 - 18.0 - 275.0 - 12.0;
    assert!(!attacks(RIFLE, 512.0, below).is_empty(), "from level 1, 275 su");
    assert_eq!(attacks(RIFLE, 511.0, below), [] as [Tick; 0], "from level 0, 274 su");
    let above = 520.0;
    assert_eq!(
        attacks(RIFLE, above - 12.0 - 200.0 - 18.0, above),
        [] as [Tick; 0],
        "uphill, 200 su"
    );
    assert!(
        !attacks(RIFLE, 400.0, 400.0 - 18.0 - 200.0 - 12.0).is_empty(),
        "level, 200 su"
    );
}

/// `factions/colonists/units/drone.md`: it moves by DragMovement (slice M3-G; `tests/movement_drag/` has the
/// model's own tests). From rest it gains 1.2 space units/frame in its first frame and approaches the equilibrium
/// `(ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio` = (0.9 + 0.3) / 0.15 = 8 space units/frame
/// (design change 0205) without ever passing it; a Peacekeeper moves 4 from its first frame.
#[test]
fn a_drone_accelerates_to_eight_space_units_a_frame() {
    let mut t = ScenarioBuilder::new(&flat(24, 5))
        .spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(centre(1, 2))) // 1: (96, 160), facing east
        .start();
    let goal = centre(20, 2); // 1216 su east
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(goal)), false);
    let mut steps = Vec::new();
    for _ in 0..200 {
        let from = t.pos(1);
        let to = t.step().pos(1);
        if to != from {
            steps.push(to.distance(from));
        }
    }
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert_eq!(t.pos(1), goal);
    assert!(
        (steps[0] - 1.2).abs() < 1e-3 && (steps[1] - 2.22).abs() < 1e-3,
        "{steps:?}"
    );
    let fastest = steps.iter().copied().fold(0.0, f32::max);
    assert!((7.92..=8.0).contains(&fastest), "{fastest}");
}

/// `systems/units.md > UnitCollision > Ground Collision`: a Drone walks around a Robo Portal (4 × 4 tiles) in its way,
/// never over its footprint, and arrives.
#[test]
fn a_drone_paths_around_a_robo_portal() {
    let mut t = ScenarioBuilder::new(&flat(20, 10))
        .spawn(ObjectKind::RoboPortal, Some(P2), tile(8, 3)) // 1: (8, 3) to (11, 6)
        .spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(centre(3, 5))) // 2
        .start();
    let goal = centre(16, 5);
    t.order_at(1, P1, &[ObjectId(2)], OrderKind::Move(Target::Point(goal)), false);
    let (lo, hi) = (Vec2::new(8.0, 3.0) * 64.0, Vec2::new(12.0, 7.0) * 64.0);
    for _ in 0..200 {
        let p = t.step().pos(2);
        let overlaps = p.x + 18.0 > lo.x && p.x - 18.0 < hi.x && p.y + 18.0 > lo.y && p.y - 18.0 < hi.y;
        assert!(!overlaps, "over the Robo Portal at {p} in tick {}", t.sim().tick());
    }
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(t.pos(2).distance(goal) < 1.0, "{}", t.pos(2));
}
