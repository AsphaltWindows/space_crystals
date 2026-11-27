//! The GDO's mine (`factions/gdo/units/twister.md > Mines`, design change 0366, slice M6-GK): "closer to a permanently
//! deployed unit" (design change 0426), so drawn as a unit's body, small and untagged: an octagon of its 16 su
//! Silhouette, its label, "ME", the first and last letters of its one-word name, on its portrait only. **Placeholder
//! (Q0001)**: its look and its hiding ("hidden underground") are slice M6-GM's, which has a Twister lay it; it has no
//! panel ("no commands in its panel yet").

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "ME",
    look: Look::Unit {
        shape: UnitShape::Octagon,
        tagged: false,
    },
    shot: None,
    template: PanelTemplate::NONE,
};
