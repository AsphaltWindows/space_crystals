//! The Airfield (`factions/gdo/structures/airfield.md`, design change 0366, slice M6-GK): a structure, labelled "AD",
//! the first and last letters of its one-word name. Since slice M6-GH1 it trains the Hornet, its panel the Production
//! template with the Dragonfly held in W (`crate::command_ui::interface::templates::gdo::airfield`). Since slice
//! M6-GH2 its aircraft land on its 4 landing slots, each drawn as a pad on its roof (`Look::Pads`), and the Deployment
//! Center constructs it, its BuildMenu\[Advanced\]'s E, greyed while the player owns no Advanced Mechanics
//! (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates::gdo;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AD",
    look: Look::Pads,
    shot: None,
    template: gdo::airfield::TEMPLATE,
};
