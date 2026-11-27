//! The Supply Tower (`factions/gdo/economy.md > SupplyTower`, design change 0366, slice M6-GK): a structure, labelled
//! "SPT", the initials of its name with Supply's P ("ST" being the Scout's, "STR" the Subterranean Transport's), with
//! no panel yet. No player constructs one until slice M6-GS, which makes its Deployment Center entry live, its
//! BuildMenu\[Basic\]'s R greyed until then (`crate::command_ui::interface::templates::gdo::deployment_center`), and its
//! Supply Chopper with it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SPT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
