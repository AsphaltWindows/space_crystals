//! The Guard (`factions/syndicate/units/guard.md`): a HeavyInfantry's square, the one unit without a label (the
//! Agent's AG tells their squares apart). Its laser rifle fires a red laser: design change 0207
//! (`factions/syndicate/units/README.md`): "every Syndicate laser-shooting unit and structure (Guard, Laser Battery,
//! Laser Turret) fires a **red** laser". Its panel is the BasicCombatUnit template.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "GD",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: false,
    },
    shot: Some(ShotStyle::RedLaser),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
