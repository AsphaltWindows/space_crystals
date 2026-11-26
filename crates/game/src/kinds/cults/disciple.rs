//! The Disciple (`factions/cults/units/disciple.md`): a LightInfantry's round body, the Militant's, with its first and
//! last letters, "DE", on it; unarmed ("AttackAttributes - None", placeholder), so it fires no shots. The Cathedral
//! trains it since slice M5-N4a, which brings its panel (`> Placeholder stats`: "ObjectInterfaceState:
//! BasicCombatUnitInterfaceState, plus **R: Shroud**"): the BasicCombatUnit template, its attack commands greyed, and
//! Shroud's slot, R, greyed until slice M5-N4b brings the spell
//! (`crate::command_ui::interface::templates::cults::disciple`). Its energy shows in a bar under its HP bar
//! (`crate::combat_effects`) and in the InfoPanel (`crate::hud::info_panel`).

use crate::command_ui::interface::templates::cults::disciple::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "DE",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: None,
    template: TEMPLATE,
};
