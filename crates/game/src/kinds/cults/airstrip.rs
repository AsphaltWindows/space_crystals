//! The Airstrip (`factions/cults/structures/airstrip.md`): a structure, labelled "AP", as its Construct menus' entry
//! was (slice M5-J1). Its numbers and its pool's capacity are in the game since slice M5-G2; since slice M5-N7 the
//! Recruits build it (Advanced D, greyed "Requires a Chop Shop" while their player owns none) and it trains Sparrows
//! from its pool, which take off along its runway: its panel is the pool buildings' ([`TEMPLATE`], design change 0337),
//! its right-click sets the rally point, and its door arrows are in at B and out at C (`crate::door_arrows`). Its
//! look is a structure's with its runway drawn down its length (`Look::Runway`, a placeholder).

use crate::command_ui::interface::templates::cults::airstrip::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AP",
    look: Look::Runway,
    shot: None,
    template: TEMPLATE,
};
