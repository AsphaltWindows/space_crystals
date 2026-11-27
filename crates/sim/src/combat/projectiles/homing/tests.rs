//! (M5-K) `systems/combat.md > HeadDisjointed` through the whole sim, with the test missile (`combat::testing`): the
//! flight at a target that stands still and at one that walks off, the shooter's Cooldown and Reloading while it flies,
//! a target that leaves the game in flight, and the store's files.

use glam::Vec2;

use super::super::tests::{blasts, hits_on, launches, open, until_launched};
use crate::combat::AttackPhase;
use crate::combat::testing::{MISSILE, grenadier};
use crate::command::DebugCommand;
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::Tick;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SHOOTER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);
const FROM: Vec2 = Vec2::new(200.0, 320.0);
/// Where the target stands: 256 su east of the shooter.
const AT: Vec2 = Vec2::new(456.0, 320.0);

/// The test missile's shooter at [`FROM`], attacking player 2's Peacekeeper at [`AT`], with auto-targeting off (the
/// Peacekeeper doesn't fight back), run until the missile leaves; and the tick it left in.
fn launched_at_a_peacekeeper() -> (TestSim, Tick) {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, FROM)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, AT)
        .order_at(2, P1, &[SHOOTER], OrderKind::Attack(TARGET), false);
    let launched = until_launched(&mut t);
    (t, launched)
}

/// Whether the launches of the run so far were all homing ones.
fn all_homing(t: &TestSim) -> bool {
    (t.events().iter()).all(|(_, e)| !matches!(e, SimEvent::ProjectileLaunched { homing: false, .. }))
}

/// The shooter's attack phase.
fn phase(t: &TestSim) -> Option<AttackPhase> {
    t.sim()
        .object(SHOOTER)
        .and_then(|o| o.attack.as_ref())
        .and_then(|a| a.phase)
}

/// "The Firing phase is short and spawns a projectile which tracks the target independently of the attacking unit's
/// further actions": at a target that stands still, the missile leaves from the shooter's centre at the end of Firing
/// for the target's, flies 8 su a tick straight at it, and lands on it after the straight flight's 32 ticks
/// (`super::super::flight_ticks`), though its shooter was told to Stop as it left: its blast of radius 0 at the
/// target's centre, and its hit on the target, its Damage less the target's PointArmor.
#[test]
fn a_homing_missile_at_a_target_that_stands_still_flies_straight_at_it() {
    let _missile = grenadier(MISSILE);
    let (mut t, launched) = launched_at_a_peacekeeper();
    assert_eq!(launches(&t), [(launched, FROM, AT, launched + 32)]);
    assert!(all_homing(&t));
    t.order_at(launched + 1, P1, &[SHOOTER], OrderKind::Stop, false);
    let flight = |t: &TestSim| t.sim().projectiles()[0].homing.expect("homing");
    assert_eq!((flight(&t).pos, flight(&t).prev, flight(&t).speed), (FROM, FROM, 8.0));
    t.step();
    assert_eq!((flight(&t).pos, flight(&t).prev), (FROM + Vec2::new(8.0, 0.0), FROM));
    t.run_until(launched + 31);
    assert_eq!(t.sim().projectiles().len(), 1, "still in flight");
    assert_eq!(hits_on(&t, TARGET), []);
    t.step();
    assert!(t.sim().projectiles().is_empty());
    assert_eq!(blasts(&t), [(launched + 32, AT, 0.0)]);
    let peacekeeper = ObjectKind::Peacekeeper.def();
    assert_eq!(hits_on(&t, TARGET), [(launched + 32, 20.0 - peacekeeper.armor.point)]);
    assert_eq!(launches(&t).len(), 1, "stopped");
}

/// "tracks the target independently of the attacking unit's further actions ... The projectile cannot miss": the
/// Peacekeeper walks east as the missile leaves, at 4 su a tick to the missile's 8, and the missile follows it and
/// lands on it, wherever it has got to, later than the straight flight would (the blast at its centre then), though
/// the target has left its shooter's sight by then, which ended the shooter's Attack (design change 0200). "The unit
/// enters Cooldown and Reloading while the projectile is in flight".
#[test]
fn a_homing_missile_follows_a_target_that_walks_off_and_hits_it() {
    let _missile = grenadier(MISSILE);
    let (mut t, launched) = launched_at_a_peacekeeper();
    let away = Target::Point(Vec2::new(1000.0, 320.0));
    t.order_at(launched + 1, P2, &[TARGET], OrderKind::Move(away), false)
        .run_until(launched + 1);
    assert_eq!(phase(&t), Some(AttackPhase::Cooldown));
    t.run_until(launched + MISSILE.cooldown_ticks + 2);
    assert_eq!(phase(&t), Some(AttackPhase::Reloading));
    assert_eq!(t.sim().projectiles().len(), 1, "while it flies");
    while hits_on(&t, TARGET).is_empty() {
        assert!(t.sim().tick() < launched + 200, "it never caught up\n{}", t.ascii(None));
        t.step();
    }
    let (hit, _) = hits_on(&t, TARGET)[0];
    assert!(hit > launched + 32, "later than at a still target: {hit}");
    let landed = blasts(&t)[0];
    assert_eq!((landed.0, landed.2), (hit, 0.0));
    assert_eq!(landed.1, t.pos(2), "where the target walked to");
    assert!(landed.1.x > AT.x + 64.0, "{}", landed.1);
    let attack = t.sim().object(SHOOTER).unwrap().attack.as_ref().unwrap();
    assert_eq!(attack.target, None, "its Attack ended meanwhile");
    assert!(all_homing(&t));
}

/// A target that leaves the game in flight (the module docs' placeholder): the Peacekeeper is removed while the missile
/// flies, and the missile flies on to where it stood, lands there in the straight flight's time, and hits nothing.
#[test]
fn a_homing_missile_whose_target_leaves_the_game_lands_where_it_stood_and_hits_nothing() {
    let _missile = grenadier(MISSILE);
    let (mut t, launched) = launched_at_a_peacekeeper();
    let remove = DebugCommand::Remove { id: TARGET };
    t.debug_at(launched + 10, remove).run_until(launched + 11);
    assert!(t.sim().object(TARGET).is_none());
    assert_eq!(t.sim().projectiles()[0].target, None, "forgotten");
    t.run_until(launched + 32);
    assert!(t.sim().projectiles().is_empty());
    assert_eq!(blasts(&t), [(launched + 32, AT, 0.0)]);
    let any_hit = t.events().iter().any(|(_, e)| matches!(e, SimEvent::Hit { .. }));
    assert!(!any_hit, "it hits nothing");
}

/// The goldens' guard: a homing projectile's flight is in the files and the state's digest, and loads back as it was.
#[test]
fn a_homing_missile_saves_and_loads_with_its_flight() {
    let _missile = grenadier(MISSILE);
    let (mut t, _) = launched_at_a_peacekeeper();
    t.step();
    let saved = serde_json::to_value(t.sim()).unwrap();
    assert!(saved["projectiles"][0].get("homing").is_some());
    let loaded: crate::sim::Sim = serde_json::from_value(saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(loaded.projectiles(), t.sim().projectiles());
}
