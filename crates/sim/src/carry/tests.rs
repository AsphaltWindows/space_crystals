use glam::Vec2;

use super::*;
use crate::ids::{ObjectId, PlayerId};
use crate::scenario::SpawnAt;
use crate::shape::Shape;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

/// A `kind` object of player 1 at (400, 400), facing `facing`, alone on an open map.
fn object(kind: ObjectKind, facing: f32) -> Object {
    let builder = ScenarioBuilder::new(&"................................\n".repeat(32)).spawn(
        kind,
        Some(PlayerId(1)),
        SpawnAt::Point(Vec2::new(400.0, 400.0)),
    );
    let sim = Sim::new(&builder.build());
    let mut o = sim.objects().find(|o| o.id == ObjectId(1)).cloned().expect("spawned");
    o.facing = facing;
    o
}

/// The circle round a Guard's centre that holds its Silhouette however it lies, as `Sim::unload` spaces the rings.
fn guard_radius() -> f32 {
    (ObjectKind::Guard.def().silhouette().expect("a unit") / 2.0).length()
}

/// The Transport above ground carries 4 infantry, Light or Heavy, a place each, and nothing else; it loads, and its
/// burrowed type, a carrier still, doesn't (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`:
/// "it must be above ground").
#[test]
fn the_transport_carries_four_infantry_above_ground() {
    let def = def_of_kind(ObjectKind::SubterraneanTransport).expect("a carrier");
    assert_eq!(def.capacity, 4);
    assert_eq!(places(def, ObjectKind::Guard), Some(1));
    assert_eq!(places(def, ObjectKind::Recruit), Some(1));
    assert_eq!(places(def, ObjectKind::AssaultBike), None);
    assert_eq!(places(def, ObjectKind::SubterraneanTransport), None);
    assert_eq!(places(def, ObjectKind::Tunnel), None);
    assert!(def_of_kind(ObjectKind::SubterraneanTransportBurrowed).is_some());
    assert!(def_of_kind(ObjectKind::Guard).is_none());

    let surfaced = object(ObjectKind::SubterraneanTransport, 0.0);
    assert!(loads(&surfaced));
    assert_eq!(admits(&surfaced, ObjectKind::Guard), Ok(()));
    assert_eq!(admits(&surfaced, ObjectKind::AssaultBike), Err(NoTransit::NotAccepted));
    assert_eq!(admits(&surfaced, ObjectKind::Tunnel), Err(NoTransit::NotAUnit));
    let mut neutral = surfaced.clone();
    neutral.owner = None;
    assert_eq!(admits(&neutral, ObjectKind::Guard), Err(NoTransit::NotATunnel));

    let mut burrowed = surfaced.clone();
    burrowed.kind = ObjectKind::SubterraneanTransportBurrowed;
    assert!(!loads(&burrowed));
    assert_eq!(admits(&burrowed, ObjectKind::Guard), Err(NoTransit::NotATunnel));
    let guard = object(ObjectKind::Guard, 0.0);
    assert!(!loads(&guard));
    assert_eq!(admits(&guard, ObjectKind::Guard), Err(NoTransit::NotATunnel));
}

/// The rings round a Transport, at every facing: each spot's Guard is clear of the Transport, and of every other
/// spot's on its ring and on the next; the first spot is the one nearest the middle of its back.
#[test]
fn the_rings_spread_round_the_carrier_without_overlap() {
    let radius = guard_radius();
    for step in 0..16 {
        let facing = step as f32 * std::f32::consts::TAU / 16.0;
        let carrier = object(ObjectKind::SubterraneanTransport, facing);
        let hull = Shape::silhouette(carrier.kind, carrier.pos, facing);
        let rings: Vec<Vec<Vec2>> = (0..3).map(|ring| ring_spots(&carrier, radius, ring)).collect();
        for (ring, spots) in rings.iter().enumerate() {
            assert!(spots.len() >= 4, "ring {ring} at {facing}: {} spots", spots.len());
            for (i, &p) in spots.iter().enumerate() {
                let guard = Shape::silhouette(ObjectKind::Guard, p, facing);
                assert!(
                    !guard.overlaps(&hull),
                    "ring {ring} spot {i} overlaps the carrier at {facing}"
                );
                let others = (spots.iter().skip(i + 1)).chain(rings.get(ring + 1).into_iter().flatten());
                for &q in others {
                    assert!(p.distance(q) >= 2.0 * radius, "{p} and {q} at {facing}");
                }
            }
        }
        // Behind it: the first spot of the nearest ring is the back's, ahead of none of the others.
        let ahead = math::facing_vector(facing);
        let along = |p: Vec2| (p - carrier.pos).dot(ahead);
        let first = along(rings[0][0]);
        assert!(first < 0.0);
        assert!(rings[0].iter().all(|&p| along(p) >= first - 1e-3));
    }
}

/// Each ring is wider than the one inside it, and has room for at least as many: the four Guards of a full Transport
/// all have a spot on the nearest one.
#[test]
fn each_ring_holds_at_least_the_one_inside_it() {
    let carrier = object(ObjectKind::SubterraneanTransport, 0.0);
    let counts: Vec<usize> = (0..3)
        .map(|ring| ring_spots(&carrier, guard_radius(), ring).len())
        .collect();
    assert!(counts[0] >= 4, "{counts:?}");
    assert!(counts.windows(2).all(|pair| pair[0] <= pair[1]), "{counts:?}");
}
