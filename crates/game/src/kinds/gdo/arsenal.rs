//! The Arsenal (`factions/gdo/structures/arsenal.md`, design change 0356, slice M5-K): a structure, labelled "AL", the
//! first and last letters of its one-word name ("AR" being the Armory's), with no panel ("Panel: none yet (its
//! research is not designed; it only unlocks the Rocketman for now)"). The Deployment Center builds it
//! (BuildMenu\[Basic\]'s S), and it lets the Barracks train Rocketmen.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AL",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
