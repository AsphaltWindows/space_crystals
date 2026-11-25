//! The Robo Portal's panel (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`): the
//! Production template (design change 0124: "Standard production panel, same conventions as GDO's Barracks"), "Q
//! Drone, W Scout, X, C".

use super::super::{InterfaceTemplate, PanelTemplate};
use super::Planned;

/// The Robo Portal's panel template: its "\[W\] Scout", not in the game yet, in its slot (design change 0124).
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(1, Planned::Scout)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
