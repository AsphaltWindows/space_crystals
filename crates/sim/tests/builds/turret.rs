//! The Laser Turret an Agent builds (slice M4-S1; `factions/syndicate/structures/laser_turret.md > Construction`: "built
//! using the same flow as Tunnel construction"): its TechPrerequisite, its price of 100 Space Crystals paid with the
//! order and refunded in full, its site with the Agent inside, its 320 frames, and its Agent coming out beside it, or
//! waiting inside while every spot beside it is taken; the site holds its fire until complete. The placeholders are
//! question Q0084's (points 4 to 7).

use super::*;
use sc_sim::construction::site::BuildPrice;
use sc_sim::defs::Resource;
use sc_sim::units::SU_PER_GU;

/// The turret's site, a tile south-east of [`base`]'s Tunnel, and the centre of its footprint.
const TURRET_SITE: TilePos = TilePos::new(16, 10);
const TURRET_CENTRE: Vec2 = Vec2::new(16.5 * 64.0, 10.5 * 64.0);

/// Player 1's Optics Lab in [`field`], in its Tunnel's area.
const LAB: ObjectId = ObjectId(3);

/// "Build time: 320 frames (20 s)".
const FRAMES: u32 = 320;

/// How far from the turret's centre, along each axis, an Agent stands beside it: half the footprint, half the Agent's
/// box, and the margin of a unit against a wall (`network::Paths`).
const BESIDE: f32 = SU_PER_GU / 2.0 + 18.0 + 1.0 / 64.0;

/// [`base`] with player 1's Agent 2 south-west of [`TURRET_SITE`], 150 Space Crystals and 5 Supplies from tick 1, and,
/// if `lab`, its Optics Lab (3) in its Tunnel's area; with a `guard`, player 2's Guard holding 4.5 grid units east of
/// the turret's tile, beyond the Guard's own Range and in the turret's.
fn field(lab: bool, guard: bool) -> TestSim {
    let mut builder = base().spawn(AGENT, Some(P1), point(12, 16)); // 2
    if lab {
        builder = builder.spawn(ObjectKind::OpticsLab, Some(P1), tile(2, 6)); // 3
    }
    let mut t = builder.start();
    let banks = DebugCommand::SetResources {
        player: P1,
        crystals: 150,
        supplies: 5,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, banks);
    if guard {
        let at = SpawnAt::Point(Vec2::new(17.0 * 64.0 + 288.0 + 18.0, TURRET_CENTRE.y));
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind: GUARD,
                owner: Some(P2),
                at,
                orientation: Default::default(),
            },
        );
    }
    step(&mut t);
    if guard {
        let guard = t.sim().objects().find(|o| o.kind == GUARD).expect("the Guard").id;
        next(
            &mut t,
            P2,
            Command::Order {
                units: vec![guard],
                kind: OrderKind::Hold,
                queue: false,
            },
        );
    }
    t
}

/// A Build of a Laser Turret at `origin` to `units`, without Shift.
fn build_turret(units: &[u32], origin: TilePos) -> Command {
    let orientation = BuildingOrientation::default();
    let kind = OrderKind::Build {
        kind: TURRET,
        origin,
        orientation,
    };
    order(units, kind, false)
}

/// Player `player`'s Space Crystals.
fn crystals(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

/// The shots of `attacker` so far: (tick, target).
fn shots(t: &TestSim, attacker: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Attacked {
                attacker: a, target, ..
            } if a == attacker => Some((*tick, target)),
            _ => None,
        })
        .collect()
}

