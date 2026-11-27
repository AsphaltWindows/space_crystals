//! A Tunnel builds the Null Chamber (`factions/syndicate/structures/null_chamber.md`: "Tier Requirement - 2",
//! "TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)", "Size 2x2; MaxHP 400;
//! PointArmor 6, FullArmor 24 (underground expansion, n = 6); Cost 200 Space Crystals; Build Time 480 frames (30
//! seconds)"), by `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254: "**Paid
//! when the ghost is placed**", "the ConstructionHP Rule applies to every expansion (starts at 10%, grows with
//! progress)"); and it serves no one until it is finished.

use sc_sim::tunnel::Operation;

use super::*;

/// A free place for a Null Chamber in [`base`]'s Tier 2 area but not its Tier 1 one: (9, 11) to (10, 12).
const CHAMBER_FREE: TilePos = TilePos::new(9, 11);

/// [`base`] with player 1's Hideout ([`HIDEOUT_AT`], id 2), the Null Chamber's prerequisite, if `hideout`.
fn with_hideout(hideout: bool) -> ScenarioBuilder {
    match hideout {
        true => base().spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(HIDEOUT_AT)),
        false => base(),
    }
}

/// The site `tunnel` builds; panics if it builds none.
fn building(t: &TestSim, tunnel: ObjectId) -> ObjectId {
    match object(t, tunnel).tunnel.as_ref().and_then(|state| state.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("Tunnel {tunnel:?} builds nothing: {other:?}"),
    }
}

/// A Tier 1 Tunnel is refused the Null Chamber for its tier ([`Reject::TierRequirement`], 2), a Tier 2 one without a
/// Hideout for the Hideout ([`Reject::Requires`]); with one, a Tier 2 Tunnel builds it in its Tier 2 area for 200 Space
/// Crystals, paid at the click, in 480 frames, from 40 of its 400 HP, and it takes no command (it has no production: a
/// Train there finds no producer). A Guard inside all the while takes no turn while it is a site ("While the player
/// owns at least one Null Chamber": one in operation, `Object::is_operational`), and takes its first tick in the tick
/// the chamber is finished.
#[test]
fn a_tier_2_tunnel_with_a_hideout_builds_the_null_chamber_for_200_in_480_frames() {
    let tier_1 = started(with_hideout(true), 1);
    let refused = Reject::TierRequirement { kind: CHAMBER, tier: 2 };
    let command = expand(T1, CHAMBER, CHAMBER_FREE);
    assert_eq!(gate(&tier_1, P1, command.clone()), Err(refused), "Tier 1");
    let no_hideout = started(with_hideout(false), 2);
    let refused = Reject::Requires(ObjectKind::Hideout);
    assert_eq!(gate(&no_hideout, P1, command.clone()), Err(refused), "no Hideout");
    let mut t = started(with_hideout(true), 2);
    let guard = guards_in(&mut t, 1)[0];
    assert_eq!(gate(&t, P1, command.clone()), Ok(()));
    let click = t.sim().tick() + 1;
    t.at(click, P1, command);
    run_checked(&mut t, click);
    assert_eq!(
        t.sim().player(P1).map(|p| p.crystals),
        Some(BANK - 200),
        "paid at the click"
    );
    let site = building(&t, T1);
    let o = object(&t, site);
    assert_eq!((o.kind, o.owner, o.is_site()), (CHAMBER, Some(P1), true));
    assert_eq!((o.hp, o.max_hp()), (Some(40.0), Some(40.0)), "10% of 400");
    assert_eq!(o.placement.map(|p| p.origin), Some(CHAMBER_FREE));
    while object(&t, site).is_site() {
        assert_eq!(turn(&t, guard), None, "tick {}: no turn at a site", t.sim().tick());
        step_checked(&mut t);
    }
    let o = object(&t, site);
    assert_eq!(
        (t.sim().tick(), o.hp, o.max_hp()),
        (click + 480, Some(400.0), Some(400.0))
    );
    let completed = SimEvent::ExpansionCompleted { tunnel: T1, site };
    assert!(t.events().iter().any(|(_, e)| *e == completed));
    assert_eq!(t.sim().parent_tunnel(site), Some(T1));
    assert_eq!(turn(&t, guard), Some(1), "from the tick it is finished");
    assert!(object(&t, site).production.is_none(), "no commands");
    assert_eq!(gate(&t, P1, train(site, GUARD)), Err(Reject::NoProducers));
}
