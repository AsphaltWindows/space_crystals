//! The Rocketman (`factions/gdo/units/rocketman.md`, design change 0356, slice M5-K): a LightInfantry's round body, the
//! Peacekeeper's, the one that turns with its unit's facing, with its label on it, "RN", the first and last letters of
//! its one-word name. Its panel is the BasicCombatUnit template, to which the grid adds its R, Target Lock
//! (`crate::command_ui::interface::templates::grid`). Its missiles are drawn as missiles ([`ShotStyle::Missile`]),
//! the dodgeable ones and the homing ones alike (placeholder look, Q0001).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "RN",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Missile),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
