//! A Tunnel builds the Forge (`factions/syndicate/structures/forge.md`: "Tier Requirement - 2", "TechPrerequisite -
//! player owns at least one Vehicle Assembly (anywhere in the Tunnel Network)", "Size 2x2; MaxHP 400; PointArmor 6,
//! FullArmor 24 (underground expansion, n = 6); Cost 200 Space Crystals; Build Time 480 frames (30 seconds)", "No
//! commands for now (prerequisite only)") and the Aerospace Assembly
//! (`factions/syndicate/structures/aerospace_assembly.md`: "Tier Requirement - 3", "TechPrerequisite - player owns at
//! least one Forge (anywhere in the Tunnel Network)", "Size 3x3; MaxHP 600; PointArmor 6, FullArmor 54 (underground
//! expansion, n = 6); Cost 300 Space Crystals; Build Time 640 frames (40 seconds)"), by
//! `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254: "**Paid when the ghost is
//! placed**", "the ConstructionHP Rule applies to every expansion (starts at 10%, grows with progress)").

use sc_sim::tunnel::Operation;

use super::*;

const FORGE_KIND: ObjectKind = ObjectKind::Forge;
const AA_KIND: ObjectKind = ObjectKind::AerospaceAssembly;
/// A free place for a Forge in [`base`]'s Tier 2 area but not its Tier 1 one: (9, 11) to (10, 12).
const FORGE_FREE: TilePos = TilePos::new(9, 11);
/// A free place for an Aerospace Assembly in [`base`]'s Tier 3 area but not its Tier 2 one: (17, 10) to (19, 12).
const AA_FREE: TilePos = TilePos::new(17, 10);

/// The site `tunnel` builds; panics if it builds none.
fn building(t: &TestSim, tunnel: ObjectId) -> ObjectId {
    match object(t, tunnel).tunnel.as_ref().and_then(|state| state.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("Tunnel {tunnel:?} builds nothing: {other:?}"),
    }
}

/// [`base`] with player 1's Vehicle Assembly ([`VA_AT`], id 2), the Forge's prerequisite, if `assembly`.
fn with_assembly(assembly: bool) -> ScenarioBuilder {
    match assembly {
        true => base().spawn(ObjectKind::VehicleAssembly, Some(P1), SpawnAt::Tile(VA_AT)),
        false => base(),
    }
}

/// [`base`] with player 1's Forge ([`FORGE_AT`], id 2), the Aerospace Assembly's prerequisite, if `forge`.
fn with_forge(forge: bool) -> ScenarioBuilder {
    match forge {
        true => base().spawn(FORGE_KIND, Some(P1), SpawnAt::Tile(FORGE_AT)),
        false => base(),
    }
}

/// `kind`'s site, which `tunnel` began building in tick `start`, paid `price`: player 1's, at 10% of `max_hp`, its
/// maximum growing every tick until it completes `frames` ticks after the click, at its full `max_hp`, the Tunnel free
/// again and its parent.
fn builds(t: &mut TestSim, kind: ObjectKind, start: Tick, (price, frames, max_hp): (u32, Tick, f32)) -> ObjectId {
    assert_eq!(bank(t, P1), BANK - price, "paid at the click");
    let site = building(t, T1);
    let o = object(t, site);
    assert_eq!((o.kind, o.owner, o.is_site()), (kind, Some(P1), true));
    let tenth = max_hp / 10.0;
    assert_eq!((o.hp, o.max_hp()), (Some(tenth), Some(tenth)), "10% of {max_hp}");
    let mut last = object(t, site).max_hp();
    while t.sim().tick() < start + frames - 1 {
        step_checked(t);
        let max = object(t, site).max_hp();
        assert!(max > last, "tick {}: {max:?} after {last:?}", t.sim().tick());
        last = max;
    }
    assert!(object(t, site).is_site(), "not before its {frames} frames");
    step_checked(t);
    let o = object(t, site);
    assert_eq!(
        (t.sim().tick(), o.is_site(), o.hp, o.max_hp()),
        (start + frames, false, Some(max_hp), Some(max_hp))
    );
    let completed = (t.events().iter()).any(|(_, e)| *e == SimEvent::ExpansionCompleted { tunnel: T1, site });
    assert!(completed);
    assert_eq!(
        object(t, T1).tunnel.as_ref().unwrap().operation,
        None,
        "the Tunnel is free"
    );
    assert_eq!(t.sim().parent_tunnel(site), Some(T1));
    site
}

