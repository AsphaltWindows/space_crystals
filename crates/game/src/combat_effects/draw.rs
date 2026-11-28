//! The effects' mesh: one vertex-coloured, alpha-blended mesh in the overlays layer, rebuilt every frame while
//! anything shows, with the death markers at the back, then (slice M5-N11) the casts and the blasts on the ground, the
//! tracers, the projectiles in the air, the HP bars, (slice M5-N5) the buffs' marks and (slice M5-K) the Target Locks'
//! reticles (placeholders until the art direction, Q0001).

use bevy::prelude::*;
use sc_sim::Map;

use super::blasts::blast_shape;
use super::buffs::{buff_marks, mark_shape};
use super::casts::cast_shape;
use super::locks::{lock_marks, lock_shape};
use super::projectiles::{homing_missiles, missile_shape, projectile_shape};
use super::{
    CombatEffects, CombatEffectsMesh, DeathMarker, HpBar, ShotStyle, Tracer, age, hp_bars, shot_spans, shows_blast,
    shows_cast, shows_death, shows_flight,
};
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::LocalSight;
use crate::object_render::player_color;
use crate::selection::ControlStates;
use crate::shapes::{ColorMesh, darken, lighten};
use crate::sim_driver::{RenderAlpha, SimRes};
use crate::tuning::ClientTuning;

/// A shot leaves a unit this far up the screen from its ground point, su, as from its body rather than its feet; so
/// does a projectile (`super::projectiles`).
pub(super) const MUZZLE_LIFT: f32 = 8.0;
/// A shield bar's fill (placeholder colour, until the art direction: Q0001).
const SHIELD_FILL: Color = Color::srgb(0.35, 0.72, 1.0);
/// (M5-N4a) An energy bar's fill: Placeholder (not in the design: Q0001's art direction), a violet, apart from the
/// HP bar's green, yellow and red and the shield bar's blue.
const ENERGY_FILL: Color = Color::srgb(0.72, 0.45, 1.0);
/// (M6-GH1) An ammo pip's fill, a round left: Placeholder (not in the design: Q0001's art direction), an amber, apart
/// from the bars' colours; a spent round's pip is its dark frame alone.
const AMMO_FILL: Color = Color::srgb(1.0, 0.68, 0.15);
/// (M6-SB) A tank bar's fill, by its kind (`PatchKind::ALL`'s order): Placeholder (not in the design: Q0001's art
/// direction), the sludge's olive brown (`crate::patches_view`) and (slice M6-SC) the smoke's grey; while the toggle
/// is on, lightened by [`TANK_ON_LIGHTEN`].
const TANK_FILLS: [Srgba; 2] = [Srgba::rgb(0.55, 0.50, 0.16), Srgba::rgb(0.62, 0.64, 0.68)];
const TANK_ON_LIGHTEN: f32 = 0.45;
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

