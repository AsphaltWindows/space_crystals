//! The Prospector (`factions/gdo/units/prospector.md`, design change 0366, slice M6-GK): a tracked hull
//! ([`UnitShape::Tracked`], the Hand of Heaven's), with its first and last letters, "PR", on it; unarmed. Its panel,
//! **placeholder** until slice M6-GP builds the design's ("Q Move, X Stop, **R: Deploy into Outpost**") and makes its
//! Factory entry live, is the BasicCombatUnit template without its attack commands (the Prayer-mobile's).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PR",
    look: Look::Unit {
        shape: UnitShape::Tracked,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate {
        without_attacks: true,
        ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
    },
};
