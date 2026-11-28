//! A structure that attacks (plan decision T49: the Laser Turret, slice M4-S1), its placeholder look until the art
//! direction (Q0001) settles it; the design gives none. A structure's footprint (`super::structures`) with a round
//! head on its roof ([`draw_head`]) and a barrel ([`TurretBarrel`]) that turns with the turret's facing, which the sim
//! turns at once to face what it fires at (plan decision T49: "A structure has no turn rate (it fires in any
//! direction)"; `sc_sim::combat`), interpolated between ticks as a unit's facing tick is ([`place_barrels`]). A site
//! has neither (`super::site`); a record of a turret seen raised has its head and no barrel (`super::remembered`:
//! where the barrel pointed isn't remembered).
//!
//! **Always a pop-up** (slices M4-S2 and M5-Y, plan decision T50; `factions/syndicate/structures/laser_turret.md >
//! Pop-up turret`, design change 0360: "There is no upright form any more"; `sc_sim::popup`): fully lowered, the
//! turret is drawn as an underground structure's outline, as an expansion (`super::MeshKey::lowered`), with neither
//! head nor barrel: its owner sees the ground and the units over it, and the fog's `Sight` shows it to nobody else
//! (`sc_sim::vision::hidden`). From its first rising frame it is the raised turret's body, its barrel already facing
//! its target and coming out as it rises, and going back in as it retracts ([`place_barrels`]). A neutral one, which
//! its lost Tunnel left (`factions/syndicate/structures/laser_turret.md > When its Tunnel is lost`), is down for good
//! and seen by everyone: its outline in the neutral colour, and so is a record of one
//! (`sc_sim::popup::remembered_lowered`). Placeholders all.
//!
//! **A unit's turret** (slice M5-U1; `sc_sim::combat::turret_facing`): a unit whose type has TurretAttributes carries
//! the same barrel on its body, with no head, turned to its turret's facing, which the sim turns at its own rate on
//! the base's. The sim keeps the base's facing of the tick before (`Object::prev_facing`) and not the turret's, so the
//! barrel follows the turret's itself between ticks ([`TurretTrack`]); on a body that turns with the unit (a round
//! one, `super::body_turns`) it turns by the turret's facing less the body's.

use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind, Tick};

use super::structures::LOOK_Z;
use crate::coords::{facing_to_rotation, lerp_facing};
use crate::shapes::{ColorMesh, darken};
use crate::sim_driver::{RenderAlpha, SimRes};

/// The barrel of a turret's visual: a child of the visual, turned by the object's facing, or (M5-U1) a unit's turret's.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct TurretBarrel(pub ObjectId);

/// (M5-U1) The facing of a unit's turret in the last two ticks its barrel saw ([`place_barrels`]), radians, as on the
/// map (`sc_sim::combat::turret_facing`), for the barrel's interpolation between them.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct TurretTrack {
    /// The tick of `to`.
    tick: Tick,
    /// The facing in the tick before `tick`.
    from: f32,
    to: f32,
}

impl TurretTrack {
    /// A turret that faces `facing` in tick `tick`.
    pub fn new(tick: Tick, facing: f32) -> Self {
        Self {
            tick,
            from: facing,
            to: facing,
        }
    }

    /// The turret faces `facing` in tick `tick`: the track goes on from where it was in the tick before, or starts
    /// there if the barrel didn't see the tick before (the sim ran several ticks in a frame, or a replay jumped).
    pub fn observe(&mut self, tick: Tick, facing: f32) {
        if tick == self.tick {
            return;
        }
        self.from = if tick == self.tick + 1 { self.to } else { facing };
        self.to = facing;
        self.tick = tick;
    }

    /// The facing `alpha` of the way from the tick before to this one (`crate::sim_driver::RenderAlpha`), the shorter
    /// way round.
    pub fn at(&self, alpha: f32) -> f32 {
        lerp_facing(self.from, self.to, alpha)
    }
}

/// (M5-U1) Whether a `kind` unit carries a turret's barrel: its type has TurretAttributes (`sc_sim::defs::UnitDef`).
pub(super) fn on_unit(kind: ObjectKind) -> bool {
    kind.def().unit.as_ref().is_some_and(|unit| unit.turret.is_some())
}

/// The head's radius, space units: well inside a 1x1 footprint's fill (19 su from its centre).
const HEAD_SU: f32 = 13.0;
/// The width of the head's dark rim.
const HEAD_RIM_SU: f32 = 3.0;
/// The barrel along local +x, space units: from inside the head to a little past the footprint's edge (32 su).
const BARREL_FROM_SU: f32 = 6.0;
const BARREL_TO_SU: f32 = 36.0;
const BARREL_WIDTH_SU: f32 = 8.0;
/// How much of its length the barrel of a pop-up turret has as it starts to rise (a placeholder): it grows from there
/// to its whole length over the rise, and back over the retraction ([`place_barrels`]).
const RISING_FROM: f32 = 0.25;
/// The barrel's dark casing and its light core (placeholder colours).
const BARREL_CASING: Color = Color::srgb(0.10, 0.11, 0.14);
const BARREL_CORE: Color = Color::srgb(0.78, 0.80, 0.84);

