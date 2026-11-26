//! The Shrine (`factions/cults/structures/shrine.md`): a structure, labelled "SR" ("SE", its first and last letters, is
//! the Storage's), as its Construct menus' entry was (slice M5-J1). Its numbers are in the game since slice M5-G2.
//! Since slice M5-N5 the Recruits build it (Advanced V, greyed "Requires a Temple" while their player owns none),
//! Recruits right-clicked into it take part in its ritual and charge its energy, and its panel casts its two area
//! abilities at a point in vision and lets one participant of a kind out, or all of them
//! (`crate::command_ui::interface::templates::cults::shrine`); its InfoPanel shows its energy and its participants
//! (`crate::hud::info_panel`), and the cursor that awaits a cast's point the ability's circle
//! (`crate::command_ui::indicators`).

use crate::command_ui::interface::templates::cults::shrine::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SR",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
