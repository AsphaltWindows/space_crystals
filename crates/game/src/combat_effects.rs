//! Placeholder combat feedback (`systems/combat.md`; the art waits on Q0001): a short tracer from the attacker to its
//! target for each attack that takes effect (`SimEvent::Attacked`), a Syndicate laser's red ([`ShotStyle`], design
//! change 0207), HP bars over damaged and selected objects, with a
//! shield bar above the HP bar of an object that has Shields (`systems/combat.md > Shields > Display`, slice M3-B,
//! design change 0148: "a shield bar sits directly above the HP bar"; its styling waits for the HUD pass), and a
//! fading marker where an object died (`SimEvent::Died`).
//!
//! Everything is one vertex-coloured, alpha-blended mesh in the overlays layer, rebuilt every frame while anything
//! shows. Effects age by the sim's ticks plus the render alpha, so a QA screenshot of a tick (alpha 1) always shows
//! them alike. HP bars follow the units' interpolated positions.
//!
//! The fog of war hides what the local player doesn't see (`crate::fog`):
//! - **A shot** shows only along the parts of its path over tiles Visible to the player ([`visible_spans`];
//!   `core/vision.md > Attacks from outside vision`, design change 0177: "a shot fired from the fog at your unit is
//!   seen coming out of the fog edge, which shows its direction"). The attacker itself is the sim's to reveal
//!   (`sc_sim::vision::reveal`): revealed, it shows like any object the player sees, and its own shots at the
//!   player's objects are drawn whole, from it to their target (design change 0206, answering Q0056: "while an
//!   attacker in the fog is revealed to a player, that attacker's own shots are drawn from the attacker to the
//!   target, with no gap at the fog edge"; design change 0244, answering Q0075, narrows it to the shots at the
//!   player's own objects: "a fully drawn shot reveals nothing only when **you** are the one attacked";
//!   [`shot_spans`]). "Other shots from the fog are still clipped to Visible tiles", a revealed attacker's at
//!   anything else too.
//! - **A death marker** only on a Visible tile or for one of the player's own objects, and **an HP bar** only over an
//!   object the player sees (`Sight::sees`): placeholder rules, the design is silent.

use bevy::prelude::*;
use sc_sim::{Map, Object, ObjectId, ObjectKind, PlayerId, SU_PER_GU, Sim, SimEvent, Tick, VisibilityState};

use crate::ClientSystems;
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::object_render::{object_height, player_color};
use crate::selection::ControlStates;
use crate::shapes::{ColorMesh, darken, lighten};
use crate::sim_driver::{RenderAlpha, SimEventMessage, SimRes};

/// How long a tracer shows, ticks. Placeholder until the art direction (Q0001).
pub const TRACER_TICKS: u32 = 5;
/// How long a death marker shows, ticks. Placeholder until the art direction (Q0001).
pub const DEATH_MARKER_TICKS: u32 = 40;
/// A shot leaves a unit this far up the screen from its ground point, su, as from its body rather than its feet.
const MUZZLE_LIFT: f32 = 8.0;
/// HP bars: height, gap above the corners of the unit's silhouette (clear of a selection ring, which lies 7 su
/// outside them: `selection::markers`) or above a tile-based object's footprint, and the narrowest bar, su. A shield
/// bar is as high, right above the HP bar, their frames shared.
const BAR_HEIGHT: f32 = 4.0;
const BAR_GAP: f32 = 9.0;
const BAR_MIN_WIDTH: f32 = 24.0;
/// A shield bar's fill (placeholder colour, until the art direction: Q0001).
const SHIELD_FILL: Color = Color::srgb(0.35, 0.72, 1.0);
/// Above the path overlay, in the overlays layer.
const EFFECTS_Z: f32 = 1.0;

/// An attack that took effect in `tick` (`SimEvent::Attacked`), from and to the ground points of both units then.
#[derive(Clone, Debug, PartialEq)]
pub struct Tracer {
    pub tick: Tick,
    /// The attacker: while it is revealed to the player, its shot is drawn whole ([`shot_spans`]).
    pub attacker: ObjectId,
    /// The form the attacker fired in (`sc_sim::conversion::form`; its type if it died by the time the client saw the
    /// shot), whose weapon the shot is drawn as ([`ShotStyle::of`]).
    pub weapon: Option<ObjectKind>,
    pub owner: Option<PlayerId>,
    /// The object hit.
    pub target: ObjectId,
    /// Its owner as the shot took effect (design change 0244: a revealed attacker's shot is drawn whole only at the
    /// player's own objects, [`shot_spans`]); `None` for a neutral object, or one the client no longer knows.
    pub target_owner: Option<PlayerId>,
    pub from: Vec2,
    pub to: Vec2,
}

