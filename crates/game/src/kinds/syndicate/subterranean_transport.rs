//! The Subterranean Transport's two types (`factions/syndicate/units/subterranean_transport.md`, design change 0367;
//! slice M6-SK): the Subterranean Transport, above ground, and the Subterranean Transport (Burrowed), underground. Both
//! are a tracked hull ([`UnitShape::Tracked`]), a placeholder (Q0001), with "STR" on the one and "STB" on the other
//! (its "ST" is the Scout's); unarmed. Neither has a panel yet: its own ("Panel: Q Move, X Stop, **R: Burrow /
//! Surface**, **F: Unload All**") comes with slices M6-ST1 and M6-ST2, which have the Vehicle Assembly build it and
//! give it its burrowing (plan decision T84). Until then each is the debug spawn's alone, and the Tunnel's EjectMenu
//! shows the Subterranean Transport's tile, greyed with a count of 0 (the burrowed one never enters).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

/// The Subterranean Transport, above ground.
pub const SURFACED: ClientKind = ClientKind {
    label: "STR",
    look: Look::Unit {
        shape: UnitShape::Tracked,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate::NONE,
};

/// The Subterranean Transport (Burrowed): the surfaced one's, but for its label.
pub const BURROWED: ClientKind = ClientKind {
    label: "STB",
    ..SURFACED
};
