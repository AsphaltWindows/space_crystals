//! The Laser Turret — `factions/syndicate/structures/laser_turret.md` (design change 0260, its placeholder stats;
//! slice M4-S1): a structure that attacks (plan decision T49), which an Agent builds by the Tunnel's flow (plan
//! decision T51).
//!
//! Slice M4-S1 builds the **upright** turret, the Agent's. Its pop-up form inside an own Tunnel Area and the Tunnel's
//! ExpandMenu entry are slice M4-S2's: until then a turret inside an own Tunnel Area stands upright too.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::units::{AttackAttrs, AttackType, FullyConnectedSubtype, TargetDomain, TargetType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/laser_turret.md`: "Syndicate defensive structure capable of attacking both ground and
/// air units". Upright (`> Upright vs Pop-up`): "a normal surface structure, always visible. Costs no Tunnel Space"; it
/// "blocks movement like any building" (design change 0296 point 3, `factions/syndicate/structures/laser_turret.md >
/// Pop-up rules`). "The turret has no commands" (design change 0298 point 8): it picks its own targets
/// (`combat::structures`).
pub(crate) static LASER_TURRET: ObjectDef = ObjectDef {
    kind: ObjectKind::LaserTurret,
    name: "Laser Turret",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Surface / pop-up)
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size - 1x1
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: `core/entities.md > Object Type > Groupable` makes only casters and the workers used up by a build
    // Ungroupable.
    groupable: true,
    sight_range: 6.0,    // SightRange: 6 grid units
    max_hp: Some(300.0), // HP: 300
    max_shields: None,
    // "Armor (building armor rule, 1x1): raised PointArmor 1, FullArmor 1" (`factions/syndicate/structures/README.md`'s
    // n = 1 for a 1x1 structure). The underground armour is the pop-up's, slice M4-S2's.
    armor: Armor { point: 1.0, full: 1.0 },
    // A surface structure (`core/map.md > Elevation > Structures only on flat terrain`): only the expansions may stand
    // on slopes (design change 0005).
    needs_flat_ground: true,
    unit: None,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None, // no commands
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        // Upright: a surface structure, which "blocks movement like any building" on its footprint (design change 0296
        // point 3). The pop-up form, underground while it has no target, is slice M4-S2's.
        underground: false,
        tunnel: None,
        site: Some(LASER_TURRET_SITE), // Constructed by - Agent (anywhere)
        // Constructed by - "Tunnel (within its own Tunnel Area, via the ExpandMenu)": slice M4-S2.
        expansion: None,
        // TechPrerequisite - "player owns at least one Optics Lab (anywhere in the Tunnel Network)".
        requires: &[ObjectKind::OpticsLab],
        attack: Some(LASER_TURRET_ATTACK),
    }),
    default_settings: None,
};

/// `factions/syndicate/structures/laser_turret.md`'s placeholder AttackAttributes: "FullyConnected red laser (instant,
/// cannot miss); TargetDomain Universal; SingleTarget; Damage 24; Range 5 grid units; MinRange 0; Aim 4, Firing 1,
/// Cooldown 3, Reload 16 frames (one shot per 24 frames, 16 damage per second)".
pub(crate) const LASER_TURRET_ATTACK: AttackAttrs = AttackAttrs {
    // FullyConnected; its Range of 5 grid units makes it the Ranged subtype (`systems/combat.md > FullyConnected >
    // Melee` is adjacent contact).
    attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
    target_domain: TargetDomain::Universal, // TargetDomain Universal
    target_type: TargetType::SingleTarget,  // SingleTarget
    damage: 24.0,                           // Damage 24
    range: 5.0,                             // Range 5 grid units
    min_range: 0.0,                         // MinRange 0
    projectile_speed: None,                 // FullyConnected: no projectile
    aim_ticks: 4,                           // Aim 4
    firing_ticks: 1,                        // Firing 1
    cooldown_ticks: 3,                      // Cooldown 3
    reload_ticks: 16,                       // Reload 16 frames
};

/// The Laser Turret's construction by an Agent (`StructureDef::site`; `factions/syndicate/structures/laser_turret.md >
/// Construction`: "**By an Agent**: built using the same flow as Tunnel construction").
pub(crate) const LASER_TURRET_SITE: SiteDef = SiteDef {
    // "Cost: 100 Space Crystals (no Supplies)", paid as the order is given, refunded in full by an order that goes
    // before its site starts, and at 75% by a cancelled site (design change 0297 points 4 and 5:
    // `SitePrice::Crystals`).
    price: SitePrice::Crystals(100),
    frames: 320, // "Build time: 320 frames (20 s)"
    // The Tunnel's flow (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`): it "starts at 10%
    // HP (ConstructionHP Rule)".
    start_hp: 0.1,
    // "On successful completion, the Agent emerges on the surface adjacent to the finished structure (unlike Tunnels,
    // where the Agent ends up inside the Tunnel Network)".
    builder_goes: BuilderGoes::Beside,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stats (design change 0260): a 1x1 surface structure of 300 HP that sees 6 grid units, at the
    /// building armour rule's 1 / 1, requiring an Optics Lab; its red laser hits ground and air for 24 at 5 grid units,
    /// one shot in 24 frames. An Agent builds it in 320 frames for 100 Space Crystals, from 10% of its HP, and comes
    /// out beside it. Not given, so placeholders: Groupable, SymmetryType AAAA (plan decision T29).
    #[test]
    fn the_laser_turret_follows_the_design() {
        let def = ObjectKind::LaserTurret.def();
        let structure = def.structure.as_ref().unwrap();
        assert_eq!(
            (def.name, def.category, def.faction, def.tile_size(), structure.symmetry),
            (
                "Laser Turret",
                Category::Structure,
                Some(Faction::Syndicate),
                Some((1, 1)),
                SymmetryType::AAAA
            )
        );
        let armor = Armor { point: 1.0, full: 1.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(300.0), None, armor, 6.0)
        );
        assert_eq!(
            (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
            (true, true, true, true)
        );
        assert_eq!(
            (
                structure.underground,
                structure.production,
                structure.expansion,
                structure.requires
            ),
            (false, None, None, &[ObjectKind::OpticsLab][..])
        );
        let attack = structure.attack.unwrap();
        assert_eq!(
            (attack.attack_type, attack.target_domain, attack.target_type),
            (
                AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
                TargetDomain::Universal,
                TargetType::SingleTarget
            )
        );
        assert_eq!(
            (attack.damage, attack.range, attack.min_range, attack.projectile_speed),
            (24.0, 5.0, 0.0, None)
        );
        let cycle = [
            attack.aim_ticks,
            attack.firing_ticks,
            attack.cooldown_ticks,
            attack.reload_ticks,
        ];
        assert_eq!(cycle, [4, 1, 3, 16]);
        assert_eq!(cycle.iter().sum::<u32>(), 24, "one shot per 24 frames");
        // 24 damage in 24 frames at 16 frames a second, against no armour.
        let per_second = attack.damage * crate::units::TICKS_PER_SECOND as f32 / 24.0;
        assert_eq!(per_second, 16.0, "16 damage per second");
        let expected = SiteDef {
            price: SitePrice::Crystals(100),
            frames: 320,
            start_hp: 0.1,
            builder_goes: BuilderGoes::Beside,
        };
        assert_eq!(structure.site, Some(expected));
        let agent = ObjectKind::Agent.def().unit.as_ref().unwrap().worker.unwrap();
        assert!(agent.builds.contains(&ObjectKind::LaserTurret), "Agent D");
    }
}
