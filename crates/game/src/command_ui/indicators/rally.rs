//! The rally points of the selected producers (`systems/controls/hud.md > CommandIndicators >
//! Indicator assignments`, design change 0087): "shown while the producing building is selected, with a line from the
//! building to the rally point so distant or off-screen rally points are easy to follow. A cleared rally point (None)
//! shows nothing."
//!
//! - a ground rally point: a green Location marker;
//! - an own or neutral object: a green Object marker around it;
//! - an enemy: a red Object marker around it. The sim turns it into a ground rally point at the enemy's last-seen
//!   position as soon as an enemy unit leaves the owner's sight (design change 0088), so the marker turns into a
//!   green Location marker there, and never follows an enemy the player doesn't see.
//! - an object the player remembers out of sight (a structure or a resource: design change 0139, "any rally marker
//!   treat[s] the remembered object as present until the player sees otherwise"): the Object marker around its
//!   record, where the record is drawn, wherever the object really is and whether or not it still exists
//!   (`super::targets`, slice M3-K2a), red for an enemy's.
//!
//! The line runs from the edge of the building's footprint to the marker's ring, solid, in the marker's colour
//! (placeholders, like the markers: Q0001).

use bevy::prelude::*;
use sc_sim::{Object, ObjectId, Rally, SU_PER_GU, Sim, TargetView};

use super::{ATTACK_COLOR, Indicator, MOVE_COLOR, marker_ring, own_selected};
use crate::coords::{drawn_position, view_to_world};
use crate::fog::Sight;
use crate::selection::{ControlStates, Relation};
use crate::shapes::ColorMesh;
use crate::tuning::ClientTuning;

/// The line's width, su, and its opacity (placeholders).
const LINE_WIDTH: f32 = 1.5;
const LINE_ALPHA: f32 = 0.7;
/// The line is cut into pieces this long (su), which follow the terrain's height.
const LINE_PIECE: f32 = 16.0;

/// A selected producer's rally point, as the indicators show it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RallyIndicator {
    /// The producer whose rally point it is: the line starts at its footprint.
    pub producer: ObjectId,
    /// The marker at the rally point (module docs).
    pub marker: Indicator,
}

/// The rally points of the selected producers of the player whose `sight` this is, in the producers' id order: one
/// for each that has one. Several producers on one rally point each have theirs, each with its line.
pub fn rally_indicators(sim: &Sim, sight: &Sight, states: &ControlStates) -> Vec<RallyIndicator> {
    own_selected(sim, states, sight.player())
        .filter_map(|producer| {
            let rally = producer.production.as_ref()?.rally?;
            Some(RallyIndicator {
                producer: producer.id,
                marker: rally_marker(sim, sight, rally)?,
            })
        })
        .collect()
}

/// `> Indicator assignments`: the marker of a rally point. An object rally point marks its object as the player knows
/// it (`sight.target_view`): the object while the player sees it, its record while the player remembers it (a
/// structure or a resource: its own and neutral ones stay rally points out of sight, design change 0088, and an
/// enemy's too, design change 0139), coloured by the owner it knows; nothing marks an object hidden from `sight`.
fn rally_marker(sim: &Sim, sight: &Sight, rally: Rally) -> Option<Indicator> {
    match rally {
        Rally::Point(at) => Some(Indicator::Location { at, color: MOVE_COLOR }),
        Rally::Object { id, .. } => {
            let owner = match sight.target_view(sim, id) {
                TargetView::Visible => sim.object(id)?.owner,
                TargetView::Remembered { .. } => sight.remembered_record(id)?.owner,
                TargetView::Gone => return None,
            };
            let color = match Relation::of(sight.player(), owner) {
                Relation::Enemy => ATTACK_COLOR,
                Relation::Own | Relation::Ally | Relation::Neutral => MOVE_COLOR,
            };
            Some(Indicator::Object { id, color })
        }
    }
}

/// Draws the line of `rally` (module docs), as drawn at the render `alpha` with `tuning`: from where it leaves the
/// producer's footprint to the marker's ring, where the player whose `sight` this is knows the rally target
/// ([`marker_ring`]). Nothing when the marker's ring reaches into the footprint.
pub(super) fn draw_line(
    mesh: &mut ColorMesh,
    sim: &Sim,
    sight: &Sight,
    rally: &RallyIndicator,
    (alpha, tuning): (f32, &ClientTuning),
    z: f32,
) {
    let ring = marker_ring(sim, sight, rally.marker, alpha, tuning);
    let (Some(producer), Some(ring)) = (sim.object(rally.producer), ring) else {
        return;
    };
    let (to, radius) = (ring.center, ring.radius);
    let Some(from) = footprint_exit(producer, to) else {
        return;
    };
    let along = to - from;
    let length = along.length() - radius;
    if length <= 0.0 {
        return;
    }
    let to = from + along.normalize() * length;
    let color = rally.marker.color().with_alpha(LINE_ALPHA);
    let pieces = (length / LINE_PIECE).ceil().clamp(1.0, 4096.0) as u32;
    let at = |i: u32| view_to_world(drawn_position(sim.map(), from.lerp(to, i as f32 / pieces as f32)));
    for i in 0..pieces {
        mesh.segment(at(i), at(i + 1), LINE_WIDTH, z, color);
    }
}

/// Where the ray from the centre of `producer`'s footprint towards `to` (ground space) leaves the footprint; none
/// when `to` lies inside it, and for an object without a footprint.
fn footprint_exit(producer: &Object, to: Vec2) -> Option<Vec2> {
    let placement = producer.placement?;
    let center = placement.center();
    let half = Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU / 2.0;
    let toward = to - center;
    // How far along `toward` each pair of sides is; the nearer one is where the ray leaves.
    let scale = |half: f32, d: f32| if d == 0.0 { f32::INFINITY } else { half / d.abs() };
    let t = scale(half.x, toward.x).min(scale(half.y, toward.y));
    (t < 1.0).then(|| center + toward * t)
}

#[cfg(test)]
mod tests {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{ObjectKind, PlayerId, SpawnAt, TilePos};

    use super::*;

    /// The line starts where it leaves the producer's footprint: a Barracks on tiles (8, 4) to (10, 5), x 512..704
    /// and y 256..384.
    #[test]
    fn the_line_leaves_the_footprint_towards_the_rally_point() {
        let scenario = ScenarioBuilder::new(&vec![".".repeat(30); 16].join("\n"))
            .spawn(
                ObjectKind::Barracks,
                Some(PlayerId(1)),
                SpawnAt::Tile(TilePos::new(8, 4)),
            )
            .build();
        let sim = Sim::new(&scenario);
        let barracks = sim.objects().next().unwrap();
        let exit = |x, y| footprint_exit(barracks, Vec2::new(x, y));
        assert_eq!(exit(992.0, 320.0), Some(Vec2::new(704.0, 320.0)), "east");
        assert_eq!(exit(608.0, 64.0), Some(Vec2::new(608.0, 256.0)), "north");
        assert_eq!(exit(416.0, 192.0), Some(Vec2::new(512.0, 256.0)), "the corner");
        assert_eq!(exit(600.0, 300.0), None, "inside");
    }
}
