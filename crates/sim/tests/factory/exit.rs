//! The Factory's way out (`factions/gdo/structures/factory.md`: "SymmetryType ABAC (exit side B, like the Barracks)";
//! `core/entities.md > Entrance and exit > Exit Doorway`): a vehicle appears "half inside the building, at the middle
//! of the exit side", facing out of it, its box turned with it (plan decision T100; the 64 × 48 Artillery Rig's box is
//! the sim's unit test, `sim/producers/tests.rs`), and drives to the rally point the Factory starts with, 3 grid units
//! out (design changes 0446 and 0447).

use sc_sim::{BuildingOrientation, Rotation, math};

use super::*;

/// Every orientation a building can be placed in: four turns, each flipped either way or not.
fn orientations() -> Vec<BuildingOrientation> {
    let mut all = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        for (flip_h, flip_v) in [(false, false), (true, false), (false, true), (true, true)] {
            all.push(BuildingOrientation {
                rotation,
                flip_h,
                flip_v,
            });
        }
    }
    all
}

/// Design change 0446's Verification, the sim's half (the client shows it, `crates/game/tests/commands/factory.rs`):
/// "a new Factory shows its rally point 3 GU out of side B". In every orientation, turned or flipped, a Factory has it
/// on the outward normal of the middle of side B, wherever that side faces then: 3 grid units beyond its 3 × 3
/// footprint's edge there. The gate would take it as a rally point set by hand.
#[test]
fn a_factory_starts_with_its_rally_point_three_grid_units_out_of_side_b() {
    for orientation in orientations() {
        let mut scenario = base().spawn(FACTORY, Some(P1), SpawnAt::Tile(WORKS_AT)).build();
        let factory = scenario.map.spawns.len() - 1;
        scenario.map.spawns[factory].orientation = orientation;
        let t = TestSim::new(scenario);
        let door = door(&t, WORKS);
        let centre = object(&t, WORKS).placement.as_ref().unwrap().center();
        let expected = centre + door.outward() * (96.0 + RALLY_OUT);
        assert_eq!(rally_of(&t, WORKS), Some(Rally::Point(expected)), "{orientation:?}");
        assert!(
            near(door.middle().distance(centre), 96.0, 1e-3),
            "{orientation:?}: the middle of side B"
        );
        let set = production(WORKS, ProductionCommand::SetRally(at(expected)));
        assert_eq!(gate(&t, P1, set), Ok(()), "{orientation:?}");
    }
}

/// Design change 0446's Verification: "five Raptors queued with no orders leave without stalling the door"; and the
/// plan's "Five Raptors queued leave by B, facing out, with no overlap". Queued at once in tick 2, they come out one
/// every 240 frames, never held back by the doorway (the Factory never waits, `ProductionStatus::waiting`), each half
/// inside the middle of side B, facing out of it, its box turned with it (48 su out of the side, 32 su along it); no
/// two Raptors' boxes ever overlap (checked every tick); and each drives off to the rally point, all five out of the
/// doorway at the end, stopped, idle, round the point, the first on it (their parking, design change 0451, is
/// `parking.rs`').
#[test]
fn five_raptors_come_out_of_side_b_facing_out_and_never_stall_it() {
    let mut t = works(0);
    for _ in 0..5 {
        t.at(2, P1, train(RAPTOR));
    }
    let door = door(&t, WORKS);
    let Some(Rally::Point(point)) = rally_of(&t, WORKS) else {
        panic!("a default rally point");
    };
    let mut seen = 0;
    let last = 1 + 5 * RAPTOR_FRAMES;
    while t.sim().tick() < last + 150 {
        t.step();
        let tick = t.sim().tick();
        let stalled = t.sim().production_status(WORKS).is_some_and(|status| status.waiting);
        assert!(!stalled, "tick {tick}: stalled\n{}", t.ascii(None));
        let raptors: Vec<ObjectId> = produced(&t, WORKS).iter().map(|&(_, unit)| unit).collect();
        for &raptor in &raptors[seen..] {
            let o = object(&t, raptor);
            assert_eq!(o.kind, RAPTOR);
            assert!(
                o.pos.distance(door.middle()) < 1e-3,
                "tick {tick}: half inside the middle of side B: {}",
                o.pos
            );
            assert!(
                math::facing_vector(o.facing).distance(door.outward()) < 1e-6,
                "tick {tick}: facing out"
            );
            let half = shape(&t, raptor).bound_half();
            let along = door.outward().abs();
            assert!(
                near(half.dot(along), 24.0, 1e-3) && near(half.dot(Vec2::ONE - along), 16.0, 1e-3),
                "{half}"
            );
        }
        seen = raptors.len();
        for (i, &a) in raptors.iter().enumerate() {
            for &b in &raptors[i + 1..] {
                assert!(
                    !shape(&t, a).overlaps(&shape(&t, b)),
                    "tick {tick}: {a:?} and {b:?} overlap\n{}",
                    t.ascii(None)
                );
            }
        }
    }
    let out: Vec<(Tick, ObjectId)> = produced(&t, WORKS);
    let ticks: Vec<Tick> = out.iter().map(|&(tick, _)| tick).collect();
    let expected: Vec<Tick> = (1..=5).map(|n| 1 + n * RAPTOR_FRAMES).collect();
    assert_eq!(ticks, expected, "one every 240 frames");
    assert_eq!(t.sim().production_status(WORKS), None, "its queue done");
    let reach = RAPTOR.def().half_extents().unwrap().length();
    for (i, &(_, raptor)) in out.iter().enumerate() {
        assert!(parked(&t, raptor), "Raptor {i}: parked: {:?}", t.mover(raptor.0));
        let o = object(&t, raptor);
        assert!(
            o.pos.distance(point) <= RALLY_OUT + reach,
            "Raptor {i}: round the rally point: {}",
            o.pos
        );
    }
    assert!(object(&t, out[0].1).pos.distance(point) < 1.0, "the first on it");
}
