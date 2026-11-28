//! The Dragonfly's missile (`factions/gdo/units/dragonfly.md > AttackAttributes`): "Attack: **HeadDisjointed** (homing
//! air-to-air missile)", "TargetDomain: Air", "Damage **90**", "Ammo: **1 missile**, then it must reload at its
//! assigned Airfield", "**RearmTimePerAmmo 64 frames**"; "No turret (fires forward)", which follows the Sparrow's rule
//! (`m6-plan.md > GL > Placeholders`: its attack is its base's, `combat::turret::Source::Base`, fired once its heading
//! faces its target); `factions/gdo/structures/airfield.md > Returning and rearming`: "**Automatic return**: when an
//! aircraft runs out of ammo, it automatically flies back to its assigned Airfield, lands and rearms".

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
