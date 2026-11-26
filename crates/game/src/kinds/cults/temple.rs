//! The Temple (`factions/cults/structures/temple.md`): a structure, labelled with its first and last letters, "TE", as
//! its Construct menus' entry was (slice M5-J1). Its numbers are in the game since slice M5-G2. Since slice M5-N3 the
//! Recruits build it (Advanced X, greyed "Requires an Altar" while their player owns none), Recruits right-clicked into
//! it staff its research, and its panel researches each line's next level, lets one Recruit of a kind out, or all of
//! them, and cancels the last research (`crate::command_ui::interface::templates::cults::temple`); its InfoPanel shows
//! its staff and its research (`crate::hud::info_panel::activity`).

use crate::command_ui::interface::templates::cults::temple::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "TE",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
