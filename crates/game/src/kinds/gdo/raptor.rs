//! The Raptor (`factions/gdo/units/raptor.md`, design change 0366, slice M6-GK): the plain unit box, its Silhouette's
//! rectangle turned with its facing, as every vehicle's, with its first and last letters, "RR", on it. Its homing
//! missiles are drawn as missiles ([`ShotStyle::Missile`], the Rocketman's), from the barrel every unit turret has
//! (`crate::object_render::turret`). Its panel is the BasicCombatUnit template ("ObjectInterfaceState:
//! BasicCombatUnitInterfaceState"). No player produces one until slice M6-GF, which makes its Factory entry live.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "RR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Missile),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
