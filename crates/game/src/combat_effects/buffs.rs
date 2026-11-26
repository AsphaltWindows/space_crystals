//! (M5-N5) The marks of the buffs under way (plan §N5: "a buff marker on affected units";
//! `factions/cults/structures/shrine.md > Abilities`: "qualifying units inside the area when the ability is cast
//! receive the buff for its duration and keep it even if they leave the area"): beside the place of its HP bar, right
//! of it, a mark for each buff a unit has (`sc_sim::Object::buffs`), in its abilities' order, whether its bars show or
//! not. Health regeneration's is a green cross, Energy regeneration's a violet one, the energy bar's
//! (`super::draw`). Every player who sees the unit sees them, as its bars: **Placeholder (not in the design: who sees a
//! buff, and its look; Q0001)**.

use bevy::prelude::*;
use sc_sim::{AreaAbility, Sim};

use super::hp_bar;
use crate::coords::view_to_world;
use crate::fog::Sight;
use crate::shapes::ColorMesh;

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
const RIM_COLOR: Color = Color::srgb(0.05, 0.05, 0.05);

/// A buff's mark: centred on `center` (view space), of `ability`'s buff.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BuffMark {
    pub center: Vec2,
    pub ability: AreaAbility,
}

/// The marks to draw: those of the buffs of each object `sight` sees, at its interpolated position (module docs), its
/// bar's place being where [`hp_bar`] puts it.
pub fn buff_marks(sim: &Sim, sight: &Sight, alpha: f32) -> Vec<BuffMark> {
    let mut marks = Vec::new();
    for o in sim.objects().filter(|o| !o.buffs.is_empty() && sight.sees(o)) {
        let bar = hp_bar(sim.map(), o, alpha, 1.0, (None, None));
        let first = bar.center + Vec2::new(bar.width / 2.0 + GAP + ARM, 0.0);
        let shown = AreaAbility::ALL
            .into_iter()
            .filter(|&ability| o.buffs.iter().any(|buff| buff.ability == ability));
        for (i, ability) in shown.enumerate() {
            let center = first + Vec2::new(i as f32 * (2.0 * ARM + GAP), 0.0);
            marks.push(BuffMark { center, ability });
        }
    }
    marks
}

/// Draws `mark`: a cross in its ability's colour inside a dark rim.
pub(super) fn mark_shape(mesh: &mut ColorMesh, mark: BuffMark, z: f32) {
    let center = view_to_world(mark.center);
    let color = match mark.ability {
        AreaAbility::HealthRegeneration => HEALTH_COLOR,
        AreaAbility::EnergyRegeneration => ENERGY_COLOR,
    };
    for (grow, color) in [(RIM, RIM_COLOR), (0.0, color)] {
        for half in [Vec2::new(ARM, THICK), Vec2::new(THICK, ARM)] {
            let half = half + Vec2::splat(grow);
            mesh.rect(center - half, center + half, z, color);
        }
    }
}
