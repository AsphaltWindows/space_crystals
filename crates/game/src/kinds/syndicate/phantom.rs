//! The Phantom (`factions/syndicate/units/phantom.md`, design change 0367; slice M6-SK): the plain unit box, a square,
//! with "PHM" on it, its first and last letters with its H (its "PM" is the Prayer-mobile's), drawn in flight as every
//! air unit is (`crate::object_render::flight`). Its laser is red (design change 0207). Its panel is the
//! BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"). The Aerospace Assembly trains it
//! into the Tunnel Network (slice M6-SD), and a Tier 3 Tunnel lets it out by its air exit door and in by its air entry
//! door (plan decision T96), its EjectMenu tile F greyed for the tier at Tiers 1 and 2; it rises out of the one and
//! descends into the other over 16 frames, drawn rising and sinking (`crate::object_render::flight`, design change
//! 0431).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PHM",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::RedLaser),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
