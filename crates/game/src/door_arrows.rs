//! The door arrows (`core/entities.md > Entrance and exit (production and research buildings)`, design changes 0390
//! and 0394): "while placing a building (AwaitingPlacement), its ghost shows an **arrow at each doorway**: an arrow
//! **pointing in** at an entrance, an arrow **pointing out** at an exit, and a **two-way arrow** at a single door used
//! both ways (e.g. the Cults Garrison, the GDO Bunker). The arrows sit at the doorway's position on its side and **turn
//! and flip with the ghost** [...]. **The same arrows show on a placed building while it is selected** [...]. This
//! applies to every faction's buildings with entrances or exits (including those that release units through an exit
//! side, e.g. the GDO Barracks)." The placement ghost draws them (`crate::placement::ghost`), and so do the selected
//! buildings (`crate::selection::doors`), both from [`door_arrows`].
//!
//! The doorways are the ones the sim lets units in and out by, from each type's defs, wherever its sides face as it
//! is placed (`sc_sim::sides`):
//! - **the entrance** (`HoldsDef::entrance`): in, at the middle of the entrance side, where units walk in
//!   (`sc_sim::holding::entry_path`: of several sides so lettered, the first clockwise from the top);
//! - **the exits** (`sc_sim::production::exit::exit_rule`, a producer's or a holder's): out, at the middle of each exit
//!   side (`ExitRule::Side`, `ExitRule::Sides`: `sc_sim::Doorway`). A door that is both, the Garrison's one door
//!   (`ExitRule::AtOnce` on its entrance side), is two-way;
//! - **a portal's energy doorway** (`ExitRule::Centre`): portals "don't use exit sides" (`core/entities.md > Exit
//!   Doorway`), their units leaving the centre through an open side (`factions/colonists/structures/README.md > Portal
//!   Exits (energy doorway)`): out, at the middle of each open side. **Placeholder (not in the design: where a
//!   portal's arrows stand, its doorway being at the centre)**;
//! - **the Airstrip's runway** (`ExitRule::Runway`, slice M5-N7; `factions/cults/structures/airstrip.md > Placeholder
//!   stats`: "EntranceSide B (Recruits walk in); a finished aircraft is placed **directly on the runway** and takes off
//!   along it toward C"): in at B, its entrance, out at C, the end its aircraft take off toward, which is also the
//!   doorway its Recruits leave by, on the ground (`exit_rule`: the holder's exit). **Placeholder (not in the design:
//!   an aircraft takes off along the whole runway, not through a doorway; the arrow out at C reads its way off the
//!   runway, as the portals' arrows read their open sides)**;
//! - **a Tunnel's Side A** (`factions/syndicate/tunnels.md`: "two paths: one for entering, one for exiting"): in at the
//!   entry path, out at the exit path, where the sim takes units in and lets them out (`sc_sim::network::Paths`);
//! - none for a building whose units stay in the Tunnel Network (`ExitRule::Network`, the Hideout's: "units released
//!   underground (e.g. Syndicate Tunnels) are not covered by this rule", `core/entities.md > Exit Doorway`), nor for a
//!   type whose defs give it no doorway (the Gunworks and the Machine Shop, whose entrance and exit the design gives,
//!   take no units in yet: `factions/cults/structures/gunworks.md`, "Until its research is designed [...] Recruits
//!   can't enter it").
//!
//! The look, a placeholder until the art (Q0001): a flat arrow on the ground just outside the side, along the way
//! units go: its head at the wall for an entrance, away from it for an exit, at both ends for a two-way door; blue in,
//! amber out, white both ways.

use bevy::prelude::*;
use sc_sim::defs::ExitRule;
use sc_sim::network::Paths;
use sc_sim::production::exit::exit_rule;
use sc_sim::sides::DIRECTIONS;
use sc_sim::{Map, ObjectKind, Placement, SU_PER_GU, SideLetter, sides};

use crate::coords::{drawn_at_height, view_to_world};
use crate::shapes::ColorMesh;

/// The arrows' colours: in, out and both ways (placeholders).
pub const IN_COLOR: Color = Color::srgb(0.35, 0.80, 1.0);
pub const OUT_COLOR: Color = Color::srgb(1.0, 0.75, 0.22);
pub const BOTH_COLOR: Color = Color::srgb(0.96, 0.96, 0.96);
/// How far outside its side an arrow begins, clear of the ghost's outline, and how long it is, space units.
const GAP_SU: f32 = 4.0;
const LENGTH_SU: f32 = 40.0;
/// The shaft's width, and the head's length and width, space units.
const SHAFT_SU: f32 = 8.0;
const HEAD_LENGTH_SU: f32 = 16.0;
const HEAD_WIDTH_SU: f32 = 24.0;

