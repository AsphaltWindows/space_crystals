//! The Specter (`factions/syndicate/units/specter.md`, design change 0367; slice M6-SK): the plain unit box, a square,
//! with "SPR" on it, its first and last letters with its P (its "SR" is the Shrine's), drawn in flight as every air
//! unit is (`crate::object_render::flight`). Its bomb is drawn falling as the grenade flies ([`ShotStyle::Grenade`]:
//! an explosive projectile with its blast where it lands), a placeholder (Q0001) until slice M6-SA's fixed fall. Its
//! panel isn't given: the BasicCombatUnit template, "used by combat units" (`systems/controls/interface.md >
//! BasicCombatUnitInterfaceState`), which gives it Attack Ground, as every attack that can miss. Slice M6-SA has the
//! Aerospace Assembly build it and brings its single payload (plan decision T84): until then it is the debug spawn's
//! alone, and the Tunnel's EjectMenu shows its tile, greyed with a count of 0.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SPR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Grenade),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
