//! The Phantom (`factions/syndicate/units/phantom.md`, design change 0367; slice M6-SK): the plain unit box, a square,
//! with "PHM" on it, its first and last letters with its H (its "PM" is the Prayer-mobile's), drawn in flight as every
//! air unit is (`crate::object_render::flight`). Its laser is red (design change 0207). Its panel is the
//! BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"). Slice M6-SD has the Aerospace
//! Assembly build it and brings the air doors (plan decision T84): until then it is the debug spawn's alone, and the
//! Tunnel's EjectMenu shows its tile, greyed with a count of 0.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PHM",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::RedLaser),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
