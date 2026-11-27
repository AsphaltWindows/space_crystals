//! The release of a producer's finished unit where a test needs a type no game has (`defs::test_defs`): (M6-GF) a
//! vehicle of the Artillery Rig's 64 × 48 su out of the Factory's side B (plan decision T100: "GDO vehicles leave by
//! the Barracks' `ExitRule::Side`, their spot fitting the oriented box, facing out. GF tests it with the 64x48 Rig's
//! box"), the Rig's entry held until slice M6-GA.

use glam::Vec2;

use crate::command::{Command, DebugCommand};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, ProductionDef, Recipe};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::object::Rotation;
use crate::production::{Doorway, ProductionCommand};
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::shape::Shape;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Factory, on tiles (12, 10) to (14, 12) of [`factory`]'s map.
const FACTORY: ObjectId = ObjectId(1);
const RIG: ObjectKind = ObjectKind::ArtilleryRig;
/// The Factory's entries for this test: the Artillery Rig's 250 Space Crystals and 480 frames
/// (`factions/gdo/structures/factory.md > Produces`), without its Advanced Mechanics and its Supplies, which aren't
/// what the test is about.
const RIG_ENTRY: &[Recipe] = &[Recipe {
    kind: RIG,
    cost: 250,
    frames: 480,
    requires: &[],
    supplies: 0,
}];

/// Player 1's Factory ([`FACTORY`]) turned by `rotation` and their Deployment Center, whose Power runs it, and player
/// 2's Deployment Center, on a flat 30 × 24 map; 1000 Space Crystals for player 1 in tick 1, auto-targeting off.
fn factory(rotation: Rotation) -> TestSim {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(30); 24].join("\n"))
        .spawn(ObjectKind::Factory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(1, 1)),
        )
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P2),
            SpawnAt::Tile(TilePos::new(25, 19)),
        )
        .build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation,
        flip_h: false,
        flip_v: false,
    };
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    t
}

/// The unit the Factory released, once it has.
fn released(t: &TestSim) -> Option<ObjectId> {
    t.events().iter().find_map(|(_, e)| match *e {
        SimEvent::UnitProduced { producer, unit } if producer == FACTORY => Some(unit),
        _ => None,
    })
}

/// The Artillery Rig, 64 × 48 su (`factions/gdo/units/artillery_rig.md`: "Silhouette 64x48 space units"), trained at
/// the Factory, which only a test can let train one today, turned each of the four ways, with player 1's idle
/// Peacekeeper right in front of the middle of side B, 40 su out. It comes out facing out of side B, its box turned
/// with it, 64 su deep out of the side and 48 su along it, moved in from half inside (its centre on the side) as far
/// as its 32 su half length and a sixty-fourth of a su take it to stand clear of the Peacekeeper, and touching the
/// Peacekeeper's box, never overlapping it (`Sim::doorway_spot`). It then drives out of the doorway to the Factory's
/// default rally point (design change 0446), the Peacekeeper making way for it, under the invariants.
#[test]
fn a_64_by_48_vehicle_comes_out_of_side_b_facing_out_its_box_turned_with_it() {
    let _trains_rigs = TestDef::structure(ObjectKind::Factory, |structure| {
        let production = structure.production.expect("the Factory produces");
        structure.production = Some(ProductionDef {
            recipes: RIG_ENTRY,
            ..production
        });
    });
    let pk_half = ObjectKind::Peacekeeper.def().half_extents().unwrap();
    let mut ways = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let mut t = factory(rotation);
        let building = t.sim().object(FACTORY).expect("the Factory");
        let door = Doorway::of(building)
            .expect("its exit doorway")
            .doors()
            .next()
            .expect("one door");
        let (middle, outward) = (door.middle(), door.outward());
        ways.push(outward);
        let blocker = ObjectId(4);
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, middle + outward * 40.0);
        let train = Command::Production {
            objects: vec![FACTORY],
            command: ProductionCommand::Train(RIG),
        };
        t.at(2, P1, train);
        while released(&t).is_none() {
            assert!(t.sim().tick() < 500, "{rotation:?}: no Rig\n{}", t.ascii(None));
            t.step();
        }
        let rig = released(&t).unwrap();
        let o = t.sim().object(rig).expect("out");
        let across = pk_half.dot(outward.abs());
        let expected = middle + outward * (40.0 - across - 32.0 - 1.0 / 64.0);
        assert!(
            o.pos.distance(expected) < 1e-3,
            "{rotation:?}: at {}, not {expected}",
            o.pos
        );
        assert!(
            math::facing_vector(o.facing).distance(outward) < 1e-6,
            "{rotation:?}: facing out"
        );
        let half = Shape::of(o).bound_half();
        let along = outward.abs();
        assert!(
            (half.dot(along) - 32.0).abs() < 1e-3 && (half.dot(Vec2::ONE - along) - 24.0).abs() < 1e-3,
            "{rotation:?}: {half}"
        );
        let pk = Shape::of(t.sim().object(blocker).expect("the Peacekeeper"));
        assert!(
            !Shape::of(o).overlaps(&pk) && Shape::of(o).gap(&pk) < 0.1,
            "{rotation:?}: against the Peacekeeper"
        );
        let out = t.sim().tick();
        while t.mover(rig.0).exit.is_some() {
            assert!(
                t.sim().tick() < out + 120,
                "{rotation:?}: out of the doorway\n{}",
                t.ascii(None)
            );
            t.step();
        }
    }
    let mut sorted: Vec<(i32, i32)> = ways.iter().map(|v| (v.x as i32, v.y as i32)).collect();
    sorted.sort_unstable();
    assert_eq!(sorted, [(-1, 0), (0, -1), (0, 1), (1, 0)], "B faced each way: {ways:?}");
}
