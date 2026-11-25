//! The effects' mesh: one vertex-coloured, alpha-blended mesh in the overlays layer, rebuilt every frame while
//! anything shows, with the death markers at the back, then the blasts on the ground, the tracers, the projectiles in
//! the air, and the HP bars (placeholders until the art direction, Q0001).

use bevy::prelude::*;
use sc_sim::Map;

use super::blasts::blast_shape;
use super::projectiles::projectile_shape;
use super::{
    BAR_HEIGHT, CombatEffects, CombatEffectsMesh, DEATH_MARKER_TICKS, DeathMarker, HpBar, ShotStyle, TRACER_TICKS,
    Tracer, age, hp_bars, shot_spans, shows_blast, shows_death, shows_flight,
};
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::LocalSight;
use crate::object_render::player_color;
use crate::selection::ControlStates;
use crate::shapes::{ColorMesh, darken, lighten};
use crate::sim_driver::{RenderAlpha, SimRes};

/// A shot leaves a unit this far up the screen from its ground point, su, as from its body rather than its feet; so
/// does a projectile (`super::projectiles`).
pub(super) const MUZZLE_LIFT: f32 = 8.0;
/// A shield bar's fill (placeholder colour, until the art direction: Q0001).
const SHIELD_FILL: Color = Color::srgb(0.35, 0.72, 1.0);
/// Above the path overlay, in the overlays layer.
const EFFECTS_Z: f32 = 1.0;

/// A red laser's glow and core (placeholders: 0207 says only "red").
const LASER_GLOW: Srgba = Srgba::rgb(1.0, 0.12, 0.10);
const LASER_CORE: Srgba = Srgba::rgb(1.0, 0.62, 0.58);

