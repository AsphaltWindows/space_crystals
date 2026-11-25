//! The Syndicate's stat blocks against the design, one test per kind: `factions/syndicate/units.md > Guard`,
//! `> Agent`; `tunnels.md > Tunnel`; `structures.md > Hideout`. The rules every kind follows are in
//! `defs_are_consistent` (`defs/tests.rs`).

use glam::Vec2;

use super::*;
use crate::combat::domain_allows;
use crate::defs::Domain;

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// shot to the next, their sum.
fn timings(attack: &AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}

/// Design change 0186 (`factions/syndicate/units.md > Guard > AttackAttributes`): the Guard's laser rifle hits
/// ground and air (TargetDomain Universal), instantly, for 18 damage, one shot every 24 frames (Aim 4, Fire 1,
/// Cooldown 3, Reload 16; placeholders). Its Range, its MinRange and the rest of the Guard are as before.
#[test]
fn the_guards_laser_rifle_hits_ground_and_air_every_24_frames() {
    let guard = ObjectKind::Guard.def();
    let attack = guard.unit.as_ref().unwrap().attack.unwrap();
    assert_eq!(attack.target_domain, TargetDomain::Universal);
    for domain in [Domain::Ground, Domain::Air] {
        assert!(domain_allows(attack.target_domain, domain), "{domain:?}");
    }
    assert_eq!(
        attack.attack_type,
        AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
        "instant hit: it can't miss"
    );
    assert_eq!(attack.projectile_speed, None);
    assert_eq!((attack.target_type, attack.damage), (TargetType::SingleTarget, 18.0));
    assert_eq!(timings(&attack), ((4, 1, 3, 16), 24), "one shot every 24 frames");
    assert_eq!((attack.range, attack.min_range), (3.0, 0.0));
    assert_eq!((guard.max_hp, guard.sight_range), (Some(80.0), 5.0));
}

/// `factions/syndicate/units.md > Agent`: a 36×36 HeavyInfantry unit, MaxHP 75, PointArmor 1, FullArmor 1, SightRange
/// 5, TunnelSpaceCost 1, Groupable false, MaxSpeed 6, no turret, with its pistol (user, 2026-11-24; "Numbers are
/// placeholders"): FullyConnected Ranged, ground only, single target, 5 damage at up to 2 grid units, "One shot every
/// 20 frames (1.25 s)". Its gathering comes with slice M4-D.
#[test]
fn the_agent_follows_the_design() {
    let agent = ObjectKind::Agent.def();
    assert_eq!(
        (agent.category, agent.faction, agent.silhouette()),
        (Category::Unit, Some(Faction::Syndicate), Some(Vec2::new(36.0, 36.0)))
    );
    let one = Armor { point: 1.0, full: 1.0 };
    assert_eq!(
        (agent.max_hp, agent.max_shields, agent.armor, agent.sight_range),
        (Some(75.0), None, one, 5.0)
    );
    assert_eq!(
        (
            agent.selectable,
            agent.destructible,
            agent.groupable,
            agent.needs_flat_ground
        ),
        (true, true, false, false)
    );
    let unit = agent.unit.as_ref().unwrap();
    assert_eq!(unit.base, UnitBase::HeavyInfantry);
    let movement = MovementAttrs::TurnRate {
        turn_rate: PI,
        acceleration: None,
        deceleration: None,
        max_speed: 6.0,
    };
    assert_eq!(unit.movement, movement);
    assert_eq!(
        (unit.turret.is_none(), unit.tunnel_space_cost, unit.worker.is_none()),
        (true, Some(1), true)
    );
    let attack = unit.attack.unwrap();
    assert_eq!(
        attack.attack_type,
        AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
    );
    assert_eq!(
        (attack.target_domain, attack.target_type, attack.damage),
        (TargetDomain::Ground, TargetType::SingleTarget, 5.0)
    );
    assert!(domain_allows(attack.target_domain, Domain::Ground));
    assert!(!domain_allows(attack.target_domain, Domain::Air), "ground only");
    assert_eq!(
        (attack.range, attack.min_range, attack.projectile_speed),
        (2.0, 0.0, None)
    );
    assert_eq!(timings(&attack), ((3, 1, 2, 14), 20), "one shot every 20 frames");
}

