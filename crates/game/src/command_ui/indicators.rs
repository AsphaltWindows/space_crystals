//! Command indicators (`systems/controls/selection_and_hud.md > CommandIndicators`), placeholders until the art
//! (Q0001):
//!
//! - **Markers** at the targets of the current orders of the local player's selected units (`> Indicator
//!   assignments`): a ring with a dot on the ground for a Location indicator (Move, AttackMove, Patrol; a Move to an
//!   object marks where the object is), a ring around the target for an Object indicator (Attack). They go when the
//!   unit is deselected or the order completes. Patrol's origin isn't marked: the sim keeps no origin yet.
//! - **Flashes** where an order was just given ([`OrderIssued`]): a ring closing in on the target while it fades,
//!   for [`FLASH_TICKS`]. They show for every order given, even one that completes at once.
//! - **Waypoints**: while the Queue modifier (Shift) is held, each selected unit's route from where it is drawn
//!   through the targets of its current and queued orders, as dashed lines, each leg in its order's colour, with
//!   the queued orders' markers.
//! - **Rally points** ([`rally`], design change 0087): each selected producer's rally point, a marker with a line
//!   from the building to it; none while it has no rally point.
//!
//! Colours follow `> Color language`: green for Move, red for Attack, orange for AttackMove and Patrol. A target
//! object marks nothing, and ends no route leg, while the fog of war hides it from the local player
//! (`crate::fog::Sight::sees`): the indicators would give away where it is. A Move to a structure or resource on
//! Explored tiles marks where the player remembers it (`crate::fog::Sight::remembered`). Everything is one
//! alpha-blended mesh in the overlays layer, between the path overlay and the combat effects, rebuilt every frame
//! while anything shows. Flashes age by the sim's ticks and the render alpha, so a QA screenshot of a tick (alpha 1)
//! always shows them alike.

pub mod rally;

use std::collections::BTreeSet;
use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::orders::Order;
use sc_sim::vision::LastKnown;
use sc_sim::{Map, Object, ObjectId, OrderKind, SU_PER_GU, Sim, Target, Tick};

use super::OrderIssued;
use crate::coords::{DrawLayer, drawn_at_height, drawn_position, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::input::{ActionState, InputAction};
use crate::object_render::object_height;
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;
use crate::sim_driver::{RenderAlpha, SimRes};
use crate::{ClientMode, ClientSystems};

pub use rally::{RallyIndicator, rally_indicators};

/// How long the flash of a given order lasts, ticks. Placeholder: the design gives none.
pub const FLASH_TICKS: u32 = 8;
/// `> Color language` (placeholder shades): peaceful movement, a hostile target, aggressive movement.
pub const MOVE_COLOR: Color = Color::srgb(0.30, 0.95, 0.40);
pub const ATTACK_COLOR: Color = Color::srgb(1.00, 0.25, 0.20);
pub const AGGRESSIVE_COLOR: Color = Color::srgb(1.00, 0.58, 0.12);
/// A Location marker: its ring's radius and the dot's, su (placeholders).
const LOCATION_RADIUS: f32 = 10.0;
const DOT_RADIUS: f32 = 3.0;
/// An Object marker lies this far outside the target's box, su.
const OBJECT_GAP: f32 = 6.0;
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
/// Above the path overlay (at the layer's base) and below the combat effects (+1).
const INDICATORS_Z: f32 = 0.5;
/// Segments of a ring.
const RING_SEGMENTS: usize = 32;

/// The flash of an order given in `tick`.
#[derive(Clone, Debug, PartialEq)]
pub struct Flash {
    pub tick: Tick,
    pub kind: OrderKind,
}

/// The flashes on screen, dropped once they have faded.
#[derive(Resource, Default, Debug)]
pub struct CommandIndicators {
    pub flashes: Vec<Flash>,
    /// The `SimRes::generation` they belong to.
    generation: Option<u32>,
}

/// An indicator, placed for this frame (`> CommandIndicators`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Indicator {
    /// `> Location Indicator`: a marker on the ground at a point (ground space).
    Location { at: Vec2, color: Color },
    /// `> Object Indicator`: a marker around an object's perimeter.
    Object { id: ObjectId, color: Color },
}

/// The entity holding the indicators' mesh.
#[derive(Component)]
pub struct IndicatorMesh;

pub struct IndicatorsPlugin;

impl Plugin for IndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandIndicators>()
            .add_systems(Startup, spawn_indicator_mesh)
            .add_systems(
                Update,
                (collect_flashes, draw_indicators)
                    .chain()
                    .in_set(ClientSystems::Present),
            );
    }
}

