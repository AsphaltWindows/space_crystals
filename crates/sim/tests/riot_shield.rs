//! The Peacekeeper's Riot Shield (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`;
//! `sc_sim::riot_shield`): an ability command raises and lowers it, each taking the toggle time, and while it is up
//! the Peacekeeper moves at half speed, never attacks (no Attack orders, no auto-targeting, on Hold Position
//! neither) and takes half the damage before armor, from every side. The gate checks the command. A game saved
//! mid-toggle, and a replay, play on the same. That an idle unit with its shield up still yields (design change
//! 0023) is tested with the other yielding rules, in `movement_yield.rs`.
//!
//! Every `TestSim` checks the invariants I1–I7 and V1 after each tick.

use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::orders::MAX_ORDER_UNITS;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, Command, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Replay, RiotShield, Sim,
    SimEvent, Target, Tick, TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;
/// The toggle time, ticks (a placeholder: `> Toggle time` is "TBD (should be under half a second)").
const T: Tick = RIOT_SHIELD_TOGGLE_TICKS;

const RAISE: Ability = Ability::RiotShield { raise: true };
const LOWER: Ability = Ability::RiotShield { raise: false };

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

fn ids(ids: &[u32]) -> Vec<ObjectId> {
    ids.iter().map(|&i| ObjectId(i)).collect()
}

/// Spawns `units` (kind, owner, position) in tick 1, as ids 1, 2, …; units fight by themselves only with
/// `auto_targeting`.
fn spawned(units: &[(ObjectKind, PlayerId, Vec2)], auto_targeting: bool) -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    if !auto_targeting {
        t.without_auto_targeting();
    }
    for &(kind, owner, pos) in units {
        t.spawn_unit_at(1, kind, owner, pos);
    }
    t
}

fn shield(t: &TestSim, id: u32) -> RiotShield {
    t.sim()
        .object(ObjectId(id))
        .unwrap()
        .riot_shield
        .expect("a Riot Shield")
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

/// The ticks in which `attacker` fired.
fn attacks_by(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Attacked { attacker: a, .. } if *a == ObjectId(attacker) => Some(*tick),
            _ => None,
        })
        .collect()
}

fn command(player: PlayerId, units: &[u32], ability: Ability) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Ability {
            units: ids(units),
            ability,
        },
    }
}

fn attack_order(units: &[u32], target: u32, queue: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Order {
            units: ids(units),
            kind: OrderKind::Attack(ObjectId(target)),
            queue,
        },
    }
}

/// `> Toggle time`: raised in tick `t`, the shield is up at the end of tick `t + T − 1`; lowered, down after as
/// long. The unit is slow from the start of the raise to the end of the lowering (placeholder: the design doesn't
/// say what the transitions do). A second raise changes nothing, and a lowering half-way up turns back from there.
#[test]
fn raising_and_lowering_take_the_toggle_time() {
    let mut t = spawned(&[(PK, P1, Vec2::new(300.0, 300.0))], false);
    let mode = |t: &TestSim| (shield(t, 1), t.mover(1).speed_factor);
    t.run(2);
    assert_eq!(mode(&t), (RiotShield::Lowered, 1.0));

    t.ability_at(3, P1, &ids(&[1]), RAISE);
    let raising: Vec<_> = (0..T).map(|_| mode(t.step())).collect();
    let expected: Vec<_> = (1..T)
        .rev()
        .map(|left| (RiotShield::Raising { left }, 0.5))
        .chain([(RiotShield::Raised, 0.5)])
        .collect();
    assert_eq!(raising, expected, "up at the end of tick 3 + T − 1");

    let mut unchanged = t.sim().clone();
    unchanged.step(&TickInputs::default());
    t.ability_at(t.sim().tick() + 1, P1, &ids(&[1]), RAISE).step();
    assert_eq!(t.sim(), &unchanged, "already up");

    t.ability_at(20, P1, &ids(&[1]), LOWER).run_until(19);
    let lowering: Vec<_> = (0..T).map(|_| mode(t.step())).collect();
    let expected: Vec<_> = (1..T)
        .rev()
        .map(|left| (RiotShield::Lowering { left }, 0.5))
        .chain([(RiotShield::Lowered, 1.0)])
        .collect();
    assert_eq!(lowering, expected, "down at the end of tick 20 + T − 1, and fast again");

    // Two ticks up, then lowered: two ticks down.
    t.ability_at(30, P1, &ids(&[1]), RAISE)
        .ability_at(32, P1, &ids(&[1]), LOWER)
        .run_until(31);
    assert_eq!(shield(&t, 1), RiotShield::Raising { left: T - 2 });
    assert_eq!(mode(t.step()), (RiotShield::Lowering { left: 1 }, 0.5));
    assert_eq!(mode(t.step()), (RiotShield::Lowered, 1.0));
}

