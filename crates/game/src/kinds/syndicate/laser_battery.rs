//! The Laser Battery's two types (`factions/syndicate/units/laser_battery.md`, design change 0367; slice M6-SK): the
//! Laser Battery and the Laser Battery (Deployed) ("**Two unit types**"). Both are a tracked hull
//! ([`UnitShape::Tracked`], "Treaded"), their Silhouette's rectangle turned with their facing, with "LB" on the mobile
//! one and "LBD" on the deployed one; both fire a red laser (design change 0207: "every Syndicate laser-shooting unit
//! and structure (Guard, Laser Battery, Laser Turret) fires a **red** laser"). The mobile one's panel is the
//! BasicCombatUnit template, "used by combat units" (`systems/controls/interface.md > BasicCombatUnitInterfaceState`),
//! with W: Reverse, as it CanReverse. The deployed one has none yet, but the A: Attack of every selection that can
//! attack (design change 0399): that template's movement commands are no use to a unit that "**cannot move**"
//! (`factions/syndicate/units/laser_battery.md > Deploy (ability)`; plan decision T87: a Move it is given ends where it
//! stands), and its own, with Undeploy, comes with slice M6-SL, which also brings the Deploy, the Amplify and the
//! battery, and has the Vehicle Assembly build it (plan decision T84). Until then each is the debug spawn's alone, and
//! the Tunnel's EjectMenu shows the Laser Battery's tile, greyed with a count of 0 (the deployed one never enters).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

/// The Laser Battery.
pub const MOBILE: ClientKind = ClientKind {
    label: "LB",
    look: Look::Unit {
        shape: UnitShape::Tracked,
        tagged: true,
    },
    shot: Some(ShotStyle::RedLaser),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};

/// The Laser Battery (Deployed): the Laser Battery's, but for its label and its panel, none until slice M6-SL.
pub const DEPLOYED: ClientKind = ClientKind {
    label: "LBD",
    template: PanelTemplate::NONE,
    ..MOBILE
};
