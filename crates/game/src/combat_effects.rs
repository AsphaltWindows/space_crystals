//! Placeholder combat feedback (`systems/combat.md`; the art waits on Q0001): a short tracer from the attacker to its
//! target for each attack that takes effect (`SimEvent::Attacked`), a Syndicate laser's red ([`ShotStyle`], design
//! change 0207); a TailDisjointed shot's to the spot it locked, whether it struck something there or missed
//! (`SimEvent::Missed`, slice M5-O3: the Gunner's, "Visually still an instant gunshot"); HP bars over damaged and
//! selected objects, with a
//! shield bar above the HP bar of an object that has Shields (`systems/combat.md > Shields > Display`, slice M3-B,
//! design change 0148: "a shield bar sits directly above the HP bar"; its styling waits for the HUD pass), and a
//! fading marker where an object died (`SimEvent::Died`). Slice M4-O adds the projectiles in flight
//! ([`projectiles`], from `SimEvent::ProjectileLaunched`, plan decision T52), each in its thrower's style (the
//! Grenadier's grenade, slice M4-U), and the blasts where they land ([`blasts`], `SimEvent::Blast`).
//!
//! Everything is one vertex-coloured, alpha-blended mesh in the overlays layer, rebuilt every frame while anything
//! shows. Effects age by the sim's ticks plus the render alpha, so a QA screenshot of a tick (alpha 1) always shows
//! them alike. HP bars follow the units' interpolated positions.
//!
//! The fog of war hides what the local player doesn't see (`crate::fog`). Whom each effect reaches, and how, is the
//! filter of the sim's events' (`crate::viewer_events`: each effect keeps its event's [`Sighting`]); what is left to
//! the drawing code here is where a path crosses the viewer's Visible tiles ([`visible_spans`]) and how long each
//! effect lasts:
//! - **A shot** shows only along the parts of its path over tiles Visible to the player ([`visible_spans`],
//!   [`shot_spans`]; `core/vision.md > Attacks from outside vision`, design change 0177: "a shot fired from the fog at
//!   your unit is seen coming out of the fog edge, which shows its direction"), **with no exception** (design change
//!   0293, answering Q0083 point 4): "every shot, tracer or projectile, whoever fires it and at whatever, is drawn
//!   **only over your Visible tiles**". The attacker itself is the sim's to reveal (`sc_sim::vision::reveal`):
//!   revealed, it shows like any object the player sees, and its shots still come out of the fog edge. 0293 replaced
//!   the exception of design changes 0206 and 0244 (Q0056, Q0075), which drew a revealed attacker's shots at the
//!   player's own objects whole, from the attacker to the target.
//! - **A death marker** only on a Visible tile or for one of the player's own objects, and **an HP bar** only over an
//!   object the player sees (`Sight::sees`): placeholder rules, the design is silent. Neither ever shows for an object
//!   hidden from the player (`sc_sim::vision::Hiding`: a Hideout under a blast, plan decision T52: "nothing is drawn
//!   for a hidden victim"; the `Hiding` its death carries, `SimEvent::Died::hiding`), and no hit is drawn for any
//!   victim.
//! - **A projectile** only while its point on the ground is over a Visible tile, whoever threw it ([`shows_flight`],
//!   design change 0293: "you see only the visible part of its projectiles"); **a blast** only while the tile under
//!   its centre is Visible ([`shows_blast`]).
//!
//! Placeholder (Q0090; the look waits on Q0001): a tracer's glow and a projectile's body are in the colour of their
//! shooter's owner, whom the viewer may not see: the design is silent on what a shot shows of its shooter.

pub mod blasts;
mod draw;
pub mod projectiles;

use bevy::prelude::*;
use sc_sim::{Map, Object, ObjectId, ObjectKind, PlayerId, SU_PER_GU, Sim, SimEvent, Tick, VisibilityState};

use crate::ClientSystems;
use crate::coords::drawn_at_height;
use crate::fog::Sight;
use crate::object_render::object_height;
use crate::sim_driver::SimRes;
use crate::viewer_events::{FilterEvents, Perception, Sighting, ViewerEvents};
use draw::{draw_effects, spawn_effects_mesh};

pub use crate::tuning::feedback::{DEATH_MARKER_TICKS, TRACER_TICKS};
pub use blasts::{BLAST_TICKS, Blast, shows_blast};
pub use projectiles::{Flight, shows_flight};