/// `> Riot Shield`: "MaxSpeed is halved (4 → 2 space units/frame)". Raised in the tick its move is ordered, a
/// Peacekeeper walks 2 su a tick; lowered in tick 12, 4 again from tick 12 + T − 1 on. The shield doesn't touch its
/// order.
#[test]
fn a_raised_shield_halves_the_speed() {
    let mut t = spawned(&[(PK, P1, Vec2::new(100.0, 300.0))], false);
    let to = Vec2::new(900.0, 300.0);
    t.order_at(2, P1, &ids(&[1]), OrderKind::Move(Target::Point(to)), false)
        .ability_at(2, P1, &ids(&[1]), RAISE)
        .ability_at(12, P1, &ids(&[1]), LOWER)
        .run(1);
    let mut steps = Vec::new();
    for _ in 2..=20 {
        let before = t.pos(1);
        t.step();
        steps.push(t.pos(1).distance(before));
    }
    let slow = (12 + T - 1 - 2) as usize;
    assert_eq!(steps, [vec![2.0; slow], vec![4.0; 19 - slow]].concat());
    let orders = t.sim().object(ObjectId(1)).unwrap().orders.clone().unwrap();
    assert_eq!(
        orders.current.map(|active| active.order.kind),
        Some(OrderKind::Move(Target::Point(to)))
    );
}

/// `> Riot Shield`: "Incoming damage is halved — a 50% pre-armor reduction (step 1 of systems/combat.md >
/// DamageModifierOrder), multiplying with other pre-armor reductions. The reduction applies to damage from all
/// directions." A Peacekeeper's hit (Damage 10) on a Peacekeeper (PointArmor 1) takes 9; on a raised shield
/// 10 × 0.5 − 1 = 4, from either side; on a raised shield on Rugged ground 10 × 0.5 × 0.5 − 1 = 1.5. Ordered in
/// tick 10, the attackers hit in ticks 12 and 29 (a cycle is 17 ticks). A shield on its way up doesn't protect yet
/// (placeholder): raised in tick 11, unit 6 takes 9 in tick 12 and 4 a cycle later.
#[test]
fn a_raised_shield_halves_the_damage_before_the_armor() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(800.0, 400.0)), // 1, plain, shielded
            (PK, P2, Vec2::new(950.0, 400.0)), // 2, east of 1
            (PK, P1, Vec2::new(160.0, 100.0)), // 3, Rugged, shielded
            (PK, P2, Vec2::new(160.0, 280.0)), // 4
            (PK, P2, Vec2::new(650.0, 400.0)), // 5, west of 1
            (PK, P1, Vec2::new(450.0, 300.0)), // 6, shielded from tick 11
            (PK, P2, Vec2::new(450.0, 450.0)), // 7
        ],
        false,
    );
    t.ability_at(2, P1, &ids(&[1, 3]), RAISE)
        .ability_at(11, P1, &ids(&[6]), RAISE);
    for (attackers, target) in [(&[2, 5][..], 1), (&[4][..], 3), (&[7][..], 6)] {
        let order = attack_order(attackers, target, false);
        t.at(10, P2, order.command);
    }
    t.run_until(30);
    assert_eq!(
        hits_on(&t, 1),
        [(12, 4.0), (12, 4.0), (29, 4.0), (29, 4.0)],
        "halved before the armor, from both sides"
    );
    assert_eq!(hits_on(&t, 3), [(12, 1.5), (29, 1.5)], "with the Rugged bonus");
    assert_eq!(hits_on(&t, 6), [(12, 9.0), (29, 4.0)], "only once it is up");
}

