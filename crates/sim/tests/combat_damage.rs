//! Damage and death (`systems/combat.md > SingleTargetDamage`, `> DamageModifierOrder`, `> MinimumDamage`,
//! `ARCHITECTURE.md > Combat`): armor and the rugged-terrain bonus, the tick's hits applied together, death at 0 HP
//! and the one removal path that leaves no reference to the dead behind.

use sc_sim::combat::{self, AttackPhase};
use sc_sim::testing::{self, ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SimEvent, Target, Tick, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// 16 × 8 tiles; tiles (2..=3, 1..=2) are Rugged.
const MAP: &str = "
    ................
    ..rr............
    ..rr............
    ................
    ................
    ................
    ................
    ................
";

/// Spawns `units` (kind, owner, position) in tick 1, as ids 1, 2, …
fn spawned(units: &[(ObjectKind, PlayerId, Vec2)]) -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    for &(kind, owner, pos) in units {
        t.spawn_unit_at(1, kind, owner, pos);
    }
    t
}

fn ids(ids: &[u32]) -> Vec<ObjectId> {
    ids.iter().map(|&i| ObjectId(i)).collect()
}

fn attack(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], target: u32, queue: bool) {
    t.order_at(tick, player, &ids(units), OrderKind::Attack(ObjectId(target)), queue);
}

fn hp(t: &TestSim, id: u32) -> Option<f32> {
    t.sim().object(ObjectId(id)).and_then(|o| o.hp)
}

/// The damage of each hit on `target`, with its tick.
fn hits_on(t: &TestSim, target: u32) -> Vec<(Tick, f32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Hit {
                target: hit, damage, ..
            } if *hit == ObjectId(target) => Some((*tick, *damage)),
            _ => None,
        })
        .collect()
}

fn died(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Died { id, .. } => Some((*tick, *id)),
            _ => None,
        })
        .collect()
}

/// `> SingleTargetDamage`: (Damage × pre-armor multipliers − PointArmor) × directional multiplier, at least 0.5.
/// A Peacekeeper (Damage 10) takes 9 from a Guard's 80 HP (PointArmor 1); a Guard (Damage 6) takes 5 from a
/// Peacekeeper's 50, and on Rugged ground the Peacekeeper's RuggedTerrainDefenseBonus halves the damage before the
/// armor: 6 × 0.5 − 1 = 2. The Guard, heavy infantry, has no such bonus.
#[test]
fn armor_and_the_rugged_terrain_bonus() {
    let mut t = spawned(&[
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 100.0)), // 1, plain
        (ObjectKind::Guard, P2, Vec2::new(550.0, 100.0)),       // 2, plain
        (ObjectKind::Peacekeeper, P1, Vec2::new(160.0, 100.0)), // 3, Rugged
        (ObjectKind::Guard, P2, Vec2::new(160.0, 280.0)),       // 4, plain
        (ObjectKind::Guard, P2, Vec2::new(220.0, 160.0)),       // 5, Rugged
    ]);
    attack(&mut t, 2, P1, &[1], 2, false);
    attack(&mut t, 2, P2, &[2], 1, false);
    attack(&mut t, 2, P2, &[4], 3, false);
    attack(&mut t, 2, P1, &[3], 5, false);
    t.run_until(4);
    assert_eq!(hits_on(&t, 2), [(4, 9.0)]);
    assert_eq!(hits_on(&t, 1), [(4, 5.0)]);
    assert_eq!(hits_on(&t, 3), [(4, 2.0)], "rugged: halved before the armor");
    assert_eq!(hits_on(&t, 5), [(4, 9.0)], "heavy infantry: no bonus");
    assert_eq!(
        [hp(&t, 1), hp(&t, 2), hp(&t, 3), hp(&t, 5)],
        [Some(45.0), Some(71.0), Some(48.0), Some(71.0)]
    );
    // The attack event carries both positions, for the client's tracer.
    assert!(t.events().contains(&(
        4,
        SimEvent::Attacked {
            attacker: ObjectId(1),
            target: ObjectId(2),
            from: Vec2::new(400.0, 100.0),
            to: Vec2::new(550.0, 100.0),
        }
    )));
}