/// The colour of an order's indicator (`> Indicator assignments`); none for orders without a target.
pub fn order_color(kind: OrderKind) -> Option<Color> {
    match kind {
        OrderKind::Move(_) => Some(MOVE_COLOR),
        OrderKind::Attack(_) => Some(ATTACK_COLOR),
        OrderKind::AttackMove(_) | OrderKind::Patrol(_) => Some(AGGRESSIVE_COLOR),
        OrderKind::Hold | OrderKind::Stop => None,
    }
}

/// The indicator of an order (`> Indicator assignments`): a Location for Move, AttackMove and Patrol (a Move to an
/// object at the object, wherever it is now, or where the player remembers it), an Object for Attack; none for
/// orders without a target, or whose target is gone or hidden from `sight`.
pub fn indicator(sim: &Sim, sight: &Sight, kind: OrderKind) -> Option<Indicator> {
    let color = order_color(kind)?;
    match kind {
        OrderKind::Move(Target::Point(at)) | OrderKind::AttackMove(at) | OrderKind::Patrol(at) => {
            Some(Indicator::Location { at, color })
        }
        OrderKind::Move(Target::Object(id)) => {
            let at = shown_at(sim, sight, id)?;
            Some(Indicator::Location { at, color })
        }
        OrderKind::Attack(id) => seen(sim, sight, id).map(|_| Indicator::Object { id, color }),
        OrderKind::Hold | OrderKind::Stop => None,
    }
}

/// Object `id`, if it exists and `sight` sees it.
fn seen<'a>(sim: &'a Sim, sight: &Sight, id: ObjectId) -> Option<&'a Object> {
    sim.object(id).filter(|o| sight.sees(o))
}

/// Where `sight` shows object `id` (ground space): where it is if the player sees it, else where the record shown
/// in its place has it.
fn shown_at(sim: &Sim, sight: &Sight, id: ObjectId) -> Option<Vec2> {
    match seen(sim, sight, id) {
        Some(o) => Some(o.pos),
        None => sight.remembered_record(id).map(LastKnown::pos),
    }
}

/// The indicators of the current orders of the selected units of the player whose `sight` this is, and of their
/// queued orders too while `queued`, each target once.
pub fn selection_indicators(sim: &Sim, sight: &Sight, states: &ControlStates, queued: bool) -> Vec<Indicator> {
    let mut seen = BTreeSet::new();
    let mut indicators = Vec::new();
    for o in own_selected(sim, states, sight.player()) {
        let Some(orders) = &o.orders else {
            continue;
        };
        let queue = orders.queue.iter().filter(|_| queued);
        for order in orders.current.iter().map(|current| &current.order).chain(queue) {
            let Some(indicator) = indicator(sim, sight, order.kind) else {
                continue;
            };
            let key = match indicator {
                Indicator::Location { at, .. } => (0, at.x.to_bits(), at.y.to_bits(), 0),
                Indicator::Object { id, .. } => (1, 0, 0, id.0),
            };
            if seen.insert((key, order_color(order.kind).map(|c| c.to_srgba().to_u8_array()))) {
                indicators.push(indicator);
            }
        }
    }
    indicators
}

/// The local player's selected units, in id order.
fn own_selected<'a>(
    sim: &'a Sim,
    states: &'a ControlStates,
    player: sc_sim::PlayerId,
) -> impl Iterator<Item = &'a Object> + 'a {
    states
        .selection(player)
        .ids()
        .filter_map(|id| sim.object(id))
        .filter(move |o| o.owner == Some(player))
}

/// Where the leg of a route that carries out `order` ends, ground space: the target object where it is shown, else
/// the unit's own destination (its place in the group's shape, `Order::destination`); none for Hold and Stop, and
/// for a target hidden from `sight`.
fn waypoint(sim: &Sim, sight: &Sight, order: &Order) -> Option<Vec2> {
    match order.kind {
        OrderKind::Move(Target::Object(id)) => shown_at(sim, sight, id),
        OrderKind::Attack(id) => seen(sim, sight, id).map(|o| o.pos),
        _ => order.destination(),
    }
}