/// Rebuilds the effects mesh: death markers at the back, then casts, blasts, tracers, projectiles (the homing missiles too),
/// HP bars, the buffs' marks and the Target Locks' reticles (alpha blending draws a mesh's triangles in order), each
/// only where the local player sees it. Hidden while nothing shows. (M5-X2) How long each lasts, the bars' sizes
/// and the air units' lift are the [`ClientTuning`]'s.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_effects(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    tuning: Res<ClientTuning>,
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
    let (tuning, feedback) = (&*tuning, &tuning.feedback);
    let z = DrawLayer::Overlays.base_z() + EFFECTS_Z;
    let mut mesh = ColorMesh::triangles();
    for death in effects.deaths.iter().filter(|d| shows_death(&sight, d)) {
        let fade = 1.0 - age(&sim, alpha.0, death.tick) / feedback.death_marker_ticks as f32;
        death_marker(&mut mesh, sim.map(), death, fade, z);
    }
    for cast in effects.casts.iter().filter(|c| shows_cast(&sim, &sight, c, tuning)) {
        let t = age(&sim, alpha.0, cast.tick) / feedback.cast_ticks as f32;
        cast_shape(&mut mesh, sim.map(), cast, t, z);
    }
    for blast in effects.blasts.iter().filter(|b| shows_blast(&sim, &sight, b, tuning)) {
        let t = age(&sim, alpha.0, blast.tick) / feedback.blast_ticks as f32;
        blast_shape(&mut mesh, sim.map(), blast, t, z);
    }
    for tracer in &effects.tracers {
        let fade = 1.0 - age(&sim, alpha.0, tracer.tick) / feedback.tracer_ticks as f32;
        for span in shot_spans(&sim, &sight, tracer, tuning) {
            tracer_line(&mut mesh, sim.map(), tracer, span, fade, z);
        }
    }
    for flight in &effects.flights {
        let progress = flight.progress(&sim, alpha.0);
        if shows_flight(&sim, &sight, flight, progress) {
            projectile_shape(&mut mesh, sim.map(), flight, progress, z);
        }
    }
    for missile in homing_missiles(&sim, &sight, alpha.0, tuning) {
        missile_shape(&mut mesh, &missile, z);
    }
    let selection = states.selection(sight.player());
    for bar in hp_bars(&sim, &sight, alpha.0, tuning, |id| selection.contains(id)) {
        hp_bar_shape(&mut mesh, bar, tuning.look.bar_height, z);
    }
    for mark in buff_marks(&sim, &sight, alpha.0, tuning) {
        mark_shape(&mut mesh, mark, z);
    }
    for mark in lock_marks(&sim, &sight, alpha.0, tuning) {
        lock_shape(&mut mesh, &mark, z);
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

/// Where a shot leaves or reaches a unit standing at `ground`, in Bevy world space; (M5-H1) one in flight, `lift`
/// levels above it (`Tracer::lift`).
fn muzzle(map: &Map, ground: Vec2, lift: f32) -> Vec2 {
    view_to_world(drawn_at_height(ground, map.height_at(ground) + lift) - Vec2::new(0.0, MUZZLE_LIFT))
}

/// A bright core inside a glow, both fading out, in the shot's style ([`ShotStyle`]: the glow in the shooter's colour,
/// or a red laser): the part `span` of the line from the attacker's muzzle to the target's ([`visible_spans`]:
/// fractions of the way).
fn tracer_line(mesh: &mut ColorMesh, map: &Map, tracer: &Tracer, span: (f32, f32), fade: f32, z: f32) {
    let fade = fade.clamp(0.0, 1.0);
    let (muzzle_from, muzzle_to) = (
        muzzle(map, tracer.from, tracer.lift.0),
        muzzle(map, tracer.to, tracer.lift.1),
    );
    let (from, to) = (muzzle_from.lerp(muzzle_to, span.0), muzzle_from.lerp(muzzle_to, span.1));
    let (glow, core) = match ShotStyle::of(tracer.weapon) {
        // A grenade takes effect by its blast, never as a shot, (M5-K) and a missile where it lands: drawn as a
        // tracer, were they one.
        ShotStyle::Tracer | ShotStyle::Grenade | ShotStyle::Missile => (
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

/// A dark frame around the bar's fill, each bar `height` su high (`LookTuning::bar_height`): green above 60%, yellow
/// above 30%, red below. The shield bar, if any, right above it, its fill in [`SHIELD_FILL`]; (M5-N4a) the energy
/// bar, if any, right under it, its fill in [`ENERGY_FILL`]; (M6-GH1) an aircraft's ammo pips under them, one square
/// a round, as high as a bar, centred under it, those of the rounds left filled in [`AMMO_FILL`] from the left;
/// (M6-SB) a unit's tank bars under those, one a tank, in its kind's [`TANK_FILLS`], lighter while its toggle is on.
fn hp_bar_shape(mesh: &mut ColorMesh, bar: HpBar, height: f32, z: f32) {
    let center = view_to_world(bar.center);
    let fill = match bar.fraction {
        f if f > 0.6 => Color::srgb(0.30, 0.85, 0.30),
        f if f > 0.3 => Color::srgb(0.95, 0.80, 0.20),
        _ => Color::srgb(0.95, 0.25, 0.20),
    };
    let size = Vec2::new(bar.width, height);
    bar_shape(mesh, center, size, bar.fraction, fill, z);
    if let Some(shields) = bar.shields {
        let above = center + Vec2::new(0.0, height + 1.0);
        bar_shape(mesh, above, size, shields, SHIELD_FILL, z);
    }
    let mut under = center;
    if let Some(energy) = bar.energy {
        under -= Vec2::new(0.0, height + 1.0);
        bar_shape(mesh, under, size, energy, ENERGY_FILL, z);
    }
    if let Some((left, max)) = bar.ammo {
        under -= Vec2::new(0.0, height + 3.0);
        let step = height + 3.0;
        let first = under.x - step * (max as f32 - 1.0) / 2.0;
        for round in 0..max {
            let at = Vec2::new(first + step * round as f32, under.y);
            let full = if round < left { 1.0 } else { 0.0 };
            bar_shape(mesh, at, Vec2::splat(height), full, AMMO_FILL, z);
        }
    }
    for (tank, fill) in bar.tanks.into_iter().zip(TANK_FILLS) {
        let Some((fraction, on)) = tank else {
            continue;
        };
        under -= Vec2::new(0.0, height + 1.0);
        let fill = if on { lighten(fill, TANK_ON_LIGHTEN) } else { fill };
        bar_shape(mesh, under, size, fraction, fill.into(), z);
    }
}

/// A bar centred on `center` (Bevy world space), of `size` su: a dark frame, `fraction` of it filled in `fill` from
/// the left.
fn bar_shape(mesh: &mut ColorMesh, center: Vec2, size: Vec2, fraction: f32, fill: Color, z: f32) {
    let half = size / 2.0;
    mesh.rect(
        center - half - 1.0,
        center + half + 1.0,
        z,
        Color::srgb(0.05, 0.05, 0.05),
    );
    let left = center - half;
    mesh.rect(left, left + Vec2::new(size.x * fraction, size.y), z, fill);
}