/// The Forge: a Tier 1 Tunnel is refused it for its tier ([`Reject::TierRequirement`], 2), a Tier 2 one without a
/// Vehicle Assembly for the Assembly ([`Reject::Requires`]); with one, a Tier 2 Tunnel builds it in its Tier 2 area for
/// 200 Space Crystals in 480 frames, from 40 of its 400 HP, and it takes no command (it has no production: a Train
/// there finds no producer).
#[test]
fn a_tier_2_tunnel_with_a_vehicle_assembly_builds_the_forge_for_200_in_480_frames() {
    let tier_1 = started(with_assembly(true), 1);
    let refused = Reject::TierRequirement {
        kind: FORGE_KIND,
        tier: 2,
    };
    assert_eq!(
        gate(&tier_1, P1, expand(T1, FORGE_KIND, FORGE_FREE)),
        Err(refused),
        "Tier 1"
    );
    let no_assembly = started(with_assembly(false), 2);
    let refused = Reject::Requires(ObjectKind::VehicleAssembly);
    assert_eq!(
        gate(&no_assembly, P1, expand(T1, FORGE_KIND, FORGE_FREE)),
        Err(refused),
        "no Vehicle Assembly"
    );
    let mut t = started(with_assembly(true), 2);
    assert_eq!(gate(&t, P1, expand(T1, FORGE_KIND, FORGE_FREE)), Ok(()));
    t.at(2, P1, expand(T1, FORGE_KIND, FORGE_FREE));
    run_checked(&mut t, 2);
    let forge = builds(&mut t, FORGE_KIND, 2, (200, 480, 400.0));
    assert_eq!(object(&t, forge).placement.map(|p| p.origin), Some(FORGE_FREE));
    assert!(object(&t, forge).production.is_none(), "a prerequisite only");
    assert_eq!(gate(&t, P1, train(forge, PHANTOM)), Err(Reject::NoProducers));
}

/// The Aerospace Assembly: a Tier 2 Tunnel is refused it for its tier ([`Reject::TierRequirement`], 3), a Tier 3 one
/// without a Forge for the Forge ([`Reject::Requires`]); with one, a Tier 3 Tunnel builds it in its Tier 3 area for 300
/// Space Crystals in 640 frames, from 60 of its 600 HP. Until then it takes no Train (its production held back,
/// [`Reject::NoProducers`]); then it takes the Phantom's, an Optics Lab spawned for it.
#[test]
fn a_tier_3_tunnel_with_a_forge_builds_the_aerospace_assembly_for_300_in_640_frames() {
    let tier_2 = started(with_forge(true), 2);
    let refused = Reject::TierRequirement { kind: AA_KIND, tier: 3 };
    assert_eq!(gate(&tier_2, P1, expand(T1, AA_KIND, AA_FREE)), Err(refused), "Tier 2");
    let no_forge = started(with_forge(false), 3);
    let refused = Reject::Requires(FORGE_KIND);
    assert_eq!(
        gate(&no_forge, P1, expand(T1, AA_KIND, AA_FREE)),
        Err(refused),
        "no Forge"
    );
    let mut t = started(with_forge(true), 3);
    assert_eq!(gate(&t, P1, expand(T1, AA_KIND, AA_FREE)), Ok(()));
    t.at(2, P1, expand(T1, AA_KIND, AA_FREE));
    run_checked(&mut t, 2);
    let site = building(&t, T1);
    assert_eq!(gate(&t, P1, train(site, PHANTOM)), Err(Reject::NoProducers));
    let assembly = builds(&mut t, AA_KIND, 2, (300, 640, 600.0));
    assert_eq!(object(&t, assembly).placement.map(|p| p.origin), Some(AA_FREE));
    assert_eq!(
        gate(&t, P1, train(assembly, PHANTOM)),
        Err(Reject::Requires(ObjectKind::OpticsLab))
    );
    let lab = id(next_id(&t));
    t.debug_at(
        t.sim().tick() + 1,
        spawn(ObjectKind::OpticsLab, P1, SpawnAt::Tile(LAB_AT)),
    );
    run_for(&mut t, 1);
    assert_eq!(object(&t, lab).kind, ObjectKind::OpticsLab);
    assert_eq!(gate(&t, P1, train(assembly, PHANTOM)), Ok(()));
}

/// The design's armours: the Forge's 6 / 24 and the Aerospace Assembly's 6 / 54, the underground expansions' n = 6
/// (design change 0264), as [`hangar`] has them.
#[test]
fn the_forge_and_the_aerospace_assembly_have_the_expansions_armour() {
    let t = hangar();
    let armour = |id: ObjectId| {
        let armor = object(&t, id).def().armor;
        (armor.point, armor.full)
    };
    assert_eq!(armour(FORGE), (6.0, 24.0));
    assert_eq!(armour(AA), (6.0, 54.0));
    assert_eq!(object(&t, FORGE).max_hp(), Some(400.0));
    assert_eq!(object(&t, AA).max_hp(), Some(600.0));
}
