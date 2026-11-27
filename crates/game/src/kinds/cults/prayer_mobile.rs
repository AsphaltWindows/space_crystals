//! The Prayer-mobile (`factions/cults/units/prayer_mobile.md`, design changes 0364 and 0365: "a wheeled vehicle
//! carrying a **megaphone**, with **no weapon**"): its Silhouette's rectangle, turned with its facing as its Silhouette
//! is (slice M5-V1), on four wheels, with a megaphone at its front ([`UnitShape::Megaphone`], placeholder until Q0001)
//! and its initials, "PM", on it; unarmed. Its numbers are in the game since slice M5-G2; since slice M5-N11 the Chop
//! Shop builds it (E, greyed "Requires a Temple" until one stands) and its panel is the BasicCombatUnit template
//! ("ObjectInterfaceState: BasicCombatUnitInterfaceState without attack commands, plus **R: Inspire** and **F:
//! Suppress**"): without its D, Attack Ground (`PanelTemplate::without_attacks`), and alone without its A, as an
//! unarmed unit's (design change 0399), its W Reverse the Seeker's (it CanReverse); its two casts in R and F
//! (`command_ui::interface::templates::grid`), issued at once, each greyed while it lacks the energy
//! (`sc_sim::Reject::NotEnoughEnergy`).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PM",
    look: Look::Unit {
        shape: UnitShape::Megaphone,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate {
        without_attacks: true,
        ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
    },
};
