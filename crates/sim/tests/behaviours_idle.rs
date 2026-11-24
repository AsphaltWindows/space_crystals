//! Units acting on their own (`systems/controls/behaviors.md > BaseAutoTargeting`, `> HoldingPosition`): an idle
//! unit acquires an enemy within its SightRange by the design's priority, chases it within IdleLeashDistance and
//! returns to where it stood; a unit on Hold Position attacks what its weapon reaches and never moves; a unit on a
//! Move doesn't engage; nobody picks its own side or nobody's units.

use sc_sim::orders::{Auto, IDLE_LEASH_GU};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Shape,
    SimConfig, SimEvent, SpawnAt, Target, Tick, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5

/// 40 × 16 open tiles: 2560 × 1024 su.
const OPEN: &str = "
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn pos(t: &TestSim, n: u32) -> Vec2 {
    t.sim().object(id(n)).expect("exists").pos
}

fn is_idle(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().is_idle()
}

/// What the unit's attack engages.
fn target(t: &TestSim, n: u32) -> Option<ObjectId> {
    t.sim().object(id(n)).unwrap().attack.as_ref().unwrap().target
}

fn auto(t: &TestSim, n: u32) -> Auto {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().auto
}

fn gap(t: &TestSim, a: u32, b: u32) -> f32 {
    let shape = |n| Shape::of(t.sim().object(id(n)).expect("exists"));
    shape(a).gap(&shape(b))
}

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

fn teleport(t: &mut TestSim, tick: Tick, n: u32, to: Vec2) {
    t.debug_at(tick, DebugCommand::SetPosition { id: id(n), pos: to });
}

/// `> Active during Idle`, the brief's "chase and leash return": an idle Peacekeeper acquires an enemy Peacekeeper
/// in sight but out of range, which walks away as fast as it follows. Where it stood when it acquired the enemy is
/// its IdleOrigin. Once it is farther than IdleLeashDistance (4 grid units) from there it disengages and walks
/// back, without looking for targets on the way, and stands on the IdleOrigin again.
#[test]
fn an_idle_unit_chases_within_its_leash_and_returns() {
    let origin = v(300.0, 500.0);
    let mut t = ScenarioBuilder::new(OPEN).start();
    // A gap of 270 su: beyond Range (256), within SightRange (320).
    t.spawn_unit_at(1, PK, P1, origin)
        .spawn_unit_at(1, PK, P2, origin + v(12.0 + 270.0 + 12.0, 0.0))
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(2400.0, 500.0))), false);
    t.run_until(2);
    assert_eq!(target(&t, 1), Some(id(2)), "acquired in its first tick");
    assert!(matches!(auto(&t, 1), Auto::Engaging { origin: at, .. } if at == origin));
    assert!(is_idle(&t, 1), "it has no order all the while");
    let mut farthest = 0.0_f32;
    let mut turned_at = None;
    while t.status(1) != MoveStatus::Arrived {
        assert!(t.sim().tick() < 400, "it never came back\n{}", t.ascii(None));
        t.step();
        farthest = farthest.max(pos(&t, 1).distance(origin));
        if turned_at.is_none() && auto(&t, 1) == Auto::Returning {
            turned_at = Some(t.sim().tick());
            assert_eq!(target(&t, 1), None, "disengaged");
        }
    }
    // 4 su a tick: 65 steps take it past 256 su, and the next tick turns it around.
    assert_eq!(farthest, gu(IDLE_LEASH_GU) + 4.0);
    assert_eq!(turned_at, Some(2 + 65));
    assert_eq!(pos(&t, 1), origin, "back on its IdleOrigin");
    // The tick that turns it around is its first step back.
    assert_eq!(t.sim().tick(), 2 + 65 + 64);
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0], "the enemy was never in range");
    t.run(50);
    assert_eq!(pos(&t, 1), origin, "the enemy is out of sight by now");
    assert_eq!(auto(&t, 1), Auto::default());
}