/// `> MinimumDamage`, `> DirectionalArmor` (no M1 unit has directional armor or out-armors a hit).
#[test]
fn minimum_damage_and_directional_armor() {
    assert_eq!(combat::hit_damage(3.0, 1.0, 5.0, 1.0), combat::MINIMUM_DAMAGE);
    assert_eq!(combat::hit_damage(3.0, 0.5, 1.0, combat::DIRECTIONAL_REAR), 0.625);
    assert_eq!(combat::hit_damage(10.0, 1.0, 2.0, combat::DIRECTIONAL_FRONT), 6.0);
    let facing_east = 0.0;
    assert_eq!(
        combat::directional_multiplier(facing_east, Vec2::new(-10.0, 1.0)),
        combat::DIRECTIONAL_REAR
    );
    assert_eq!(
        combat::directional_multiplier(facing_east, Vec2::new(0.0, 10.0)),
        combat::DIRECTIONAL_SIDE
    );
}

/// A Guard's hits (5) bring a Peacekeeper's 50 HP to exactly 0 on the tenth: that is death.
#[test]
fn zero_hp_is_death() {
    let mut t = spawned(&[
        (ObjectKind::Guard, P2, Vec2::new(400.0, 300.0)),
        (ObjectKind::Peacekeeper, P1, Vec2::new(550.0, 300.0)),
    ]);
    attack(&mut t, 2, P2, &[1], 2, false);
    t.run_until(75);
    assert_eq!(hp(&t, 2), Some(5.0));
    t.run_until(76);
    assert_eq!(hits_on(&t, 2).len(), 10);
    assert_eq!(died(&t), [(76, ObjectId(2))]);
    assert!(t.sim().object(ObjectId(2)).is_none());
    let last: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == 76)
        .map(|(_, e)| e)
        .collect();
    assert_eq!(
        last[1..],
        [
            &SimEvent::Hit {
                attacker: ObjectId(1),
                target: ObjectId(2),
                damage: 5.0,
            },
            &SimEvent::Died {
                id: ObjectId(2),
                kind: ObjectKind::Peacekeeper,
                owner: Some(P1),
                pos: Vec2::new(550.0, 300.0),
            },
            &SimEvent::Removed(ObjectId(2)),
        ],
        "attack, hit, death, removal: {last:?}"
    );
    // The Guard's order is complete, and its attack has nothing left to aim at.
    let guard = t.sim().object(ObjectId(1)).unwrap();
    assert!(guard.orders.as_ref().unwrap().is_idle());
    assert_eq!(guard.attack.as_ref().unwrap().target, None);
}

/// The tick's hits are applied together, once every unit has acted. Two hits of 9 in one tick take a Guard with 17
/// HP down although neither would alone; and two Peacekeepers in a duel kill each other in the same tick, each one's
/// last shot landing although it dies in that tick.
#[test]
fn hits_in_one_tick_all_count() {
    let mut t = spawned(&[
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 300.0)), // 1
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 400.0)), // 2
        (ObjectKind::Guard, P2, Vec2::new(550.0, 350.0)),       // 3
    ]);
    attack(&mut t, 2, P1, &[1], 3, false);
    // Unit 1 hits in ticks 4, 21, …, 106 (7 hits: 63) and 123; unit 2, ordered in 121, hits in 123 too.
    attack(&mut t, 121, P1, &[2], 3, false);
    t.run_until(122);
    assert_eq!(hp(&t, 3), Some(17.0));
    t.run_until(123);
    let last: Vec<(Tick, f32)> = hits_on(&t, 3).into_iter().filter(|&(tick, _)| tick == 123).collect();
    assert_eq!(last, [(123, 9.0), (123, 9.0)]);
    assert_eq!(died(&t), [(123, ObjectId(3))]);

    let mut duel = spawned(&[
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 300.0)),
        (ObjectKind::Peacekeeper, P2, Vec2::new(550.0, 300.0)),
    ]);
    attack(&mut duel, 2, P1, &[1], 2, false);
    attack(&mut duel, 2, P2, &[2], 1, false);
    duel.run_until(100);
    // 9 per hit: the sixth, in tick 4 + 5 × 17, is fatal to both.
    assert_eq!(died(&duel), [(89, ObjectId(1)), (89, ObjectId(2))]);
    assert_eq!(duel.sim().objects().count(), 0);
}