pub(super) fn spawn_effects_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // A mesh without vertices makes the renderer warn: a zero-area stand-in until the first effect.
    let mut placeholder = ColorMesh::triangles();
    placeholder.rect(Vec2::ZERO, Vec2::ZERO, 0.0, Color::NONE);
    commands.spawn((
        Name::new("Combat effects"),
        CombatEffectsMesh,
        Mesh2d(meshes.add(placeholder.into_mesh())),
        // White and alpha-blended (`ColorMaterial::default`): the vertex colours fade the effects.
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}

/// Rebuilds the effects mesh: death markers at the back, then blasts, tracers, projectiles and HP bars (alpha blending
/// draws a mesh's triangles in order), each only where the local player sees it. Hidden while nothing shows.
pub(super) fn draw_effects(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    effects: Res<CombatEffects>,
    states: Res<ControlStates>,
    sight: LocalSight,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<CombatEffectsMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let sight = sight.get();
    let z = DrawLayer::Overlays.base_z() + EFFECTS_Z;
    let mut mesh = ColorMesh::triangles();
    for death in effects.deaths.iter().filter(|d| shows_death(&sight, d)) {
        let fade = 1.0 - age(&sim, alpha.0, death.tick) / DEATH_MARKER_TICKS as f32;
        death_marker(&mut mesh, sim.map(), death, fade, z);
    }
    for blast in effects.blasts.iter().filter(|b| shows_blast(&sim, &sight, b)) {
        blast_shape(&mut mesh, sim.map(), blast, age(&sim, alpha.0, blast.tick), z);
    }
    for tracer in &effects.tracers {
        let fade = 1.0 - age(&sim, alpha.0, tracer.tick) / TRACER_TICKS as f32;
        for span in shot_spans(&sim, &sight, tracer) {
            tracer_line(&mut mesh, sim.map(), tracer, span, fade, z);
        }
    }
    for flight in &effects.flights {
        let progress = flight.progress(&sim, alpha.0);
        if shows_flight(&sim, &sight, flight, progress) {
            projectile_shape(&mut mesh, sim.map(), flight, progress, z);
        }
    }
    let selection = states.selection(sight.player());
    for bar in hp_bars(&sim, &sight, alpha.0, |id| selection.contains(id)) {
        hp_bar_shape(&mut mesh, bar, z);
    }
    if mesh.vertex_count() == 0 {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    if let Err(error) = meshes.insert(&mesh2d.0, mesh.into_mesh()) {
        error!("combat effects rebuild failed: {error}");
    }
    visibility.set_if_neq(Visibility::Inherited);
}

/// Where a shot leaves or reaches a unit standing at `ground`, in Bevy world space.
fn muzzle(map: &Map, ground: Vec2) -> Vec2 {
    view_to_world(drawn_at_height(ground, map.height_at(ground)) - Vec2::new(0.0, MUZZLE_LIFT))
}

/// A bright core inside a glow, both fading out, in the shot's style ([`ShotStyle`]: the glow in the shooter's colour,
/// or a red laser): the part `span` of the line from the attacker's muzzle to the target's ([`visible_spans`]:
/// fractions of the way).
fn tracer_line(mesh: &mut ColorMesh, map: &Map, tracer: &Tracer, span: (f32, f32), fade: f32, z: f32) {
    let fade = fade.clamp(0.0, 1.0);
    let (muzzle_from, muzzle_to) = (muzzle(map, tracer.from), muzzle(map, tracer.to));
    let (from, to) = (muzzle_from.lerp(muzzle_to, span.0), muzzle_from.lerp(muzzle_to, span.1));
    let (glow, core) = match ShotStyle::of(tracer.weapon) {
        // A grenade takes effect by its blast, never as a shot: drawn as a tracer, were it one.
        ShotStyle::Tracer | ShotStyle::Grenade => (
            lighten(player_color(tracer.owner).to_srgba(), 0.3),
            Srgba::rgb(1.0, 0.96, 0.7),
        ),
        ShotStyle::RedLaser => (LASER_GLOW, LASER_CORE),
    };
    mesh.segment(from, to, 5.0, z, glow.with_alpha(0.45 * fade).into());
    mesh.segment(from, to, 2.0, z, core.with_alpha(fade).into());
}

/// A dark scorch with a cross in the dead object's colour, fading out.
fn death_marker(mesh: &mut ColorMesh, map: &Map, death: &DeathMarker, fade: f32, z: f32) {
    let fade = fade.clamp(0.0, 1.0);
    let center = view_to_world(drawn_at_height(death.pos, map.height_at(death.pos)));
    let radius = death.kind.def().half_extents().map_or(12.0, |h| h.max_element()) + 4.0;
    let scorch: Vec<Vec2> = (0..12)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 12.0;
            center + Vec2::new(a.cos(), a.sin() * 0.7) * radius
        })
        .collect();
    mesh.polygon(&scorch, z, Color::srgba(0.06, 0.05, 0.04, 0.7 * fade));
    let color = darken(player_color(death.owner).to_srgba(), 0.8).with_alpha(fade);
    let arm = radius * 0.6;
    for diagonal in [Vec2::new(arm, arm), Vec2::new(arm, -arm)] {
        mesh.segment(center - diagonal, center + diagonal, 3.0, z, color.into());
    }
}

/// A dark frame around the bar's fill: green above 60%, yellow above 30%, red below. The shield bar, if any, right
/// above it, its fill in [`SHIELD_FILL`].
fn hp_bar_shape(mesh: &mut ColorMesh, bar: HpBar, z: f32) {
    let center = view_to_world(bar.center);
    let fill = match bar.fraction {
        f if f > 0.6 => Color::srgb(0.30, 0.85, 0.30),
        f if f > 0.3 => Color::srgb(0.95, 0.80, 0.20),
        _ => Color::srgb(0.95, 0.25, 0.20),
    };
    bar_shape(mesh, center, bar.width, bar.fraction, fill, z);
    if let Some(shields) = bar.shields {
        let above = center + Vec2::new(0.0, BAR_HEIGHT + 1.0);
        bar_shape(mesh, above, bar.width, shields, SHIELD_FILL, z);
    }
}

/// A bar centred on `center` (Bevy world space), `width` su wide: a dark frame, `fraction` of it filled in `fill`
/// from the left.
fn bar_shape(mesh: &mut ColorMesh, center: Vec2, width: f32, fraction: f32, fill: Color, z: f32) {
    let half = Vec2::new(width / 2.0, BAR_HEIGHT / 2.0);
    mesh.rect(
        center - half - 1.0,
        center + half + 1.0,
        z,
        Color::srgb(0.05, 0.05, 0.05),
    );
    let left = center - half;
    mesh.rect(left, left + Vec2::new(width * fraction, BAR_HEIGHT), z, fill);
}
