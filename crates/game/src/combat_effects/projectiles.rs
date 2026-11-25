//! (M4-O) Projectiles in flight (plan decision T52): each drawn from its launch (`SimEvent::ProjectileLaunched`) to its
//! landing, along the straight path from where it left to the point it was locked on
//! (`systems/combat.md > DoublyDisjointed`: it "travels to the location the target was at at the end of the Aiming
//! phase"), at the place the ticks gone of its flight put it, lifted off the ground by a cosmetic arc with its shadow
//! under it. Placeholders until the art direction (Q0001): a lob as a grenade's, for every projectile so far, the
//! only ones being DoublyDisjointed (HeadDisjointed and TailDisjointed, which may fly flat, aren't built).
//!
//! The fog: `core/vision.md > Attacks from outside vision`: "a shot, tracer or projectile is visible to a player only
//! along the part of its path that crosses tiles **Visible** to that player": a projectile shows while its point on
//! the ground is over a Visible tile ([`shows_flight`]), as a shot's path is cut ([`super::visible_spans`]). **No
//! exceptions** (design change 0293, answering Q0083 point 4): a revealed attacker's projectile at the player's own
//! objects too, "you see only the visible part of its projectiles".

use bevy::prelude::*;
use sc_sim::{Map, ObjectId, PlayerId, Sim, Tick, VisibilityState};

use super::draw::MUZZLE_LIFT;
use crate::coords::{drawn_at_height, view_to_world};
use crate::fog::Sight;
use crate::object_render::player_color;
use crate::shapes::{ColorMesh, lighten};

/// How high the arc of a projectile rises halfway, as a share of its path's length. Placeholder (not in the design): a
/// lob, so that a grenade reads as thrown.
pub const ARC: f32 = 0.25;
/// The projectile's radius and its shadow's, su (placeholders, Q0001).
const RADIUS: f32 = 4.0;
const SHADOW: Vec2 = Vec2::new(5.0, 3.0);

/// A projectile in flight, as its launch said (`SimEvent::ProjectileLaunched`).
#[derive(Clone, Debug, PartialEq)]
pub struct Flight {
    /// The tick it was launched in.
    pub tick: Tick,
    pub shooter: ObjectId,
    /// The shooter's owner as it launched, whose colour it is drawn in.
    pub owner: Option<PlayerId>,
    /// The object whose place it was locked on, if it was fired at one (none at a location: an Attack Ground).
    pub target: Option<ObjectId>,
    /// Where it left and the point it lands on, ground space.
    pub from: Vec2,
    pub to: Vec2,
    /// The tick it lands in.
    pub lands_at: Tick,
}

impl Flight {
    /// Whether it is still in the air: launched by the sim's tick and not landed yet. It lands in the step to its
    /// `lands_at`, whose blast shows instead (`super::blasts`).
    pub fn flying(&self, sim: &Sim) -> bool {
        self.tick <= sim.tick() && sim.tick() < self.lands_at
    }

    /// How far along its path it is drawn now, 0 where it left to 1 where it lands: the sim's tick shows at alpha 1
    /// (`super::age`).
    pub fn progress(&self, sim: &Sim, alpha: f32) -> f32 {
        let flight = self.lands_at.saturating_sub(self.tick).max(1) as f32;
        (super::age(sim, alpha, self.tick) / flight).clamp(0.0, 1.0)
    }

    /// Its point on the ground at `progress`, ground space.
    pub fn ground(&self, progress: f32) -> Vec2 {
        self.from.lerp(self.to, progress)
    }

    /// How high above its point on the ground it is drawn at `progress`, su: the arc's parabola, [`ARC`] of its path's
    /// length halfway, over a line from the shooter's muzzle (`draw::MUZZLE_LIFT` up) down to the ground it lands on.
    pub fn height(&self, progress: f32) -> f32 {
        let arc = 4.0 * ARC * self.from.distance(self.to) * progress * (1.0 - progress);
        arc + MUZZLE_LIFT * (1.0 - progress)
    }
}