/// HP bars: height, gap above the corners of the unit's silhouette (clear of a selection ring, which lies 7 su
/// outside them: `selection::markers`) or above a tile-based object's footprint, and the narrowest bar, su. A shield
/// bar is as high, right above the HP bar, their frames shared.
const BAR_HEIGHT: f32 = 4.0;
const BAR_GAP: f32 = 9.0;
const BAR_MIN_WIDTH: f32 = 24.0;

/// An attack that took effect in `tick` (`SimEvent::Attacked`), from and to the ground points of both units then; a
/// TailDisjointed one's to the spot it locked, where it struck or missed (`SimEvent::Missed`, slice M5-O3).
#[derive(Clone, Debug, PartialEq)]
pub struct Tracer {
    pub tick: Tick,
    /// The attacker: revealed to the player or not, its shot shows only over the player's Visible tiles
    /// ([`shot_spans`]).
    pub attacker: ObjectId,
    /// The form the attacker fired in (`sc_sim::conversion::form`; its type if it died by the time the client saw the
    /// shot), whose weapon the shot is drawn as ([`ShotStyle::of`]).
    pub weapon: Option<ObjectKind>,
    pub owner: Option<PlayerId>,
    /// The object hit; none for a shot that hit nothing.
    pub target: Option<ObjectId>,
    pub from: Vec2,
    pub to: Vec2,
    /// Whom it reaches, and how: along its path, over each viewer's Visible tiles (`crate::viewer_events`,
    /// [`shot_spans`]).
    pub seen: Sighting,
}

/// How a shot or a projectile is drawn, by the weapon that fired it: each shooting kind's style is in its module
/// (`crate::kinds::ClientKind::shot`), which says which weapons are lasers of which colour, and which throw grenades.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShotStyle {
    /// A pale tracer inside a glow in the shooter's colour: every shot but a Syndicate laser's (placeholder, Q0001).
    Tracer,
    /// A red laser: design change 0207 (`factions/syndicate/units/README.md`): "every Syndicate laser-shooting unit
    /// and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser", whoever owns it.
    RedLaser,
    /// A grenade (`factions/syndicate/units/grenadier.md`, slice M4-U): no laser, and no shot either: a projectile,
    /// drawn in flight as a lobbed grenade, its fuse lit ([`projectiles`]), that takes effect by its blast, never as
    /// a shot (`SimEvent::Attacked`). Placeholder look (Q0001).
    Grenade,
}

impl ShotStyle {
    /// The style of a shot or a projectile fired in the form `weapon`, its kind's: the Guard's laser rifle and the
    /// Laser Turret's laser (slice M4-S1) are red lasers (the Laser Battery's will be when it exists), the Grenadier's
    /// grenade a grenade; every other weapon, the Agent's pistol included, a tracer, as is a shot whose shooter the
    /// client no longer knows.
    pub fn of(weapon: Option<ObjectKind>) -> Self {
        weapon
            .and_then(|kind| crate::kinds::of(kind).shot)
            .unwrap_or(ShotStyle::Tracer)
    }
}

/// An object that died in `tick`, where it stood (`SimEvent::Died`).
#[derive(Clone, Debug, PartialEq)]
pub struct DeathMarker {
    pub tick: Tick,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    /// Whom it reaches, and how (`crate::viewer_events`, [`shows_death`]): its owner, and the others while `pos` is on
    /// a Visible tile of theirs, none of those it was hidden from.
    pub seen: Sighting,
}

