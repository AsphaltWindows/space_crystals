//! The Disciple (`factions/cults/units/disciple.md`): a LightInfantry's round body, the Militant's, with its first and
//! last letters, "DE", on it; unarmed ("AttackAttributes - None", placeholder). Its numbers are in the game since slice
//! M5-G2, and nothing makes one yet: the Cathedral trains it with its slice, which brings its panel too (`>
//! ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **R: Shroud**`), its Shroud with it; until then it has
//! none.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "DE",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate::NONE,
};
