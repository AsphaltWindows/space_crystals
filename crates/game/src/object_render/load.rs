//! What a worker carries, shown on its body (slice M3-J2): a placeholder marker until the art (Q0001), a Space
//! Crystal shard by the top-right corner of a worker that carries Space Crystals (`factions/colonists/units/drone.md >
//! Gathering`: it picks up its CarryCapacity at the patch and delivers it at its drop-off, and keeps a load through
//! other orders, design change 0129), and a supply crate there on an Agent that carries a Supply instead
//! (`factions/syndicate/units/agent.md > Gathering > Supplies`, slice M4-J1: **placeholder look**, an amber crate
//! with a dark band, so that the two loads never look alike). The marker shows on every worker the local player sees
//! that carries something, anyone's, as the body that carries it shows (placeholder reading: the design doesn't say
//! who sees a load).

use bevy::prelude::*;
use sc_sim::defs::Resource;
use sc_sim::{Object, ObjectId, ObjectKind};

use crate::shapes::{ColorMesh, darken};
use crate::sim_driver::SimRes;

/// The load marker of a worker's visual for loads of one resource: a child of the visual, hidden while the worker
/// carries none of it, else inheriting the visual's visibility, so that the fog hides it with its worker
/// (`crate::fog`). A worker that gathers Supplies has one of each ([`carried`]).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LoadMarker(pub ObjectId, pub Resource);

/// The shard's colour: the Space Crystals' cyan (`super::resources`).
const SHARD_COLOR: Color = Color::srgb(0.36, 0.90, 0.98);
/// Its half width and half height, su.
const SHARD_HALF: Vec2 = Vec2::new(4.5, 7.0);
/// The supply crate's colour, and its half side, su (placeholders).
const CRATE_COLOR: Color = Color::srgb(0.95, 0.66, 0.22);
const CRATE_HALF: f32 = 5.5;

/// Whether `object` is a worker that carries a load.
pub fn carries(object: &Object) -> bool {
    object.worker.is_some_and(|worker| worker.load > 0)
}

/// Whether `object` is a worker that carries a load of `resource`.
pub fn carries_of(object: &Object, resource: Resource) -> bool {
    object
        .worker
        .is_some_and(|worker| worker.load > 0 && worker.resource == resource)
}

/// The resources whose loads a `kind` object's visual shows a marker for: a worker's Space Crystals
/// (`UnitDef::worker`), and its Supplies if it gathers them (`WorkerDef::supplies`: the Agent); none for anything else.
pub(super) fn carried(kind: ObjectKind) -> Vec<Resource> {
    let worker = kind.def().unit.as_ref().and_then(|unit| unit.worker.as_ref());
    match worker {
        Some(worker) if worker.supplies.is_some() => vec![Resource::Crystals, Resource::Supplies],
        Some(_) => vec![Resource::Crystals],
        None => Vec::new(),
    }
}

/// The marker of a `kind` worker's loads of `resource`, by the top-right corner of its body (local space, which
/// doesn't turn: a worker's body stays axis-aligned), above its tag: a diamond, cyan inside a dark rim, for Space
/// Crystals; a square crate, amber inside a dark rim and crossed by a dark band, for Supplies.
pub(super) fn mesh(kind: ObjectKind, resource: Resource) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let center = half - SHARD_HALF * Vec2::new(0.6, 0.4);
    let mut mesh = ColorMesh::triangles();
    match resource {
        Resource::Crystals => {
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
            mesh.polygon(&diamond(1.5), 0.0, darken(SHARD_COLOR.to_srgba(), 0.25).into());
            mesh.polygon(&diamond(0.0), 0.001, SHARD_COLOR);
        }
        Resource::Supplies => {
            let dark: Color = darken(CRATE_COLOR.to_srgba(), 0.3).into();
            let rim = Vec2::splat(CRATE_HALF + 1.5);
            mesh.rect(center - rim, center + rim, 0.0, dark);
            mesh.rect(
                center - Vec2::splat(CRATE_HALF),
                center + Vec2::splat(CRATE_HALF),
                0.001,
                CRATE_COLOR,
            );
            let band = Vec2::new(CRATE_HALF, 1.2);
            mesh.rect(center - band, center + band, 0.002, dark);
        }
    }
    mesh.into_mesh()
}

/// Shows each load marker whose worker carries a load of its resource. Never `Visible`: a shown marker inherits its
/// worker's visibility.
pub(super) fn place_load_markers(sim: Res<SimRes>, mut markers: Query<(&LoadMarker, &mut Visibility)>) {
    for (marker, mut visibility) in &mut markers {
        let shown = sim.object(marker.0).is_some_and(|o| carries_of(o, marker.1));
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
