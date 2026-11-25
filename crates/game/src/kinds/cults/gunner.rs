//! The Gunner (`factions/cults/units/gunner.md`, design change 0304): a LightInfantry's round body, the Militant's,
//! with its label, "GU", on it ("GR", its first and last letters, is the Grenadier's); its shots are tracers ("Visually
//! still an instant gunshot"). Its panel is the BasicCombatUnit template (`> ObjectInterfaceState:
//! BasicCombatUnitInterfaceState`), which offers it Attack Ground, as it does every attack that can miss (`>
//! AttackAttributes`: "CanTargetGround: true (derived)"; `crate::command_ui::interface::templates::can_target_ground`).
//! The sim takes the order, and its TailDisjointed attack lets go of the location until slice M5-O3 builds it.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "GU",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
