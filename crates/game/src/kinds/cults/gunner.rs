//! The Gunner (`factions/cults/units/gunner.md`, design change 0304): a LightInfantry's round body, the Militant's,
//! with its label, "GU", on it ("GR", its first and last letters, is the Grenadier's); its shots are tracers ("Visually
//! still an instant gunshot"), each drawn to the spot its TailDisjointed attack locked, not to its target, so that a
//! miss shows (slice M5-O3: `SimEvent::Missed`, `crate::combat_effects`). Its panel is the BasicCombatUnit template
//! (`> ObjectInterfaceState: BasicCombatUnitInterfaceState`), which offers it Attack Ground, as it does every attack
//! that can miss (`> AttackAttributes`: "CanTargetGround: true (derived)";
//! `crate::command_ui::interface::templates::can_target_ground`).

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
