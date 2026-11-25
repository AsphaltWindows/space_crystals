//! Command indicators (`systems/controls/selection_and_hud.md > CommandIndicators`), placeholders until the art
//! (Q0001):
//!
//! - **Markers** at the targets of the current orders of the local player's selected units (`> Indicator
//!   assignments`): a ring with a dot on the ground for a Location indicator (Move, AttackMove, Patrol, AttackGround;
//!   a Move to an object marks where the object is), a ring around the target for an Object indicator (Attack). They
//!   go when the unit is deselected or the order completes (an Attack Ground's when another order replaces it: it
//!   keeps firing until then, design change 0293). Patrol's origin isn't marked: the sim keeps no origin yet.
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
//! Colours follow `> Color language`: green for Move, red for Attack and AttackGround, orange for AttackMove and
//! Patrol; the gathering
//! markers have colours of their own (`gathering::GATHER_COLOR`, the pointers'). A target
//! object is marked, and ends a route leg, where the local player knows it ([`targets`], design change 0139): where
//! it is drawn while the player sees it, where its record is drawn while the player remembers it (a structure or a
//! resource on Explored tiles), wherever the object really is; and not at all while the fog of war hides it otherwise
//! (a unit out of sight): the indicators would give away where it is. Everything is one alpha-blended mesh in the
//! overlays layer, between the path overlay and the combat effects, rebuilt every frame while anything shows.
//! Flashes age by the sim's ticks and the render alpha, so a QA screenshot of a tick (alpha 1) always shows them
//! alike.

mod draw;
pub mod gathering;
pub mod rally;
mod targets;

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::orders::Order;
use sc_sim::{Object, ObjectId, OrderKind, Sim, Target, Tick};

use super::OrderIssued;
use crate::ClientSystems;
use crate::fog::Sight;
use crate::selection::ControlStates;

pub use draw::marker_ring;
pub use gathering::{GATHER_COLOR, GatheringMarker, RETURN_COLOR, SET_DROP_OFF_COLOR, drop_off_indicators};
pub use rally::{RallyIndicator, rally_indicators};

use draw::{OBJECT_GAP, collect_flashes, draw_indicators, ring, spawn_indicator_mesh};

pub use crate::tuning::feedback::FLASH_TICKS;

/// `> Color language` (placeholder shades): peaceful movement, a hostile target, aggressive movement.
pub const MOVE_COLOR: Color = Color::srgb(0.30, 0.95, 0.40);
pub const ATTACK_COLOR: Color = Color::srgb(1.00, 0.25, 0.20);
pub const AGGRESSIVE_COLOR: Color = Color::srgb(1.00, 0.58, 0.12);
/// Above the path overlay (at the layer's base) and below the combat effects (+1), in the overlays layer; the units'
/// tags go above it (`object_render::units`).
pub const INDICATORS_Z: f32 = 0.5;
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
        // beside its ghost (slice M3-J1); a Build too (slice M4-E).
        OrderKind::Move(_) | OrderKind::Deploy { .. } | OrderKind::Build { .. } => Some(MOVE_COLOR),
        // `> Indicator assignments`: "Enter | Object | Green", the Move's green (slice M4-C; its look is M4-J1's).
        OrderKind::Enter(_) => Some(MOVE_COLOR),
        // Design change 0201: the gathering markers' own colours.
        OrderKind::Gather(_) => Some(GATHER_COLOR),
        OrderKind::DropOff(_) => Some(RETURN_COLOR),
        // `> Indicator assignments`: "AttackGround | Location | Red" (slice M4-O).
        OrderKind::Attack(_) | OrderKind::AttackGround(_) => Some(ATTACK_COLOR),
        OrderKind::AttackMove(_) | OrderKind::Patrol(_) => Some(AGGRESSIVE_COLOR),
        OrderKind::Hold | OrderKind::Stop => None,
    }
}

/// The indicator of an order (`> Indicator assignments`): a Location for Move, AttackMove, Patrol and AttackGround (a
/// Move to an object at the object as the player knows it, [`targets`]), an Object for Attack, around the object as the player
/// knows it, and the Gather marker and the Return Resources marker for Gather and Drop Off, around theirs (design
/// change 0201: "replacing the Move marker"); none for orders without a target, or whose target the player knows
/// nothing of.
pub fn indicator(sim: &Sim, sight: &Sight, kind: OrderKind) -> Option<Indicator> {
    let color = order_color(kind)?;
    let known = |id| sight.target_view(sim, id).is_known();
    match kind {
        OrderKind::Move(Target::Point(at))
        | OrderKind::AttackMove(at)
        | OrderKind::Patrol(at)
        | OrderKind::AttackGround(at) => Some(Indicator::Location { at, color }),
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
        OrderKind::Build { .. } => Some(Indicator::Location {
            at: sc_sim::orders::build::build_site(kind)?.1.center(),
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
/// ([`targets`]; a Gather's source and a Drop Off's drop-off too, which their markers surround), an Attack Ground's
/// location, as an Attack's leg ends at its target, else the unit's own destination (its place in the group's shape,
/// `Order::destination`); none for Hold and Stop, and for a target the player knows nothing of.
fn waypoint(sim: &Sim, sight: &Sight, order: &Order) -> Option<Vec2> {
    if let OrderKind::AttackGround(at) = order.kind {
        return Some(at);
    }
    match order.target() {
        Some(id) => targets::shown_at(sim, sight, id),
        None => order.destination(),
    }
}

#[cfg(test)]
mod tests;
