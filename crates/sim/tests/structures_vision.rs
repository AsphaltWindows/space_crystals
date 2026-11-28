//! The GDO structures in sight and in mind (slice M2-A): they are sight sources with the ElevationModifier
//! (`core/vision.md`), enemies remember them in the fog (`core/vision.md > Explored`), a plate shares its tile with the
//! patch it is placed onto (`factions/gdo/economy.md > ExtractionPlate`) and nothing else overlaps (design change
//! 0096), and units attack enemy structures on their own, after the targets that shoot back (design change 0103).
//! `TestSim` checks the invariants after every tick, the vision invariant V1 included.

use sc_sim::testing::{ScenarioBuilder, TestSim};
const FOG_DELAY_TICKS: sc_sim::Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;
use sc_sim::vision::PlayerVision;
use sc_sim::{
    DebugCommand, Faction, LayoutObject, ObjectId, ObjectKind, OrderKind, PlacementCheck, PlayerId, ScenarioProblem,
    SimEvent, SlotId, Spawn, SpawnAt, Tick, TilePos, TileProblem, Vec2, VisibilityState, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

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

fn is_visible(t: &TestSim, player: PlayerId, id: u32) -> bool {
    t.sim().is_visible(player, object(t, id))
}

/// The tiles `player` sees, row by row.
fn seen(t: &TestSim, player: PlayerId) -> Vec<TilePos> {
    let map = t.sim().map();
    let tiles = (0..map.height() as i32).flat_map(|y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)));
    tiles
        .filter(|&tile| t.sim().visibility(player, tile) == VisibilityState::Visible)
        .collect()
}

/// What `player` sees and remembers.
fn vision(t: &TestSim, player: PlayerId) -> &PlayerVision {
    t.sim().vision().player(player).expect("a player")
}

fn teleport(t: &mut TestSim, tick: u32, id: u32, to: Vec2) {
    t.debug_at(
        tick,
        DebugCommand::SetPosition {
            id: ObjectId(id),
            pos: to,
        },
    );
}

/// What unit `id`'s attack engages.
fn target(t: &TestSim, id: u32) -> Option<ObjectId> {
    object(t, id).attack.as_ref().unwrap().target
}

/// `core/vision.md > Vision`: a structure gives its owner sight of the tiles whose centres lie within its SightRange
/// of its centre: 6 grid units for a Deployment Center, 3 for a Generator and an Extraction Facility, 4 for a
/// Barracks, none for a plate.
#[test]
fn structures_see_with_their_sight_range() {
    for (kind, sight) in [
        (ObjectKind::DeploymentCenter, 6.0),
        (ObjectKind::Generator, 3.0),
        (ObjectKind::Barracks, 4.0),
        (ObjectKind::ExtractionFacility, 3.0),
        (ObjectKind::ExtractionPlate, 0.0),
    ] {
        let mut builder = ScenarioBuilder::new(&flat(24, 20)).spawn(kind, Some(P1), tile(9, 8));
        if kind == ObjectKind::ExtractionPlate {
            builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(9, 8)); // a plate goes onto a patch
        }
        let mut t = builder.start();
        t.run_until(1);
        let at = object(&t, 1).pos;
        let expected: Vec<TilePos> = (0..20)
            .flat_map(|y| (0..24).map(move |x| TilePos::new(x, y)))
            .filter(|tile| sight > 0.0 && tile.center().distance(at) <= gu(sight))
            .collect();
        assert_eq!(seen(&t, P1), expected, "{kind:?}");
        assert_eq!(kind.def().sight_range, sight);
    }
}

/// 30 × 16 tiles: level 0 in the west, a ramp on column 15 (its centre at level 0), level 1 from column 16 on.
fn plateau() -> String {
    let corners = format!("{}{}", "0".repeat(16), "1".repeat(15));
    format!("{}\n\n{}", flat(30, 16), vec![corners; 17].join("\n"))
}

