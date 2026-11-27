//! The Missile Turret — `factions/gdo/structures/missile_turret.md` (design change 0366, slice M6-GK): "GDO defensive
//! structure. Attacks both air and ground targets". Its numbers are the design's "Placeholder stats" (plan decision
//! T84). Slice M6-GR gives it its homing missile ([`MISSILE_TURRET_ATTACK`], `StructureDef::attack`), which it fires
//! with the Laser Turret's targeting (`crate::combat::structures`), and makes its Defense System entry ([`RECIPE`])
//! live. A consumer of Power, its whole attack cycle slows with its owner's Power ratio (design change 0423,
//! `crate::combat::power`).

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::units::{AttackAttrs, AttackType, TargetDomain, TargetType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/missile_turret.md`: its placeholder stats (design change 0366).
pub(crate) const MISSILE_TURRET: ObjectDef = ObjectDef {
    kind: ObjectKind::MissileTurret,
    name: "Missile Turret",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size 1x1
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 7.0,    // SightRange 7 grid units
    max_hp: Some(250.0), // MaxHP 250
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 2.0,  // FullArmor 2
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 0,    // BuildRadiusExtension 0
        power: -10,                   // Power −10
        requires: &[],                // TechPrerequisite - None
        // (M6-GR) "Targeting as the Laser Turret's (BaseAutoTargeting, closest, keeps its target)": a structure's own
        // pick (`crate::combat::structures`). No pop-up: it stands raised, firing from its first tick.
        attack: Some(MISSILE_TURRET_ATTACK),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/defense_system.md > Constructs`: "Missile Turret: 100 Space Crystals, 240 frames (15
/// seconds) (placeholder)". Live since slice M6-GR, in the Defense System's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::MissileTurret,
    cost: 100,
    frames: 240,
    requires: &[],
    supplies: 0,
};

/// (M6-GR) `> Placeholder stats`: "Attack: **HeadDisjointed** (homing missiles, cannot miss; user, 2026-11-27),
/// TargetDomain Universal, SingleTarget; Damage 18; Range 6 grid units; MinRange 0; ProjectileSpeed 16 su/frame; Aim 4,
/// Firing 2, Cooldown 4, Reload 14 frames (one missile per 24 frames)" (`systems/combat.md > HeadDisjointed`; design
/// change 0368: "homing").
pub(crate) const MISSILE_TURRET_ATTACK: AttackAttrs = AttackAttrs {
    attack_type: AttackType::HeadDisjointed, // homing missiles, cannot miss
    target_domain: TargetDomain::Universal,  // TargetDomain Universal
    target_type: TargetType::SingleTarget,   // SingleTarget
    damage: 18.0,                            // Damage 18
    range: 6.0,                              // Range 6 grid units
    min_range: 0.0,                          // MinRange 0
    projectile_speed: Some(16.0),            // ProjectileSpeed 16 su/frame
    aim_ticks: 4,                            // Aim 4
    firing_ticks: 2,                         // Firing 2
    cooldown_ticks: 4,                       // Cooldown 4
    reload_ticks: 14,                        // Reload 14 frames
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, with no TechPrerequisite; (M6-GR) its homing missile, one per 24 frames, and no
    /// pop-up; its Defense System entry, 100 Space Crystals and 240 frames, is live.
    #[test]
    fn the_missile_turret_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::MissileTurret,
            (1, 1),
            SymmetryType::AAAA,
            250.0,
            (2.0, 2.0),
            7.0,
            true,
            0,
            -10,
        );
        assert_eq!(structure.requires, [] as [ObjectKind; 0]);
        assert_eq!((structure.attack, structure.popup), (Some(MISSILE_TURRET_ATTACK), None));
        let a = MISSILE_TURRET_ATTACK;
        assert_eq!(
            (a.attack_type, a.target_domain),
            (AttackType::HeadDisjointed, TargetDomain::Universal)
        );
        assert_eq!(
            (a.damage, a.range, a.min_range, a.projectile_speed),
            (18.0, 6.0, 0.0, Some(16.0))
        );
        let cycle = (a.aim_ticks, a.firing_ticks, a.cooldown_ticks, a.reload_ticks);
        assert_eq!(cycle, (4, 2, 4, 14));
        assert_eq!(cycle.0 + cycle.1 + cycle.2 + cycle.3, 24, "one missile per 24 frames");
        assert_eq!(RECIPE, super::super::held(ObjectKind::MissileTurret, 100, 240));
        let ds = ObjectKind::DefenseSystem.def().structure.as_ref().unwrap();
        assert_eq!(ds.constructs, [RECIPE]);
    }
}
