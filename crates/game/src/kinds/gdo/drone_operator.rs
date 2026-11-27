//! The Drone Operator and its drone (`factions/gdo/units/drone_operator.md`, design change 0366, slice M6-GK), a type
//! each. The operator: a LightInfantry's round body, the Peacekeeper's, the one that turns with its unit's facing,
//! with its label on it, "DO", the initials of its name; unarmed ("**no weapon** of its own"). Its drone, named "Drone"
//! as the design names it: the plain unit box, a square, with "DRN" on it, its name's first two letters and its last
//! but one ("DR" being the Colonists' Drone's, "DE" the Disciple's), drawn in flight as every air unit is
//! (`crate::object_render::flight`); unarmed until its explosion. Their panels, **placeholders** until slice M6-GD
//! builds the design's (the operator's Build Drone and Launch Drone, the drone flown directly) and makes the operator's
//! Barracks entry live, are the BasicCombatUnit template without its attack commands (the Prayer-mobile's).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

/// The Drone Operator.
pub const KIND: ClientKind = ClientKind {
    label: "DO",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate {
        without_attacks: true,
        ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
    },
};

/// Its drone (`sc_sim::ObjectKind::OperatorDrone`).
pub const DRONE: ClientKind = ClientKind {
    label: "DRN",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    ..KIND
};
