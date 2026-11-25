//! The Militant (`factions/cults/units/militant.md`, design change 0302): a LightInfantry's round body, the
//! Peacekeeper's, with its label, "MT", on it; its shots are tracers. Its panel is the BasicCombatUnit template (`>
//! ObjectInterfaceState: BasicCombatUnitInterfaceState`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "MT",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
