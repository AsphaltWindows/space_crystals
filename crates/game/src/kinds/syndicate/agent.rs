//! The Agent (`factions/syndicate/units/agent.md`): a HeavyInfantry's square with its label (slice M4-A); its
//! pistol's shots are tracers. Its panel is its own (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`,
//! slice M4-J1: `crate::command_ui::interface::templates::syndicate::agent`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::syndicate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "AG",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: syndicate::agent::TEMPLATE,
};
