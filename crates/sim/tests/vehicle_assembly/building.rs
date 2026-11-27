//! A Tunnel builds the Vehicle Assembly (`factions/syndicate/structures/vehicle_assembly.md`: "Tier Requirement - 2",
//! "TechPrerequisite - player owns at least one Chemical Plant (anywhere in the Tunnel Network)", "Size 3x3; MaxHP
//! 600; PointArmor 6, FullArmor 54 (underground expansion, n = 6); Cost 250 Space Crystals; Build Time 480 frames";
//! `factions/syndicate/tunnel_menus.md > Expansions under construction`, design change 0254), and it regenerates
//! (`factions/syndicate/structures/README.md > Tunnel Expansions`: "Placeholder rate (designer): **1% of MaxHP per
//! second**, always on, **including during combat**").

use std::f32::consts::PI;

use sc_sim::tunnel::Operation;

use super::*;

/// `factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`: "Cost 250 Space Crystals; Build Time 480
/// frames (30 seconds)".
const PRICE: u32 = 250;
const FRAMES: Tick = 480;
/// What the Vehicle Assembly regenerates in a tick: 1% of its 600 HP a second, of 16 ticks.
const REGEN: f32 = 0.375;

/// The site `tunnel` builds; panics if it builds none.
fn building(t: &TestSim, tunnel: ObjectId) -> ObjectId {
    match object(t, tunnel).tunnel.as_ref().and_then(|state| state.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("Tunnel {tunnel:?} builds nothing: {other:?}"),
    }
}

fn hp(t: &TestSim, id: ObjectId) -> f32 {
    object(t, id).hp.expect("destructible")
}

/// The plan's acceptance, "Expand is refused at Tier 1 ("Tier 2"), and without a Chemical Plant"
/// (`factions/syndicate/tunnel_menus.md > ExpandMenu`: "Expansion types whose Tier Requirement exceeds this Tunnel's
/// tier are visible but greyed out", "its TechPrerequisite (if any) is satisfied anywhere in the Tunnel Network"): with
/// the bank and the place, a Tier 1 Tunnel is refused it for its tier ([`Reject::TierRequirement`], 2), a Tier 2
/// one without a Chemical Plant for the Plant ([`Reject::Requires`]); with the Plant at Tier 2 it is taken, and 249
/// Space Crystals are too few. Sent anyway, a refused Expand starts nothing and costs nothing.
#[test]
fn the_expand_is_refused_at_tier_1_and_without_a_chemical_plant() {
    let tier_1 = started(plant(), 1);
    let refused = Reject::TierRequirement {
        kind: ASSEMBLY,
        tier: 2,
    };
    assert_eq!(gate(&tier_1, P1, expand(T1, VA_AT)), Err(refused), "Tier 1");
    let mut no_plant = started(base(), 2);
    let refused = Reject::Requires(ObjectKind::ChemicalPlant);
    assert_eq!(
        gate(&no_plant, P1, expand(T1, VA_AT)),
        Err(refused),
        "no Chemical Plant"
    );
    no_plant.at(2, P1, expand(T1, VA_AT));
    run_checked(&mut no_plant, 2);
    assert_eq!(object(&no_plant, T1).tunnel.as_ref().unwrap().operation, None);
    assert_eq!(bank(&no_plant, P1), BANK);
    let mut t = started(plant(), 2);
    assert_eq!(gate(&t, P1, expand(T1, VA_AT)), Ok(()), "Tier 2, with a Chemical Plant");
    t.debug_at(2, crystals(P1, PRICE - 1));
    run_checked(&mut t, 2);
    assert_eq!(gate(&t, P1, expand(T1, VA_AT)), Err(Reject::InsufficientCrystals));
}

