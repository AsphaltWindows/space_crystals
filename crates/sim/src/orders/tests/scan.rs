//! A target scan's SightRange (`systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`): measured
//! from the unit's centre to the nearest point of the target's shape, so that the unit's own width never counts.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;
use crate::units::{SU_PER_GU, TilePos};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SCANNER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);
/// The Guard's SightRange (`factions/syndicate/units/guard.md`: "SightRange - 5"), space units.
const SIGHT_SU: f32 = 5.0 * SU_PER_GU;

/// On 16 × 10 open tiles, player 1's idle Guard (36 su wide) at `at`, player 2's `target` at `spawn`, on Hold, and,
/// if `spotter` is given, player 1's Shadebringer (no weapon) there, which sees the target from the start: whether
/// the Guard's first scans took the target.
fn first_scan(at: Vec2, target: ObjectKind, spawn: SpawnAt, spotter: Option<Vec2>) -> bool {
    let mut builder = ScenarioBuilder::new(&"................\n".repeat(10))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(at))
        .spawn(target, Some(P2), spawn);
    if let Some(spotter) = spotter {
        builder = builder.spawn(ObjectKind::Shadebringer, Some(P1), SpawnAt::Point(spotter));
    }
    let mut t = builder.start();
    if target.def().unit.is_some() {
        t.order_at(1, P2, &[TARGET], OrderKind::Hold, false);
    }
    t.run_until(3);
    if spotter.is_some() {
        assert!(
            t.sim().is_visible(P1, t.sim().object(TARGET).unwrap()),
            "the spotter sees it"
        );
    }
    t.sim().object(SCANNER).unwrap().attack.as_ref().unwrap().target == Some(TARGET)
}

/// Asserts that the Guard, its centre `d` su west of `edge` (the point of the target's west edge on the Guard's row),
/// takes the target exactly while `d` is at most its SightRange, the boundary included: swept 1 su at a time across
/// it.
fn takes_from_sight_range_off_its_edge(edge: Vec2, target: ObjectKind, spawn: SpawnAt, spotter: Option<Vec2>) {
    for d in (0..=40).map(|i| SIGHT_SU - 20.0 + i as f32) {
        let took = first_scan(edge - Vec2::new(d, 0.0), target, spawn, spotter);
        assert_eq!(
            took,
            d <= SIGHT_SU,
            "{target:?}, the Guard's centre {d} su off its edge"
        );
    }
}

/// A lone Guard and a Peacekeeper (24 su wide) whose west edge runs through the sight point of tile (11, 3), so that
/// the Guard's own eye sees it from as far as the scan reaches: taken from SightRange off its edge, and no farther
/// (an edge-to-edge scan would reach 18 su farther, the Guard's half width).
#[test]
fn a_lone_unit_takes_a_target_from_its_sight_range_off_the_targets_edge() {
    let edge = TilePos::new(11, 3).center();
    let half = ObjectKind::Peacekeeper.def().half_extents().unwrap().x;
    let spawn = SpawnAt::Point(edge + Vec2::new(half, 0.0));
    takes_from_sight_range_off_its_edge(edge, ObjectKind::Peacekeeper, spawn, None);
}

/// A Deployment Center on tiles (10, 2) to (13, 5), spotted: taken from SightRange off its west edge, x 640, though
/// the Guard's own eye sees it only from 32 su nearer (the sight point of its nearest tile); the scan, not the eye,
/// decides how far.
#[test]
fn a_spotted_structure_is_taken_from_sight_range_off_its_edge() {
    let edge = TilePos::new(10, 3).corner() + Vec2::new(0.0, SU_PER_GU / 2.0);
    let spotter = TilePos::new(15, 3).center();
    let spawn = SpawnAt::Tile(TilePos::new(10, 2));
    takes_from_sight_range_off_its_edge(edge, ObjectKind::DeploymentCenter, spawn, Some(spotter));
}