/// Which way units go through a doorway.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Way {
    /// An entrance: the arrow points in.
    In,
    /// An exit: the arrow points out.
    Out,
    /// One door used both ways (the Garrison's): a two-way arrow.
    Both,
}

impl Way {
    pub fn color(self) -> Color {
        match self {
            Way::In => IN_COLOR,
            Way::Out => OUT_COLOR,
            Way::Both => BOTH_COLOR,
        }
    }
}

/// The arrow of one doorway of a placed building (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DoorArrow {
    pub way: Way,
    /// The doorway's spot on its side's line, a ground point (space units).
    pub at: Vec2,
    /// The unit vector out of its side, in ground axes (+y south).
    pub outward: Vec2,
}

/// The arrows of the doorways of a `kind` building placed at `placement` (module docs): side by side clockwise from
/// the side facing north, a Tunnel's entry then its exit. None for a type without a doorway.
pub fn door_arrows(kind: ObjectKind, placement: &Placement) -> Vec<DoorArrow> {
    let (Some(structure), Some(letters)) = (kind.def().structure.as_ref(), sides(kind, placement.orientation())) else {
        return Vec::new();
    };
    if structure.tunnel.is_some() {
        let Some(paths) = Paths::of(kind, placement) else {
            return Vec::new();
        };
        let arrow = |way, at| DoorArrow {
            way,
            at,
            outward: paths.outward,
        };
        return vec![arrow(Way::In, paths.entry), arrow(Way::Out, paths.exit)];
    }
    let rule = exit_rule(structure);
    let is_exit = |letter: SideLetter| match rule {
        Some(ExitRule::Side(exit) | ExitRule::Sides(exit) | ExitRule::AtOnce(exit)) => letter == exit,
        // The end its aircraft take off toward (module docs: a placeholder reading), had its exit no doorway.
        Some(ExitRule::Runway { toward, .. }) => letter == toward,
        Some(ExitRule::Centre(open)) => open.contains(&letter),
        Some(ExitRule::Network) | None => false,
    };
    let mut ways: [Option<Way>; 4] = letters.map(|letter| is_exit(letter).then_some(Way::Out));
    if let Some(entrance) = structure
        .holds
        .and_then(|holds| letters.iter().position(|&l| l == holds.entrance))
    {
        ways[entrance] = Some(if ways[entrance].is_some() { Way::Both } else { Way::In });
    }
    let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
    (ways.into_iter().zip(DIRECTIONS))
        .filter_map(|(way, (dx, dy))| {
            let outward = Vec2::new(dx as f32, dy as f32);
            Some(DoorArrow {
                way: way?,
                at: placement.center() + outward * outward.abs().dot(half),
                outward,
            })
        })
        .collect()
}

/// Adds `arrows` to `mesh`, each lying on the ground of `map` as it is drawn at its doorway's height, at local z `z`:
/// Bevy world coordinates (module docs: the look).
pub fn draw_arrows(mesh: &mut ColorMesh, map: &Map, arrows: &[DoorArrow], z: f32) {
    for arrow in arrows {
        let height = map.height_at(arrow.at);
        let drawn = |ground: Vec2| view_to_world(drawn_at_height(ground, height));
        let (n, across) = (arrow.outward, arrow.outward.perp() * (HEAD_WIDTH_SU / 2.0));
        let (near, far) = (arrow.at + n * GAP_SU, arrow.at + n * (GAP_SU + LENGTH_SU));
        let (head_in, head_out) = match arrow.way {
            Way::In => (true, false),
            Way::Out => (false, true),
            Way::Both => (true, true),
        };
        let head = |tip: Vec2, back: Vec2| [tip, back + across, back - across].map(drawn);
        let color = arrow.way.color();
        let mut shaft = (near, far);
        if head_in {
            shaft.0 = near + n * HEAD_LENGTH_SU;
            mesh.polygon(&head(near, shaft.0), z, color);
        }
        if head_out {
            shaft.1 = far - n * HEAD_LENGTH_SU;
            mesh.polygon(&head(far, shaft.1), z, color);
        }
        mesh.segment(drawn(shaft.0), drawn(shaft.1), SHAFT_SU, z, color);
    }
}

#[cfg(test)]
mod tests;
