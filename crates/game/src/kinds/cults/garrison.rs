//! The Garrison (`factions/cults/structures/garrison.md`): a structure, labelled "GA" ("GN", its first and last
//! letters, is the Generator's), as its Construct menus' entry was (slice M5-J1). Its numbers are in the game since
//! slice M5-G2; the Recruits build it, and its panel lets everyone inside out (slice M5-N2,
//! `crate::command_ui::interface::templates::cults::garrison`). Its occupants fire from it, their shots drawn from
//! its footprint's edge (`crate::combat_effects`), and its InfoPanel lists them
//! (`crate::hud::info_panel::activity::holding`).

use crate::command_ui::interface::templates::cults::garrison::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "GA",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
