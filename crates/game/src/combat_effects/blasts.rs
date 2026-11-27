//! (M4-O) Blasts (plan decision T52): a ring the size of an area attack's circle where its projectile landed
//! (`SimEvent::Blast`; `systems/combat.md > AoEDamage`: "Damage is uniform across the AoE circle"), flashing and fading
//! for [`BLAST_TICKS`]. Placeholder look until the art direction (Q0001). (M5-K) A single-target projectile's landing
//! (radius 0: a Rocketman's missile) is a small puff where it struck instead, at a body's height over its point, in the
//! air over an aircraft ([`Blast::lift`]).
//!
//! The fog: a blast shows only while the tile under its centre is Visible to the player ([`shows_blast`]), however far
//! its ring reaches (a placeholder reading of "where the viewer has vision"; its sighting, `crate::viewer_events`).
//! What it hit shows nothing of itself: a `SimEvent::Hit` is drawn for no victim, and an object the player doesn't
//! see, a hidden Hideout under the blast among them, shows no HP bar and no death marker (`super::hp_bars`,
//! `super::shows_death`).

use bevy::prelude::*;
use sc_sim::{Map, Sim, Tick};

use super::draw::MUZZLE_LIFT;
use super::shows;
use crate::coords::{drawn_at_height, view_to_world};
use crate::fog::Sight;
use crate::shapes::ColorMesh;
use crate::viewer_events::{Perception, Sighting};

pub use crate::tuning::feedback::BLAST_TICKS;

/// The ring's width, su, and its look: a fire's orange round a pale flash (placeholders, Q0001).
const RING_WIDTH: f32 = 4.0;
const RING: Srgba = Srgba::rgb(1.0, 0.55, 0.15);
const FLASH: Srgba = Srgba::rgb(1.0, 0.92, 0.7);
const SEGMENTS: usize = 32;
/// (M5-K) An impact puff's radius as it starts, su (placeholder, Q0001): it grows to twice that as it fades.
const PUFF: f32 = 5.0;

/// An area attack that landed in `tick` (`SimEvent::Blast`): its circle's centre, ground space, and radius, su.
#[derive(Clone, Debug, PartialEq)]
pub struct Blast {
    pub tick: Tick,
    pub at: Vec2,
    pub radius: f32,
    /// (M5-K) How far above the ground a single-target projectile struck, terrain levels: an aircraft's flight
    /// (`crate::object_render::flight_lift`), its puff drawn there; 0 for an area blast, on the ground.
    pub lift: f32,
    /// Whom it reaches, and how: each viewer while its centre's tile is Visible to them (`crate::viewer_events`,
    /// [`shows_blast`]).
    pub seen: Sighting,
}

/// Whether `sight` shows `blast` now: for [`BLAST_TICKS`] from its tick, as its sighting reaches the player
/// (`crate::viewer_events`): while the tile under its centre is Visible to the player, always while the fog doesn't
/// apply.
pub fn shows_blast(sim: &Sim, sight: &Sight, blast: &Blast) -> bool {
    shows(sim, blast.tick, BLAST_TICKS) && blast.seen.perceived_by(sight) == Perception::Whole
}

/// The blast `age` ticks after it landed: a pale disc that fades in its first third, inside a ring that grows from
/// three quarters of its radius to all of it as it fades; (M5-K) a single-target projectile's, an impact puff
/// ([`impact_shape`]).
pub(super) fn blast_shape(mesh: &mut ColorMesh, map: &Map, blast: &Blast, age: f32, z: f32) {
    let t = (age / BLAST_TICKS as f32).clamp(0.0, 1.0);
    if blast.radius <= 0.0 {
        return impact_shape(mesh, map, blast, t, z);
    }
    let center = view_to_world(drawn_at_height(blast.at, map.height_at(blast.at)));
    let flash = (1.0 - 3.0 * t).max(0.0);
    if flash > 0.0 {
        let disc: Vec<Vec2> = (0..SEGMENTS).map(|i| center + direction(i) * blast.radius).collect();
        mesh.polygon(&disc, z, FLASH.with_alpha(0.35 * flash).into());
    }
    ring(
        mesh,
        center,
        blast.radius * (0.75 + 0.25 * t),
        RING.with_alpha(0.9 * (1.0 - t)).into(),
        z,
    );
}

/// (M5-K) A single-target projectile's impact, `t` of the way through its [`BLAST_TICKS`]: a puff where it struck, at a
/// body's height over its point (`super::draw::MUZZLE_LIFT`, where a shot reaches a unit), [`Blast::lift`] levels up
/// for an aircraft: a pale flash that fades in its first half, inside a ring that grows from [`PUFF`] to twice that
/// as it fades.
fn impact_shape(mesh: &mut ColorMesh, map: &Map, blast: &Blast, t: f32, z: f32) {
    let drawn = drawn_at_height(blast.at, map.height_at(blast.at) + blast.lift) - Vec2::new(0.0, MUZZLE_LIFT);
    let center = view_to_world(drawn);
    let radius = PUFF * (1.0 + t);
    let flash = (1.0 - 2.0 * t).max(0.0);
    if flash > 0.0 {
        let disc: Vec<Vec2> = (0..SEGMENTS).map(|i| center + direction(i) * radius).collect();
        mesh.polygon(&disc, z, FLASH.with_alpha(0.8 * flash).into());
    }
    ring(mesh, center, radius, RING.with_alpha(0.9 * (1.0 - t)).into(), z);
}

/// A ring of [`RING_WIDTH`] round `center` (Bevy world space), `radius` su to its middle.
fn ring(mesh: &mut ColorMesh, center: Vec2, radius: f32, color: Color, z: f32) {
    let (inner, outer) = ((radius - RING_WIDTH / 2.0).max(0.0), radius + RING_WIDTH / 2.0);
    for i in 0..SEGMENTS {
        let (a, b) = (direction(i), direction(i + 1));
        let corners = [
            center + a * inner,
            center + a * outer,
            center + b * outer,
            center + b * inner,
        ];
        mesh.quad(corners.map(|p| p.extend(z)), color);
    }
}

/// The unit vector of the ring's `i`-th corner.
fn direction(i: usize) -> Vec2 {
    let a = i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
    Vec2::new(a.cos(), a.sin())
}