/// A turret's head on its roof, in shades of `color`, its owner's: a dark disc with a darker rim, under the tag
/// (`super::structures::tag`), which stays legible on it.
pub(super) fn draw_head(mesh: &mut ColorMesh, color: Color) {
    let srgba = color.to_srgba();
    let disc = |radius: f32| -> Vec<Vec2> {
        (0..20)
            .map(|i| Vec2::from_angle(i as f32 * std::f32::consts::TAU / 20.0) * radius)
            .collect()
    };
    mesh.polygon(&disc(HEAD_SU), LOOK_Z, darken(srgba, 0.25).into());
    mesh.polygon(&disc(HEAD_SU - HEAD_RIM_SU), LOOK_Z + 0.0005, darken(srgba, 0.5).into());
}

/// The barrel along local +x, which its visual's rotation turns to the turret's facing: a light core in a dark casing,
/// over the head.
fn draw_barrel(mesh: &mut ColorMesh) {
    let (from, to) = (Vec2::new(BARREL_FROM_SU, 0.0), Vec2::new(BARREL_TO_SU, 0.0));
    mesh.segment(from, to, BARREL_WIDTH_SU, LOOK_Z + 0.001, BARREL_CASING);
    let core_to = to - Vec2::new(2.0, 0.0);
    mesh.segment(from, core_to, BARREL_WIDTH_SU - 4.0, LOOK_Z + 0.0015, BARREL_CORE);
}

/// The barrel alone, for its own entity ([`TurretBarrel`]).
pub(super) fn barrel_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    draw_barrel(&mut mesh);
    mesh.into_mesh()
}

/// A turret's head and its barrel pointing east (local +x), on a structure's mesh: a turret's placeholder shape, where
/// no object's facing turns it (`super::object_mesh`: the map editor's markers).
pub(super) fn draw_fixed(mesh: &mut ColorMesh, color: Color) {
    draw_head(mesh, color);
    draw_barrel(mesh);
}

/// Turns each barrel to its turret's facing, interpolated between the last tick's and this one's
/// (`docs/architecture/client-architecture-visuals.md > Client architecture > Visuals > Interpolation`), as `super::place_object_visuals` does a
/// unit's facing tick: a structure's facing, or (M5-U1) a unit's turret's ([`TurretTrack`]), less its body's turn.
pub(super) fn place_barrels(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut barrels: Query<(&TurretBarrel, Option<&mut TurretTrack>, &mut Transform)>,
) {
    for (barrel, track, mut transform) in &mut barrels {
        let Some(object) = sim.object(barrel.0) else {
            continue;
        };
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        let body = if super::body_turns(super::drawn_kind(object)) {
            facing
        } else {
            0.0
        };
        let aim = match (track, sc_sim::combat::turret_facing(object)) {
            (Some(mut track), Some(now)) => {
                track.observe(sim.tick(), now);
                track.at(alpha.0)
            }
            _ => facing,
        };
        let facing = aim - body;
        // A pop-up on its way up or down (slice M4-S2): the barrel as long as the turret is far up.
        let out = object
            .popup
            .map_or(1.0, |popup| RISING_FROM + (1.0 - RISING_FROM) * popup.height());
        transform.set_if_neq(Transform::from_rotation(facing_to_rotation(facing)).with_scale(Vec3::new(out, 1.0, 1.0)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit's turret's barrel goes from the tick before's facing to this one's over a tick, the shorter way round
    /// (across the back here), stays put while the tick does, and starts afresh after a tick it didn't see.
    #[test]
    fn a_turret_s_track_follows_it_from_tick_to_tick() {
        let mut track = TurretTrack::new(5, 0.0);
        assert_eq!(track.at(0.5), 0.0);
        track.observe(6, 1.0);
        assert!((track.at(0.25) - 0.25).abs() < 1e-6 && track.at(1.0) == 1.0);
        track.observe(6, 2.0);
        assert_eq!(track.at(0.0), 0.0, "the same tick: no change");
        track.observe(8, 2.0);
        assert_eq!((track.at(0.0), track.at(1.0)), (2.0, 2.0), "a tick missed: from there");
        let pi = std::f32::consts::PI;
        let mut track = TurretTrack::new(1, pi - 0.1);
        track.observe(2, -pi + 0.1);
        assert!(
            (sc_sim::math::wrap_angle(track.at(0.5)) - pi).abs() < 1e-5,
            "across the back"
        );
    }
}