/// `core/vision.md > ElevationModifier`: a Deployment Center on higher ground sees 7 grid units over lower tiles,
/// and 6 over its own level.
#[test]
fn a_deployment_center_on_high_ground_sees_one_further_down() {
    let mut t = ScenarioBuilder::new(&plateau())
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(18, 6)) // (18, 6) to (21, 9), centred on (1280, 512)
        .start();
    t.run_until(1);
    let state = |x, y| t.sim().visibility(P1, TilePos::new(x, y));
    assert_eq!(t.sim().map().elevation_level(object(&t, 1).pos), 1);
    // Along row 7, 32 su off the centre's row: tile (13, 7) lies 6.52 grid units west, at level 0.
    assert_eq!(state(13, 7), VisibilityState::Visible, "within 7, lower");
    assert_eq!(state(12, 7), VisibilityState::Unexplored, "7.51 away");
    assert_eq!(state(25, 7), VisibilityState::Visible, "5.52 away, same level");
    assert_eq!(state(26, 7), VisibilityState::Unexplored, "6.52 away, same level");
}

/// `core/vision.md > Explored`: an enemy Barracks seen and then out of sight stays on the player's Explored tiles as
/// it was last seen; destroyed in the fog, it is remembered until its place is seen again.
#[test]
fn an_enemy_barracks_is_remembered_in_the_fog() {
    let mut t = ScenarioBuilder::new(&flat(32, 10))
        .spawn(ObjectKind::Barracks, Some(P2), tile(12, 4)) // 1: (12, 4) to (14, 5)
        .spawn(PK, Some(P1), SpawnAt::Point(centre(8, 5))) // 2: sees it
        // Player 2's other building, never in player 1's sight: losing the Barracks doesn't defeat them (plan
        // decision T8), and the game goes on.
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 0)) // 3
        .start();
    // Looking only: no shot at the Barracks (they attack enemy buildings, `targets_that_shoot_back_come_first`).
    t.without_auto_targeting();
    t.run_until(2);
    assert!(is_visible(&t, P1, 1));
    let record = vision(&t, P1).record(ObjectId(1)).cloned().unwrap();
    assert_eq!(
        (record.kind, record.owner, record.hp, record.placement),
        (
            ObjectKind::Barracks,
            Some(P2),
            Some(300.0),
            object(&t, 1).placement.unwrap()
        )
    );
    assert!(
        vision(&t, P1).remembered_record(ObjectId(1)).is_none(),
        "seen, not remembered"
    );

    // Away in tick 3: in sight for the fog delay (design change 0178), through tick 10.
    teleport(&mut t, 3, 2, centre(28, 5));
    t.run_until(3);
    assert!(is_visible(&t, P1, 1), "for the fog delay");
    let out = 3 + FOG_DELAY_TICKS;
    t.run_until(out);
    assert!(!is_visible(&t, P1, 1));
    assert_eq!(t.sim().visibility(P1, TilePos::new(12, 4)), VisibilityState::Explored);
    let remembered = vision(&t, P1).remembered_record(ObjectId(1));
    assert_eq!(
        remembered.map(|r| (r.kind, r.seen)),
        Some((ObjectKind::Barracks, out - 1))
    );

    t.debug_at(out + 1, DebugCommand::Remove { id: ObjectId(1) })
        .run_until(out + 1);
    assert!(
        vision(&t, P1).remembered_record(ObjectId(1)).is_some(),
        "destroyed in the fog"
    );
    teleport(&mut t, out + 2, 2, centre(8, 5));
    t.run_until(out + 2);
    assert!(vision(&t, P1).record(ObjectId(1)).is_none(), "seen gone");
}

/// A plate on a patch (`factions/gdo/economy.md > ExtractionPlate`): the scenario is valid and both objects exist,
/// the tile blocks ground units, and an enemy who sees the tile sees, and then remembers, both.
#[test]
fn a_plate_on_a_patch() {
    let scenario = ScenarioBuilder::new(&flat(14, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(5, 3)) // 1
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(5, 3)) // 2
        .spawn(PK, Some(P2), SpawnAt::Point(centre(2, 3))) // 3: sees (5, 3)
        .build();
    assert_eq!(scenario.validate(), []);
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    t.run_until(1);
    assert_eq!(object(&t, 1).placement, object(&t, 2).placement);
    let grid = t.sim().nav().grid(sc_sim::nav::NavClass::GroundRugged, 1).unwrap();
    assert!(grid.is_blocked(TilePos::new(5, 3)));
    assert!(is_visible(&t, P2, 1) && is_visible(&t, P2, 2));
    let records = |t: &TestSim| {
        let vision = vision(t, P2);
        (
            vision.records().map(|r| r.id.0).collect::<Vec<_>>(),
            vision.is_record_tile(TilePos::new(5, 3)),
        )
    };
    assert_eq!(records(&t), (vec![1, 2], true), "both recorded");
    teleport(&mut t, 2, 3, centre(12, 3));
    t.run_until(2 + FOG_DELAY_TICKS);
    assert!(!is_visible(&t, P2, 1) && !is_visible(&t, P2, 2));
    assert_eq!(records(&t), (vec![1, 2], true), "both remembered");
}

