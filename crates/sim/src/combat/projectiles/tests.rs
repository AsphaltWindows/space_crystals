//! Plan §3.3 O's acceptance through the whole sim, with the test grenade (`combat::testing`): the flight, dodging, a
//! hidden Hideout under a blast and whom it reveals the thrower to, a shot whose thrower died, and the hash.

use glam::Vec2;

use super::*;
use crate::combat::testing::{GRENADE, grenadier};
use crate::command::DebugCommand;
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::math::PI;
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SHOOTER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The launches of the run so far: (tick, from, to, lands_at).
fn launches(t: &TestSim) -> Vec<(Tick, Vec2, Vec2, Tick)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched { from, to, lands_at, .. } => Some((*tick, from, to, lands_at)),
            _ => None,
        })
        .collect()
}

/// The blasts of the run so far: (tick, centre, radius).
fn blasts(t: &TestSim) -> Vec<(Tick, Vec2, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Blast { at, radius } => Some((*tick, at, radius)),
            _ => None,
        })
        .collect()
}

/// The HP damage `id` took in each hit of the run so far: (tick, damage).
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<(Tick, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit { target, damage, .. } if target == id => Some((*tick, damage)),
            _ => None,
        })
        .collect()
}

/// Runs until the first launch, and returns its tick.
fn until_launched(t: &mut TestSim) -> Tick {
    while launches(t).is_empty() {
        assert!(t.sim().tick() < 200, "nothing was launched\n{}", t.ascii(None));
        t.step();
    }
    t.sim().tick()
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Plan §3.3 O's acceptance, the flight: "A shot at 4 GU lands 32 ticks after launch (256 su at 8 su per tick: 0260's
/// 'about 2 s over 4 GU')". An Attack Ground 256 su east of the thrower: the projectile leaves from its centre at the
/// end of Firing, flies 32 ticks, and its blast hits the Peacekeeper of player 2 standing there with its share less its
/// FullArmor, all of it inside the circle. The store holds the projectile just as long; the order completes in the
/// tick after the blast, the first phase 5a that finds nothing of it in flight, having fired once.
#[test]
fn a_shot_at_four_grid_units_lands_32_ticks_after_it_leaves() {
    let _grenadier = grenadier(GRENADE);
    let from = Vec2::new(200.0, 320.0);
    let at = from + Vec2::new(256.0, 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at)
        .order_at(2, P1, &[SHOOTER], OrderKind::AttackGround(at), false);
    let launched = until_launched(&mut t);
    assert_eq!(flight_ticks(from, at, Some(8.0)), 32);
    assert_eq!(launches(&t), [(launched, from, at, launched + 32)]);
    assert_eq!(t.sim().projectiles().len(), 1);
    t.run_until(launched + 31);
    assert_eq!(t.sim().projectiles().len(), 1, "still in flight");
    assert!(blasts(&t).is_empty());
    t.step();
    assert_eq!(blasts(&t), [(launched + 32, at, 40.0)]);
    assert!(t.sim().projectiles().is_empty());
    let peacekeeper = ObjectKind::Peacekeeper.def();
    let share = 100.0 * (24.0 * 24.0 / (PI * 40.0 * 40.0));
    assert_eq!(hits_on(&t, TARGET), [(launched + 32, share - peacekeeper.armor.full)]);
    assert_eq!(
        current(&t, SHOOTER),
        Some(OrderKind::AttackGround(at)),
        "until the blast"
    );
    t.step();
    assert_eq!(current(&t, SHOOTER), None, "complete");
    t.run(100);
    assert_eq!(launches(&t).len(), 1, "it fires once");
}

/// Plan §3.3 O's acceptance, dodging: "A target walking away at 5 su per tick is clear of the 40 su circle when it lands,
/// and takes nothing" (design change 0260: "a walker escapes"). The thrower's Attack locks where the Peacekeeper stood at
/// the end of its Aiming; the Peacekeeper walks off as the grenade leaves, and the blast lands behind it.
#[test]
fn a_target_that_walks_off_is_clear_when_it_lands() {
    let _grenadier = grenadier(GRENADE);
    let from = Vec2::new(200.0, 320.0);
    let stood = from + Vec2::new(256.0, 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, stood)
        .order_at(2, P1, &[SHOOTER], OrderKind::Attack(TARGET), false);
    let launched = until_launched(&mut t);
    assert_eq!(launches(&t)[0].2, stood, "locked where it stood");
    let away = Target::Point(Vec2::new(1000.0, 320.0));
    t.order_at(launched + 1, P2, &[TARGET], OrderKind::Move(away), false)
        .run_until(launched + 32);
    assert_eq!(blasts(&t), [(launched + 32, stood, 40.0)]);
    assert!(t.pos(2).x - stood.x > 40.0 + 12.0, "clear of the circle: {}", t.pos(2));
    assert_eq!(hits_on(&t, TARGET), []);
    assert_eq!(t.sim().object(TARGET).unwrap().hp, ObjectKind::Peacekeeper.def().max_hp);
}

/// Plan §3.3 O's acceptance, hidden information, in the sim: player 2 throws a grenade at the ground over player 1's
/// Hideout, which player 2 can't see (plan decision T32). The Hideout takes 100 − 24 × 5026.5 / 16384 ≈ 92.6, and the
/// thrower, which player 1 doesn't see, is revealed to player 1 at the blast (`// Placeholder: Q0083 point 2`), for
/// the reveal's 24 ticks. Player 2 still sees nothing of the Hideout.
#[test]
fn a_blast_over_a_hidden_hideout_reveals_the_thrower_to_its_owner() {
    let _grenadier = grenadier(GRENADE);
    let mut t = ScenarioBuilder::new(&open())
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(TilePos::new(8, 4)))
        .start();
    let hideout = t.sim().objects().next().unwrap().id;
    let centre = t.sim().object(hideout).unwrap().pos;
    let shooter = ObjectId(hideout.0 + 1);
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P2, centre - Vec2::new(64.0 + 180.0, 0.0))
        .order_at(2, P2, &[shooter], OrderKind::AttackGround(centre), false);
    until_launched(&mut t);
    let revealed = |t: &TestSim| t.sim().vision().player(P1).unwrap().is_revealed(shooter);
    t.run_until(t.sim().projectiles()[0].lands_at - 1);
    assert!(!revealed(&t), "not before the blast");
    t.step();
    assert_eq!(blasts(&t).len(), 1);
    let [(_, taken)] = hits_on(&t, hideout)[..] else {
        panic!("one hit on the Hideout");
    };
    assert_eq!((taken * 10.0).round(), 926.0, "{taken}");
    assert!(revealed(&t), "revealed at the blast");
    let hideout_object = t.sim().object(hideout).unwrap().clone();
    assert!(!t.sim().is_visible(P2, &hideout_object));
    t.run(24);
    assert!(!revealed(&t), "for 24 ticks");
}