/// Whether `sight` shows `flight` now, at `progress` of its way: while it flies, and its point on the ground is over a
/// tile Visible to the player (`core/vision.md > Attacks from outside vision`), whoever threw it and at whatever: a
/// shooter revealed to the player, at the player's own object, too (design change 0293, which replaced the
/// placeholder that drew those whole, as design change 0244 drew its shots).
pub fn shows_flight(sim: &Sim, sight: &Sight, flight: &Flight, progress: f32) -> bool {
    flight.flying(sim) && sight.at(flight.ground(progress)) == VisibilityState::Visible
}

/// The projectile at `progress` of its way, in its shooter's colour, above its shadow on the ground.
pub(super) fn projectile_shape(mesh: &mut ColorMesh, map: &Map, flight: &Flight, progress: f32, z: f32) {
    let ground = flight.ground(progress);
    let drawn = drawn_at_height(ground, map.height_at(ground));
    oval(mesh, view_to_world(drawn), SHADOW, z, Color::srgba(0.0, 0.0, 0.0, 0.35));
    let center = view_to_world(drawn - Vec2::new(0.0, flight.height(progress)));
    oval(
        mesh,
        center,
        Vec2::splat(RADIUS + 1.0),
        z,
        Color::srgb(0.08, 0.08, 0.08),
    );
    let body = lighten(player_color(flight.owner).to_srgba(), 0.35);
    oval(mesh, center, Vec2::splat(RADIUS - 0.5), z, body.into());
}

/// A filled oval around `center` (Bevy world space), `radii` su across and up.
fn oval(mesh: &mut ColorMesh, center: Vec2, radii: Vec2, z: f32, color: Color) {
    let points: Vec<Vec2> = (0..10)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 10.0;
            center + Vec2::new(a.cos(), a.sin()) * radii
        })
        .collect();
    mesh.polygon(&points, z, color);
}

#[cfg(test)]
mod tests {
    use sc_sim::TickInputs;
    use sc_sim::testing::ScenarioBuilder;

    use super::*;

    /// A projectile from the origin to 4 GU east, launched in `tick`, landing in `lands_at`.
    fn flight(tick: Tick, lands_at: Tick) -> Flight {
        Flight {
            tick,
            shooter: ObjectId(1),
            owner: None,
            target: None,
            from: Vec2::ZERO,
            to: Vec2::new(256.0, 0.0),
            lands_at,
        }
    }

    fn steps(sim: &mut Sim, ticks: u32) {
        for _ in 0..ticks {
            sim.step(&TickInputs::default());
        }
    }

    /// It flies at the pace of the sim's ticks, the render alpha drawing it between them (alpha 1 shows the sim's
    /// tick): where it left in its own tick, a quarter of the way 8 of its 32 ticks on, and in the air until the step
    /// to its `lands_at`, where it lands.
    #[test]
    fn a_projectile_flies_at_the_pace_of_the_ticks() {
        let mut sim = Sim::new(&ScenarioBuilder::new(&vec![".".repeat(8); 8].join("\n")).build());
        steps(&mut sim, 10);
        let f = flight(10, 42);
        assert!(f.flying(&sim));
        assert_eq!(f.progress(&sim, 1.0), 0.0);
        steps(&mut sim, 8);
        assert_eq!(f.progress(&sim, 1.0), 0.25);
        assert_eq!(f.progress(&sim, 0.5), 7.5 / 32.0);
        assert_eq!(f.ground(0.25), Vec2::new(64.0, 0.0));
        steps(&mut sim, 23);
        assert!(f.flying(&sim), "tick 41");
        steps(&mut sim, 1);
        assert!(!f.flying(&sim), "landed in the step to tick 42");
    }

    /// The cosmetic arc: up from the shooter's muzzle, [`ARC`] of its way's length high halfway, down to the ground
    /// where it lands.
    #[test]
    fn the_arc_leaves_the_muzzle_and_comes_down_on_the_ground() {
        let f = flight(0, 32);
        assert_eq!(f.height(0.0), MUZZLE_LIFT);
        assert_eq!(f.height(0.5), ARC * 256.0 + MUZZLE_LIFT / 2.0);
        assert_eq!(f.height(1.0), 0.0);
    }
}