/// The plan's acceptance, "At Tier 2 it builds at 250/480" (design change 0254: "**Paid when the ghost is placed**",
/// "the ConstructionHP Rule applies to every expansion (starts at 10%, grows with progress)"): the click pays 250
/// Space Crystals and starts the site, player 1's, at 60 of its HP; its maximum grows every tick, and the site is
/// complete 480 ticks after the click, at its 600 HP, the Tunnel free again. Until then it takes no Train (its
/// production held back, [`Reject::NoProducers`]); then it trains a Bike.
#[test]
fn a_tier_2_tunnel_builds_it_for_250_in_480_frames() {
    let mut t = started(plant(), 2);
    t.at(2, P1, expand(T1, VA_AT));
    run_checked(&mut t, 2);
    assert_eq!(bank(&t, P1), BANK - PRICE, "paid at the click");
    let site = building(&t, T1);
    let o = object(&t, site);
    assert_eq!(
        (o.kind, o.owner, o.is_site(), o.placement.map(|p| p.origin)),
        (ASSEMBLY, Some(P1), true, Some(VA_AT))
    );
    assert_eq!((o.hp, o.max_hp()), (Some(60.0), Some(60.0)), "10% of 600");
    let started = (t.events().iter()).any(|(_, e)| *e == SimEvent::ExpansionStarted { tunnel: T1, site });
    assert!(started);
    assert_eq!(gate(&t, P1, train_bike(site)), Err(Reject::NoProducers));
    let mut last = object(&t, site).max_hp();
    while t.sim().tick() < 2 + FRAMES - 1 {
        step_checked(&mut t);
        let max = object(&t, site).max_hp();
        assert!(max > last, "tick {}: {max:?} after {last:?}", t.sim().tick());
        last = max;
    }
    assert!(object(&t, site).is_site(), "not before its 480 frames");
    step_checked(&mut t);
    let o = object(&t, site);
    assert_eq!(
        (t.sim().tick(), o.is_site(), o.hp, o.max_hp()),
        (2 + FRAMES, false, Some(600.0), Some(600.0))
    );
    let completed = (t.events().iter()).any(|(_, e)| *e == SimEvent::ExpansionCompleted { tunnel: T1, site });
    assert!(completed);
    assert_eq!(
        object(&t, T1).tunnel.as_ref().unwrap().operation,
        None,
        "the Tunnel is free"
    );
    assert_eq!(t.sim().parent_tunnel(site), Some(T1));
    assert_eq!(gate(&t, P1, train_bike(site)), Ok(()));
}

/// The plan's acceptance, "with armour 6/54, regenerating 1%/s": player 2's Bike throws its mortar at the ground over
/// player 1's Vehicle Assembly, which player 2 can't see (an Attack Ground). Each round, its 48 su circle wholly over
/// the 3 × 3, does its 120 less the FullArmor of 54 scaled by the share of the Assembly the circle covers
/// (`systems/combat.md > AoEDamage`: "FullArmor × (unit overlap area / unit total area)"), 120 − 54 × 7238.2 / 36864,
/// about 109.4; and in every tick without a blast the Assembly regains 0.375 HP, 1% of its 600 a second, under fire
/// too. Its thrower called off, it climbs back to its full 600.
#[test]
fn it_takes_its_armours_share_of_a_blast_and_regenerates_1_percent_a_second() {
    let mut t = assembly();
    let centre = object(&t, VA).pos;
    assert_eq!(centre, Vec2::new(672.0, 544.0));
    let thrower = id(next_id(&t));
    t.spawn_unit_at(2, BIKE, P2, centre - Vec2::new(300.0, 0.0));
    t.at(3, P2, order(&[thrower], OrderKind::AttackGround(centre)));
    let circle = PI * 48.0 * 48.0;
    let blast = 120.0 - 54.0 * circle / (192.0 * 192.0);
    assert!((blast - 109.4).abs() < 0.01, "{blast}");
    let (mut blasts, mut regenerated) = (0, 0);
    let mut last = hp(&t, VA);
    run_checked(&mut t, 3);
    for _ in 0..200 {
        step_checked(&mut t);
        let now = hp(&t, VA);
        if now < last {
            // The economy phase, after the combat's: its share of the blast, then a tick's regeneration.
            blasts += 1;
            assert!((last - blast + REGEN - now).abs() < 0.01, "a blast: {last} to {now}");
        } else if blasts == 0 {
            assert_eq!(now, 600.0, "full until hit");
        } else {
            assert!((now - last - REGEN).abs() < 1e-3, "{last} to {now}");
            regenerated += 1;
        }
        last = now;
    }
    assert!(
        blasts >= 3 && regenerated >= 100,
        "{blasts} blasts, {regenerated} ticks regenerating"
    );
    // Called off; a round already on its way lands all the same.
    t.at(t.sim().tick() + 1, P2, order(&[thrower], OrderKind::Stop));
    run_for(&mut t, 60);
    let missing = 600.0 - hp(&t, VA);
    run_for(&mut t, (missing / REGEN).ceil() as Tick + 1);
    assert_eq!(hp(&t, VA), 600.0, "back to full");
}