/// `> Riot Shield`: "Attack orders are not possible: the player must lower the shield before ordering an attack."
/// The gate takes no Attack order for a Peacekeeper whose shield is up or on its way up or down, queued or not; in a
/// group, the others take it. A move, an AttackMove or a Patrol it does take.
#[test]
fn no_attack_orders_with_the_shield_up() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 300.0)),    // 1
            (GUARD, P1, Vec2::new(360.0, 300.0)), // 2
            (PK, P2, Vec2::new(520.0, 300.0)),    // 3
        ],
        false,
    );
    t.ability_at(2, P1, &ids(&[1]), RAISE).run_until(2);
    let validate = |t: &TestSim, cmd: &PlayerCommand| t.sim().validate(cmd);
    assert_eq!(validate(&t, &attack_order(&[1], 3, false)), Err(Reject::CannotAttack));
    assert_eq!(validate(&t, &attack_order(&[1], 3, true)), Err(Reject::CannotAttack));
    let to = Vec2::new(600.0, 200.0);
    for kind in [
        OrderKind::Move(Target::Point(to)),
        OrderKind::Move(Target::Object(ObjectId(3))),
        OrderKind::AttackMove(to),
        OrderKind::Patrol(to),
    ] {
        let order = PlayerCommand {
            player: P1,
            command: Command::Order {
                units: ids(&[1]),
                kind,
                queue: true,
            },
        };
        assert_eq!(validate(&t, &order), Ok(()), "{kind:?}");
    }
    // In a group, the Guard takes it alone.
    t.at(3, P1, attack_order(&[1, 2], 3, false).command).step();
    let current = |t: &TestSim, id: u32| {
        let orders = t.sim().object(ObjectId(id)).unwrap().orders.clone().unwrap();
        orders.current.map(|active| active.order.kind)
    };
    assert_eq!(
        (current(&t, 1), current(&t, 2)),
        (None, Some(OrderKind::Attack(ObjectId(3))))
    );
    // Lowering: not yet. Down: yes.
    t.ability_at(10, P1, &ids(&[1]), LOWER).run_until(10);
    assert_eq!(validate(&t, &attack_order(&[1], 3, false)), Err(Reject::CannotAttack));
    t.run_until(10 + T - 1);
    assert_eq!(shield(&t, 1), RiotShield::Lowered);
    assert_eq!(validate(&t, &attack_order(&[1], 3, false)), Ok(()));
}

/// `> Riot Shield`: "The Peacekeeper does not auto-target enemies while the shield is up", idle or on Hold Position.
/// Two Peacekeepers of player 1, one idle and one holding, stand in range of a Peacekeeper of player 2, all three
/// shields raised as they arrive: nobody fires. Player 1 lowers its shields in tick 61: down in tick 61 + T − 1,
/// they acquire the enemy at once and hit two ticks later (Aim 2, Firing 1). The enemy's shield stays up, and it
/// never fires back.
#[test]
fn no_auto_targeting_with_the_shield_up() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 250.0)), // 1, idle
            (PK, P2, Vec2::new(500.0, 300.0)), // 2
            (PK, P1, Vec2::new(300.0, 350.0)), // 3, on Hold
        ],
        true,
    );
    t.ability_at(1, P1, &ids(&[1, 3]), RAISE)
        .ability_at(1, P2, &ids(&[2]), RAISE)
        .order_at(2, P1, &ids(&[3]), OrderKind::Hold, false)
        .run_until(60);
    assert_eq!(
        t.events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
            .count(),
        0
    );
    for id in [1, 2, 3] {
        let attack = t.sim().object(ObjectId(id)).unwrap().attack.clone().unwrap();
        assert_eq!((attack.target, attack.phase), (None, None), "#{id}");
    }
    t.ability_at(61, P1, &ids(&[1, 3]), LOWER).run_until(80);
    let down = 61 + T - 1;
    assert_eq!(attacks_by(&t, 1).first(), Some(&(down + 2)));
    assert_eq!(attacks_by(&t, 3).first(), Some(&(down + 2)));
    assert_eq!(attacks_by(&t, 2), Vec::<Tick>::new());
}

/// Raising the shield during an Attack order ends the attack: the order completes (the unit can't attack its target
/// any more, `systems/controls/behaviors.md > AttackingObject`) and the unit stands idle with nothing engaged.
/// Raised while still aiming, nothing is fired (`systems/combat.md > Aiming`: interruptible); raised while
/// reloading, the shot fired before stays the only one.
#[test]
fn raising_the_shield_ends_an_attack() {
    for (raised, hits) in [(3, vec![]), (8, vec![(4, 9.0)])] {
        let mut t = spawned(
            &[
                (PK, P1, Vec2::new(300.0, 300.0)),    // 1
                (GUARD, P2, Vec2::new(530.0, 300.0)), // 2, beyond its own Range of 1
            ],
            false,
        );
        let order = attack_order(&[1], 2, false);
        t.at(2, P1, order.command)
            .ability_at(raised, P1, &ids(&[1]), RAISE)
            .run_until(raised);
        let unit = t.sim().object(ObjectId(1)).unwrap();
        assert!(unit.orders.as_ref().unwrap().is_idle(), "raised in tick {raised}");
        assert_eq!(unit.attack.as_ref().unwrap().target, None);
        t.run_until(60);
        assert_eq!(hits_on(&t, 2), hits, "raised in tick {raised}");
    }
}

