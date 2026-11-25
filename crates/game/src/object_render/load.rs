//! What a worker carries, shown on its body (slice M3-J2): a placeholder marker until the art (Q0001), a Space
//! Crystal shard by the top-right corner of a Drone that carries a load (`factions/colonists/units/drone.md >
//! Gathering`: it picks up its CarryCapacity at the patch and delivers it at its drop-off, and keeps a load through
//! other orders, design change 0129). The marker shows on every worker the local player sees that carries something,
//! anyone's, as the body that carries it shows (placeholder reading: the design doesn't say who sees a load).

use bevy::prelude::*;
use sc_sim::{Object, ObjectId, ObjectKind};

use crate::shapes::{ColorMesh, darken};
use crate::sim_driver::SimRes;

/// The load marker of a worker's visual: a child of the visual, hidden while the worker carries nothing, else
/// inheriting the visual's visibility, so that the fog hides it with its worker (`crate::fog`).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LoadMarker(pub ObjectId);

/// The shard's colour: the Space Crystals' cyan (`super::resources`).
const SHARD_COLOR: Color = Color::srgb(0.36, 0.90, 0.98);
/// Its half width and half height, su.
const SHARD_HALF: Vec2 = Vec2::new(4.5, 7.0);

/// Whether `object` is a worker that carries a load.
pub fn carries(object: &Object) -> bool {
    object.worker.is_some_and(|worker| worker.load > 0)
}

/// Whether a `kind` object's visual gets a load marker: a worker's (`UnitDef::worker`).
pub(super) fn marked(kind: ObjectKind) -> bool {
    kind.def().unit.as_ref().is_some_and(|unit| unit.worker.is_some())
}

/// The marker of a `kind` worker: a diamond, cyan inside a dark rim, by the top-right corner of its body (local
/// space, which doesn't turn: a Drone's body stays axis-aligned), above its tag.
pub(super) fn mesh(kind: ObjectKind) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let center = half - SHARD_HALF * Vec2::new(0.6, 0.4);
    let diamond = |grow: f32| -> Vec<Vec2> {
        let h = SHARD_HALF + Vec2::splat(grow);
        [
            Vec2::new(0.0, h.y),
            Vec2::new(h.x, 0.0),
            Vec2::new(0.0, -h.y),
            Vec2::new(-h.x, 0.0),
        ]
        .map(|p| center + p)
        .to_vec()
    };
    let mut mesh = ColorMesh::triangles();
    mesh.polygon(&diamond(1.5), 0.0, darken(SHARD_COLOR.to_srgba(), 0.25).into());
    mesh.polygon(&diamond(0.0), 0.001, SHARD_COLOR);
    mesh.into_mesh()
}

/// Shows each load marker whose worker carries a load. Never `Visible`: a shown marker inherits its worker's
/// visibility.
pub(super) fn place_load_markers(sim: Res<SimRes>, mut markers: Query<(&LoadMarker, &mut Visibility)>) {
    for (marker, mut visibility) in &mut markers {
        let shown = sim.object(marker.0).is_some_and(carries);
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
