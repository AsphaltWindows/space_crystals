//! The minimap's alerts (`systems/controls/hud.md > Minimap > Alerts`, slice M5-MM2, design change 0443): when "one of
//! the player's units or buildings **takes damage while it is off screen** (outside the camera's view)", "a red ping on
//! the minimap at that spot: a ring that shrinks and fades over about 2 s (placeholder). No text and no sound."
//!
//! - **What pings** ([`take_alerts`]): a `SimEvent::Hit` of the frame on an object of the player the minimap shows, as
//!   that player perceives the frame's events (`crate::viewer_events`: the owner always perceives a hit on their own
//!   object). Placeholder (not in the design: it says "takes damage" alone): whoever's the attack, an ally's or the
//!   player's own included; and placeholder (likewise): a hit that only takes Shields counts too. Where: the object's
//!   position now, or for one the hit killed, where it died (its `SimEvent::Died` of the same frame). A crush pings
//!   nothing (design change 0453: "**A crush gives no alert** [...] (a crush is not damage)"): it is a
//!   `SimEvent::Crushed`, with no Hit.
//! - **Off screen**: outside the ground the world view shows ([`super::viewed_ground`]), the rectangle the camera's
//!   outline shows on the minimap, so that the two agree: a hit inside the outline never pings.
//! - **How often** ([`MinimapAlerts::alert`]): "at most one ping per area every 10 s (placeholder; an area: within
//!   about 8 grid units of an earlier ping still in its 10 s)": a hit within `FeedbackTuning::ping_area_gu` of a ping
//!   made in the last `ping_throttle_ticks` makes none, though that ping faded long before.
//! - **Whose**: each ping is the player's the minimap showed as it was made, and shows only to them, as the fog does:
//!   after the debug switch, F2 (a replay's too), the other player's hits ping, and Space goes to their latest.
//! - **Drawn** ([`place_pings`]): a UI node's round border over the minimap's image ([`PingRing`]), as the camera's
//!   outline is, sized and faded by its age in sim ticks as drawn (`crate::combat_effects::age`, [`ring_at`]), so that
//!   QA's held ticks show it exactly, and gone after `FeedbackTuning::ping_ticks`.
//! - **Space** ([`jump_to_alert`]): "Jump to the latest alert: **Space** ... centres the camera on the **latest**
//!   alert's spot (snap, as Centering ...)" (`core/camera.md > Window Mode > Centering`). A reading: the latest ping
//!   made, faded or not (not the latest hit, which the throttle may have kept from pinging); nothing before the first.
//! - **A new sim** (a restart, a play-test) has none: they belong to the sim's generation (`SimRes::generation`).
//!
//! Colours and sizes are placeholders (Q0001).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{PlayerId, SimEvent, Tick, gu};

use super::{CameraSource, MinimapFrame, place};
use crate::ClientMode;
use crate::camera::{CameraView, map_extent_su};
use crate::combat_effects::age;
use crate::coords::drawn_position;
use crate::fog::LocalSight;
use crate::hud::results::has_left;
use crate::input::{ActionState, InputAction};
use crate::platform::SurfaceSize;
use crate::sim_driver::{LocalPlayer, RenderAlpha, SimRes};
use crate::tuning::ClientTuning;
use crate::tuning::feedback::FeedbackTuning;
use crate::viewer_events::ViewerEvents;

/// A ping's ring as it is made and as it ends, its outer size (logical pixels); its border's width; its colour.
/// Placeholders (Q0001).
const RING_START_PX: f32 = 28.0;
const RING_END_PX: f32 = 6.0;
const RING_BORDER_PX: f32 = 2.0;
const RING_COLOR: Color = Color::srgb(1.0, 0.12, 0.08);

// A ping is kept for the throttle at least as long as it shows.

/// A ping on the minimap.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ping {
    /// Whose alert it is: the player the minimap showed as it was made.
    pub player: PlayerId,
    /// The spot, a ground point on the map (space units).
    pub at: Vec2,
    /// The sim's tick of the hit that made it.
    pub tick: Tick,
}

/// The minimap's alerts: the recent pings, and each player's latest.
#[derive(Resource, Debug, Default)]
pub struct MinimapAlerts {
    /// The pings made in the last `FeedbackTuning::ping_throttle_ticks` (or `ping_ticks`, if a data file made them
    /// longer), oldest first: those that show, and those that faded but still keep their area from pinging.
    pings: Vec<Ping>,
    /// Each player's latest ping's spot, where Space centres the camera.
    latest: Vec<(PlayerId, Vec2)>,
    /// How many pings were made, for tests and QA.
    pub made: u32,
    /// The `SimRes::generation` they belong to: a replaced sim has none.
    generation: Option<u32>,
}