/// Design change 0096 (`core/entities.md > Placement Validation > All placement methods: no buildings on
/// resources`): a structure over a patch or a station makes a map invalid, and so does one in a Starting Layout; a
/// plate on an uncovered patch is valid, also from a layout, and one on no patch isn't (`factions/gdo/economy.md >
/// ExtractionPlate`, slice M2-J2). The debug spawn's side is in `structures.rs`.
#[test]
fn only_a_plate_may_cover_a_resource() {
    let gdo = |kind, x, y| Spawn::new(kind, Some(SlotId(1)), tile(x, y));
    let mut s = ScenarioBuilder::new(&flat(14, 8))
        .spawn(ObjectKind::CrystalPatch, None, tile(2, 2)) // 0
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(8, 2)) // 1: (8, 2) to (9, 3)
        .build();
    s.map.spawns.extend([
        gdo(ObjectKind::Generator, 1, 1),        // 2: over the patch at (2, 2)
        gdo(ObjectKind::Barracks, 9, 3),         // 3: over the station at (9, 3)
        gdo(ObjectKind::ExtractionPlate, 2, 2),  // 4: on the patch, which the Generator covers too
        gdo(ObjectKind::ExtractionPlate, 10, 6), // 5: on no patch
    ]);
    let overlap = |first, second, x, y| ScenarioProblem::OverlappingFootprints {
        first,
        second,
        tile: TilePos::new(x, y),
    };
    let off_patch = ScenarioProblem::InvalidSpawn {
        spawn: 5,
        kind: ObjectKind::ExtractionPlate,
        problem: sc_sim::SpawnProblem::NotOnPatch,
    };
    assert_eq!(
        s.validate(),
        [overlap(0, 2, 2, 2), overlap(1, 3, 9, 3), overlap(2, 4, 2, 2), off_patch]
    );

    // From a Starting Layout: a plate on the map's patch is fine, a Generator over the station is not (the rules of a
    // layout, design change 0097: the plate and the Generator in the area of the layout's own Deployment Center).
    s.map.spawns.truncate(2);
    let layout = |kind, x, y| LayoutObject {
        kind,
        at: tile(x, y),
        orientation: Default::default(),
    };
    *s.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![
        layout(ObjectKind::ExtractionPlate, 2, 2),
        layout(ObjectKind::DeploymentCenter, 3, 4), // (3, 4) to (6, 7)
    ];
    assert_eq!(s.validate(), []);
    let sim = sc_sim::Sim::try_new(&s).unwrap();
    let on_tile: Vec<_> = sim
        .objects()
        .filter(|o| o.pos == centre(2, 2))
        .map(|o| (o.kind, o.owner))
        .collect();
    assert_eq!(
        on_tile,
        [
            (ObjectKind::CrystalPatch, None),
            (ObjectKind::ExtractionPlate, Some(P1))
        ]
    );
    s.map.slots[0]
        .layouts
        .get_mut(Faction::Gdo)
        .push(layout(ObjectKind::Generator, 7, 1));
    assert_eq!(
        s.validate(),
        [ScenarioProblem::LayoutPlacement {
            slot: SlotId(1),
            faction: Faction::Gdo,
            index: 2,
            kind: ObjectKind::Generator,
            check: PlacementCheck {
                tiles: vec![(TilePos::new(8, 2), TileProblem::Resource)],
                whole: vec![],
            },
        }]
    );
}

