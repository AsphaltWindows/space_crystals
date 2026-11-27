//! The Vehicle Assembly's panel (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`: "Panel as the
//! Hideout: **Q** Assault Bike, **W** Shadebringer, **E** Laser Battery, **R** Subterranean Transport (greyed until
//! their prerequisites are owned), **X** Cancel Production, **C** Set Rally Point"; slice M6-SV): the Production
//! template (plan decision T23, `super::super::production_grid`), as the Hideout's ([`super::hideout`]); its
//! right-click sets the rally point. Q is its recipe, the Assault Bike's, trained into the Tunnel Network (plan
//! decision T100); W, E and R hold the units it doesn't train yet, each not available ([`Planned`]) until its slice:
//! M6-SB's Shadebringer, M6-SL's Laser Battery and M6-ST1's Subterranean Transport, whose recipes take their slots.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Vehicle Assembly's panel template: the Production template, its recipe in Q, its units not in the game yet in
/// W, E and R.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[
        (1, Planned::Shadebringer),
        (2, Planned::LaserBattery),
        (3, Planned::SubterraneanTransport),
    ],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
