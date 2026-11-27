//! The Chop Shop (`factions/cults/structures/chop_shop.md`): a structure, labelled with its initials, "CS", as its
//! Construct menus' entry was (slice M5-J1). Its numbers and its pool's capacity are in the game since slice M5-G2;
//! since slice M5-N6 the Recruits build it (Advanced A, greyed "Requires an Armory" while their player owns none) and
//! it trains Seekers of two Recruits from its pool, which drive out of side C: its panel is the pool buildings'
//! ([`TEMPLATE`], design change 0337), its right-click sets the rally point, and its door arrows are in at B and out at
//! C (`crate::door_arrows`).

use crate::command_ui::interface::templates::cults::chop_shop::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "CS",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
