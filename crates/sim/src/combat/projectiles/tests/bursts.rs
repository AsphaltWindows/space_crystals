//! (M5-S8) The Rocketman's missiles' bursts (`factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`,
//! design change 0445, answering Q0173) through the whole sim, with its own def: the domain each keeps, of what it was
//! fired at (`area::blast_domain`), fixed as it leaves; and a locked one's whose target left the game in flight (the
//! placeholder of `super::super::homing`). The design change's Verification is played in
//! `crates/sim/tests/rocketman/bursts.rs`.

use glam::Vec2;

use super::{blasts, hits_on, launches, open, until_launched};
use crate::command::DebugCommand;
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, TargetDomain, UnitBase};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::testing::{ScenarioBuilder, TestSim};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ROCKETMAN: ObjectId = ObjectId(1);
/// Player 2's Peacekeeper, on the ground at [`AT`].
const GROUND: ObjectId = ObjectId(2);
/// Player 2's Militant on a Hover Craft base, an air unit (`defs::test_defs`), at [`OVER`], over the Peacekeeper.
const AIR: ObjectId = ObjectId(3);
const FROM: Vec2 = Vec2::new(200.0, 320.0);
/// 4 grid units east of the Rocketman: within its Range of 6, outside its MinRange of 2; and the air unit 16 su off it.
/// Each of the two 24 × 24 units lies wholly inside a burst of 32 su on the other's centre (the far corners 30.5 su
/// out).
const AT: Vec2 = Vec2::new(456.0, 320.0);
const OVER: Vec2 = Vec2::new(456.0, 336.0);

/// The Rocketman at [`FROM`], player 2's Peacekeeper at [`AT`] and its aircraft at [`OVER`], with `more` aircraft of
/// player 2's, auto-targeting off (nobody fights but on orders); the Rocketman given `order` in tick 2; run until its
/// first missile leaves.
fn launched(order: OrderKind, more: &[Vec2]) -> TestSim {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Rocketman, P1, FROM)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, AT)
        .spawn_unit_at(1, ObjectKind::Militant, P2, OVER);
    for &at in more {
        t.spawn_unit_at(1, ObjectKind::Militant, P2, at);
    }
    t.order_at(2, P1, &[ROCKETMAN], order, false);
    until_launched(&mut t);
    t
}

/// "**Domain of the burst** (user, 2026-11-26): it keeps the domain of what it was fired at: fired at an **air** unit it
/// hits only air units; fired at a **ground** unit or the ground, only ground (and underground) objects". Each of the
/// Rocketman's missiles bursts over both of player 2's units: fired at the Peacekeeper, a Ground burst, which hits it
/// and not the aircraft over it; at the aircraft, an Air burst, which hits it and not the Peacekeeper under it; at the
/// ground there (an Attack Ground), a Ground burst again. Each projectile holds its domain from its launch on.
#[test]
fn a_rocketmans_missile_bursts_in_the_domain_of_what_it_was_fired_at() {
    let _flyer = TestDef::unit(ObjectKind::Militant, |unit| unit.base = UnitBase::HoverCraft);
    for (order, spot, domain, hit, spared) in [
        (OrderKind::Attack(GROUND), AT, TargetDomain::Ground, GROUND, AIR),
        (OrderKind::Attack(AIR), OVER, TargetDomain::Air, AIR, GROUND),
        (OrderKind::AttackGround(AT), AT, TargetDomain::Ground, GROUND, AIR),
    ] {
        let mut t = launched(order, &[]);
        let [(_, _, to, lands_at)] = launches(&t)[..] else {
            panic!("{order:?}: one launch");
        };
        assert_eq!(to, spot, "{order:?}");
        assert_eq!(t.sim().projectiles()[0].domain, domain, "{order:?}: fixed as it leaves");
        t.run_until(lands_at);
        assert_eq!(blasts(&t), [(lands_at, spot, 32.0)], "{order:?}");
        assert_eq!(hits_on(&t, hit).len(), 1, "{order:?}");
        assert_eq!(hits_on(&t, spared), [], "{order:?}: the other domain's");
        assert_eq!(hits_on(&t, ROCKETMAN), [], "{order:?}");
    }
}

/// The placeholder of `super::super::homing` (M5-K's, as design change 0445 left it): a locked missile whose target
/// leaves the game in flight flies on to where it was and bursts there, in the target's domain, fixed as it left;
/// harmlessly or not, by the same rule. The Rocketman locks onto the aircraft over the Peacekeeper, with a second of
/// player 2's aircraft (#4) 26 su beside it; the first is removed as the locked missile flies, and the burst where it
/// was hits the second, in the air, and not the Peacekeeper under it.
#[test]
fn a_locked_missile_whose_target_left_bursts_where_it_was_in_its_domain() {
    let _flyer = TestDef::unit(ObjectKind::Militant, |unit| unit.base = UnitBase::HoverCraft);
    let beside = ObjectId(4);
    let mut t = launched(OrderKind::TargetLock(AIR), &[OVER + Vec2::new(26.0, 0.0)]);
    let launched = t.sim().tick();
    let [(_, _, to, lands_at)] = launches(&t)[..] else {
        panic!("one launch");
    };
    assert_eq!(to, OVER);
    let missile = &t.sim().projectiles()[0];
    assert!(missile.homing.is_some(), "a locked one");
    assert_eq!(missile.domain, TargetDomain::Air);
    t.debug_at(launched + 4, DebugCommand::Remove { id: AIR })
        .run_until(lands_at);
    assert!(t.sim().object(AIR).is_none(), "left the game");
    assert_eq!(blasts(&t), [(lands_at, OVER, 32.0)], "where it was");
    assert_eq!(hits_on(&t, beside).len(), 1, "the aircraft beside it");
    assert_eq!(hits_on(&t, GROUND), [], "not the Peacekeeper under it");
}
