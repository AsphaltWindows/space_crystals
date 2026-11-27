//! The Disciple (`factions/cults/units/disciple.md`): a LightInfantry's round body, the Militant's, with its first and
//! last letters, "DE", on it; unarmed ("AttackAttributes - None", placeholder), so it fires no shots. The Cathedral
//! trains it since slice M5-N4a, which brings its panel (`> Placeholder stats`: "ObjectInterfaceState:
//! BasicCombatUnitInterfaceState, plus **R: Shroud** and **F: Mirage**"): the BasicCombatUnit template, its attack
//! commands greyed, Shroud in R, which slice M5-N4b brings, and (M6-CM) Mirage in F
//! (`crate::command_ui::interface::templates::cults::disciple`); its clouds are drawn by `crate::shroud_view`, and its
//! channelling stance too (`crate::shroud_view::stance`). Its energy shows in a bar under its HP bar
//! (`crate::combat_effects`) and in the InfoPanel (`crate::hud::info_panel`).
//!
//! (M6-CM) Its illusions (`sc_sim::mirage`) are Disciples to every enemy: this kind, this look, this name. Their owner
//! alone sees them washed out (`crate::object_render::illusion_color`) and reads "Disciple (illusion)" in the
//! InfoPanel (`factions/cults/units/disciple.md > Spells > Mirage`: "**Seen by the owner**: they **look different on
//! the screen**, and their name reads **"Disciple (illusion)"**"); their energy bar and line show their parent's
//! energy, to everyone (design change 0452).

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
