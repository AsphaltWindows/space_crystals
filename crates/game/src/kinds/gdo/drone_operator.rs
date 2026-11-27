//! The Drone Operator and its drone (`factions/gdo/units/drone_operator.md`, design change 0366, slice M6-GK), a type
//! each. The operator: a LightInfantry's round body, the Peacekeeper's, the one that turns with its unit's facing,
//! with its label on it, "DO", the initials of its name; unarmed ("**no weapon** of its own"). Its drone, named "Drone"
//! as the design names it: the plain unit box, a square, with "DRN" on it, its name's first two letters and its last
//! but one ("DR" being the Colonists' Drone's, "DE" the Disciple's), drawn in flight as every air unit is
//! (`crate::object_render::flight`); armed with its explosion, a melee attack, whose style is a melee weapon's, a
//! tracer, though it fires none: its blast's ring shows where it went off, on the ground, as a grenade's does
//! (`crate::combat_effects::blasts`). Since slice M6-GD the Barracks trains the operator (R), whose panel is the
//! BasicCombatUnit template without its attack commands, with Launch Drone in R and Build Drone in F
//! (`crate::command_ui::interface::templates::gdo::drone_operator`), and the drone's is Move, Attack and Stop
//! (`crate::command_ui::interface::templates::gdo::operator_drone`); a ring round the operator shows the drone's leash
//! (`crate::command_ui::indicators::leash`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::gdo::{drone_operator, operator_drone};
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
    template: drone_operator::TEMPLATE,
};

/// Its drone (`sc_sim::ObjectKind::OperatorDrone`).
pub const DRONE: ClientKind = ClientKind {
    label: "DRN",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: operator_drone::TEMPLATE,
};
