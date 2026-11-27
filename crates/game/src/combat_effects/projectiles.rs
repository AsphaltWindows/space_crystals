//! (M4-O) Projectiles in flight (plan decision T52): each drawn from its launch (`SimEvent::ProjectileLaunched`) to its
//! landing, along the straight path from where it left to the point it was locked on
//! (`systems/combat.md > DoublyDisjointed`: it "travels to the location the target was at at the end of the Aiming
//! phase"), at the place the ticks gone of its flight put it, lifted off the ground by a cosmetic arc with its shadow
//! under it. Placeholders until the art direction (Q0001): a lob as a grenade's, for every projectile but a missile;
//! each in its thrower's style ([`ShotStyle::of`], from its kind's client module, plan decision T52): the Grenadier's
//! a grenade with its fuse lit (slice M4-U), any other a plain ball. (M5-K) A missile, the Rocketman's
//! ([`ShotStyle::Missile`]), flies flat instead, from the shooter's muzzle to the target's body as a shot does
//! (`super::draw::MUZZLE_LIFT`), up to an aircraft's flight ([`Flight::lift`]), nose first with its exhaust lit
//! ([`missile_shape`]). A homing missile (HeadDisjointed, `sc_sim::combat::projectiles::Homing`) has no path known at
//! its launch: it is drawn from the sim's store as the sim steers it ([`homing_missiles`]), between where the last two
//! ticks left it, and its launch draws nothing (`SimEvent::ProjectileLaunched::homing`).
//!
//! The fog: `core/vision.md > Attacks from outside vision`: "a shot, tracer or projectile is visible to a player only
//! along the part of its path that crosses tiles **Visible** to that player": a projectile shows while its point on
//! the ground is over a Visible tile ([`shows_flight`]), as a shot's path is cut ([`super::visible_spans`]). **No
//! exceptions** (design change 0293, answering Q0083 point 4): a revealed attacker's projectile at the player's own
//! objects too, "you see only the visible part of its projectiles". Its sighting says so (`crate::viewer_events`); a
//! homing missile shows while its point on the ground is over a Visible tile alike.

use bevy::prelude::*;
use sc_sim::{Map, ObjectId, ObjectKind, PlayerId, Sim, Tick, VisibilityState};

use super::ShotStyle;
use super::draw::MUZZLE_LIFT;
use crate::coords::{drawn_at_height, view_to_world};
use crate::fog::Sight;
use crate::object_render::{flight_lift, player_color};
use crate::shapes::{ColorMesh, lighten};
use crate::viewer_events::{Perception, Sighting};

/// How high the arc of a projectile rises halfway, as a share of its path's length. Placeholder (not in the design): a
/// lob, so that a grenade reads as thrown.
pub const ARC: f32 = 0.25;
/// The projectile's radius and its shadow's, su (placeholders, Q0001).
const RADIUS: f32 = 4.0;
const SHADOW: Vec2 = Vec2::new(5.0, 3.0);
/// A grenade's lit fuse: a spark on its top, its radius su (placeholders, Q0001).
const FUSE: Color = Color::srgb(1.0, 0.85, 0.35);
const FUSE_RADIUS: f32 = 2.0;
/// (M5-K) A missile's body, its length and width, its exhaust's flame's length and width, and its shadow's radii, su,
/// and the flame's and the nose's colours (placeholders, Q0001).
const MISSILE_LENGTH: f32 = 14.0;
const MISSILE_WIDTH: f32 = 3.0;
const FLAME_LENGTH: f32 = 8.0;
const FLAME_WIDTH: f32 = 4.0;
const MISSILE_SHADOW: Vec2 = Vec2::new(5.0, 2.5);
const FLAME: Srgba = Srgba::rgb(1.0, 0.62, 0.2);
const NOSE: Srgba = Srgba::rgb(0.92, 0.92, 0.88);

