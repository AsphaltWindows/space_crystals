//! The Dragonfly's missile (`factions/gdo/units/dragonfly.md > AttackAttributes`): "Attack: **HeadDisjointed** (homing
//! air-to-air missile)", "TargetDomain: Air", "Damage **90**", "Ammo: **1 missile**, then it must reload at its
//! assigned Airfield", "**RearmTimePerAmmo 64 frames**"; "No turret (fires forward)", which follows the Sparrow's rule
//! (`m6-plan.md > GL > Placeholders`: its attack is its base's, `combat::turret::Source::Base`, fired once its heading
//! faces its target); `factions/gdo/structures/airfield.md > Returning and rearming`: "**Automatic return**: when an
//! aircraft runs out of ammo, it automatically flies back to its assigned Airfield, lands and rearms".

use sc_sim::mover::Pass;

use super::*;

/// Player 2's Sparrow, which circles where it was spawned (its idle circle), within the Dragonfly's sight of [`A`]'s
/// first slot: "MaxHP 50", armour 0 (`factions/cults/units/sparrow.md`), so that one missile kills it.
const SPARROW: u32 = 7;
const SPARROW_AT: Vec2 = Vec2::new(1100.0, 500.0);
/// Player 1's Dragonfly, landed on [`A`]'s first slot.
const DRAGONFLY_ID: u32 = 8;

/// [`field`] with [`SPARROW`], and [`DRAGONFLY_ID`] landed on [`A`]'s first slot, home.
fn sparrow_and_dragonfly() -> TestSim {
    let mut t = field(|builder| builder.spawn(ObjectKind::Sparrow, Some(P2), at(SPARROW_AT.x, SPARROW_AT.y)));
    one_landed(&mut t, DRAGONFLY_ID);
    t
}

/// The plan's acceptance, "One missile shoots down a Sparrow. Then the Dragonfly flies home, lands in 16 ticks and
/// rearms in 64", and design change 0428's Verification: "A Dragonfly returns: it slows to IdleSpeed over its slot,
/// descends, rearms, lifts straight up". The Attack lifts it off; it flies at the Sparrow and fires its one missile as
/// its heading faces it, which kills it. Out of ammo, it flies home by itself (`OrderKind::ReturnToBase`), never
/// stopping in the air: it flies the last of its approach at IdleSpeed, stops over its slot, lands in 16 ticks and
/// rearms its missile in 64, from the tick it is down; its flight home over, it stays landed, still, until a Move lifts
/// it straight up.
#[test]
fn one_missile_shoots_down_a_sparrow_then_it_flies_home_lands_in_16_and_rearms_in_64() {
    let mut t = sparrow_and_dragonfly();
    let n = DRAGONFLY_ID;
    assert_eq!(ammo(t.sim(), n), AMMO);
    shoots_down(&mut t, n, SPARROW);
    assert_eq!(ammo(t.sim(), n), 0, "its one missile fired");
    assert_eq!(doing(t.sim(), n), Some(OrderKind::ReturnToBase), "home by itself");

    let frames = fly_down(&mut t, n, 600);
    lands_on(&frames, A_SLOTS[0], 10);
    let down_at = t.sim().tick();
    assert_eq!(ammo(t.sim(), n), 0);
    while ammo(t.sim(), n) < AMMO {
        assert!(t.sim().tick() < down_at + 2 * REARM, "rearmed");
        t.step();
        assert_eq!((t.pos(n), speed(t.sim(), n)), (A_SLOTS[0], 0.0), "still on its slot");
        assert_eq!(doing(t.sim(), n), None, "its flight home over as it came down");
    }
    assert_eq!(t.sim().tick() - down_at + 1, REARM, "its missile rearmed in 64 ticks");
    assert!(home(t.sim(), n));
    for _ in 0..100 {
        t.step();
        assert_eq!((t.pos(n), speed(t.sim(), n)), (A_SLOTS[0], 0.0), "it stays landed");
    }
    takes_off(&mut t, n, move_to(&[n], Vec2::new(1600.0, 900.0)));
}

/// "TargetDomain: Air": the gate refuses the Dragonfly an Attack on a ground unit (`Reject::CannotHit`), and an Attack
/// Ground; out of ammo, an Attack on an air unit too (`Reject::OutOfAmmo`), its right-click on one no longer attacking.
#[test]
fn it_hits_air_alone_and_with_its_missile_spent_nothing() {
    let mut t = field(|builder| {
        builder
            .spawn(ObjectKind::Sparrow, Some(P2), at(SPARROW_AT.x, SPARROW_AT.y))
            .spawn(ObjectKind::Peacekeeper, Some(P2), at(900.0, 420.0))
    });
    let (peacekeeper, n) = (8, 9);
    one_landed(&mut t, n);
    let ground = order(&[n], OrderKind::Attack(id(peacekeeper)));
    assert_eq!(gate(t.sim(), P1, &ground), Err(Reject::CannotHit(id(peacekeeper))));
    let spot = order(&[n], OrderKind::AttackGround(Vec2::new(900.0, 420.0)));
    assert_ne!(gate(t.sim(), P1, &spot), Ok(()), "air alone");
    let other = t.sim().default_order(id(n), Target::Object(id(peacekeeper)));
    assert!(!matches!(other, Some(OrderKind::Attack(_))), "{other:?}");
    assert_eq!(
        t.sim().default_order(id(n), Target::Object(id(SPARROW))),
        Some(OrderKind::Attack(id(SPARROW)))
    );

    // A second Sparrow, for its spent missile.
    t.spawn_unit_at(t.sim().tick() + 1, ObjectKind::Sparrow, P2, Vec2::new(1100.0, 300.0));
    t.step();
    let second = 10;
    shoots_down(&mut t, n, SPARROW);
    let attack = order(&[n], OrderKind::Attack(id(second)));
    assert_eq!(gate(t.sim(), P1, &attack), Err(Reject::OutOfAmmo));
    let click = t.sim().default_order(id(n), Target::Object(id(second)));
    assert!(!matches!(click, Some(OrderKind::Attack(_))), "{click:?}");
}

