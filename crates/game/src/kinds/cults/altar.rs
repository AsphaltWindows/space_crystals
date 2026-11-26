//! The Altar (`factions/cults/augmentation.md > Altar`): a structure, labelled with its first and last letters, "AR",
//! as its Construct menus' entry was (slice M5-J1). Its panel (`> ObjectInterfaceState[Altar]`, slice M5-N1b,
//! `crate::command_ui::interface::templates::cults::altar`): a unit type, then its state, then the orders it may take
//! or Release, and Eject All, Cancel and Set Rally Point; its right-click sets the rally point.

use crate::command_ui::interface::templates::cults::altar::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AR",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