impl MinimapAlerts {
    /// `systems/controls/hud.md > Minimap > Alerts`: "at most **one ping per area every 10 s** (placeholder; an
    /// area: within about 8 grid units of an earlier ping still in its 10 s)". An alert of `player`'s at `at` in tick
    /// `tick` pings unless one of their pings made less than `tuning`'s `ping_throttle_ticks` before lies within its
    /// `ping_area_gu` of it, shown or faded. Returns whether it pinged.
    pub fn alert(&mut self, player: PlayerId, at: Vec2, tick: Tick, tuning: &FeedbackTuning) -> bool {
        let area = gu(tuning.ping_area_gu);
        let throttle = tuning.ping_throttle_ticks;
        let near = |p: &Ping| p.player == player && tick.saturating_sub(p.tick) < throttle && p.at.distance(at) <= area;
        if self.pings.iter().any(near) {
            return false;
        }
        self.pings.push(Ping { player, at, tick });
        match self.latest.iter_mut().find(|(p, _)| *p == player) {
            Some((_, latest)) => *latest = at,
            None => self.latest.push((player, at)),
        }
        self.made += 1;
        true
    }

    /// The pings made in the last `FeedbackTuning::ping_throttle_ticks` (or `ping_ticks`, if longer), oldest first.
    pub fn pings(&self) -> &[Ping] {
        &self.pings
    }

    /// Where `player`'s latest ping was: where Space centres the camera. `None` before their first.
    pub fn latest(&self, player: PlayerId) -> Option<Vec2> {
        self.latest.iter().find(|(p, _)| *p == player).map(|&(_, at)| at)
    }

    /// Forgets the pings made `tuning`'s `ping_throttle_ticks` or more before `tick` (its `ping_ticks`, if a data file
    /// made them longer): they neither show nor throttle any more.
    fn forget(&mut self, tick: Tick, tuning: &FeedbackTuning) {
        let kept = tuning.ping_throttle_ticks.max(tuning.ping_ticks);
        self.pings.retain(|p| tick.saturating_sub(p.tick) < kept);
    }

    /// Starts over for a sim of `generation` that isn't theirs: no pings, no latest.
    fn follow(&mut self, generation: u32) {
        if self.generation != Some(generation) {
            *self = Self {
                generation: Some(generation),
                ..default()
            };
        }
    }
}

/// A ping's ring at `age` ticks, as drawn: its outer size (logical pixels) and its alpha, from [`RING_START_PX`] and
/// opaque as it is made, shrinking and fading evenly to [`RING_END_PX`] and clear at `ping_ticks`
/// (`FeedbackTuning::ping_ticks`); `None` from then on.
pub fn ring_at(age: f32, ping_ticks: Tick) -> Option<(f32, f32)> {
    let t = age.max(0.0) / ping_ticks.max(1) as f32;
    (t < 1.0).then_some((RING_START_PX + (RING_END_PX - RING_START_PX) * t, 1.0 - t))
}

/// The layer over the minimap's image that holds the pings' rings, clipped to the map area.
#[derive(Component)]
pub struct PingLayer;

/// A ping's ring: a round node with a border, in the [`PingLayer`]. Rings are kept once made, hidden while no ping
/// needs them.
#[derive(Component)]
pub struct PingRing;

/// The [`PingLayer`] node, a child of the minimap's image node that covers it (`super::spawn_minimap`).
pub(super) fn ping_layer() -> impl Bundle {
    (
        Name::new("Minimap pings"),
        PingLayer,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            overflow: Overflow::clip(),
            ..default()
        },
    )
}