/// A projectile in flight, as its launch said (`SimEvent::ProjectileLaunched`).
#[derive(Clone, Debug, PartialEq)]
pub struct Flight {
    /// The tick it was launched in.
    pub tick: Tick,
    pub shooter: ObjectId,
    /// The form its shooter launched it in (`sc_sim::conversion::form`; its type if it died by the time the client saw
    /// the launch), whose style it is drawn in ([`ShotStyle::of`]), as a shot is (`super::Tracer::weapon`).
    pub weapon: Option<ObjectKind>,
    /// The shooter's owner as it launched, whose colour it is drawn in.
    pub owner: Option<PlayerId>,
    /// The object whose place it was locked on, if it was fired at one (none at a location: an Attack Ground).
    pub target: Option<ObjectId>,
    /// Where it left and the point it lands on, ground space.
    pub from: Vec2,
    pub to: Vec2,
    /// The tick it lands in.
    pub lands_at: Tick,
    /// (M5-K) How far above the ground its two ends are drawn, terrain levels, as a shot's are (`super::Tracer::lift`):
    /// its shooter's flight and its target's at its launch (`crate::object_render::flight_lift`), so that a missile at
    /// an aircraft flies up to it.
    pub lift: (f32, f32),
    /// Whom it reaches, and how: along its path, over each viewer's Visible tiles (`crate::viewer_events`,
    /// [`shows_flight`]).
    pub seen: Sighting,
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
    /// length halfway, over a line from the shooter's muzzle (`draw::MUZZLE_LIFT` up) down to the ground it lands on;
    /// (M5-K) a missile's at a muzzle's height all the way, flat, from its shooter's body to its target's.
    pub fn height(&self, progress: f32) -> f32 {
        if ShotStyle::of(self.weapon) == ShotStyle::Missile {
            return MUZZLE_LIFT;
        }
        let arc = 4.0 * ARC * self.from.distance(self.to) * progress * (1.0 - progress);
        arc + MUZZLE_LIFT * (1.0 - progress)
    }

    /// (M5-K) Its lift at `progress`, terrain levels: from its shooter's to its target's ([`Flight::lift`]).
    pub fn lift_at(&self, progress: f32) -> f32 {
        self.lift.0 + (self.lift.1 - self.lift.0) * progress
    }

    /// Where it is drawn at `progress`, view space: over its point on the ground, lifted ([`Flight::lift_at`]), and
    /// [`Flight::height`] up from there.
    fn drawn(&self, map: &Map, progress: f32) -> Vec2 {
        let ground = self.ground(progress);
        drawn_at_height(ground, map.height_at(ground) + self.lift_at(progress)) - Vec2::new(0.0, self.height(progress))
    }
}

/// Whether `sight` shows `flight` now, at `progress` of its way: while it flies, as its sighting reaches the player
/// (`crate::viewer_events`): while its point on the ground is over a tile Visible to the player
/// (`core/vision.md > Attacks from outside vision`), always while the fog doesn't apply; whoever threw it and at
/// whatever: a shooter revealed to the player, at the player's own object, too (design change 0293, which replaced the
/// placeholder that drew those whole, as design change 0244 drew its shots).
pub fn shows_flight(sim: &Sim, sight: &Sight, flight: &Flight, progress: f32) -> bool {
    flight.flying(sim)
        && match flight.seen.perceived_by(sight) {
            Perception::Whole => true,
            Perception::Clipped { from, to } => sight.at(from.lerp(to, progress)) == VisibilityState::Visible,
            Perception::Nothing => false,
        }
}

