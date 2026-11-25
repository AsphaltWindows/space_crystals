//! The Bio Portal's panel (`factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`): the
//! Production template (design change 0124: "Standard production panel, same conventions as GDO's Barracks"), "Q
//! Sabre, W Lancer, E psionic unit, X, C".

use super::super::{InterfaceTemplate, PanelTemplate};
use super::Planned;

/// The Bio Portal's panel template: its "\[W\] Lancer" and "\[E\] Psionic unit (TBD)", not in the game yet, in their
/// slots (design change 0124).
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(1, Planned::Lancer), (2, Planned::PsionicUnit)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