/// `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`: an idle unit takes the enemy
/// that can shoot back first, though an enemy Generator lies closer and straight ahead; then it attacks the
/// Generator on its own (design change 0103: enemy structures are ValidTargets, non-threatening while they can't
/// attack; its verification 2).
#[test]
fn targets_that_shoot_back_come_first() {
    let mut t = ScenarioBuilder::new(&flat(16, 12))
        .spawn(PK, Some(P1), SpawnAt::Point(centre(5, 5))) // 1, facing east
        .spawn(ObjectKind::Generator, Some(P2), tile(7, 5)) // 2: 84 su east of it
        .spawn(PK, Some(P2), SpawnAt::Point(centre(5, 9))) // 3: 232 su south
        .start();
    t.run_until(3);
    assert_eq!(target(&t, 1), Some(ObjectId(3)));
    t.debug_at(4, DebugCommand::Remove { id: ObjectId(3) }).run_until(8);
    assert_eq!(target(&t, 1), Some(ObjectId(2)));
}

/// Design change 0103's verification 1: Peacekeepers on an AttackMove into an enemy base defended by a Peacekeeper
/// shoot the defender first, then the buildings, and go on to their destination.
#[test]
fn an_attack_move_takes_the_defender_first_then_the_buildings() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P2), tile(15, 5)) // 1: (15, 5) to (16, 6)
        .spawn(PK, Some(P2), SpawnAt::Point(centre(12, 6))) // 2: the defender, in front of it
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 5))) // 3
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 7))) // 4
        .start();
    let destination = centre(21, 6);
    t.order_at(
        1,
        P1,
        &[ObjectId(3), ObjectId(4)],
        OrderKind::AttackMove(destination),
        false,
    );
    t.run_until(1000);
    let died = |id: u32| {
        let tick = t.events().iter().find_map(|(tick, e)| match e {
            SimEvent::Died { id: dead, .. } if *dead == ObjectId(id) => Some(*tick),
            _ => None,
        });
        tick.unwrap_or_else(|| panic!("{id} is destroyed"))
    };
    let (defender, generator) = (died(2), died(1));
    let hits_on = |target: u32| -> Vec<Tick> {
        let by_attackers = |attacker: &ObjectId| [ObjectId(3), ObjectId(4)].contains(attacker);
        t.events()
            .iter()
            .filter_map(|(tick, e)| match e {
                SimEvent::Hit {
                    attacker, target: hit, ..
                } if by_attackers(attacker) && *hit == ObjectId(target) => Some(*tick),
                _ => None,
            })
            .collect()
    };
    assert!(!hits_on(2).is_empty());
    let first_on_generator = hits_on(1)[0];
    assert!(
        defender < first_on_generator && first_on_generator < generator,
        "{defender}, {first_on_generator}"
    );
    // Each at its own place of the group's destination (the magic box keeps their offsets), done.
    for id in [3, 4] {
        let unit = object(&t, id);
        assert!(unit.orders.as_ref().unwrap().is_idle(), "{id} is done");
        assert!(unit.pos.distance(destination) <= gu(1.0), "{id} at {}", unit.pos);
    }
}

/// 20 × 10 tiles: level 1 up to corner column 8, level 0 from column 9 on (column 8 is the ramp).
fn ledge() -> String {
    let corners = format!("{}{}", "1".repeat(9), "0".repeat(12));
    format!("{}\n\n{}", flat(20, 10), vec![corners; 11].join("\n"))
}

/// A scan finds a structure whose nearest tile is in reach, however far its centre lies: a Peacekeeper on high ground
/// (SightRange 5, 6 over lower ground) takes an enemy Deployment Center the centre of whose nearest tile lies exactly
/// 6 grid units from its own centre (`systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`: from
/// the unit's centre, as vision does, the boundary included), the Deployment Center's centre 7.5 grid units away
/// (`World::pick`'s cells hold candidates by their centres). Its own eye shows it the Deployment Center.
#[test]
fn a_scan_reaches_a_large_structure_by_its_nearest_tile() {
    let mut t = ScenarioBuilder::new(&ledge())
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(480.0, 288.0))) // 1: on tile (7, 4), level 1
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(13, 3)) // 2: (13, 3) to (16, 6), from x 832
        .start();
    assert_eq!(centre(13, 4).distance(object(&t, 1).pos), gu(6.0));
    // The first scan. (Chasing it down the ramp, the Peacekeeper loses the modifier, and with it the target.)
    t.run_until(1);
    assert!(is_visible(&t, P1, 2));
    assert_eq!(target(&t, 1), Some(ObjectId(2)));
}