/// The projectile at `progress` of its way, in its shooter's colour, above its shadow on the ground; a grenade
/// ([`ShotStyle::Grenade`]) with its fuse lit on its top; (M5-K) a missile ([`ShotStyle::Missile`]) as
/// [`missile_shape`] draws one, heading along its path.
pub(super) fn projectile_shape(mesh: &mut ColorMesh, map: &Map, flight: &Flight, progress: f32, z: f32) {
    let ground = flight.ground(progress);
    let drawn = drawn_at_height(ground, map.height_at(ground));
    if ShotStyle::of(flight.weapon) == ShotStyle::Missile {
        let heading = flight.drawn(map, 1.0) - flight.drawn(map, 0.0);
        let missile = Missile {
            body: flight.drawn(map, progress),
            heading: heading.try_normalize().unwrap_or(Vec2::X),
            shadow: drawn,
            owner: flight.owner,
        };
        return missile_shape(mesh, &missile, z);
    }
    oval(mesh, view_to_world(drawn), SHADOW, z, Color::srgba(0.0, 0.0, 0.0, 0.35));
    let center = view_to_world(flight.drawn(map, progress));
    oval(
        mesh,
        center,
        Vec2::splat(RADIUS + 1.0),
        z,
        Color::srgb(0.08, 0.08, 0.08),
    );
    let body = lighten(player_color(flight.owner).to_srgba(), 0.35);
    oval(mesh, center, Vec2::splat(RADIUS - 0.5), z, body.into());
    if ShotStyle::of(flight.weapon) == ShotStyle::Grenade {
        let fuse = center + Vec2::new(RADIUS * 0.5, RADIUS);
        oval(mesh, fuse, Vec2::splat(FUSE_RADIUS), z, FUSE);
    }
}

/// (M5-K) A missile to draw, view space: its body's middle, the way it heads (a unit vector), its shadow's place on the
/// ground under it, and its shooter's owner, whose colour its body is.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Missile {
    pub body: Vec2,
    pub heading: Vec2,
    pub shadow: Vec2,
    pub owner: Option<PlayerId>,
}

/// (M5-K) The homing missiles in flight that `sight` shows (module docs): each of the sim's store with a flight of its
/// own (`sc_sim::combat::projectiles::Projectile::homing`), between where the last two ticks left it, `alpha` of the
/// way (`Homing::prev` to `Homing::pos`, as an object is drawn), while its point on the ground is over a tile Visible
/// to the player (`core/vision.md > Attacks from outside vision`, design change 0293). It is lifted from its shooter's
/// flight to its target's as it closes in on the target (by its distances to the two), heading the way it flies, or at
/// its target as it leaves. A missile whose target has left the game flies on at the ground's height (placeholder).
pub fn homing_missiles(sim: &Sim, sight: &Sight, alpha: f32) -> Vec<Missile> {
    let map = sim.map();
    let lift_of = |id: Option<ObjectId>| id.and_then(|id| sim.object(id)).map_or(0.0, |o| flight_lift(o.kind));
    let missile_at =
        |ground: Vec2, lift: f32| drawn_at_height(ground, map.height_at(ground) + lift) - Vec2::new(0.0, MUZZLE_LIFT);
    (sim.projectiles().iter())
        .filter_map(|p| {
            let homing = p.homing.as_ref()?;
            let ground = homing.prev.lerp(homing.pos, alpha);
            if sight.at(ground) != VisibilityState::Visible {
                return None;
            }
            let (from_lift, to_lift) = (lift_of(Some(p.shooter)), lift_of(p.target));
            let lift = |at: Vec2| {
                let (gone, left) = (p.from.distance(at), at.distance(p.to));
                from_lift + (to_lift - from_lift) * (gone / (gone + left).max(1.0))
            };
            let body = missile_at(ground, lift(ground));
            let flown = missile_at(homing.pos, lift(homing.pos)) - missile_at(homing.prev, lift(homing.prev));
            let heading = (flown.try_normalize())
                .or_else(|| (missile_at(p.to, to_lift) - body).try_normalize())
                .unwrap_or(Vec2::X);
            Some(Missile {
                body,
                heading,
                shadow: drawn_at_height(ground, map.height_at(ground)),
                owner: p.owner,
            })
        })
        .collect()
}

