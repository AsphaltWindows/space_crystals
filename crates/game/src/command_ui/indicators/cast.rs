//! (M5-N5) The circle of a cast that awaits its point (plan §N5: "the AwaitingTarget cursor, with a 4 GU ring";
//! `factions/cults/structures/shrine.md > Placeholder stats`: "circle of radius 4 grid units"): while a Shrine's Q or W
//! awaits its point, a ring on the ground around the point under the pointer, of the ability's radius, following the
//! terrain's height, with a faint fill, so that the player sees which units the cast would reach ("a unit is affected
//! when its **centre** is inside"). In the Cast pointer's pink over a point the player sees now
//! (`crate::command_ui::cursor::CAST_COLOR`; `sc_sim::Sim::sees_point`), else in a muted grey: a click there would be
//! an invalid target. (M6-GR) A Radar Station's Scan shows the circle it reveals, "a circle of radius **6 grid
//! units**" (`factions/gdo/structures/radar_station.md > Placeholder stats`), pink anywhere on the map, seen or not
//! ("left-click any point on the map"), grey off it ([`takes_point`]). Placeholder look (Q0001).

use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::{AbilityArea, AreaAbility, Map, PlayerId, SU_PER_GU, Sim};

use crate::command_ui::cursor::CAST_COLOR;
use crate::command_ui::interface::{InterfaceState, PanelCommand};
use crate::coords::{drawn_position, view_to_world};
use crate::shapes::ColorMesh;

/// The ring's line width, su, its segments, and the opacity of the fill inside it (placeholders).
const WIDTH: f32 = 3.0;
const SEGMENTS: usize = 64;
const FILL_ALPHA: f32 = 0.12;
/// The ring's colour over a point the player doesn't see now: the Inactive pointer's grey, darker.
const OUT_OF_SIGHT_COLOR: Color = Color::srgb(0.62, 0.64, 0.68);

/// The radius of the circle a cast of `ability` reaches, su: its "Area" (`sc_sim::AbilityDef::radius`, grid units).
pub fn radius(ability: AreaAbility) -> f32 {
    ability.def().radius * SU_PER_GU
}

/// Whether a click at the ground point `at` casts `ability` for `player`, as the gate judges its point
/// (`crate::command_ui::resolution::target_resolution`): a Shrine's cast a point the player sees now
/// (`Sim::sees_point`), (M6-GR) a Scan any point on the map (`AbilityArea::AnyPoint`).
pub fn takes_point(sim: &Sim, player: PlayerId, ability: AreaAbility, at: Vec2) -> bool {
    match ability.def().area {
        AbilityArea::AnyPoint => sim.map().contains_point(at),
        _ => sim.sees_point(player, at),
    }
}

/// The ability whose point `interface` awaits: a Shrine's Q or W (`PanelCommand::Cast`), (M6-GR) a Radar Station's
/// Q; none for anything else.
pub fn awaited(interface: InterfaceState) -> Option<AreaAbility> {
    match interface.pending()? {
        PanelCommand::Cast(ability) => Some(ability),
        _ => None,
    }
}

/// Draws the circle of a cast of `ability` around the ground point `at` (module docs), at `z`: a faint fill and a
/// ring, each point of it drawn at the terrain's height there; in the Cast pink while a click there casts it
/// (`valid`, [`takes_point`]), else grey.
pub(super) fn draw(mesh: &mut ColorMesh, map: &Map, ability: AreaAbility, at: Vec2, valid: bool, z: f32) {
    let color = if valid { CAST_COLOR } else { OUT_OF_SIGHT_COLOR };
    let r = radius(ability);
    let edge = |i: usize| {
        let a = i as f32 * TAU / SEGMENTS as f32;
        view_to_world(drawn_position(map, at + Vec2::new(a.cos(), a.sin()) * r))
    };
    let center = view_to_world(drawn_position(map, at)).extend(z);
    let fill = color.with_alpha(FILL_ALPHA);
    for i in 0..SEGMENTS {
        mesh.triangle([center, edge(i).extend(z), edge(i + 1).extend(z)], fill);
    }
    for i in 0..SEGMENTS {
        mesh.segment(edge(i), edge(i + 1), WIDTH, z, color);
    }
}
