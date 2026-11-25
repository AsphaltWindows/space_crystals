//! The Agent (`factions/syndicate/units/agent.md`): a HeavyInfantry's square with its label (slice M4-A); its
//! pistol's shots are tracers. Its panel comes with slice M4-J1 (`factions/syndicate/units/agent.md > Agent
//! ObjectInterfaceState`): none yet.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "AG",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::NONE,
};