/// `factions/syndicate/tunnels.md > Tunnel`: a 4×4 ABCD surface structure, Destructible, Groupable false, SightRange
/// 5, whose tier table is `> Tiers`: HP 600 / 800 / 1000, PointArmor 1 and FullArmor 16 at every tier, Tunnel Area
/// Radius 2 / 3 / 4, Transit Infantry, then Vehicles, then Air. Inert so far (slice M4-A): no drop-off, production or
/// construction yet.
#[test]
fn the_tunnel_follows_the_design() {
    let def = ObjectKind::Tunnel.def();
    let structure = def.structure.as_ref().unwrap();
    assert_eq!(
        (def.category, def.faction, def.tile_size(), structure.symmetry),
        (
            Category::Structure,
            Some(Faction::Syndicate),
            Some((4, 4)),
            SymmetryType::ABCD
        )
    );
    let armor = Armor { point: 1.0, full: 16.0 };
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(600.0), None, armor, 5.0)
    );
    assert_eq!(
        (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
        (true, true, false, true)
    );
    let tiers: Vec<(f32, u32, &[Transit])> = (structure.tunnel.unwrap().tiers.iter())
        .map(|tier| (tier.max_hp, tier.area_radius, tier.transit))
        .collect();
    use Transit::{Air, Infantry, Vehicles};
    assert_eq!(
        tiers,
        [
            (600.0, 2, &[Infantry][..]),
            (800.0, 3, &[Infantry, Vehicles][..]),
            (1000.0, 4, &[Infantry, Vehicles, Air][..]),
        ]
    );
    let inert = StructureDef {
        symmetry: SymmetryType::ABCD,
        build_radius_extension: 0,
        power: 0,
        production: None,
        mining: None,
        constructs: &[],
        warp_in: &[],
        drop_off: false,
        underground: false,
        tunnel: structure.tunnel,
    };
    assert_eq!(structure, &inert);
}

/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: "Tier 1+: Infantry (Light
/// Infantry, Heavy Infantry)", "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)", "Tier 3+: Air units
/// (Hover Craft, Glider)".
#[test]
fn each_tier_admits_its_unit_bases() {
    use Transit::{Air, Infantry, Vehicles};
    use UnitBase as B;
    let tiers = TUNNEL_TIERS.tiers;
    let first_tier = [
        (B::LightInfantry, Infantry, 1),
        (B::HeavyInfantry, Infantry, 1),
        (B::WheeledVehicle, Vehicles, 2),
        (B::TrackedVehicle, Vehicles, 2),
        (B::DrillUnit, Vehicles, 2),
        (B::HoverVehicle, Vehicles, 2),
        (B::Mech, Vehicles, 2),
        (B::HoverCraft, Air, 3),
        (B::Glider, Air, 3),
    ];
    for (base, transit, first) in first_tier {
        assert_eq!(Transit::of(base), transit, "{base:?}");
        for (tier, row) in (1..).zip(tiers) {
            assert_eq!(row.admits(base), tier >= first, "{base:?} at Tier {tier}");
        }
    }
}

/// `factions/syndicate/structures.md > Hideout`: an underground structure ("Structure Type (Underground)"), 2×2, HP
/// 400, PointArmor 1, FullArmor 4, which may stand on slopes (design change 0005) and has no tiers. Not given, so
/// placeholders: SightRange 0 (Q0065 point 5), Groupable true (Q0065 point 6), SymmetryType AAAA (plan decision
/// T29). Its BuildQueue comes with slice M4-C.
#[test]
fn the_hideout_follows_the_design() {
    let def = ObjectKind::Hideout.def();
    let structure = def.structure.as_ref().unwrap();
    assert_eq!(
        (def.category, def.faction, def.tile_size(), structure.symmetry),
        (
            Category::Structure,
            Some(Faction::Syndicate),
            Some((2, 2)),
            SymmetryType::AAAA
        )
    );
    let armor = Armor { point: 1.0, full: 4.0 };
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(400.0), None, armor, 0.0)
    );
    assert_eq!(
        (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
        (true, true, true, false)
    );
    assert_eq!(
        (
            structure.underground,
            structure.tunnel,
            structure.production,
            structure.drop_off
        ),
        (true, None, None, false)
    );
}