fn spawn_indicator_mesh(
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
fn collect_flashes(
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

/// Rebuilds the indicators' mesh: the routes and the rally lines at the back, then the markers (the orders' and the
/// rally points'), then the flashes. Hidden while nothing shows, and outside the game modes.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn draw_indicators(
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
        let queued = actions.held(InputAction::Queue);
        if queued {
            for o in own_selected(sim, &states, sight.player()) {
                route(&mut mesh, sim, &sight, o, alpha.0, z);
            }
        }
        let rallies = rally_indicators(sim, &sight, &states);
        for rally in &rallies {
            rally::draw_line(&mut mesh, sim, rally, alpha.0, z);
        }
        let markers = selection_indicators(sim, &sight, &states, queued);
        for indicator in markers.into_iter().chain(rallies.iter().map(|rally| rally.marker)) {
            marker(&mut mesh, sim, indicator, alpha.0, 0.0, 1.0, z);
        }
        for flash in &indicators.flashes {
            let progress = flash_progress(sim, alpha.0, flash.tick);
            if let Some(indicator) = indicator(sim, &sight, flash.kind) {
                let fade = 1.0 - progress;
                marker(&mut mesh, sim, indicator, alpha.0, FLASH_REACH * fade, fade, z);
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

/// Draws an indicator: its ring `grow` su wider than at rest, at `opacity`. A Location also gets its dot while at
/// rest or nearly.
fn marker(mesh: &mut ColorMesh, sim: &Sim, indicator: Indicator, alpha: f32, grow: f32, opacity: f32, z: f32) {
    let width = if grow > 0.0 { FLASH_WIDTH } else { MARKER_WIDTH };
    match indicator {
        Indicator::Location { at, color } => {
            let center = view_to_world(drawn_position(sim.map(), at));
            let color = color.with_alpha(opacity);
            ring(mesh, center, LOCATION_RADIUS + grow, width, z, color);
            if grow < FLASH_REACH / 2.0 {
                disc(mesh, center, DOT_RADIUS, z, color);
            }
        }
        Indicator::Object { id, color } => {
            let Some(o) = sim.object(id) else {
                return;
            };
            let pos = o.prev_pos.lerp(o.pos, alpha);
            let height = object_height(sim.map(), pos, o.placement.as_ref());
            let (center, half) = object_box(o, alpha);
            let center = view_to_world(drawn_at_height(center, height));
            ring(
                mesh,
                center,
                half.length() + OBJECT_GAP + grow,
                width,
                z,
                color.with_alpha(opacity),
            );
        }
    }
}

/// The box an Object marker surrounds: its centre (ground space, where the object is drawn at the render `alpha`)
/// and its half extents, su. A structure's footprint, else the type's box.
fn object_box(o: &Object, alpha: f32) -> (Vec2, Vec2) {
    match &o.placement {
        Some(p) => (p.center(), Vec2::new(p.w as f32, p.h as f32) * SU_PER_GU / 2.0),
        None => (
            o.prev_pos.lerp(o.pos, alpha),
            o.def().half_extents().unwrap_or(Vec2::splat(SU_PER_GU / 4.0)),
        ),
    }
}

/// Where `indicator`'s marker is, ground space, and its ring's radius at rest, su, as [`marker`] draws it at the
/// render `alpha`; none for an Object marker whose object is gone.
fn marker_ring(sim: &Sim, indicator: Indicator, alpha: f32) -> Option<(Vec2, f32)> {
    match indicator {
        Indicator::Location { at, .. } => Some((at, LOCATION_RADIUS)),
        Indicator::Object { id, .. } => {
            let (center, half) = object_box(sim.object(id)?, alpha);
            Some((center, half.length() + OBJECT_GAP))
        }
    }
}

/// A ring around `center` (Bevy world space).
fn ring(mesh: &mut ColorMesh, center: Vec2, radius: f32, width: f32, z: f32, color: Color) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_colours_follow_the_colour_language() {
        let p = Vec2::new(10.0, 20.0);
        assert_eq!(order_color(OrderKind::Move(Target::Point(p))), Some(MOVE_COLOR));
        assert_eq!(
            order_color(OrderKind::Move(Target::Object(ObjectId(3)))),
            Some(MOVE_COLOR)
        );
        assert_eq!(order_color(OrderKind::Attack(ObjectId(3))), Some(ATTACK_COLOR));
        assert_eq!(order_color(OrderKind::AttackMove(p)), Some(AGGRESSIVE_COLOR));
        assert_eq!(order_color(OrderKind::Patrol(p)), Some(AGGRESSIVE_COLOR));
        assert_eq!(order_color(OrderKind::Hold), None);
        assert_eq!(order_color(OrderKind::Stop), None);
    }
}