/// (M5-K) A missile (placeholder look, Q0001): its shadow on the ground, and over it a slim body in its shooter's
/// colour, dark-rimmed, with a pale nose ahead and its exhaust's flame behind.
pub(super) fn missile_shape(mesh: &mut ColorMesh, missile: &Missile, z: f32) {
    oval(
        mesh,
        view_to_world(missile.shadow),
        MISSILE_SHADOW,
        z,
        Color::srgba(0.0, 0.0, 0.0, 0.3),
    );
    let (center, heading) = (view_to_world(missile.body), view_to_world(missile.heading));
    let nose = center + heading * (MISSILE_LENGTH / 2.0);
    let tail = center - heading * (MISSILE_LENGTH / 2.0);
    let flame = tail - heading * FLAME_LENGTH;
    mesh.segment(flame, tail, FLAME_WIDTH, z, FLAME.with_alpha(0.8).into());
    let body = lighten(player_color(missile.owner).to_srgba(), 0.35);
    mesh.segment(tail, nose, MISSILE_WIDTH + 2.0, z, Color::srgb(0.08, 0.08, 0.08));
    mesh.segment(tail, nose, MISSILE_WIDTH, z, body.into());
    mesh.segment(nose - heading * 2.5, nose, MISSILE_WIDTH, z, NOSE.into());
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
        let (from, to) = (Vec2::ZERO, Vec2::new(256.0, 0.0));
        Flight {
            tick,
            shooter: ObjectId(1),
            weapon: None,
            owner: None,
            target: None,
            from,
            to,
            lands_at,
            lift: (0.0, 0.0),
            seen: Sighting::path(from, to),
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

    /// (M5-K) A missile (a Rocketman's) flies flat, at a muzzle's height all the way, and up from its shooter's flight
    /// to its target's: half way up to an aircraft at half its way.
    #[test]
    fn a_missile_flies_flat_up_to_its_target() {
        let missile = Flight {
            weapon: Some(ObjectKind::Rocketman),
            lift: (0.0, 3.0),
            ..flight(0, 32)
        };
        for progress in [0.0, 0.5, 1.0] {
            assert_eq!(missile.height(progress), MUZZLE_LIFT);
        }
        assert_eq!(missile.lift_at(0.5), 1.5);
        assert_eq!(missile.lift_at(1.0), 3.0);
    }

    /// (M5-K) A homing missile is drawn from the sim's store (`Sim::projectiles`): between where the last two ticks
    /// left it, `alpha` of the way, at a muzzle's height over the ground (a ground unit at each end), heading the way
    /// it flies, east at its target; in its shooter's owner's colour. Only a viewer who sees its point on the ground
    /// sees it: none for one who sees nothing.
    #[test]
    fn a_homing_missile_is_drawn_where_the_sim_steers_it() {
        use sc_sim::testing::TestSim;
        use sc_sim::{Faction, OrderKind, PlayerId, SpawnAt};
        let (p1, p2) = (PlayerId(1), PlayerId(2));
        let (rocketman, target) = (ObjectId(1), ObjectId(2));
        let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
            .faction(p1, Faction::Gdo)
            .faction(p2, Faction::Gdo)
            .spawn(ObjectKind::Rocketman, Some(p1), SpawnAt::Point(Vec2::new(200.0, 384.0)))
            .spawn(
                ObjectKind::Peacekeeper,
                Some(p2),
                SpawnAt::Point(Vec2::new(520.0, 384.0)),
            )
            .build();
        let mut t = TestSim::new(scenario);
        t.without_auto_targeting();
        t.order_at(2, p1, &[rocketman], OrderKind::TargetLock(target), false);
        while !t
            .sim()
            .projectiles()
            .iter()
            .any(|p| p.homing.is_some_and(|h| h.pos != h.prev))
        {
            assert!(t.sim().tick() < 100, "a locked missile in flight by now");
            t.step();
        }
        let sim = t.sim();
        let homing = sim.projectiles()[0].homing.unwrap();
        let drawn = homing_missiles(sim, &Sight::new(sim, p1, true), 0.5);
        let body = homing.prev.lerp(homing.pos, 0.5) - Vec2::new(0.0, MUZZLE_LIFT);
        assert_eq!(drawn.len(), 1, "{drawn:?}");
        assert!(drawn[0].body.distance(body) < 1e-3, "{drawn:?}");
        assert!(drawn[0].heading.distance(Vec2::X) < 1e-3, "{drawn:?}");
        assert_eq!(drawn[0].owner, Some(p1));
        assert_eq!(homing_missiles(sim, &Sight::new(sim, PlayerId(3), true), 0.5), []);
    }
}
