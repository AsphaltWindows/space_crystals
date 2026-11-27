//! The GDO's mine — `factions/gdo/units/twister.md > Mines` (design change 0366, slice M6-GK): what a Twister lays.
//! "**What a mine is** (user, 2026-11-26; Q0160): **closer to a permanently deployed unit** than to a structure. It is
//! **not a structure** [...] and it **can be selected** by its owner, like a unit" (design change 0426): an object of
//! a category of its own ([`Category::Mine`], plan decision T90), with a Silhouette and neither unit nor structure
//! attributes, so that it neither moves nor collides, nor counts as a building. Its numbers are the design's
//! "Placeholder stats". Nothing lays one until slice M6-GM, whose are its trigger and its blast ("activation radius
//! **32 su (0.5 GU)**; delay **8 frames**; explosion Damage **200** total, AoERadius **64 su**"), its hiding and its
//! Unit Control of 0.

use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/units/twister.md > Mines`: its placeholder stats (design change 0366).
pub(crate) const MINE: ObjectDef = ObjectDef {
    kind: ObjectKind::Mine,
    name: "Mine",
    faction: Some(Faction::Gdo), // the GDO's Twister lays it
    category: Category::Mine,
    footprint: Footprint::Silhouette { w: 16.0, h: 16.0 }, // "mine Size: a 16x16 su object, hidden underground"
    selectable: true, // "it **can be selected** by its owner, like a unit" (design change 0426)
    // "**Destroyable:** enemies can attack and kill mines"; "**Destroyed by damage, it doesn't explode**".
    destructible: true,
    groupable: true, // "Designer's placeholders: Unit Control 0, Groupable true, no commands in its panel yet."
    // **Placeholder (not in the design: a mine's SightRange)**: 0, as a map resource's: it is hidden underground.
    sight_range: 0.0,
    max_hp: Some(40.0), // "mine HP 40"
    // **Placeholder (not in the design: a mine's armour)**: none.
    armor: Armor::NONE,
    needs_flat_ground: false,
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;

    /// A GDO mine: a 16×16 su object of its own category, selectable and Groupable, MaxHP 40, with neither unit nor
    /// structure attributes, and so no movement, no tile footprint and no Unit Control (design change 0426).
    #[test]
    fn the_mine_follows_the_design() {
        let def = ObjectKind::Mine.def();
        assert_eq!(
            (def.name, def.category, def.faction),
            ("Mine", Category::Mine, Some(Faction::Gdo))
        );
        assert_eq!((def.silhouette(), def.tile_size()), (Some(Vec2::new(16.0, 16.0)), None));
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor),
            (Some(40.0), None, Armor::NONE)
        );
        assert_eq!((def.selectable, def.destructible, def.groupable), (true, true, true));
        assert!(def.unit.is_none() && def.structure.is_none() && def.default_settings.is_none());
        let mines: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
            .filter(|kind| kind.def().category == Category::Mine)
            .collect();
        assert_eq!(mines, [ObjectKind::Mine]);
    }
}
