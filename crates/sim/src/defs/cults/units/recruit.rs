//! The Recruit — `factions/cults/units/recruit.md`.

use crate::defs::units::{
    AttackAttrs, AttackType, Augmentable, DropOffChoice, FullyConnectedSubtype, LoadDef, MELEE_CONTACT_GU,
    MovementAttrs, RepairDef, RepairTargets, TargetDomain, TargetType, UnitBase, UnitDef, WorkerDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/cults/units/recruit.md`: "the Cults' basic worker unit, automatically produced by Recruitment Halls", with
/// "a melee attack for basic self-defense". What it does besides fight comes with each system's slice (plan §3.2): its
/// gathering (`> Gathering`, slice M5-D) and its building (`> Building`, slice M5-E) in its `worker`; its entering an
/// Armory (`> Armory Entry`), slice M5-F1.
pub(crate) static RECRUIT: ObjectDef = ObjectDef {
    kind: ObjectKind::Recruit,
    name: "Recruit",
    faction: Some(Faction::Cults),                         // Faction - TheCults
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true (user, 2026-11-24)
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(40.0),                                    // MaxHP - 40
    max_shields: None,
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase - LightInfantry
        // UnitBaseAttributes[LightInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 4.0,     // MaxSpeed: 4 space units/frame
        },
        rugged_terrain_defense_bonus: Some(0.5), // RuggedTerrainDefenseBonus: 50%
        turret: None,                            // TurretAttributes - None
        attack: Some(AttackAttrs {
            // AttackType: FullyConnected (Melee)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 3.0,                           // Damage: 3
            // Range: melee (adjacent contact): a reach of 8 su, a placeholder (design change 0149).
            range: MELEE_CONTACT_GU,
            min_range: 0.0,         // none listed: contact is as near as a target gets
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 2,           // AimDuration: 2 frames
            firing_ticks: 8,        // FiringDuration: 8 frames
            cooldown_ticks: 1,      // CooldownDuration: 1 frame
            reload_ticks: 1,        // ReloadDuration: 1 frame
        }),
        // Its Unit Control: the one Recruit it is (plan decision T55; `factions/cults/overview.md > CultsResources >
        // Unit Control`: "Each Cult unit has a Unit Control cost"; `factions/cults/structures/recruitment_hall.md >
        // BaseRecruitProductionTime`: "every Cults unit (miners, builders, staff and army) counts against its
        // originating hall's capacity", and "a starting hall (6 Recruits → 20) fills in 14 × 12 s").
        unit_control_cost: Some(1),
        tunnel_space_cost: None,
        beacon_capacity_cost: None,
        riot_shield: None,
        conversion: None,
        deploy: None,
        // `> Gathering` (slice M5-D, plan decision T58): "Recruits gather Space Crystals and deliver them to
        // Storage". One Gather job runs every worker; this is the Recruit's data.
        worker: Some(WorkerDef {
            crystals: LoadDef {
                carry_capacity: 2,   // CarryCapacity: 2 Space Crystals per load
                pick_up_ticks: 8,    // MiningDuration: 8 frames
                drop_off_ticks: 2,   // DropOffDuration: 2 frames
                pick_up_reach: 8.0,  // MiningReach: 8 space units (the design's placeholder)
                drop_off_reach: 8.0, // DropOffReach: 8 space units (the design's placeholder)
            },
            supplies: None,
            shares_patches: true, // Patch sharing: any number of Recruits may mine the same patch
            // Design changes 0345 and 0375 (answering Q0102 and Q0136; `> Gathering`, Which Storage): the own
            // completed Storage nearest to the Recruit itself, edge to edge, when it finishes mining, at every return.
            drop_off: DropOffChoice::EachReturn,
            // Placeholder (not in the design: the Recruit's sections name no search radius for a patch that runs
            // out; the Drone's and the Agent's 6 grid units, themselves placeholders, design changes 0123 and 0295).
            search_radius: 6.0,
            // `factions/colonists/units/drone.md > Gathering Drones pass through each other`: "Drones only. Other
            // factions' workers (e.g. Cults Recruits) keep normal collision while mining" (design change 0106).
            pass_through: false,
            // `> Building`: "Recruits construct Cults buildings from outside the site and can assist in-progress
            // construction"; `> Constructs`, the buildings of plan slice M5-A (slice M5-E), and the Gunworks, the
            // Machine Shop and the Scrap Hangar (slice M5-G2), which only unlock, research nothing yet and hold no one,
            // the Garrison (slice M5-N2), before the Gunworks as the list has it, the Altar (slice M5-N1b), after the
            // Armory, the Temple (slice M5-N3) and the Cathedral (slice M5-N4a), after the Altar as their menu has
            // them; each other building joins with the slice that makes it work (plan decision T69). A group shares
            // one build (`factions/cults/economy.md > Build Command`: "All selected Recruits walk to the location and
            // begin construction from outside the site"; plan decision T59).
            builds: &[
                ObjectKind::RecruitmentHall,
                ObjectKind::Storage,
                ObjectKind::Armory,
                ObjectKind::Altar,
                ObjectKind::Temple,
                ObjectKind::Cathedral,
                ObjectKind::Garrison,
                ObjectKind::Gunworks,
                ObjectKind::MachineShop,
                ObjectKind::ScrapHangar,
            ],
            shares_builds: true,
        }),
        // (M5-N1a) `factions/cults/augmentation.md > Increments`: "Unarmed units ... currently only the Recruit".
        augmentable: Some(Augmentable { unarmed: true }),
        // `> Repair` (slice M5-N8): "Recruits can **repair** own **buildings and vehicles**. Repair **costs Space
        // Crystals**".
        repairs: Some(RepairDef {
            targets: RepairTargets::OwnBuildingsAndVehicles,
        }),
    }),
    structure: None,
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::{assert_infantry, timings};

    /// The Cults' worker: LightInfantry, 24×24, MaxHP 40, no armour, SightRange 5, Groupable (user, 2026-11-24),
    /// MaxSpeed 4, with its melee attack "for basic self-defense": FullyConnected Melee, ground only, single target, 3
    /// damage at contact, one blow every 12 frames (Aim 2, Fire 8, Cooldown 1, Reload 1); Unit Control 1.
    /// `> Gathering`: 2 a load, 8 frames of mining and 2 of unloading, reaches of 8 su, shared patches, normal
    /// collision (design change 0106), no Supplies; the Storage nearest to it at each return (design changes 0375 and
    /// 0345). `> Building` (slice M5-E): it builds the Recruitment Hall, the Storage and the Armory, the Gunworks,
    /// the Machine Shop and the Scrap Hangar (slice M5-G2), the Garrison (slice M5-N2), the Altar (slice M5-N1b) and
    /// the Temple (slice M5-N3) and the Cathedral (slice M5-N4a), a group sharing one build.
    #[test]
    fn the_recruit_follows_the_design() {
        let attack = assert_infantry(ObjectKind::Recruit);
        assert_eq!(
            attack.attack_type,
            AttackType::FullyConnected(FullyConnectedSubtype::Melee)
        );
        assert_eq!(
            (attack.target_domain, attack.target_type, attack.damage),
            (TargetDomain::Ground, TargetType::SingleTarget, 3.0)
        );
        assert_eq!(
            (attack.range, attack.min_range, attack.projectile_speed),
            (MELEE_CONTACT_GU, 0.0, None)
        );
        assert_eq!(timings(&attack), ((2, 8, 1, 1), 12), "one blow every 12 frames");
        let def = ObjectKind::Recruit.def();
        assert_eq!(def.name, "Recruit");
        let worker = def.unit.as_ref().unwrap().worker.expect("a worker");
        let crystals = LoadDef {
            carry_capacity: 2,
            pick_up_ticks: 8,
            drop_off_ticks: 2,
            pick_up_reach: 8.0,
            drop_off_reach: 8.0,
        };
        assert_eq!((worker.crystals, worker.supplies), (crystals, None));
        assert!(worker.shares_patches && !worker.pass_through);
        assert_eq!(
            worker.drop_off,
            DropOffChoice::EachReturn,
            "design change 0375: the nearest to the Recruit, at every return"
        );
        assert_eq!(worker.search_radius, 6.0, "a placeholder: the other workers' radius");
        // `> Building`, `> Constructs` (slice M5-E): the three buildings of plan slice M5-A, slice M5-G2's three,
        // slice M5-N2's Garrison, slice M5-N1b's Altar, slice M5-N3's Temple and slice M5-N4a's Cathedral, as one
        // group's build.
        assert_eq!(
            worker.builds,
            [
                ObjectKind::RecruitmentHall,
                ObjectKind::Storage,
                ObjectKind::Armory,
                ObjectKind::Altar,
                ObjectKind::Temple,
                ObjectKind::Cathedral,
                ObjectKind::Garrison,
                ObjectKind::Gunworks,
                ObjectKind::MachineShop,
                ObjectKind::ScrapHangar
            ]
        );
        assert!(worker.shares_builds, "a group shares one build (plan decision T59)");
    }
}
