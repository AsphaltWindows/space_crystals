//! The Airfield (`factions/gdo/structures/airfield.md`, design change 0366, slice M6-GK): a structure, labelled "AD",
//! the first and last letters of its one-word name. Since slice M6-GH1 it trains the Hornet, its panel the Production
//! template with the Dragonfly held in W (`crate::command_ui::interface::templates::gdo::airfield`). No player
//! constructs one until slice M6-GH2, which lands its aircraft and makes its Deployment Center entry live, its
//! BuildMenu\[Advanced\]'s E greyed until then (`crate::command_ui::interface::templates::gdo::deployment_center`):
//! only a debug spawn places one.

use crate::command_ui::interface::templates::gdo;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AD",
    look: Look::Structure,
    shot: None,
    template: gdo::airfield::TEMPLATE,
};
