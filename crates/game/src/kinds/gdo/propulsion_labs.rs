//! Propulsion Labs (`factions/gdo/structures/propulsion_labs.md`, design change 0366, slice M6-GK): a structure,
//! labelled "PL", the initials of its name, with no panel ("Panel: none yet"). (M6-GL) The Deployment Center's
//! BuildMenu\[Advanced\] constructs it, its R, greyed while the player owns no Airfield
//! (`crate::command_ui::interface::templates::gdo::deployment_center`); it held its slot, greyed, from slice M6-GK
//! until slice M6-GL. The Airfield's W, the Dragonfly, is greyed while the player owns none
//! (`crate::command_ui::interface::templates::gdo::airfield`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "PL",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
