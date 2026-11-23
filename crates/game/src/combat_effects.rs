//! Placeholder combat feedback (`systems/combat.md`; the art waits on Q0001): a short tracer from the attacker to
//! its target for each attack that takes effect (`SimEvent::Attacked`), HP bars over damaged and selected objects,
//! and a fading marker where an object died (`SimEvent::Died`).
//!
//! Everything is one vertex-coloured, alpha-blended mesh in the overlays layer, rebuilt every frame while anything
//! shows. Effects age by the sim's ticks plus the render alpha, so a QA screenshot of a tick (alpha 1) always shows
//! them alike. HP bars follow the units' interpolated positions.

use bevy::prelude::*;
use sc_sim::{Map, Object, ObjectId, ObjectKind, PlayerId, Sim, SimEvent, Tick};

use crate::ClientSystems;
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::object_render::{object_height, player_color};
use crate::selection::ControlStates;
use crate::shapes::{ColorMesh, darken, lighten};
use crate::sim_driver::{LocalPlayer, RenderAlpha, SimEventMessage, SimRes};

/// How long a tracer shows, ticks. Placeholder until the art direction (Q0001).
pub const TRACER_TICKS: u32 = 5;
/// How long a death marker shows, ticks. Placeholder until the art direction (Q0001).
pub const DEATH_MARKER_TICKS: u32 = 40;
/// A shot leaves a unit this far up the screen from its ground point, su, as from its body rather than its feet.
const MUZZLE_LIFT: f32 = 8.0;
/// HP bars: height, gap above the corners of the unit's silhouette (clear of a selection ring, which lies 7 su
/// outside them: `selection::markers`), and the narrowest bar, su.
const BAR_HEIGHT: f32 = 4.0;
const BAR_GAP: f32 = 9.0;
const BAR_MIN_WIDTH: f32 = 24.0;
/// Above the path overlay, in the overlays layer.
const EFFECTS_Z: f32 = 1.0;

/// An attack that took effect in `tick` (`SimEvent::Attacked`), from and to the ground points of both units then.
#[derive(Clone, Debug, PartialEq)]
pub struct Tracer {
    pub tick: Tick,
    pub owner: Option<PlayerId>,
    pub from: Vec2,
    pub to: Vec2,
}

/// An object that died in `tick`, where it stood (`SimEvent::Died`).
#[derive(Clone, Debug, PartialEq)]
pub struct DeathMarker {
    pub tick: Tick,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
}

/// The tracers and death markers on screen, from the sim's events, dropped once they have faded.
#[derive(Resource, Default, Debug)]
pub struct CombatEffects {
    pub tracers: Vec<Tracer>,
    pub deaths: Vec<DeathMarker>,
    /// The `SimRes::generation` they belong to.
    generation: Option<u32>,
}

/// An HP bar: centred on `center` (view space), `width` su wide, `fraction` of it full.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HpBar {
    pub center: Vec2,
    pub width: f32,
    pub fraction: f32,
}

/// The entity holding the effects mesh.
#[derive(Component)]
pub struct CombatEffectsMesh;

pub struct CombatEffectsPlugin;

impl Plugin for CombatEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatEffects>()
            .add_systems(Startup, spawn_effects_mesh)
            .add_systems(Update, collect_effects.in_set(ClientSystems::SyncSim))
            .add_systems(Update, draw_effects.in_set(ClientSystems::Present));
    }
}

/// How long ago `tick` was, in ticks, as drawn now: the sim's tick shows at alpha 1, so an effect of the latest tick
/// is 0 ticks old then.
fn age(sim: &Sim, alpha: f32, tick: Tick) -> f32 {
    (sim.tick().saturating_sub(tick) as f32 + alpha - 1.0).max(0.0)
}

/// Whether something that happened in `tick` and lasts `ticks` still shows.
fn shows(sim: &Sim, tick: Tick, ticks: u32) -> bool {
    tick <= sim.tick() && sim.tick() - tick < ticks
}

/// The HP bars to draw: one over each object that is damaged or `selected`, at its interpolated position.
pub fn hp_bars(sim: &Sim, alpha: f32, selected: impl Fn(ObjectId) -> bool) -> Vec<HpBar> {
    sim.objects()
        .filter_map(|o| {
            let (hp, max_hp) = (o.hp?, o.def().max_hp?);
            (hp < max_hp || selected(o.id)).then(|| hp_bar(sim.map(), o, alpha, hp / max_hp))
        })
        .collect()
}

