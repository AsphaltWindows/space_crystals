//! The Career Campus (`factions/gdo/structures/career_campus.md`, design change 0328, slice M5-N9): a structure,
//! labelled "CC", its initials, with no panel ("Panel: none yet (its ability research is not designed)"). The
//! Deployment Center builds it (BuildMenu\[Basic\]'s D), and it lets the Barracks train Field Engineers.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "CC",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
