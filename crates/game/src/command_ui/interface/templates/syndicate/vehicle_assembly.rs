//! The Vehicle Assembly's panel (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`: "Panel as the
//! Hideout: **Q** Assault Bike, **W** Shadebringer, **E** Laser Battery, **R** Subterranean Transport (greyed until
//! their prerequisites are owned), **X** Cancel Production, **C** Set Rally Point"; slice M6-SV): the Production
//! template (plan decision T23, `super::super::production_grid`), as the Hideout's ([`super::hideout`]); its
//! right-click sets the rally point. Q, W and R are its recipes, the Assault Bike's, (M6-SB) the Shadebringer's and
//! (M6-ST1) the Subterranean Transport's, trained into the Tunnel Network (plan decision T100), W greyed while the
//! player owns no Vault (`sc_sim::Reject::Requires`, "Requires a Vault"), and R while it owns no Forge
//! (`factions/syndicate/units/subterranean_transport.md > TechPrerequisite`: "player owns at least one **Forge**",
//! "Requires a Forge"); E holds the unit it doesn't train yet, not available ([`Planned`]) until its slice: M6-SL's
//! Laser Battery, whose recipe takes its slot.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Vehicle Assembly's panel template: the Production template, its recipes in Q, W and R (the Transport's Train in
/// R, past E), its unit not in the game yet in E.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(2, Planned::LaserBattery)],
    trains: &[(3, ObjectKind::SubterraneanTransport)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