/// `> Active during Idle`: "If the target is destroyed or leaves SightRange, the unit returns to idle at its current
/// position (new IdleOrigin)." A Guard outruns the idle Peacekeeper that goes for it: once it is out of sight the
/// Peacekeeper stops where it is, and doesn't walk back.
#[test]
fn a_target_that_leaves_sight_leaves_the_idle_unit_where_it_is() {
    let origin = v(300.0, 500.0);
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, origin)
        .spawn_unit_at(1, GUARD, P2, origin + v(12.0 + 262.0 + 18.0, 0.0))
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(2400.0, 500.0))), false);
    t.run_until(2);
    assert_eq!(target(&t, 1), Some(id(2)));
    t.run_until(120);
    assert_eq!(target(&t, 1), None);
    assert_eq!(auto(&t, 1), Auto::default());
    let stopped = pos(&t, 1);
    assert!(
        stopped.x > origin.x && stopped.distance(origin) < gu(IDLE_LEASH_GU),
        "{stopped}"
    );
    assert_eq!(t.status(1), MoveStatus::Idle, "it stopped, and didn't go back");
    t.run(50);
    assert_eq!(pos(&t, 1), stopped);
}

/// `> Active during HoldPosition`, `> HoldingPosition`: a unit on Hold Position never moves. It attacks an enemy
/// within its weapon's range, turning to face it, and disengages at once when the enemy leaves that range, though
/// it still sees it.
#[test]
fn a_unit_on_hold_attacks_what_it_reaches_and_never_moves() {
    let at = v(300.0, 500.0);
    let enemy_at = |gap: f32| at + v(0.0, 12.0 + gap + 18.0);
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, at)
        .spawn_unit_at(1, GUARD, P2, enemy_at(270.0))
        .order_at(2, P1, &[id(1)], OrderKind::Hold, false)
        .order_at(2, P2, &[id(2)], OrderKind::Hold, false);
    t.run_until(40);
    assert!(t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()));
    assert_eq!(target(&t, 1), None, "in sight, out of range: nothing to do");
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0]);
    // In range: it turns south and fires.
    teleport(&mut t, 41, 2, enemy_at(250.0));
    t.run_until(41);
    assert_eq!(target(&t, 1), Some(id(2)));
    t.run_until(60);
    assert_eq!(attack_ticks(&t, 1), [43, 60]);
    let unit = t.sim().object(id(1)).unwrap();
    assert!(
        (unit.facing - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
        "{}",
        unit.facing
    );
    // Out of range again: it lets go, and stays.
    teleport(&mut t, 61, 2, enemy_at(262.0));
    t.run_until(61);
    assert_eq!(target(&t, 1), None, "disengaged in the same tick");
    t.run_until(200);
    assert_eq!(attack_ticks(&t, 1), [43, 60]);
    assert_eq!(pos(&t, 1), at, "it never moved");
    assert_eq!(
        pos(&t, 2),
        enemy_at(262.0),
        "nor did the Guard, out of its own range all along"
    );
    assert!(!is_idle(&t, 1), "Hold never completes");
}

/// `> Target Selection Priority`: among the enemies in sight (all of them threaten a ground unit here), the one
/// needing the least rotation from the unit's facing, then the closest.
#[test]
fn an_idle_unit_picks_by_rotation_then_by_distance() {
    let at = v(600.0, 500.0);
    let scene = |enemies: &[Vec2]| {
        let mut t = ScenarioBuilder::new(OPEN).start();
        t.spawn_unit_at(1, PK, P1, at);
        for &offset in enemies {
            t.spawn_unit_at(1, PK, P2, at + offset);
        }
        // The enemies hold, so that they stay where they are.
        let held: Vec<ObjectId> = (2..2 + enemies.len() as u32).map(id).collect();
        t.order_at(2, P2, &held, OrderKind::Hold, false);
        t.run_until(2);
        t
    };
    // The unit faces east. One enemy is near, to the north; one is farther, due east: the one ahead.
    let t = scene(&[v(0.0, -100.0), v(200.0, 0.0)]);
    assert_eq!(target(&t, 1), Some(id(3)), "the least rotation");
    // Both due east: the nearer one, whatever their ids.
    let t = scene(&[v(220.0, 0.0), v(150.0, 0.0)]);
    assert_eq!(target(&t, 1), Some(id(3)), "the closest");
    let t = scene(&[v(150.0, 0.0), v(220.0, 0.0)]);
    assert_eq!(target(&t, 1), Some(id(2)));
    // In sight but out of range counts too: the unit goes for it.
    let mut t = scene(&[v(294.0, 0.0)]);
    assert_eq!(target(&t, 1), Some(id(2)));
    t.run(10);
    assert!(pos(&t, 1).x > at.x);
    // Beyond its SightRange (a gap of 5 grid units), though player 1 sees it through a scout: nothing.
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, at)
        .spawn_unit_at(1, PK, P2, at + v(12.0 + 330.0 + 12.0, 0.0))
        .spawn_unit_at(1, PK, P1, at + v(354.0, 280.0))
        .order_at(2, P2, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(3)], OrderKind::Hold, false);
    t.run_until(30);
    assert!(t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()));
    assert_eq!((target(&t, 1), pos(&t, 1)), (None, at));
}

