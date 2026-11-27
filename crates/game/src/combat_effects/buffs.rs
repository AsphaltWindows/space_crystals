//! (M5-N5) The marks of the buffs under way (plan §N5: "a buff marker on affected units";
//! `factions/cults/structures/shrine.md > Abilities`: "qualifying units inside the area when the ability is cast
//! receive the buff for its duration and keep it even if they leave the area"): beside the place of its HP bar, right
//! of it, a mark for each buff a unit has (`sc_sim::Object::buffs`), in its abilities' order, whether its bars show or
//! not. Health regeneration's is a green cross, Energy regeneration's a violet one, the energy bar's
//! (`super::draw`); (M5-N11) the Prayer-mobile's Inspire's a gold chevron pointing up ("+1.5 su/frame MaxSpeed"), its
//! Suppress's a red X ("can't attack"), on every unit Suppress took in, of any player; (M6-SN) the Null Chamber exit
//! buff's a pale blue diamond, an eye's shape for its "+2 grid units SightRange" (`sc_sim::network::services`). Every
//! player who sees the unit sees them, as its bars: **Placeholder (not in the design: who sees a buff, and its look;
//! Q0001)**.

use bevy::prelude::*;
use sc_sim::buffs::BuffKind;
use sc_sim::{AreaAbility, Sim};

use super::hp_bar;
use crate::coords::view_to_world;
use crate::fog::Sight;
use crate::shapes::ColorMesh;
use crate::tuning::ClientTuning;

/// A mark's cross: its arms' half length and half width, its dark rim, and the gap between the bar's place and the
/// first mark, and between two marks, su (placeholders).
const ARM: f32 = 4.5;
const THICK: f32 = 1.6;
const RIM: f32 = 1.2;
const GAP: f32 = 4.0;
/// Health regeneration's colour: a green brighter than the HP bar's.
const HEALTH_COLOR: Color = Color::srgb(0.45, 1.0, 0.50);
/// Energy regeneration's: the energy bar's violet.
const ENERGY_COLOR: Color = Color::srgb(0.72, 0.45, 1.0);
/// (M5-N11) Inspire's gold and Suppress's red (placeholders).
const INSPIRE_COLOR: Color = Color::srgb(1.0, 0.80, 0.25);
const SUPPRESS_COLOR: Color = Color::srgb(1.0, 0.28, 0.22);
/// (M6-SN) The Null Chamber exit buff's pale blue (a placeholder).
const NULL_CHAMBER_COLOR: Color = Color::srgb(0.45, 0.85, 1.0);
const RIM_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

/// A buff's mark: centred on `center` (view space), of the buff `ability` (`sc_sim::buffs::Buff::ability`): an
/// ability's, or (M6-SN) the Null Chamber's.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BuffMark {
    pub center: Vec2,
    pub ability: BuffKind,
}

/// The marks to draw: those of the buffs of each object `sight` sees, at its interpolated position (module docs), its
/// bar's place being where [`hp_bar`] puts it, as `tuning` sizes the bars.
pub fn buff_marks(sim: &Sim, sight: &Sight, alpha: f32, tuning: &ClientTuning) -> Vec<BuffMark> {
    let mut marks = Vec::new();
    for o in sim.objects().filter(|o| !o.buffs.is_empty() && sight.sees(o)) {
        let bar = hp_bar(sim.map(), o, alpha, (1.0, (None, None)), &tuning.look);
        let first = bar.center + Vec2::new(bar.width / 2.0 + GAP + ARM, 0.0);
        // A unit's buffs are in their kinds' order, one of each (`sc_sim::buffs::Buff`).
        for (i, buff) in o.buffs.iter().enumerate() {
            let center = first + Vec2::new(i as f32 * (2.0 * ARM + GAP), 0.0);
            marks.push(BuffMark {
                center,
                ability: buff.ability,
            });
        }
    }
    marks
}

/// Draws `mark` in its buff's colour inside a dark rim: a cross, (M5-N11) Inspire's chevron and Suppress's X, (M6-SN)
/// the Null Chamber's diamond.
pub(super) fn mark_shape(mesh: &mut ColorMesh, mark: BuffMark, z: f32) {
    let center = view_to_world(mark.center);
    let ability = match mark.ability {
        BuffKind::Area(ability) => ability,
        BuffKind::NullChamber => {
            let corners = [
                Vec2::new(0.0, ARM),
                Vec2::new(ARM, 0.0),
                Vec2::new(0.0, -ARM),
                Vec2::new(-ARM, 0.0),
            ]
            .map(|corner| center + corner);
            let sides = |from: usize| {
                [
                    (corners[from], corners[from + 1]),
                    (corners[from + 1], corners[(from + 2) % 4]),
                ]
            };
            strokes(mesh, sides(0), NULL_CHAMBER_COLOR, z);
            strokes(mesh, sides(2), NULL_CHAMBER_COLOR, z);
            return;
        }
    };
    match ability {
        // (M6-GR) Scan gives no buff, so no mark: its sight is its owner's (`sc_sim::vision::scans`).
        AreaAbility::Scan => {}
        AreaAbility::HealthRegeneration => cross(mesh, center, HEALTH_COLOR, z),
        AreaAbility::EnergyRegeneration => cross(mesh, center, ENERGY_COLOR, z),
        AreaAbility::Inspire => {
            // A chevron pointing up the screen (Bevy world space: +y): two arms from its tip.
            let tip = center + Vec2::new(0.0, ARM * 0.7);
            let arms = [Vec2::new(-ARM, -ARM * 0.5), Vec2::new(ARM, -ARM * 0.5)];
            strokes(mesh, arms.map(|arm| (tip, center + arm)), INSPIRE_COLOR, z);
        }
        AreaAbility::Suppress => {
            let corners = [
                Vec2::new(-ARM, -ARM),
                Vec2::new(ARM, ARM),
                Vec2::new(-ARM, ARM),
                Vec2::new(ARM, -ARM),
            ];
            let strokes_of = [(corners[0], corners[1]), (corners[2], corners[3])];
            strokes(
                mesh,
                strokes_of.map(|(a, b)| (center + a, center + b)),
                SUPPRESS_COLOR,
                z,
            );
        }
    }
}

/// A cross round `center` (Bevy world space) in `color` inside a dark rim.
fn cross(mesh: &mut ColorMesh, center: Vec2, color: Color, z: f32) {
    for (grow, color) in [(RIM, RIM_COLOR), (0.0, color)] {
        for half in [Vec2::new(ARM, THICK), Vec2::new(THICK, ARM)] {
            let half = half + Vec2::splat(grow);
            mesh.rect(center - half, center + half, z, color);
        }
    }
}

/// The strokes `lines` (Bevy world space), each as thick as a cross's arm, in `color` inside a dark rim.
fn strokes(mesh: &mut ColorMesh, lines: [(Vec2, Vec2); 2], color: Color, z: f32) {
    for (grow, color) in [(RIM, RIM_COLOR), (0.0, color)] {
        for (a, b) in lines {
            let along = (b - a).normalize_or_zero() * grow;
            mesh.segment(a - along, b + along, 2.0 * (THICK + grow), z, color);
        }
    }
}