/// The gate (`Sim::validate`): an ability names only the player's own units, at most `MAX_ORDER_UNITS` of them; ids
/// that no longer exist, repeated ids and units without the ability are dropped. A rejected command changes nothing.
#[test]
fn the_gate_checks_the_ability() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 300.0)),    // 1
            (GUARD, P1, Vec2::new(360.0, 300.0)), // 2
            (PK, P2, Vec2::new(600.0, 300.0)),    // 3
        ],
        false,
    );
    t.run_until(1);
    let too_many: Vec<u32> = vec![1; MAX_ORDER_UNITS + 1];
    let rejected = [
        (command(P1, &[1, 3], RAISE), Reject::NotOwned(ObjectId(3))),
        (command(P2, &[1], LOWER), Reject::NotOwned(ObjectId(1))),
        (command(P1, &[2], RAISE), Reject::LacksAbility),
        (command(P1, &[42], RAISE), Reject::NoUnits),
        (command(P1, &[], RAISE), Reject::NoUnits),
        (command(PlayerId(9), &[1], RAISE), Reject::UnknownPlayer(PlayerId(9))),
        (command(P1, &too_many, RAISE), Reject::TooManyUnits),
    ];
    for (cmd, reject) in &rejected {
        assert_eq!(t.sim().validate(cmd), Err(*reject), "{cmd:?}");
    }
    let mut unchanged = t.sim().clone();
    unchanged.step(&TickInputs::default());
    for (cmd, _) in rejected {
        t.at(2, cmd.player, cmd.command);
    }
    t.step();
    assert_eq!(t.sim(), &unchanged);
    assert!(t.sim().events().is_empty(), "{:?}", t.sim().events());

    let fine = command(P1, &[42, 1, 2, 1], RAISE);
    assert_eq!(t.sim().validate(&fine), Ok(()));
    t.at(3, P1, fine.command).step();
    assert_eq!(shield(&t, 1), RiotShield::Raising { left: T - 1 });
    assert_eq!(t.sim().object(ObjectId(2)).unwrap().riot_shield, None);
}

/// `ARCHITECTURE.md > Determinism`: a game saved half-way up a raise loads to the same state and plays on the same,
/// through a fight and a lowering; and the whole run, recorded, replays to the same hash, also after a JSON round
/// trip. Unit 1 walks a little with its shield going up while unit 2 of player 2 fires at it; lowered in tick 30,
/// it fires back.
#[test]
fn saved_mid_toggle_and_replayed_it_plays_on_the_same() {
    let mut t = spawned(
        &[
            (PK, P1, Vec2::new(300.0, 300.0)), // 1
            (PK, P2, Vec2::new(500.0, 300.0)), // 2
        ],
        true,
    );
    let step_aside = OrderKind::Move(Target::Point(Vec2::new(300.0, 330.0)));
    t.order_at(2, P1, &ids(&[1]), step_aside, false)
        .ability_at(2, P1, &ids(&[1]), RAISE)
        .ability_at(30, P1, &ids(&[1]), LOWER)
        .run_until(4);
    assert_eq!(shield(&t, 1), RiotShield::Raising { left: T - 3 });
    assert_ne!(t.pos(1), t.sim().object(ObjectId(1)).unwrap().prev_pos, "on its way");
    let json = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    for tick in 5..=80 {
        let commands = if tick == 30 {
            vec![command(P1, &[1], LOWER)]
        } else {
            Vec::new()
        };
        loaded.step(&TickInputs { commands });
        t.step();
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {tick}");
    }
    assert_eq!(shield(&t, 1), RiotShield::Lowered);
    assert!(!attacks_by(&t, 1).is_empty(), "lowered, it fights");

    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    let json = serde_json::to_string(t.replay()).unwrap();
    let back: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(back.run().state_hash(), t.sim().state_hash());
}