/// The objects holding a reference to `id` ([`testing::referenced_ids`]).
fn references_to(sim: &Sim, id: ObjectId) -> Vec<ObjectId> {
    sim.objects()
        .filter(|o| testing::referenced_ids(o).contains(&id))
        .map(|o| o.id)
        .collect()
}

/// A death goes through the removal path: the object is gone, and so is every reference to it. The orders naming it
/// complete (the current one) or are dropped (queued ones), and the next order starts; attacks on it stop. A debug
/// removal takes the same path.
#[test]
fn death_removes_the_object_and_every_reference_to_it() {
    let mut t = spawned(&[
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 200.0)), // 1
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 260.0)), // 2
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 320.0)), // 3
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 380.0)), // 4
        (ObjectKind::Guard, P2, Vec2::new(560.0, 290.0)),       // 5
        (ObjectKind::Guard, P2, Vec2::new(560.0, 360.0)),       // 6
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 140.0)), // 7
        (ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 80.0)),  // 8
    ]);
    // 1–3 attack Guard 5 (27 a volley: dead on the third, in tick 38); 2 then attacks Guard 6.
    attack(&mut t, 2, P1, &[1, 2, 3], 5, false);
    attack(&mut t, 2, P1, &[2], 6, true);
    attack(&mut t, 2, P1, &[2], 5, true);
    // 4 holds, with an attack on 5 and a move to it queued behind.
    t.order_at(2, P1, &[ObjectId(4)], OrderKind::Hold, false);
    attack(&mut t, 2, P1, &[4], 5, true);
    t.order_at(
        2,
        P1,
        &[ObjectId(4)],
        OrderKind::Move(Target::Object(ObjectId(5))),
        true,
    );
    // When 5 dies, 7 is about to fire at it and 8 has just begun to aim.
    attack(&mut t, 37, P1, &[7], 5, false);
    attack(&mut t, 38, P1, &[8], 5, false);
    t.run_until(37);
    assert_eq!(references_to(t.sim(), ObjectId(5)), ids(&[1, 2, 3, 4, 7]));
    t.run_until(38);
    assert_eq!(died(&t), [(38, ObjectId(5))]);
    assert!(t.sim().object(ObjectId(5)).is_none());
    assert_eq!(references_to(t.sim(), ObjectId(5)), []);
    assert_eq!(testing::dangling_references(t.sim()), []);
    let phase = |t: &TestSim, id: u32| t.sim().object(ObjectId(id)).unwrap().attack.clone().unwrap().phase;
    assert_eq!(
        phase(&t, 7),
        Some(AttackPhase::Firing { target: None }),
        "it fires at nothing"
    );
    assert_eq!(phase(&t, 8), None, "the aim is cancelled");
    let orders = |t: &TestSim, id: u32| t.sim().object(ObjectId(id)).unwrap().orders.clone().unwrap();
    assert!(orders(&t, 1).is_idle());
    assert_eq!(
        orders(&t, 2).queue.len(),
        1,
        "the order on 6 is next; the second one on 5 is dropped"
    );
    assert_eq!(orders(&t, 4).queue.len(), 0);
    assert_eq!(orders(&t, 4).current.map(|a| a.order.kind), Some(OrderKind::Hold));
    t.run_until(39);
    assert_eq!(
        t.sim().object(ObjectId(2)).unwrap().attack.as_ref().unwrap().target,
        Some(ObjectId(6)),
        "unit 2 engages its next target"
    );
    assert_eq!(phase(&t, 7), Some(AttackPhase::Cooldown));
    let shots_by_7 = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == ObjectId(7)))
        .count();
    assert_eq!(shots_by_7, 0);

    t.debug_at(45, DebugCommand::Remove { id: ObjectId(6) });
    t.run_until(45);
    assert!(t.sim().object(ObjectId(6)).is_none());
    assert_eq!(references_to(t.sim(), ObjectId(6)), []);
    assert!(orders(&t, 2).is_idle());
    assert_eq!(died(&t), [(38, ObjectId(5))], "a removal is not a death");
    assert!(t.events().contains(&(45, SimEvent::Removed(ObjectId(6)))));
}