/// The tracers, death markers, projectiles and blasts on screen, from the sim's events, dropped once they have faded
/// or landed.
#[derive(Resource, Default, Debug)]
pub struct CombatEffects {
    pub tracers: Vec<Tracer>,
    pub deaths: Vec<DeathMarker>,
    /// The projectiles in flight, in launch order (slice M4-O).
    pub flights: Vec<Flight>,
    pub blasts: Vec<Blast>,
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
            .add_systems(
                Update,
                collect_effects.in_set(ClientSystems::SyncSim).after(FilterEvents),
            )
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

/// How `sight` shows the shot of `tracer` now: for [`TRACER_TICKS`] (design change 0177), as its sighting reaches the
/// player (`crate::viewer_events`): the [`visible_spans`] of its path, all of it while the fog doesn't apply; none
/// once it has faded. The same for every shot, a revealed attacker's at the player's own objects included (design
/// change 0293, answering Q0083 point 4: "No exceptions ... every shot, tracer or projectile, whoever fires it and at
/// whatever, is drawn only over your Visible tiles"; it replaced design change 0244's whole path for those).
pub fn shot_spans(sim: &Sim, sight: &Sight, tracer: &Tracer) -> Vec<(f32, f32)> {
    if !shows(sim, tracer.tick, TRACER_TICKS) {
        return Vec::new();
    }
    match tracer.seen.perceived_by(sight) {
        Perception::Whole => vec![(0.0, 1.0)],
        Perception::Clipped { from, to } => visible_spans(sight, from, to),
        Perception::Nothing => Vec::new(),
    }
}

/// Whether `sight` shows `death`, as its sighting reaches the player (`crate::viewer_events`): one of the player's own
/// objects, or on a Visible tile; and, while the fog applies, never one that was hidden from the player as it died
/// (`SimEvent::Died::hiding`: a Hideout that a blast killed unseen, plan decision T52).
pub fn shows_death(sight: &Sight, death: &DeathMarker) -> bool {
    death.seen.perceived_by(sight) == Perception::Whole
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

/// Takes the new shots, deaths, projectiles and blasts from the frame's events, each with its sighting
/// (`ViewerEvents::sightings`), and drops the effects that have faded and the projectiles that have landed. A replaced
/// sim starts from none.
fn collect_effects(sim: Res<SimRes>, events: Res<ViewerEvents>, mut effects: ResMut<CombatEffects>) {
    if effects.generation != Some(sim.generation()) {
        effects.tracers.clear();
        effects.deaths.clear();
        effects.flights.clear();
        effects.blasts.clear();
        effects.generation = Some(sim.generation());
    }
    // An attacker gone by now died in one of these ticks: its type and owner are its death's.
    let fallen = |id: ObjectId| {
        events.sightings().find_map(|(_, event, _)| match *event {
            SimEvent::Died {
                id: dead, kind, owner, ..
            } if dead == id => Some((kind, owner)),
            _ => None,
        })
    };
    // The form a shooter fired in (`sc_sim::conversion::form`), whose weapon it was, and its owner.
    let shooter_of = |id: ObjectId| match sim.object(id) {
        Some(o) => Some((sc_sim::conversion::form(o), o.owner)),
        None => fallen(id),
    };
    for (tick, event, seen) in events.sightings() {
        // A shot: a hit, or a TailDisjointed one's miss (slice M5-O3), drawn alike, to the spot.
        let shot = match *event {
            SimEvent::Attacked {
                attacker,
                target,
                from,
                to,
            } => Some((attacker, Some(target), from, to)),
            SimEvent::Missed { attacker, from, to, .. } => Some((attacker, None, from, to)),
            _ => None,
        };
        if let Some((attacker, target, from, to)) = shot {
            let shooter = shooter_of(attacker);
            effects.tracers.push(Tracer {
                tick,
                attacker,
                weapon: shooter.map(|(form, _)| form),
                owner: shooter.and_then(|(_, owner)| owner),
                target,
                from,
                to,
                seen,
            });
        }
        match *event {
            SimEvent::ProjectileLaunched {
                shooter,
                target,
                from,
                to,
                lands_at,
            } => {
                let thrower = shooter_of(shooter);
                effects.flights.push(Flight {
                    tick,
                    shooter,
                    weapon: thrower.map(|(form, _)| form),
                    owner: thrower.and_then(|(_, owner)| owner),
                    target,
                    from,
                    to,
                    lands_at,
                    seen,
                });
            }
            SimEvent::Blast { at, radius } => effects.blasts.push(Blast { tick, at, radius, seen }),
            SimEvent::Died { kind, owner, pos, .. } => effects.deaths.push(DeathMarker {
                tick,
                kind,
                owner,
                pos,
                seen,
            }),
            _ => {}
        }
    }
    effects.tracers.retain(|t| shows(&sim, t.tick, TRACER_TICKS));
    effects.deaths.retain(|d| shows(&sim, d.tick, DEATH_MARKER_TICKS));
    effects.flights.retain(|f| sim.tick() < f.lands_at);
    effects.blasts.retain(|b| shows(&sim, b.tick, BLAST_TICKS));
}