/// `factions/syndicate/structures/laser_turret.md`: "TechPrerequisite - player owns at least one Optics Lab", checked
/// as the order is given (`core/tech_and_research.md > Tech Prerequisites > Check Timing`); "Cost: 100 Space Crystals
/// (no Supplies)", which the bank must hold (design change 0233: "The order is refused, with feedback, if the player
/// can't pay"; the client greys its D for either, with the gate's reason).
#[test]
fn the_turret_needs_an_optics_lab_and_100_space_crystals() {
    let t = field(false, false);
    assert_eq!(
        gate(&t, P1, build_turret(&[2], TURRET_SITE)),
        Err(Reject::Requires(ObjectKind::OpticsLab))
    );
    let mut t = field(true, false);
    let price = BuildPrice {
        resource: Resource::Crystals,
        amount: 100,
    };
    assert_eq!(t.sim().build_price(P1, TURRET), Some(price));
    assert_eq!(gate(&t, P1, build_turret(&[2], TURRET_SITE)), Ok(()));
    let short = DebugCommand::SetResources {
        player: P1,
        crystals: 99,
        supplies: 5,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    next_debug(&mut t, short);
    assert_eq!(
        gate(&t, P1, build_turret(&[2], TURRET_SITE)),
        Err(Reject::InsufficientCrystals)
    );
}

/// The Tunnel's flow with the turret's numbers: the order pays 100 Space Crystals at once, and no Supplies (question
/// Q0084 point 4, a placeholder). On arrival the site appears with the Agent inside, off the map and so untargetable,
/// at 30 HP, 10% of 300 (`core/entities.md > ConstructionHP Rule`); it holds its fire at player 2's Guard in its range
/// (design change 0234: a site "does nothing but Cancel"), and goes on though the Optics Lab is lost ("Constructions
/// already started ... continue"). 320 ticks later it is a turret in operation, and the Agent stands beside it, at the
/// corner it came from, south-west (Q0084 point 7, a placeholder), idle and facing away; the turret then fires.
#[test]
fn the_order_pays_100_and_the_site_grows_into_a_turret_with_the_agent_beside_it() {
    let mut t = field(true, true);
    let guard = t.sim().objects().find(|o| o.kind == GUARD).expect("the Guard").id;
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    assert_eq!((crystals(&t, P1), supplies(&t, P1)), (50, 5));
    assert_eq!(ghosts(&t, P1), [(id(2), TURRET_SITE)]);

    let (started, site) = until_started(&mut t, 2);
    let s = object(&t, site);
    assert_eq!((s.kind, s.owner), (TURRET, Some(P1)));
    assert_eq!(s.placement.map(|p| (p.origin, p.w, p.h)), Some((TURRET_SITE, 1, 1)));
    assert_eq!((s.pos, s.hp, s.max_hp()), (TURRET_CENTRE, Some(30.0), Some(30.0)));
    assert!(s.is_site() && !s.is_operational());
    assert_eq!(t.sim().held_unit(id(2)).map(|held| held.place), Some(Place::Site(site)));
    assert!(t.sim().object(id(2)).is_none(), "the Agent is off the map");
    next_debug(&mut t, DebugCommand::Remove { id: LAB });
    assert!(t.sim().object(LAB).is_none());

    run_until(&mut t, started + FRAMES - 1);
    assert!(object(&t, site).is_site(), "still a site");
    assert_eq!(shots(&t, site), [], "a site holds its fire");
    step(&mut t);
    let done = started + FRAMES;
    let s = object(&t, site);
    assert!(!s.is_site() && s.is_operational());
    assert_eq!((s.hp, s.max_hp()), (Some(300.0), Some(300.0)));
    assert!(
        t.events()
            .contains(&(done, SimEvent::SiteCompleted { site, worker: id(2) }))
    );
    assert!(t.events().contains(&(done, SimEvent::LeftSite { worker: id(2), site })));
    assert_eq!(t.sim().held_unit(id(2)), None);
    assert_eq!(
        t.pos(2),
        TURRET_CENTRE + Vec2::new(-BESIDE, BESIDE),
        "the south-west corner"
    );
    let agent = object(&t, id(2));
    assert!(
        (agent.facing - 3.0 * std::f32::consts::FRAC_PI_4).abs() < 1e-5,
        "{}",
        agent.facing
    );
    assert!(idle(&t, 2));
    assert_eq!((crystals(&t, P1), supplies(&t, P1), refunds(&t)), (50, 5, vec![]));

    step_until(&mut t, 40, "the turret fires", |t| !shots(t, site).is_empty());
    assert_eq!(shots(&t, site)[0].1, guard);
}

/// Cancel Construction gives back the whole 100 Space Crystals (question Q0084 point 5, a placeholder: the price has no
/// scaling, so the "current top price" is the price), and the Agent comes out at the centre of the footprint
/// (`factions/syndicate/units/agent.md > While the Tunnel is under construction`).
#[test]
fn a_cancelled_site_refunds_100_and_lets_the_agent_out_at_its_centre() {
    let mut t = field(true, false);
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    let (_, site) = until_started(&mut t, 2);
    assert_eq!(crystals(&t, P1), 50);
    next(&mut t, P1, cancel(&[site]));
    let tick = t.sim().tick();
    let cancelled = SimEvent::SiteCancelled {
        site,
        kind: TURRET,
        refund: 100,
    };
    assert!(t.events().contains(&(tick, cancelled)));
    assert!(t.events().contains(&(tick, SimEvent::LeftSite { worker: id(2), site })));
    assert_eq!((crystals(&t, P1), supplies(&t, P1), sites(&t)), (150, 5, vec![]));
    assert_eq!(t.pos(2), TURRET_CENTRE);
    assert!(idle(&t, 2));
}

/// "if the structure is destroyed or cancelled mid-construction the Agent survives and emerges at the location", and
/// the Space Crystals are lost, as a Tunnel's Supplies are: player 2's Guards destroy the site.
#[test]
fn a_destroyed_site_loses_its_crystals_and_lets_the_agent_out() {
    let mut t = field(true, false);
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    let (_, site) = until_started(&mut t, 2);
    for (x, y) in [(18, 10), (14, 10)] {
        let spawn = DebugCommand::Spawn {
            kind: GUARD,
            owner: Some(P2),
            at: point(x, y),
            orientation: Default::default(),
        };
        t.debug_at(t.sim().tick() + 1, spawn);
    }
    step(&mut t);
    next(
        &mut t,
        P2,
        order(&[site.0 + 1, site.0 + 2], OrderKind::Attack(site), false),
    );
    step_until(&mut t, 200, "the Guards destroy the site", |t| {
        t.sim().object(site).is_none()
    });
    let died = (t.events().iter()).any(|(_, e)| matches!(*e, SimEvent::Died { id: dead, .. } if dead == site));
    assert!(died);
    assert!(
        t.events()
            .iter()
            .any(|(_, e)| *e == SimEvent::LeftSite { worker: id(2), site })
    );
    assert_eq!(t.pos(2), TURRET_CENTRE);
    assert_eq!((crystals(&t, P1), refunds(&t)), (50, vec![]));
}

/// An order that goes before its site starts is refunded in full (question Q0084 point 5), in its own resource: the
/// refund of a turret's Build pays toward the turret's Build that replaces it (design change 0233's rule for the
/// Tunnel's), and a Stop gives the 100 Space Crystals back; a turret's Build that replaces a Tunnel's gives the
/// Tunnel's Supply back and takes the 100 Space Crystals, the Supply paying none of them.
#[test]
fn an_order_that_goes_refunds_its_own_resource() {
    let mut t = field(true, false);
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    assert_eq!(crystals(&t, P1), 50);
    let elsewhere = at(16, 12);
    assert_eq!(
        gate(&t, P1, build_turret(&[2], elsewhere)),
        Ok(()),
        "its own 100 back pay for it"
    );
    next(&mut t, P1, order(&[2], OrderKind::Stop, false));
    assert_eq!((crystals(&t, P1), refunds(&t)), (150, vec![(id(2), 100)]));

    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!((crystals(&t, P1), supplies(&t, P1)), (150, 4));
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    assert_eq!((crystals(&t, P1), supplies(&t, P1)), (50, 5));
    next(&mut t, P1, build(&[2], SITE, false));
    assert_eq!((crystals(&t, P1), supplies(&t, P1)), (150, 4));
    let set = DebugCommand::SetResources {
        player: P1,
        crystals: 99,
        supplies: 4,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    next_debug(&mut t, set);
    assert_eq!(
        gate(&t, P1, build_turret(&[2], TURRET_SITE)),
        Err(Reject::InsufficientCrystals),
        "the Tunnel's Supply pays no Space Crystals"
    );
}

/// [`field`] with Agent 2's turret site, and player 1's eight debug-spawned turrets on the tiles around it, taking every
/// spot beside it, until it is complete: the site and the turrets, in order from the north-west, row by row.
fn surrounded() -> (TestSim, ObjectId, Vec<ObjectId>) {
    let mut t = field(true, false);
    next(&mut t, P1, build_turret(&[2], TURRET_SITE));
    let (started, site) = until_started(&mut t, 2);
    let around: Vec<TilePos> = (9..=11)
        .flat_map(|y| (15..=17).map(move |x| at(x, y)))
        .filter(|&tile| tile != TURRET_SITE)
        .collect();
    for &tile in &around {
        let spawn = DebugCommand::Spawn {
            kind: TURRET,
            owner: Some(P1),
            at: SpawnAt::Tile(tile),
            orientation: Default::default(),
        };
        t.debug_at(t.sim().tick() + 1, spawn);
    }
    step(&mut t);
    let turrets: Vec<ObjectId> = (t.sim().objects())
        .filter(|o| o.kind == TURRET && o.id != site)
        .map(|o| o.id)
        .collect();
    assert_eq!(turrets.len(), 8);
    run_until(&mut t, started + FRAMES);
    assert!(object(&t, site).is_operational());
    (t, site, turrets)
}

/// Question Q0084 point 7 (a placeholder): "if none is clear, it waits inside until one is". With every spot beside it
/// taken, the Agent stays in the finished turret; once the turret north of it goes, it comes out there, in that tick.
#[test]
fn with_no_spot_beside_it_clear_the_agent_waits_inside() {
    let (mut t, site, turrets) = surrounded();
    assert_eq!(t.sim().held_unit(id(2)).map(|held| held.place), Some(Place::Site(site)));
    assert!(!t.events().iter().any(|(_, e)| matches!(e, SimEvent::LeftSite { .. })));
    step(&mut t);
    assert!(t.sim().object(id(2)).is_none(), "still waiting");
    let north = turrets[1];
    assert_eq!(object(&t, north).placement.map(|p| p.origin), Some(at(16, 9)));
    next_debug(&mut t, DebugCommand::Remove { id: north });
    let tick = t.sim().tick();
    assert!(t.events().contains(&(tick, SimEvent::LeftSite { worker: id(2), site })));
    assert_eq!(t.pos(2), TURRET_CENTRE + Vec2::new(0.0, -BESIDE));
}

/// A finished turret that leaves the game with its Agent still waiting inside lets it out at the centre of its
/// footprint, as a site does (the removal path's rule for a held unit, plan decision T30): nothing is lost with it.
#[test]
fn a_turret_that_goes_with_its_agent_inside_lets_it_out() {
    let (mut t, site, _) = surrounded();
    next_debug(&mut t, DebugCommand::Remove { id: site });
    let tick = t.sim().tick();
    assert!(t.events().contains(&(tick, SimEvent::LeftSite { worker: id(2), site })));
    assert_eq!(t.pos(2), TURRET_CENTRE);
    assert_eq!(t.sim().held_unit(id(2)), None);
}