fn hp_bar(map: &Map, o: &Object, alpha: f32, fraction: f32) -> HpBar {
    let pos = o.prev_pos.lerp(o.pos, alpha);
    let half = o.def().half_extents().unwrap_or(Vec2::splat(BAR_MIN_WIDTH / 2.0));
    let drawn = drawn_at_height(pos, object_height(map, pos, o.placement.as_ref()));
    HpBar {
        center: drawn - Vec2::new(0.0, half.length() + BAR_GAP + BAR_HEIGHT / 2.0),
        width: (half.x * 2.0).max(BAR_MIN_WIDTH),
        fraction: fraction.clamp(0.0, 1.0),
    }
}

fn spawn_effects_mesh(
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

/// Takes the new shots and deaths from the tick's events and drops the effects that have faded. A replaced sim
/// starts from none.
fn collect_effects(sim: Res<SimRes>, mut messages: MessageReader<SimEventMessage>, mut effects: ResMut<CombatEffects>) {
    if effects.generation != Some(sim.generation()) {
        effects.tracers.clear();
        effects.deaths.clear();
        effects.generation = Some(sim.generation());
    }
    for message in messages.read() {
        match message.event {
            SimEvent::Attacked { attacker, from, to, .. } => {
                let owner = sim.object(attacker).and_then(|o| o.owner);
                effects.tracers.push(Tracer {
                    tick: message.tick,
                    owner,
                    from,
                    to,
                });
            }
            SimEvent::Died { kind, owner, pos, .. } => effects.deaths.push(DeathMarker {
                tick: message.tick,
                kind,
                owner,
                pos,
            }),
            _ => {}
        }
    }
    effects.tracers.retain(|t| shows(&sim, t.tick, TRACER_TICKS));
    effects.deaths.retain(|d| shows(&sim, d.tick, DEATH_MARKER_TICKS));
}

/// Rebuilds the effects mesh: death markers at the back, then tracers, then HP bars (alpha blending draws a mesh's
/// triangles in order). Hidden while nothing shows.
fn draw_effects(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    effects: Res<CombatEffects>,
    states: Res<ControlStates>,
    player: Res<LocalPlayer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<CombatEffectsMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let z = DrawLayer::Overlays.base_z() + EFFECTS_Z;
    let mut mesh = ColorMesh::triangles();
    for death in &effects.deaths {
        let fade = 1.0 - age(&sim, alpha.0, death.tick) / DEATH_MARKER_TICKS as f32;
        death_marker(&mut mesh, sim.map(), death, fade, z);
    }
    for tracer in &effects.tracers {
        let fade = 1.0 - age(&sim, alpha.0, tracer.tick) / TRACER_TICKS as f32;
        tracer_line(&mut mesh, sim.map(), tracer, fade, z);
    }
    let selection = states.selection(player.0);
    for bar in hp_bars(&sim, alpha.0, |id| selection.contains(id)) {
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

/// A bright core inside a glow in the shooter's colour, both fading out.
fn tracer_line(mesh: &mut ColorMesh, map: &Map, tracer: &Tracer, fade: f32, z: f32) {
    let fade = fade.clamp(0.0, 1.0);
    let (from, to) = (muzzle(map, tracer.from), muzzle(map, tracer.to));
    let glow = lighten(player_color(tracer.owner).to_srgba(), 0.3).with_alpha(0.45 * fade);
    mesh.segment(from, to, 5.0, z, glow.into());
    mesh.segment(from, to, 2.0, z, Color::srgba(1.0, 0.96, 0.7, fade));
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

/// A dark frame around the bar's fill: green above 60%, yellow above 30%, red below.
fn hp_bar_shape(mesh: &mut ColorMesh, bar: HpBar, z: f32) {
    let half = Vec2::new(bar.width / 2.0, BAR_HEIGHT / 2.0);
    let center = view_to_world(bar.center);
    mesh.rect(
        center - half - 1.0,
        center + half + 1.0,
        z,
        Color::srgb(0.05, 0.05, 0.05),
    );
    let fill = match bar.fraction {
        f if f > 0.6 => Color::srgb(0.30, 0.85, 0.30),
        f if f > 0.3 => Color::srgb(0.95, 0.80, 0.20),
        _ => Color::srgb(0.95, 0.25, 0.20),
    };
    let left = center - half;
    mesh.rect(left, left + Vec2::new(bar.width * bar.fraction, BAR_HEIGHT), z, fill);
}