/// How a shot is drawn, by the weapon that fired it: each shooting kind's style is in its module
/// (`crate::kinds::ClientKind::shot`), which says which weapons are lasers of which colour.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShotStyle {
    /// A pale tracer inside a glow in the shooter's colour: every weapon but a Syndicate laser (placeholder, Q0001).
    Tracer,
    /// A red laser: design change 0207 (`factions/syndicate/units/README.md`): "every Syndicate laser-shooting unit
    /// and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser", whoever owns it.
    RedLaser,
}

impl ShotStyle {
    /// The style of a shot fired in the form `weapon`, its kind's: the Guard's laser rifle is a red laser (the Laser
    /// Battery and the Laser Turret will be when they exist); every other weapon, the Agent's pistol included, a
    /// tracer, as is a shot whose shooter the client no longer knows.
    pub fn of(weapon: Option<ObjectKind>) -> Self {
        weapon
            .and_then(|kind| crate::kinds::of(kind).shot)
            .unwrap_or(ShotStyle::Tracer)
    }
}

/// A red laser's glow and core (placeholders: 0207 says only "red").
const LASER_GLOW: Srgba = Srgba::rgb(1.0, 0.12, 0.10);
const LASER_CORE: Srgba = Srgba::rgb(1.0, 0.62, 0.58);

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

/// An HP bar: centred on `center` (view space), `width` su wide, `fraction` of it full; with a shield bar above it,
/// `shields` of it full, for an object that has Shields.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HpBar {
    pub center: Vec2,
    pub width: f32,
    pub fraction: f32,
    /// The share of its MaxShields its Shields fill (`Object::shields`), if it has Shields.
    pub shields: Option<f32>,
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

/// The HP bars to draw: one over each object that is damaged (its HP or its Shields short of their maximum) or
/// `selected`, at its interpolated position, among those `sight` sees. Out of its MaxHP and MaxShields now: a
/// transforming object's interpolated ones (`Object::max_hp`, `Object::max_shields`).
pub fn hp_bars(sim: &Sim, sight: &Sight, alpha: f32, selected: impl Fn(ObjectId) -> bool) -> Vec<HpBar> {
    sim.objects()
        .filter(|o| sight.sees(o))
        .filter_map(|o| {
            let (hp, max_hp) = (o.hp?, o.max_hp()?);
            let shields = o.shields.zip(o.max_shields()).filter(|&(_, max)| max > 0.0);
            let damaged = hp < max_hp || shields.is_some_and(|(shields, max)| shields < max);
            let shields = shields.map(|(shields, max)| shields / max);
            (damaged || selected(o.id)).then(|| hp_bar(sim.map(), o, alpha, hp / max_hp, shields))
        })
        .collect()
}

/// The parts of the straight path from `from` to `to` (ground points, su) that lie over tiles `sight` shows Visible,
/// as fractions of the path, 0 at `from` and 1 at `to`, in order (`core/vision.md > Attacks from outside vision`,
/// design change 0177: "a shot, tracer or projectile is visible to a player only along the part of its path that
/// crosses tiles Visible to that player"). The path is cut where it crosses a tile's edge, and each piece shows with
/// its tile: one span `(0, 1)` over Visible tiles only, none over no Visible tile. A shot's whole path, its tiles
/// looked up once each: no cost beyond the few tiles a shot crosses.
pub fn visible_spans(sight: &Sight, from: Vec2, to: Vec2) -> Vec<(f32, f32)> {
    let mut cuts = vec![0.0, 1.0];
    for (a, b) in [(from.x, to.x), (from.y, to.y)] {
        if a == b {
            continue;
        }
        // Each tile edge strictly between the two, along this axis.
        let mut edge = ((a.min(b) / SU_PER_GU).floor() + 1.0) * SU_PER_GU;
        while edge < a.max(b) {
            cuts.push((edge - a) / (b - a));
            edge += SU_PER_GU;
        }
    }
    cuts.sort_by(f32::total_cmp);
    let mut spans: Vec<(f32, f32)> = Vec::new();
    for piece in cuts.windows(2) {
        let (start, end) = (piece[0], piece[1]);
        if end <= start || sight.at(from.lerp(to, (start + end) / 2.0)) != VisibilityState::Visible {
            continue;
        }
        match spans.last_mut() {
            Some(last) if last.1 == start => last.1 = end,
            _ => spans.push((start, end)),
        }
    }
    spans
}

