//! The Supply Tower (`factions/gdo/economy.md > SupplyTower`, design change 0366, slice M6-GK): a structure, labelled
//! "SPT", the initials of its name with Supply's P ("ST" being the Scout's, "STR" the Subterranean Transport's). The
//! Deployment Center constructs it, its BuildMenu\[Basic\]'s R
//! (`crate::command_ui::interface::templates::gdo::deployment_center`), and its panel trains the Supply Chopper and
//! schedules deliveries (`crate::command_ui::interface::templates::gdo::supply_tower`).

use crate::command_ui::interface::templates::gdo::supply_tower;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SPT",
    look: Look::Structure,
    shot: None,
    template: supply_tower::TEMPLATE,
};
