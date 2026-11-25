//! Command indicators (`systems/controls/selection_and_hud.md > CommandIndicators`), placeholders until the art
//! (Q0001):
//!
//! - **Markers** at the targets of the current orders of the local player's selected units (`> Indicator
//!   assignments`): a ring with a dot on the ground for a Location indicator (Move, AttackMove, Patrol; a Move to an
//!   object marks where the object is), a ring around the target for an Object indicator (Attack). They go when the
//!   unit is deselected or the order completes. Patrol's origin isn't marked: the sim keeps no origin yet.
//! - **Gathering markers** ([`gathering`], design change 0201): a Gather's on its source, a Drop Off's on its
//!   drop-off, and the Set Drop-off marker on each selected worker's AssignedDropOff, each with a look of its own.
//! - **Flashes** where an order was just given ([`OrderIssued`]): a ring closing in on the target while it fades,
//!   for [`FLASH_TICKS`]. They show for every order given, even one that completes at once.
//! - **Waypoints**: while the Queue modifier (Shift) is held, each selected unit's route from where it is drawn
//!   through the targets of its current and queued orders, as dashed lines, each leg in its order's colour, with
//!   the queued orders' markers.
//! - **Rally points** ([`rally`], design change 0087): each selected producer's rally point, a marker with a line
//!   from the building to it; none while it has no rally point.
//!
//! Colours follow `> Color language`: green for Move, red for Attack, orange for AttackMove and Patrol; the gathering
//! markers have colours of their own (`gathering::GATHER_COLOR`, the pointers'). A target
//! object is marked, and ends a route leg, where the local player knows it ([`targets`], design change 0139): where
//! it is drawn while the player sees it, where its record is drawn while the player remembers it (a structure or a
//! resource on Explored tiles), wherever the object really is; and not at all while the fog of war hides it otherwise
//! (a unit out of sight): the indicators would give away where it is. Everything is one alpha-blended mesh in the
//! overlays layer, between the path overlay and the combat effects, rebuilt every frame while anything shows.
//! Flashes age by the sim's ticks and the render alpha, so a QA screenshot of a tick (alpha 1) always shows them
//! alike.

pub mod gathering;
pub mod rally;
mod targets;

use std::collections::BTreeSet;
use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::orders::Order;
use sc_sim::{Map, Object, ObjectId, OrderKind, Sim, Target, Tick};

use super::OrderIssued;
use crate::coords::{DrawLayer, drawn_at_height, drawn_position, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::input::{ActionState, InputAction};
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;
use crate::sim_driver::{RenderAlpha, SimRes};
use crate::{ClientMode, ClientSystems};

pub use gathering::{GATHER_COLOR, GatheringMarker, RETURN_COLOR, SET_DROP_OFF_COLOR, drop_off_indicators};
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
/// Above the path overlay (at the layer's base) and below the combat effects (+1), in the overlays layer; the units'
/// tags go above it (`object_render::units`).
pub const INDICATORS_Z: f32 = 0.5;
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
    /// `> Object Indicator`: a marker around an object's perimeter, as the local player knows the object
    /// ([`targets`]).
    Object { id: ObjectId, color: Color },
    /// `> Gathering markers` (design change 0201): an Object indicator with a look of its own, one of the three
    /// gathering markers ([`gathering`]), around an object as the local player knows it.
    Gathering { id: ObjectId, marker: GatheringMarker },
}

impl Indicator {
    /// Its colour (`> Color language`; a gathering marker's own).
    pub fn color(self) -> Color {
        match self {
            Indicator::Location { color, .. } | Indicator::Object { color, .. } => color,
            Indicator::Gathering { marker, .. } => marker.color(),
        }
    }

    /// What tells it apart from another marker, so that each shows once: its kind, its place and its colour.
    fn key(self) -> (u8, [u32; 2], [u8; 4]) {
        let color = self.color().to_srgba().to_u8_array();
        match self {
            Indicator::Location { at, .. } => (0, [at.x.to_bits(), at.y.to_bits()], color),
            Indicator::Object { id, .. } => (1, [id.0, 0], color),
            Indicator::Gathering { id, marker } => (2, [id.0, marker as u32], color),
        }
    }
}

