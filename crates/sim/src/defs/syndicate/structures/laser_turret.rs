//! The Laser Turret — `factions/syndicate/structures/laser_turret.md` (design change 0260, its placeholder stats;
//! slice M4-S1): a structure that attacks (plan decision T49), always a **pop-up** (slice M4-S2, plan decision T50:
//! `crate::popup`), which only a Tunnel builds, from the ExpandMenu's tenth slot (`StructureDef::expansion`).
//!
//! Slice M5-Y (design changes 0359 and 0360): "**Only Tunnels build it** ... it is always a hidden **pop-up
//! turret**, and is **only active inside its Tunnel's area** ... There is no upright form any more". Slice M4-S1's
//! Agent-built turret and its upright form are gone.

use crate::defs::structures::{ExpansionDef, PopUpDef, StructureDef, SymmetryType};
use crate::defs::units::{AttackAttrs, AttackType, FullyConnectedSubtype, TargetDomain, TargetType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/laser_turret.md`: "Syndicate defensive structure capable of attacking both ground and
/// air units". A pop-up (`> Pop-up turret`): underground and hidden while it has no target, up and firing at one;
/// raised, it "blocks movement like any building" (`factions/syndicate/structures/laser_turret.md > Pop-up rules`).
/// "The turret has no commands" (design change 0298 point 8): it picks its own targets (`combat::structures`).
pub(crate) static LASER_TURRET: ObjectDef = ObjectDef {
    kind: ObjectKind::LaserTurret,
    name: "Laser Turret",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (pop-up)
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
    // n = 1 for a 1x1 structure). The underground armour is the pop-up's (`LASER_TURRET_POPUP`).
    armor: Armor { point: 1.0, full: 1.0 },
    // Design change 0358 point 3 (`> Pop-up turret`): "on ground that is **Buildable, flat and Drillable** (it must be
    // able to rise)": a surface structure's flat ground (`core/map.md > Elevation > Structures only on flat terrain`),
    // and an expansion's Drillable (`placement::check`).
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
        // Raised, a surface structure that "blocks movement like any building" on its footprint (`> Pop-up rules`).
        // No underground structure by its type: it is underground while lowered, by its state (`crate::popup`).
        underground: false,
        tunnel: None,
        // No worker builds it (design change 0359: "Agents no longer build it").
        site: None,
        // Constructed by - "Tunnel only (within its own Tunnel Area, via the ExpandMenu ...)".
        expansion: Some(LASER_TURRET_EXPANSION),
        // TechPrerequisite - "player owns at least one Optics Lab (anywhere in the Tunnel Network)".
        requires: &[ObjectKind::OpticsLab],
        attack: Some(LASER_TURRET_ATTACK),
        popup: Some(LASER_TURRET_POPUP),
        holds: None,
        occupants_fire: None,
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

/// The Laser Turret's construction by a Tunnel (`StructureDef::expansion`, slice M4-S2;
/// `factions/syndicate/structures/laser_turret.md > Construction`: "**By a Tunnel**: from the Tunnel's ExpandMenu,
/// placed within that Tunnel's Tunnel Area like an expansion. It **uses the Tunnel's single operation slot** ... Paid
/// when placed, like an expansion. **Cancelling** the site refunds **75%** (like an expansion)").
pub(crate) const LASER_TURRET_EXPANSION: ExpansionDef = ExpansionDef {
    // Design change 0358 point 4 (`> Pop-up turret`): "from **Tier 1** (no Tier Requirement)", the Tier Requirement a
    // reclaiming Tunnel meets too (`> When its Tunnel is lost`: "Tier 1 suffices").
    tier: 1,
    cost: 100,   // "Cost: 100 Space Crystals (no Supplies)"
    frames: 320, // "Build time: 320 frames (20 s)"
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

/// The Laser Turret's pop-up form (`StructureDef::popup`, slice M4-S2;
/// `factions/syndicate/structures/laser_turret.md`'s placeholder stats): "Pop-up: rises in 8 frames (already facing its
/// target); retracts after 32 frames with no target, taking 8 frames"; "underground (pop-up) PointArmor 6, FullArmor
/// 6" (the expansions' n = 6 on its one square).
pub(crate) const LASER_TURRET_POPUP: PopUpDef = PopUpDef {
    rise_frames: 8,
    idle_frames: 32,
    retract_frames: 8,
    lowered_armor: Armor { point: 6.0, full: 6.0 },
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stats (design change 0260): a 1x1 structure of 300 HP on flat ground that sees 6 grid units, at
    /// the building armour rule's 1 / 1 raised, requiring an Optics Lab; its red laser hits ground and air for 24 at 5
    /// grid units, one shot in 24 frames. A Tunnel builds it in 320 frames for 100 Space Crystals, from 10% of its HP,
    /// at any tier (design change 0358 point 4), and no worker does (design change 0359). It rises in 8 frames,
    /// retracts after 32 with no target, in 8, and has 6 / 6 underground. Not given, so placeholders: Groupable,
    /// SymmetryType AAAA (plan decision T29).
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
            (structure.underground, structure.production, structure.requires),
            (false, None, &[ObjectKind::OpticsLab][..])
        );
        let by_a_tunnel = ExpansionDef {
            tier: 1,
            cost: 100,
            frames: 320,
            start_hp: 0.1,
        };
        assert_eq!(structure.expansion, Some(by_a_tunnel));
        let popup = PopUpDef {
            rise_frames: 8,
            idle_frames: 32,
            retract_frames: 8,
            lowered_armor: Armor { point: 6.0, full: 6.0 },
        };
        assert_eq!(structure.popup, Some(popup));
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
        assert_eq!(structure.site, None, "only Tunnels build it");
        let agent = ObjectKind::Agent.def().unit.as_ref().unwrap().worker.unwrap();
        assert!(
            !agent.builds.contains(&ObjectKind::LaserTurret),
            "Agents no longer build it"
        );
    }
}
