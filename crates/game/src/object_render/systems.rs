//! The visuals' systems: making and removing them as the sim's objects come and go ([`sync_object_visuals`]), and
//! placing them every frame ([`place_object_visuals`], [`place_riot_shields`]).

use bevy::prelude::*;

use super::{
    FacingTick, LoadMarker, MeshKey, ObjectVisual, ObjectVisualAssets, ObjectVisuals, RiotShieldVisual, body_turns,
    drawn_kind, has_tick, load, object_height, object_transform, structures, underground, units, visual_mesh,
};
use crate::coords::{facing_to_rotation, lerp_facing};
use crate::kinds::object_label;
use crate::shapes::vertex_color_material;
use crate::sim_driver::{RenderAlpha, SimRes};

/// Spawns a visual for every object without one and despawns visuals whose object is gone, or is drawn otherwise
/// now ([`MeshKey`]: its kind, owner, size or tint changed), which then get a new one, children included (tag, tick,
/// Riot Shield, load marker). A replaced sim (new generation) starts from scratch, because its ids may repeat the old
/// ones.
pub(super) fn sync_object_visuals(
    mut commands: Commands,
    sim: Res<SimRes>,
    mut visuals: ResMut<ObjectVisuals>,
    mut assets: ResMut<ObjectVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !sim.is_changed() {
        return;
    }
    let ObjectVisuals {
        by_id,
        made,
        generation,
    } = &mut *visuals;
    if *generation != Some(sim.generation()) {
        for entity in std::mem::take(by_id).into_values() {
            commands.entity(entity).despawn();
        }
        made.clear();
        *generation = Some(sim.generation());
    }
    by_id.retain(|id, entity| {
        let current = sim
            .object(*id)
            .is_some_and(|o| made.get(id) == Some(&MeshKey::of_object(o)));
        if !current {
            commands.entity(*entity).despawn();
            made.remove(id);
        }
        current
    });
    let assets = &mut *assets;
    let material = assets
        .material
        .get_or_insert_with(|| materials.add(vertex_color_material()))
        .clone();
    let blended = assets
        .underground
        .get_or_insert_with(|| materials.add(structures::underground_material(1.0)))
        .clone();
    for object in sim.objects() {
        if by_id.contains_key(&object.id) {
            continue;
        }
        let key = MeshKey::of_object(object);
        let drawn = key.kind;
        let mesh = assets
            .meshes
            .entry(key)
            .or_insert_with(|| meshes.add(visual_mesh(key)))
            .clone();
        let height = object_height(sim.map(), object.pos, object.placement.as_ref());
        let facing = if body_turns(drawn) { object.facing } else { 0.0 };
        let body = if underground(drawn) { &blended } else { &material };
        let entity = commands
            .spawn((
                Name::new(format!("{:?} #{}", drawn, object.id.0)),
                ObjectVisual(object.id),
                Mesh2d(mesh),
                MeshMaterial2d(body.clone()),
                object_transform(drawn, object.pos, height, facing),
            ))
            .id();
        if let Some(size) = key.size {
            let label = object_label(drawn, object.tunnel.clone());
            commands.spawn((structures::tag(object.id, label, size, 1.0), ChildOf(entity)));
        }
        if units::tagged(drawn) {
            commands.spawn((units::tag(object.id, drawn), ChildOf(entity)));
        }
        if has_tick(drawn) {
            let tick = assets
                .ticks
                .entry(drawn)
                .or_insert_with(|| meshes.add(units::tick_mesh(drawn)))
                .clone();
            commands.spawn((
                Name::new("Facing tick"),
                FacingTick(object.id),
                Mesh2d(tick),
                MeshMaterial2d(material.clone()),
                Transform::from_rotation(facing_to_rotation(object.facing)),
                ChildOf(entity),
            ));
        }
        if object.riot_shield.is_some() {
            let shield = assets
                .shields
                .entry(drawn)
                .or_insert_with(|| meshes.add(units::riot_shield_mesh(drawn)))
                .clone();
            commands.spawn((
                Name::new("Riot Shield"),
                RiotShieldVisual(object.id),
                Mesh2d(shield),
                MeshMaterial2d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
                ChildOf(entity),
            ));
        }
        if load::marked(drawn) {
            let marker = assets
                .loads
                .entry(drawn)
                .or_insert_with(|| meshes.add(load::mesh(drawn)))
                .clone();
            commands.spawn((
                Name::new("Load marker"),
                LoadMarker(object.id),
                Mesh2d(marker),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.004),
                Visibility::Hidden,
                ChildOf(entity),
            ));
        }
        by_id.insert(object.id, entity);
        made.insert(object.id, key);
    }
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`.
pub(super) fn place_object_visuals(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut visuals: Query<(&ObjectVisual, &mut Transform), Without<FacingTick>>,
    mut ticks: Query<(&FacingTick, &mut Transform), Without<ObjectVisual>>,
) {
    for (visual, mut transform) in &mut visuals {
        let Some(object) = sim.object(visual.0) else {
            continue;
        };
        let drawn = drawn_kind(object);
        let pos = object.prev_pos.lerp(object.pos, alpha.0);
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        let height = object_height(sim.map(), pos, object.placement.as_ref());
        let facing = if body_turns(drawn) { facing } else { 0.0 };
        transform.set_if_neq(object_transform(drawn, pos, height, facing));
    }
    for (tick, mut transform) in &mut ticks {
        let Some(object) = sim.object(tick.0) else {
            continue;
        };
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        transform.set_if_neq(Transform::from_rotation(facing_to_rotation(facing)));
    }
}

/// Shows each Riot Shield that isn't down, sized by how far up it is (from 60% of its size, close to the body, to
/// all of it: `sc_sim::ShieldMode::progress`, from the unit's type and the toggle's state, design changes 0171 and
/// 0173), so that only a shield all the way up, the one that protects, is drawn whole; turned with its unit's facing
/// when the body doesn't turn. A raise that waits for its unit's shot (`sc_sim::ShieldMode::Deferred`, design change
/// 0198: the unit is a Peacekeeper (Shielded) from the press) is a shield still down, and shows nothing. Never
/// `Visible`: a shown shield inherits its unit's visibility.
pub(super) fn place_riot_shields(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut shields: Query<(&RiotShieldVisual, &mut Transform, &mut Visibility)>,
) {
    for (shield, mut transform, mut visibility) in &mut shields {
        let Some(object) = sim.object(shield.0) else {
            continue;
        };
        let (Some(mode), Some(def)) = (sc_sim::riot_shield::mode(object), sc_sim::riot_shield::def_of(object)) else {
            continue;
        };
        let shown = !matches!(mode, sc_sim::ShieldMode::Lowered | sc_sim::ShieldMode::Deferred);
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let scale = 0.6 + 0.4 * mode.progress(def.toggle_ticks);
        let rotation = if body_turns(drawn_kind(object)) {
            Quat::IDENTITY
        } else {
            facing_to_rotation(lerp_facing(object.prev_facing, object.facing, alpha.0))
        };
        transform.set_if_neq(Transform::from_rotation(rotation).with_scale(Vec3::new(scale, scale, 1.0)));
    }
}
