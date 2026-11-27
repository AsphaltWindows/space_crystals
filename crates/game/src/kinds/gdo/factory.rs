//! The Factory (`factions/gdo/structures/factory.md`, design change 0366, slice M6-GK): a structure, labelled "FY",
//! the first and last letters of its one-word name. (M6-GF) The Deployment Center constructs it, its
//! BuildMenu\[Advanced\]'s Q (`crate::command_ui::interface::templates::gdo::deployment_center`), and it trains the
//! Raptor and the Twister: the Production template, its Prospector and Artillery Rig held in E and R
//! (`crate::command_ui::interface::templates::gdo::factory`).

use crate::command_ui::interface::templates::gdo;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "FY",
    look: Look::Structure,
    shot: None,
    template: gdo::factory::TEMPLATE,
};