/// How `sight` shows the shot of `tracer` now, for [`TRACER_TICKS`], none once it has faded: whole if its target is
/// one of the player's objects and its attacker is revealed to the player ([`Sight::reveals`]; design change 0206:
/// "from the attacker to the target, with no gap at the fog edge"; "No ground around the attacker is revealed"),
/// else the [`visible_spans`] of its path (design change 0177). Design change 0244, answering Q0075: "draw the whole
/// path only if the attacker is revealed to the viewer **and** the shot's target is owned by the viewer", as then
/// both ends show already; drawn whole, a shot at a neutral's or a third player's object would show where it stands.
///
/// A literal reading, the viewer being [`Sight::player`]: a defeated player watching an ally's vision (design change
/// 0102) sees the shots at the ally's objects clipped too. Moot until the sim has teams: a defeated player has no
/// allies yet.
pub fn shot_spans(sim: &Sim, sight: &Sight, tracer: &Tracer) -> Vec<(f32, f32)> {
    if !shows(sim, tracer.tick, TRACER_TICKS) {
        return Vec::new();
    }
    if sight.reveals(tracer.attacker) && tracer.target_owner == Some(sight.player()) {
        return vec![(0.0, 1.0)];
    }
    visible_spans(sight, tracer.from, tracer.to)
}

/// Whether `sight` shows `death`: one of the player's own objects, or on a Visible tile.
pub fn shows_death(sight: &Sight, death: &DeathMarker) -> bool {
    death.owner == Some(sight.player()) || sight.at(death.pos) == VisibilityState::Visible
}

/// The bars of `o`: over a unit, clear of its silhouette's corners however it turns, as wide as the silhouette; over a
/// tile-based object, along the top edge of its footprint, as wide as the footprint (slice M3-K2: before, a
/// structure's bars were a unit's, inside it).
fn hp_bar(map: &Map, o: &Object, alpha: f32, fraction: f32, shields: Option<f32>) -> HpBar {
    let pos = o.prev_pos.lerp(o.pos, alpha);
    let (half, above) = match (o.def().half_extents(), o.placement.as_ref()) {
        (Some(half), _) => (half, half.length()),
        (None, Some(p)) => {
            let half = Vec2::new(p.w as f32, p.h as f32) * (SU_PER_GU / 2.0);
            (half, half.y)
        }
        (None, None) => {
            let half = Vec2::splat(BAR_MIN_WIDTH / 2.0);
            (half, half.length())
        }
    };
    let drawn = drawn_at_height(pos, object_height(map, pos, o.placement.as_ref()));
    HpBar {
        center: drawn - Vec2::new(0.0, above + BAR_GAP + BAR_HEIGHT / 2.0),
        width: (half.x * 2.0).max(BAR_MIN_WIDTH),
        fraction: fraction.clamp(0.0, 1.0),
        shields: shields.map(|shields| shields.clamp(0.0, 1.0)),
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
    let messages: Vec<&SimEventMessage> = messages.read().collect();
    // An attacker gone by now died in one of these ticks: its type and owner are its death's. So did a target gone by
    // now, which a shot of these ticks killed: its owner is its death's too (design change 0244,
    // `Tracer::target_owner`).
    let fallen = |id: ObjectId| {
        messages.iter().find_map(|m| match m.event {
            SimEvent::Died {
                id: dead, kind, owner, ..
            } if dead == id => Some((kind, owner)),
            _ => None,
        })
    };
    for message in &messages {
        match message.event {
            SimEvent::Attacked {
                attacker,
                target,
                from,
                to,
            } => {
                let shooter = match sim.object(attacker) {
                    Some(o) => Some((sc_sim::conversion::form(o), o.owner)),
                    None => fallen(attacker),
                };
                effects.tracers.push(Tracer {
                    tick: message.tick,
                    attacker,
                    weapon: shooter.map(|(form, _)| form),
                    owner: shooter.and_then(|(_, owner)| owner),
                    target,
                    target_owner: match sim.object(target) {
                        Some(o) => o.owner,
                        None => fallen(target).and_then(|(_, owner)| owner),
                    },
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
/// triangles in order), each only where the local player sees it. Hidden while nothing shows.
fn draw_effects(
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
    for tracer in &effects.tracers {
        let fade = 1.0 - age(&sim, alpha.0, tracer.tick) / TRACER_TICKS as f32;
        for span in shot_spans(&sim, &sight, tracer) {
            tracer_line(&mut mesh, sim.map(), tracer, span, fade, z);
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
        ShotStyle::Tracer => (
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