/// Plan decision T45: "a projectile lands even if its shooter died". The thrower is removed as its grenade flies: the
/// blast still hits the Peacekeeper, and reveals no one (`// Placeholder: Q0083 point 2`: the thrower "if still
/// alive").
#[test]
fn a_grenade_lands_after_its_thrower_is_gone() {
    let _grenadier = grenadier(GRENADE);
    let from = Vec2::new(200.0, 320.0);
    let at = from + Vec2::new(200.0, 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at)
        .order_at(2, P1, &[SHOOTER], OrderKind::Attack(TARGET), false);
    let launched = until_launched(&mut t);
    t.debug_at(launched + 1, DebugCommand::Remove { id: SHOOTER }).run(1);
    assert!(t.sim().object(SHOOTER).is_none());
    let p = t.sim().projectiles()[0].clone();
    assert_eq!((p.shooter, p.target), (SHOOTER, Some(TARGET)));
    t.run_until(p.lands_at);
    assert_eq!(hits_on(&t, TARGET).len(), 1);
    assert!(!t.sim().vision().player(P2).unwrap().is_revealed(SHOOTER));
}

/// The removal path: a projectile aimed at an object that leaves the game no longer names it, and still lands where
/// it was locked.
#[test]
fn a_projectile_forgets_its_target_and_flies_on() {
    let mut store = vec![Projectile {
        shooter: ObjectId(1),
        owner: Some(P1),
        target: Some(ObjectId(7)),
        from: Vec2::ZERO,
        to: Vec2::new(100.0, 0.0),
        lands_at: 20,
        damage: 100.0,
        radius: 40.0,
        domain: TargetDomain::Ground,
    }];
    forget(&mut store, &[ObjectId(3), ObjectId(9)]);
    assert_eq!(store[0].target, Some(ObjectId(7)));
    forget(&mut store, &[ObjectId(1), ObjectId(7)]);
    assert_eq!((store[0].shooter, store[0].target), (ObjectId(1), None));
}

/// Plan decision T45: the flight takes the straight distance over the speed, rounded up to whole ticks; none without
/// a speed, or to the shooter's own centre.
#[test]
fn the_flight_is_the_distance_over_the_speed_rounded_up() {
    let o = Vec2::new(10.0, 10.0);
    assert_eq!(flight_ticks(o, o + Vec2::new(256.0, 0.0), Some(8.0)), 32);
    assert_eq!(flight_ticks(o, o + Vec2::new(256.5, 0.0), Some(8.0)), 33);
    assert_eq!(flight_ticks(o, o + Vec2::new(30.0, 40.0), Some(10.0)), 5);
    assert_eq!(flight_ticks(o, o, Some(8.0)), 0);
    assert_eq!(flight_ticks(o, o + Vec2::X, None), 0);
}

/// The goldens' guard (Director's note 4): the store is hashed and serialized only while it holds a projectile. A sim
/// with none hashes and saves as one from before the store existed; one with a projectile differs, and loads back.
#[test]
fn the_store_counts_only_while_a_projectile_flies() {
    let _grenadier = grenadier(GRENADE);
    let from = Vec2::new(200.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, from)
        .order_at(
            2,
            P1,
            &[SHOOTER],
            OrderKind::AttackGround(from + Vec2::new(150.0, 0.0)),
            false,
        );
    let json = |t: &TestSim| serde_json::to_value(t.sim()).unwrap();
    assert!(json(&t).get("projectiles").is_none());
    until_launched(&mut t);
    let saved = json(&t);
    assert_eq!(saved["projectiles"].as_array().map(Vec::len), Some(1));
    let loaded: crate::sim::Sim = serde_json::from_value(saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(loaded.projectiles(), t.sim().projectiles());
}