/// Where an indicator's marker is drawn at rest ([`marker_ring`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MarkerRing {
    /// Its centre on the ground, space units.
    pub center: Vec2,
    /// The height it is drawn at, levels: the ground's under a Location marker, the object's under an Object
    /// marker (`object_render::object_height`).
    pub height: f32,
    /// Its ring's radius, su.
    pub radius: f32,
    /// The half extents of the box an Object marker surrounds, su (`targets::object_box`); none for a Location
    /// marker. A gathering marker that isn't a ring goes by it ([`gathering`]).
    pub half: Vec2,
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

/// The colour of an order's indicator (`> Indicator assignments`), and of its leg of a route; none for orders without
/// a target.
pub fn order_color(kind: OrderKind) -> Option<Color> {
    match kind {
        // `> Indicator assignments` has no row for a Deploy: placeholder, it shows as a Move to its site's centre
        // beside its ghost (slice M3-J1).
        OrderKind::Move(_) | OrderKind::Deploy { .. } => Some(MOVE_COLOR),
        // `> Indicator assignments`: "Enter | Object | Green", the Move's green (slice M4-C; its look is M4-J1's).
        OrderKind::Enter(_) => Some(MOVE_COLOR),
        // Design change 0201: the gathering markers' own colours.
        OrderKind::Gather(_) => Some(GATHER_COLOR),
        OrderKind::DropOff(_) => Some(RETURN_COLOR),
        OrderKind::Attack(_) => Some(ATTACK_COLOR),
        OrderKind::AttackMove(_) | OrderKind::Patrol(_) => Some(AGGRESSIVE_COLOR),
        OrderKind::Hold | OrderKind::Stop => None,
    }
}

/// The indicator of an order (`> Indicator assignments`): a Location for Move, AttackMove and Patrol (a Move to an
/// object at the object as the player knows it, [`targets`]), an Object for Attack, around the object as the player
/// knows it, and the Gather marker and the Return Resources marker for Gather and Drop Off, around theirs (design
/// change 0201: "replacing the Move marker"); none for orders without a target, or whose target the player knows
/// nothing of.
pub fn indicator(sim: &Sim, sight: &Sight, kind: OrderKind) -> Option<Indicator> {
    let color = order_color(kind)?;
    let known = |id| sight.target_view(sim, id).is_known();
    match kind {
        OrderKind::Move(Target::Point(at)) | OrderKind::AttackMove(at) | OrderKind::Patrol(at) => {
            Some(Indicator::Location { at, color })
        }
        OrderKind::Move(Target::Object(id)) => {
            let at = targets::shown_at(sim, sight, id)?;
            Some(Indicator::Location { at, color })
        }
        OrderKind::Attack(id) | OrderKind::Enter(id) => known(id).then_some(Indicator::Object { id, color }),
        OrderKind::Gather(id) => known(id).then_some(Indicator::Gathering {
            id,
            marker: GatheringMarker::Gather,
        }),
        OrderKind::DropOff(id) => known(id).then_some(Indicator::Gathering {
            id,
            marker: GatheringMarker::ReturnResources,
        }),
        OrderKind::Deploy { origin } => Some(Indicator::Location {
            at: sc_sim::deploy::site(origin).center(),
            color,
        }),
        OrderKind::Hold | OrderKind::Stop => None,
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
            if seen.insert(indicator.key()) {
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

/// Where the leg of a route that carries out `order` ends, ground space: the target object as the player knows it
/// ([`targets`]; a Gather's source and a Drop Off's drop-off too, which their markers surround), else the unit's own
/// destination (its place in the group's shape, `Order::destination`); none for Hold and Stop, and for a target the
/// player knows nothing of.
fn waypoint(sim: &Sim, sight: &Sight, order: &Order) -> Option<Vec2> {
    match order.target() {
        Some(id) => targets::shown_at(sim, sight, id),
        None => order.destination(),
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

/// Rebuilds the indicators' mesh: the routes and the rally lines at the back, then the markers (the orders', the
/// selected workers' drop-offs' and the rally points'), then the flashes. Hidden while nothing shows, and outside the
/// game modes.
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
mod tests;
