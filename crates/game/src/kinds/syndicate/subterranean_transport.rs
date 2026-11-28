//! The Subterranean Transport's two types (`factions/syndicate/units/subterranean_transport.md`, design change 0367;
//! slices M6-SK and M6-ST1): the Subterranean Transport, above ground, and the Subterranean Transport (Burrowed),
//! underground. Both are a tracked hull ([`UnitShape::Tracked`]), a placeholder (Q0001), with "STR" on the one and
//! "STB" on the other (its "ST" is the Scout's); unarmed. Their panel is the Transport's ("Panel: Q Move, X Stop,
//! **R: Burrow / Surface**, **F: Unload All**", `crate::command_ui::interface::templates::syndicate::transport`), its R
//! held until slice M6-ST2 gives it its burrowing; the Vehicle Assembly's R trains the surfaced one (slice M6-ST1), and
//! the burrowed one is the debug spawn's alone until then. A loaded Transport shows its passengers as pips over its HP
//! bar to its owner (`crate::combat_effects`).

use crate::command_ui::interface::templates::syndicate::transport;
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
    template: transport::SURFACED,
};

/// The Subterranean Transport (Burrowed): the surfaced one's, but for its label and its R.
pub const BURROWED: ClientKind = ClientKind {
    label: "STB",
    template: transport::BURROWED,
    ..SURFACED
};
