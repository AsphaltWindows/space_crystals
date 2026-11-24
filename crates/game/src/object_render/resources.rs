//! The map resources' placeholder visuals (until the art direction, Q0001, settles them): a Space Crystals patch and a
//! Supply Delivery Station, each in colours of its own, never an owner's.

use bevy::prelude::*;

use crate::shapes::ColorMesh;

/// `core/map.md > SpaceCrystalsPatch` (1×1): a cluster of three cyan crystals on a dark bed.
pub(super) fn crystal_patch_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    let bed: Vec<Vec2> = (0..8)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            Vec2::new(25.0 * a.cos(), -10.0 + 11.0 * a.sin())
        })
        .collect();
    mesh.polygon(&bed, 0.0, Color::srgb(0.09, 0.14, 0.18));
    crystal_shard(&mut mesh, Vec2::new(-13.0, -8.0), Vec2::new(7.0, 15.0), 0.001);
    crystal_shard(&mut mesh, Vec2::new(13.0, -10.0), Vec2::new(7.0, 13.0), 0.001);
    crystal_shard(&mut mesh, Vec2::new(0.0, -2.0), Vec2::new(10.0, 24.0), 0.002);
    mesh.into_mesh()
}

/// A five-sided shard standing on its base, its facets shaded as if lit from the top left.
fn crystal_shard(mesh: &mut ColorMesh, center: Vec2, half: Vec2, z: f32) {
    let apex = center + Vec2::new(0.0, half.y);
    let upper_right = center + Vec2::new(half.x, half.y * 0.35);
    let lower_right = center + Vec2::new(half.x * 0.7, -half.y);
    let lower_left = center + Vec2::new(-half.x * 0.7, -half.y);
    let upper_left = center + Vec2::new(-half.x, half.y * 0.35);
    let core = center + Vec2::new(0.0, half.y * 0.1);
    let facets = [
        (upper_left, apex, Color::srgb(0.72, 1.0, 1.0)),
        (apex, upper_right, Color::srgb(0.36, 0.90, 0.98)),
        (upper_right, lower_right, Color::srgb(0.07, 0.48, 0.66)),
        (lower_right, lower_left, Color::srgb(0.04, 0.32, 0.47)),
        (lower_left, upper_left, Color::srgb(0.17, 0.72, 0.86)),
    ];
    for (a, b, color) in facets {
        mesh.polygon(&[core, a, b], z, color);
    }
}

/// `core/map.md > SupplyDeliveryStation` (2×2): an amber landing pad with a cross and corner lights.
pub(super) fn supply_station_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    mesh.rect(
        Vec2::splat(-60.0),
        Vec2::splat(60.0),
        0.0,
        Color::srgb(0.28, 0.18, 0.07),
    );
    mesh.rect(
        Vec2::splat(-52.0),
        Vec2::splat(52.0),
        0.001,
        Color::srgb(0.85, 0.55, 0.13),
    );
    let mark = Color::srgb(0.40, 0.24, 0.06);
    mesh.rect(Vec2::new(-30.0, -5.0), Vec2::new(30.0, 5.0), 0.002, mark);
    mesh.rect(Vec2::new(-5.0, -30.0), Vec2::new(5.0, 30.0), 0.002, mark);
    for corner in [
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
    ] {
        let c = corner * 42.0;
        mesh.rect(c - 5.0, c + 5.0, 0.002, Color::srgb(1.0, 0.93, 0.55));
    }
    mesh.into_mesh()
}