/// `> ValidTarget`, question Q0014 point 1: automatic target picking never chooses the unit's own side or nobody's
/// objects, and a Move is pure movement (`> Not active during: Move`): the unit passes an enemy without a shot and
/// engages it once it has arrived and is idle.
#[test]
fn own_and_neutral_units_are_never_picked_and_a_move_doesnt_engage() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(300.0, 500.0))
        .spawn_unit_at(1, GUARD, P1, v(400.0, 500.0))
        .debug_at(
            1,
            DebugCommand::Spawn {
                kind: GUARD,
                owner: None,
                at: SpawnAt::Point(v(300.0, 600.0)),
                orientation: Default::default(),
            },
        );
    t.run_until(60);
    assert_eq!((target(&t, 1), target(&t, 2), target(&t, 3)), (None, None, None));
    assert_eq!(
        t.events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
            .count(),
        0
    );

    // An enemy Guard holds 200 su off the Peacekeeper's way; the Peacekeeper walks past it and on.
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(300.0, 500.0))
        .spawn_unit_at(1, GUARD, P2, v(700.0, 700.0))
        .order_at(2, P2, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], OrderKind::Move(Target::Point(v(900.0, 500.0))), false);
    t.run_until(2);
    while t.status(1) != MoveStatus::Arrived {
        t.step();
        assert_eq!(target(&t, 1), None, "tick {}", t.sim().tick());
    }
    let arrived = t.sim().tick();
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0]);
    assert!(!attack_ticks(&t, 2).is_empty(), "the Guard shot at it as it passed");
    // Arrived and idle, with the Guard in sight and in range: it turns on it by itself.
    t.run(60);
    assert_eq!(target(&t, 1), Some(id(2)));
    assert!(attack_ticks(&t, 1).first().is_some_and(|&first| first > arrived));
    assert!(gap(&t, 1, 2) <= gu(4.0));
}

/// `DebugCommand::SetAutoTargeting`: with auto-targeting off nobody acquires a target, idle or on Hold; an Attack
/// order works as ever; and turned on again, the units pick their fight.
#[test]
fn the_debug_switch_turns_auto_targeting_off_and_on() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, PK, P1, v(300.0, 500.0))
        .spawn_unit_at(1, GUARD, P2, v(450.0, 500.0))
        .spawn_unit_at(1, PK, P1, v(300.0, 600.0))
        .order_at(2, P1, &[id(3)], OrderKind::Hold, false);
    t.run_until(50);
    assert!(!t.sim().auto_targeting());
    assert_eq!(
        t.events()
            .iter()
            .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
            .count(),
        0
    );
    t.order_at(51, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .run_until(60);
    assert_eq!(attack_ticks(&t, 1), [53]);
    assert_eq!(attack_ticks(&t, 2), [] as [Tick; 0], "the Guard doesn't fight back");
    t.debug_at(61, DebugCommand::SetAutoTargeting { on: true })
        .run_until(70);
    assert!(t.sim().auto_targeting());
    assert_eq!(attack_ticks(&t, 2), [63], "now it does");
    assert_eq!(attack_ticks(&t, 3), [63], "and so does the unit on Hold");

    // A debug command like the others: a game without them ignores it.
    let real = TestSim::with_config(ScenarioBuilder::new(OPEN).build(), SimConfig::default());
    let off = PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    };
    assert_eq!(real.sim().validate(&off), Err(Reject::DebugDisabled));
    assert!(real.sim().auto_targeting());
}