/// Pings for the frame's hits on the shown player's objects off screen (module docs), once the camera is clamped for
/// the frame. Only in the game modes, the HUD's; a new sim's alerts start over.
pub(super) fn take_alerts(
    camera: CameraSource,
    sight: LocalSight,
    events: Res<ViewerEvents>,
    tuning: Res<ClientTuning>,
    mut alerts: ResMut<MinimapAlerts>,
) {
    let tuning = &tuning.feedback;
    alerts.follow(camera.sim.generation());
    if !camera.mode.is_game() {
        return;
    }
    let sim = sight.sim();
    let seen = sight.get();
    let player = seen.player();
    alerts.forget(sim.tick(), tuning);
    let perceived: Vec<_> = events
        .perceived_by(sim, seen)
        .map(|(tick, event, _)| (tick, event))
        .collect();
    if perceived.is_empty() {
        return;
    }
    let died = |id| {
        perceived.iter().find_map(|(_, event)| match **event {
            SimEvent::Died {
                id: dead, owner, pos, ..
            } if dead == id => Some((owner, pos)),
            _ => None,
        })
    };
    let viewed = camera.viewed_ground();
    for &(tick, event) in &perceived {
        let SimEvent::Hit {
            target,
            damage,
            shield_damage,
            ..
        } = *event
        else {
            continue;
        };
        if damage <= 0.0 && shield_damage <= 0.0 {
            continue;
        }
        let Some((owner, pos)) = sim.object(target).map(|o| (o.owner, o.pos)).or_else(|| died(target)) else {
            continue;
        };
        if owner == Some(player) && !viewed.contains(pos) {
            alerts.alert(player, sim.map().clamp_point(pos), tick, tuning);
        }
    }
}

/// What the rings are placed from.
#[derive(SystemParam)]
pub(super) struct RingSource<'w> {
    mode: Res<'w, ClientMode>,
    size: Res<'w, SurfaceSize>,
    sim: Res<'w, SimRes>,
    player: Res<'w, LocalPlayer>,
    alpha: Res<'w, RenderAlpha>,
    alerts: Res<'w, MinimapAlerts>,
    tuning: Res<'w, ClientTuning>,
}

/// Shows the shown player's pings as rings at their spots on the minimap, each as large and as clear as its age makes
/// it ([`ring_at`]), and hides the rings no ping needs; makes a ring more when there are too few. Only in the game
/// modes.
pub(super) fn place_pings(
    mut commands: Commands,
    source: RingSource,
    layers: Query<Entity, With<PingLayer>>,
    mut rings: Query<(&mut Node, &mut BorderColor), With<PingRing>>,
) {
    let sim = &**source.sim;
    let frame = MinimapFrame::on(*source.size, map_extent_su(sim.map()));
    let scale = frame.scale();
    let snap = |p: Vec2| (p * scale).round() / scale;
    let mut shown = (source.alerts.pings().iter())
        .filter(|_| source.mode.is_game())
        .filter(|p| p.player == source.player.0)
        .filter_map(|p| {
            let (size, alpha) = ring_at(age(sim, source.alpha.0, p.tick), source.tuning.feedback.ping_ticks)?;
            let ring = Rect::from_center_size(frame.screen_at(p.at), Vec2::splat(size));
            let ring = Rect::from_corners(snap(ring.min), snap(ring.max));
            Some((ring, RING_COLOR.with_alpha(alpha)))
        });
    for (mut node, mut border) in &mut rings {
        let Some((ring, color)) = shown.next() else {
            super::show_node(&mut node, false);
            continue;
        };
        super::show_node(&mut node, true);
        place(&mut node, ring.min - frame.area.min, ring.size());
        border.set_if_neq(BorderColor::all(color));
    }
    let Ok(layer) = layers.single() else {
        return;
    };
    for (ring, color) in shown {
        let at = ring.min - frame.area.min;
        commands.spawn((
            Name::new("Minimap ping"),
            PingRing,
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x),
                top: px(at.y),
                width: px(ring.width()),
                height: px(ring.height()),
                border: UiRect::all(px(RING_BORDER_PX)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(color),
            ChildOf(layer),
        ));
    }
}

/// `systems/controls/hud.md > Minimap > Alerts`: "Jump to the latest alert: **Space** ... centres the camera on the
/// **latest** alert's spot (snap, as Centering ...)" (`core/camera.md > Window Mode > Centering`): on its
/// [`drawn_position`], kept on the map afterwards as every view is (`crate::camera`). Nothing before the first ping,
/// outside the game modes, or while the results screen covers the HUD (`super::super::results`).
pub(super) fn jump_to_alert(
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut alerts: ResMut<MinimapAlerts>,
    mut view: ResMut<CameraView>,
) {
    if !mode.is_game() || !actions.just_pressed(InputAction::JumpToAlert) || has_left(&sim, player.0) {
        return;
    }
    alerts.follow(sim.generation());
    if let Some(at) = alerts.latest(player.0) {
        view.center = drawn_position(sim.map(), at);
    }
}

#[cfg(test)]
mod tests;
