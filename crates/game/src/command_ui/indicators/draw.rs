//! The indicators' mesh: one alpha-blended mesh in the overlays layer, rebuilt every frame while anything shows,
//! with a cast's circle (slice M5-N5), the routes and the rally lines at the back, then the markers, then the flashes
//! (placeholders until the art, Q0001).

use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::{Map, Object, Sim, Tick};

use super::{
    CommandIndicators, FLASH_TICKS, Flash, INDICATORS_Z, Indicator, IndicatorMesh, MarkerRing, OrderIssued, cast,
    drop_off_indicators, gathering, indicator, order_color, own_selected, rally, rally_indicators,
    selection_indicators, targets, waypoint,
};
use crate::ClientMode;
use crate::coords::{DrawLayer, drawn_at_height, drawn_position, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::input::{ActionState, InputAction};
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;
use crate::sim_driver::{RenderAlpha, SimRes};

/// A Location marker: its ring's radius and the dot's, su (placeholders).
const LOCATION_RADIUS: f32 = 10.0;
const DOT_RADIUS: f32 = 3.0;
/// An Object marker lies this far outside the target's box, su.
pub(super) const OBJECT_GAP: f32 = 6.0;
/// Line widths, su.
const MARKER_WIDTH: f32 = 2.5;
const FLASH_WIDTH: f32 = 3.0;
const ROUTE_WIDTH: f32 = 2.0;
/// A flash closes in from this far outside its marker, su.
const FLASH_REACH: f32 = 18.0;
/// The routes' dashes: each leg is cut into pieces this long (su), which follow the terrain's height; every other
/// one is drawn.
const DASH: f32 = 12.0;
/// The routes' opacity.
const ROUTE_ALPHA: f32 = 0.8;
/// Segments of a ring.
const RING_SEGMENTS: usize = 32;

pub(super) fn spawn_indicator_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // A mesh without vertices makes the renderer warn: a zero-area stand-in until the first indicator.
    let mut placeholder = ColorMesh::triangles();
    placeholder.rect(Vec2::ZERO, Vec2::ZERO, 0.0, Color::NONE);
    commands.spawn((
        Name::new("Command indicators"),
        IndicatorMesh,
        Mesh2d(meshes.add(placeholder.into_mesh())),
        // White and alpha-blended (`ColorMaterial::default`): the vertex colours fade the flashes.
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}

/// Takes the orders just given and drops the flashes that have faded. A replaced sim starts from none.
pub(super) fn collect_flashes(
    sim: Res<SimRes>,
    mut issued: MessageReader<OrderIssued>,
    mut indicators: ResMut<CommandIndicators>,
) {
    if indicators.generation != Some(sim.generation()) {
        indicators.flashes.clear();
        indicators.generation = Some(sim.generation());
    }
    for order in issued.read() {
        if order_color(order.kind).is_some() {
            indicators.flashes.push(Flash {
                tick: order.tick,
                kind: order.kind,
            });
        }
    }
    let tick = sim.tick();
    indicators
        .flashes
        .retain(|flash| tick.saturating_sub(flash.tick) < FLASH_TICKS);
}

/// How far a flash given in `tick` has run, 0 to 1, as drawn now: the sim's tick shows at alpha 1, and an order
/// applies in the tick after the one it was given in.
fn flash_progress(sim: &Sim, alpha: f32, tick: Tick) -> f32 {
    ((sim.tick().saturating_sub(tick) as f32 + alpha - 1.0) / FLASH_TICKS as f32).clamp(0.0, 1.0)
}

/// Rebuilds the indicators' mesh: a cast's circle, the routes and the rally lines at the back, then the markers (the
/// orders', the selected workers' drop-offs' and the rally points'), then the flashes. Hidden while nothing shows, and
/// outside the game modes.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_indicators(
    mode: Res<ClientMode>,
    local: LocalSight,
    alpha: Res<RenderAlpha>,
    actions: Res<ActionState>,
    states: Res<ControlStates>,
    indicators: Res<CommandIndicators>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<IndicatorMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let mut mesh = ColorMesh::triangles();
    if mode.is_game() {
        let (sim, sight) = (local.sim(), local.get());
        let z = DrawLayer::Overlays.base_z() + INDICATORS_Z;
        // (M5-N5) While a Shrine's cast awaits its point, its circle around the point under the pointer, over the map.
        let pointer = actions.pointer();
        if let Some(ability) = cast::awaited(states.of(sight.player()).interface)
            && let Some(ground) = pointer.ground.filter(|_| !pointer.over_ui)
        {
            let seen = sim.sees_point(sight.player(), ground);
            cast::draw(&mut mesh, sim.map(), ability, ground, seen, z);
        }
        let queued = actions.held(InputAction::Queue);
        if queued {
            for o in own_selected(sim, &states, sight.player()) {
                route(&mut mesh, sim, &sight, o, alpha.0, z);
            }
        }
        let rallies = rally_indicators(sim, &sight, &states);
        for rally in &rallies {
            rally::draw_line(&mut mesh, sim, &sight, rally, alpha.0, z);
        }
        let at = |indicator| marker_ring(sim, &sight, indicator, alpha.0);
        let markers = selection_indicators(sim, &sight, &states, queued);
        let drop_offs = drop_off_indicators(sim, &sight, &states);
        let rally_markers = rallies.iter().map(|rally| rally.marker);
        for indicator in markers.into_iter().chain(drop_offs).chain(rally_markers) {
            if let Some(at) = at(indicator) {
                marker(&mut mesh, indicator, at, 0.0, 1.0, z);
            }
        }
        for flash in &indicators.flashes {
            let progress = flash_progress(sim, alpha.0, flash.tick);
            if let Some(indicator) = indicator(sim, &sight, flash.kind)
                && let Some(at) = at(indicator)
            {
                let fade = 1.0 - progress;
                marker(&mut mesh, indicator, at, FLASH_REACH * fade, fade, z);
            }
        }
    }
    if mesh.vertex_count() == 0 {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    if let Err(error) = meshes.insert(&mesh2d.0, mesh.into_mesh()) {
        error!("command indicators rebuild failed: {error}");
    }
    visibility.set_if_neq(Visibility::Inherited);
}

/// A unit's route: dashed legs from where it is drawn through the waypoints of its current and queued orders.
fn route(mesh: &mut ColorMesh, sim: &Sim, sight: &Sight, o: &Object, alpha: f32, z: f32) {
    let Some(orders) = &o.orders else {
        return;
    };
    let mut from = o.prev_pos.lerp(o.pos, alpha);
    let current = orders.current.iter().map(|current| &current.order);
    for order in current.chain(&orders.queue) {
        let (Some(to), Some(color)) = (waypoint(sim, sight, order), order_color(order.kind)) else {
            continue;
        };
        dashed_on_terrain(mesh, sim.map(), from, to, z, color.with_alpha(ROUTE_ALPHA));
        from = to;
    }
}

/// The ground segment from `a` to `b`, dashed, as drawn on the terrain.
fn dashed_on_terrain(mesh: &mut ColorMesh, map: &Map, a: Vec2, b: Vec2, z: f32, color: Color) {
    let pieces = ((b - a).length() / DASH).ceil().clamp(1.0, 4096.0) as u32;
    let at = |i: u32| view_to_world(drawn_position(map, a.lerp(b, i as f32 / pieces as f32)));
    for i in (0..pieces).step_by(2) {
        mesh.segment(at(i), at(i + 1), ROUTE_WIDTH, z, color);
    }
}

/// Draws `indicator`'s marker `at` its place ([`marker_ring`]): its ring `grow` su wider than at rest, at `opacity`.
/// A Location also gets its dot while at rest or nearly; a gathering marker has a look of its own ([`gathering`]).
fn marker(mesh: &mut ColorMesh, indicator: Indicator, at: MarkerRing, grow: f32, opacity: f32, z: f32) {
    let width = if grow > 0.0 { FLASH_WIDTH } else { MARKER_WIDTH };
    let center = view_to_world(drawn_at_height(at.center, at.height));
    let color = indicator.color().with_alpha(opacity);
    if let Indicator::Gathering { marker, .. } = indicator {
        gathering::draw(mesh, marker, center.extend(z), at, grow, width, color);
        return;
    }
    ring(mesh, center, at.radius + grow, width, z, color);
    if matches!(indicator, Indicator::Location { .. }) && grow < FLASH_REACH / 2.0 {
        disc(mesh, center, DOT_RADIUS, z, color);
    }
}

/// Where `indicator`'s marker is drawn at rest, at the render `alpha`, as the player whose `sight` this is knows its
/// target: a Location at its point on the ground; an Object or a gathering marker around the object's box, where the
/// object is drawn while the player sees it, around its record's footprint while the player remembers it, wherever
/// the object really is ([`targets`], design change 0139); none while the player knows of nothing there.
pub fn marker_ring(sim: &Sim, sight: &Sight, indicator: Indicator, alpha: f32) -> Option<MarkerRing> {
    match indicator {
        Indicator::Location { at, .. } => Some(MarkerRing {
            center: at,
            height: sim.map().height_at(at),
            radius: LOCATION_RADIUS,
            half: Vec2::ZERO,
        }),
        Indicator::Object { id, .. } | Indicator::Gathering { id, .. } => {
            let (center, height, half) = targets::object_box(sim, sight, id, alpha)?;
            Some(MarkerRing {
                center,
                height,
                radius: half.length() + OBJECT_GAP,
                half,
            })
        }
    }
}

/// A ring around `center` (Bevy world space).
pub(super) fn ring(mesh: &mut ColorMesh, center: Vec2, radius: f32, width: f32, z: f32, color: Color) {
    let (inner, outer) = ((radius - width / 2.0).max(0.0), radius + width / 2.0);
    let dir = |i: usize| {
        let a = i as f32 * TAU / RING_SEGMENTS as f32;
        Vec2::new(a.cos(), a.sin())
    };
    for i in 0..RING_SEGMENTS {
        let (d0, d1) = (dir(i), dir(i + 1));
        let corners = [
            center + d0 * inner,
            center + d0 * outer,
            center + d1 * outer,
            center + d1 * inner,
        ];
        mesh.quad(corners.map(|p| p.extend(z)), color);
    }
}

/// A filled disc around `center` (Bevy world space).
fn disc(mesh: &mut ColorMesh, center: Vec2, radius: f32, z: f32, color: Color) {
    let points: Vec<Vec2> = (0..12)
        .map(|i| {
            let a = i as f32 * TAU / 12.0;
            center + Vec2::new(a.cos(), a.sin()) * radius
        })
        .collect();
    mesh.polygon(&points, z, color);
}