/// Design change 0470's Verification 1 and 2 (`factions/gdo/units/hornet.md > Placeholder stats`,
/// `factions/gdo/units/dragonfly.md > Placeholder stats`: limited-ammo aircraft fire "**from their base, with no
/// turret**, so they never spend their payload on enemies they pass on a Move (base engagement isn't active on
/// Move)"): with auto-targeting on, two Hornets and two Dragonflies fly a Move east straight over two of player 2's
/// Prospectors, which the Hornets can hit, and past its Sparrow circling just off their line, which the Dragonflies
/// can, and launch nothing; flown back on an Attack Move, both kinds engage.
#[test]
fn on_a_move_hornets_and_dragonflies_fire_nothing_and_on_an_attack_move_they_engage() {
    const LINE: f32 = 1450.0;
    let mut t = field(|builder| {
        builder
            .spawn(ObjectKind::Prospector, Some(P2), at(1100.0, LINE))
            .spawn(ObjectKind::Prospector, Some(P2), at(1220.0, LINE))
            .spawn(ObjectKind::Sparrow, Some(P2), at(1700.0, LINE + 160.0))
    });
    let tick = t.sim().tick() + 1;
    for (kind, dy) in [
        (ObjectKind::Hornet, -40.0),
        (ObjectKind::Hornet, 40.0),
        (DRAGONFLY, -120.0),
        (DRAGONFLY, 120.0),
    ] {
        t.spawn_unit_at(tick, kind, P1, Vec2::new(300.0, LINE + dy));
    }
    t.step();
    let (hornets, dragonflies) = ([10, 11], [12, 13]);
    let all = [10, 11, 12, 13];
    debug(&mut t, DebugCommand::SetAutoTargeting { on: true });
    let east = Vec2::new(2600.0, LINE);
    next(&mut t, P1, move_to(&all, east));
    step_until(&mut t, 900, "past them all", |sim| {
        all.iter().all(|&n| object(sim, n).pos.x > 2200.0)
    });
    for n in all {
        assert_eq!(launches(&t, n), [], "#{n} fired nothing on its Move");
    }
    let west = Vec2::new(300.0, LINE);
    next(&mut t, P1, order(&all, OrderKind::AttackMove(west)));
    let engaged = |t: &TestSim, units: &[u32]| units.iter().filter(|&&n| !launches(t, n).is_empty()).count();
    let until = t.sim().tick() + 900;
    while engaged(&t, &hornets) < hornets.len() || engaged(&t, &dragonflies) == 0 {
        assert!(t.sim().tick() < until, "both kinds engage: not by tick {until}");
        t.step();
    }
}

/// Design change 0470's Verification 3, "A Dragonfly attacking a circling Sparrow flies out, comes back head-on and
/// launches within a pass or two" (`systems/controls/command_behaviors.md > AttackingObject`, the GliderMovement
/// exception: past its target it flies on until the target's centre is out of range, then loops back): a Dragonfly in
/// the air just east of the Sparrow's idle circle, flying away from it, attacks it; it comes round, and launches its
/// missile within its first two passes (`mover::Pass::Run`, the Sparrow ahead of it).
#[test]
fn a_dragonfly_attacking_a_circling_sparrow_launches_within_two_passes() {
    let mut t = sparrow_and_dragonfly();
    let n = 9;
    t.spawn_unit_at(t.sim().tick() + 1, DRAGONFLY, P1, SPARROW_AT + Vec2::new(160.0, 0.0));
    t.step();
    assert_eq!(object(t.sim(), n).kind, DRAGONFLY);
    next(&mut t, P1, order(&[n], OrderKind::Attack(id(SPARROW))));
    let (mut runs, mut closest, mut was_run) = (0, Vec::<f32>::new(), false);
    let until = t.sim().tick() + 800;
    while launches(&t, n).is_empty() {
        assert!(
            t.sim().tick() < until,
            "no launch by tick {until}: {runs} passes, closest {closest:?} rad"
        );
        t.step();
        let run = mover(t.sim(), n).strafe.is_some_and(|strafe| strafe.pass == Pass::Run);
        if run && !was_run {
            runs += 1;
            closest.push(f32::MAX);
        }
        was_run = run;
        let (o, sparrow) = (object(t.sim(), n), object(t.sim(), SPARROW));
        let off = math::wrap_angle(math::vector_angle(sparrow.pos - o.pos) - o.facing).abs();
        if let Some(last) = closest.last_mut() {
            *last = last.min(off);
        }
    }
    assert!(
        (1..=2).contains(&runs),
        "launched in pass {runs} (closest {closest:?} rad)"
    );
}
