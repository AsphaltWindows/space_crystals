//! The fog delay (`core/vision.md > Visible > Fog delay`, design change 0178, answering question Q0018 point 1): a
//! tile that leaves the sight of all of a player's units and structures stays Visible for [`FOG_DELAY_TICKS`]
//! before it turns Explored. What stands on it stays seen for as long: drawn, selectable and targetable. Attack and
//! Follow have no grace period of their own: design change 0034's last-seen rule triggers only once the target is
//! no longer Visible. `TestSim` checks V1 after every tick, the tiles that linger included.

use sc_sim::mover::MoveGoal;
use sc_sim::orders::Behaviour;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::vision::FOG_DELAY_TICKS;
use sc_sim::{
    Command, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Shape, Target, Tick,
    TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn open(width: usize, height: usize) -> String {
    vec![".".repeat(width); height].join("\n")
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn move_to(at: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(at))
}

/// Whether one of `player`'s sources sees object `id` now: a tile under its box is Visible, and not only for the
/// fog delay. (A box only touching a tile's edge doesn't cover it, as for vision.)
fn in_sight(t: &TestSim, player: PlayerId, id: u32) -> bool {
    let (lo, hi) = Shape::of(t.sim().object(ObjectId(id)).unwrap()).bounds();
    let span = |lo: f32, hi: f32| {
        let first = (lo / 64.0).floor() as i32;
        first..=((hi / 64.0).ceil() as i32 - 1).max(first)
    };
    let vision = t.sim().vision().player(player).unwrap();
    span(lo.y, hi.y).any(|y| {
        span(lo.x, hi.x).any(|x| {
            let tile = TilePos::new(x, y);
            vision.state(tile) == VisibilityState::Visible && !vision.lingers(tile)
        })
    })
}

fn is_visible(t: &TestSim, player: PlayerId, id: u32) -> bool {
    t.sim().is_visible(player, t.sim().object(ObjectId(id)).unwrap())
}

fn attack(player: PlayerId, unit: u32, target: u32) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: vec![ObjectId(unit)],
            kind: OrderKind::Attack(ObjectId(target)),
            queue: false,
        },
    }
}

/// Whether unit `n`'s Attack or Follow lost its target (design change 0034: it walks to where it last saw it).
fn lost(t: &TestSim, n: u32) -> bool {
    let orders = t.sim().object(ObjectId(n)).unwrap().orders.as_ref().unwrap();
    match orders.current.as_ref().map(|active| &active.state) {
        Some(Behaviour::Attack(state)) => state.lost,
        Some(Behaviour::Approach(state)) => state.lost,
        other => panic!("#{n} has no Attack or Follow: {other:?}"),
    }
}

/// The acceptance's "a scout walks past hidden enemies, and they stay visible (selectable, targetable) for 8 ticks
/// after its sight leaves them, then not". Player 1's Peacekeeper walks east along row 5; player 2's units stand
/// 4 rows off its way, in the fog. Each is Visible from the first tick the scout's sight reaches its tile to the
/// `FOG_DELAY_TICKS`-th after it left, and only then: `Sim::is_visible`, which the client's selection asks, and the
/// gate's verdict on an Attack on it agree tick by tick. Auto-targeting is off: nobody shoots.
#[test]
fn a_scout_walking_past_leaves_them_visible_for_the_fog_delay() {
    let mut t = ScenarioBuilder::new(&open(40, 12)).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(2, 5)) // 1: the scout
        .spawn_unit_at(1, ObjectKind::Guard, P2, centre(12, 9)) // 2..=4: in the fog
        .spawn_unit_at(1, ObjectKind::Guard, P2, centre(20, 1))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(28, 9))
        .order_at(2, P1, &[ObjectId(1)], move_to(centre(37, 5)), false)
        .run_until(2);
    let enemies = [2, 3, 4];
    assert!(
        enemies.iter().all(|&enemy| !is_visible(&t, P1, enemy)),
        "hidden at first"
    );
    let mut last_in_sight: [Option<Tick>; 3] = [None; 3];
    let mut delayed = [0; 3];
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 1000, "the scout never arrived");
        t.step();
        let tick = t.sim().tick();
        for (i, &enemy) in enemies.iter().enumerate() {
            if in_sight(&t, P1, enemy) {
                last_in_sight[i] = Some(tick);
            } else if is_visible(&t, P1, enemy) {
                delayed[i] += 1;
            }
            let expected = last_in_sight[i].is_some_and(|last| tick - last <= FOG_DELAY_TICKS);
            assert_eq!(is_visible(&t, P1, enemy), expected, "#{enemy} in tick {tick}");
            let verdict = if expected {
                Ok(())
            } else {
                Err(Reject::NotVisible(ObjectId(enemy)))
            };
            assert_eq!(
                t.sim().validate(&attack(P1, 1, enemy)),
                verdict,
                "#{enemy} in tick {tick}"
            );
        }
    }
    assert!(last_in_sight.iter().all(Option::is_some), "the scout saw each");
    assert_eq!(delayed, [FOG_DELAY_TICKS; 3], "each Visible for the delay after, once");
    assert!(enemies.iter().all(|&enemy| !is_visible(&t, P1, enemy)), "hidden again");
}

/// How the hand-over of [`a_short_gap_in_sight_keeps_the_attack_and_the_follow`] went: the ticks in which no source
/// of player 1's saw the Guard, each with whether the Guard was Visible then; and the first tick the chaser's order
/// had lost it, if it did.
struct Handover {
    gap: Vec<(Tick, bool)>,
    lost: Option<Tick>,
}

/// Player 2's Guard (1) stands on (20, 5). Player 1's scout (2) on (16, 3) sees it at the edge of its sight and walks
/// away west from tick 2; player 1's Peacekeeper (3), `distance` su west of the Guard, is ordered after it (`order`)
/// in tick 2 and closes in, and its own sight takes the Guard over. Run to tick 80.
fn handover(distance: f32, order: OrderKind) -> Handover {
    let mut t = ScenarioBuilder::new(&open(40, 12)).start();
    let guard = centre(20, 5);
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P2, guard)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(16, 3))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, guard - Vec2::new(distance, 0.0))
        .order_at(2, P1, &[ObjectId(2)], move_to(centre(2, 3)), false)
        .order_at(2, P1, &[ObjectId(3)], order, false)
        .run_until(2);
    let mut handover = Handover {
        gap: Vec::new(),
        lost: None,
    };
    while t.sim().tick() < 80 {
        t.step();
        let tick = t.sim().tick();
        if !in_sight(&t, P1, 1) {
            handover.gap.push((tick, is_visible(&t, P1, 1)));
        }
        if handover.lost.is_none() && lost(&t, 3) {
            handover.lost = Some(tick);
            let walk = t.mover(3).goal;
            assert_eq!(walk, MoveGoal::Point(guard), "to where it was last seen");
        }
    }
    handover
}

/// Design change 0178 (the acceptance's "chasing an enemy at the edge of sight no longer drops to the last-seen
/// position on a one-tick gap"): "Attack/Follow don't drop to last-seen position on one-tick gaps". Across the
/// distances [`handover`] tries, the gap between the scout's sight leaving the Guard's tile and the chaser's own
/// reaching it runs from none to more than the delay. While it lasts at most `FOG_DELAY_TICKS`, the Guard stays
/// Visible all through it, and neither an Attack nor a Follow loses it: with no delay, each of those gaps would
/// have sent the chaser to the last-seen position (0034). A longer gap loses it in the tick after the delay ran out,
/// as before.
#[test]
fn a_short_gap_in_sight_keeps_the_attack_and_the_follow() {
    let (mut shortest, mut longest) = (Tick::MAX, 0);
    for distance in (344..=420).step_by(2) {
        for order in [
            OrderKind::Attack(ObjectId(1)),
            OrderKind::Move(Target::Object(ObjectId(1))),
        ] {
            let Handover { gap, lost } = handover(distance as f32, order);
            let length = gap.len() as Tick;
            if let Some(&(first, _)) = gap.first() {
                let ticks: Vec<Tick> = gap.iter().map(|&(tick, _)| tick).collect();
                assert_eq!(
                    ticks,
                    (first..first + length).collect::<Vec<_>>(),
                    "one gap: {distance} su"
                );
            }
            if length <= FOG_DELAY_TICKS {
                assert!(gap.iter().all(|&(_, visible)| visible), "{distance} su: {gap:?}");
                assert_eq!(lost, None, "{distance} su, {order:?}: kept through a gap of {length}");
            } else {
                let delay = FOG_DELAY_TICKS as usize;
                assert!(
                    gap[..delay].iter().all(|&(_, visible)| visible),
                    "{distance} su: {gap:?}"
                );
                assert!(!gap[delay].1, "{distance} su: hidden once the delay ran out");
                assert_eq!(lost, Some(gap[delay].0 + 1), "{distance} su, {order:?}");
            }
            if length > 0 {
                shortest = shortest.min(length);
            }
            longest = longest.max(length);
        }
    }
    assert_eq!(shortest, 1, "a one-tick gap is among them");
    assert!(longest > FOG_DELAY_TICKS, "and one longer than the delay");
}

/// The fog delay keeps only tiles that were in sight, so it leaks nothing (`> Fog delay`: "during the delay the tile
/// is genuinely Visible"). Player 1's Peacekeeper attacks player 2's, 4.8 grid units ahead at the edge of its sight,
/// which runs east as fast as it: the target steps onto a tile ahead that no source of player 1's has seen yet, so it
/// is out of sight at once, and the attack goes to where it was last seen (design change 0034), as before the delay.
/// The delay smooths what leaves sight, not a target running ahead of it.
#[test]
fn a_target_stepping_onto_a_tile_never_seen_is_lost_at_once() {
    let mut t = ScenarioBuilder::new(&open(48, 12)).start();
    let start = centre(4, 5) + Vec2::new(4.8 * 64.0, 0.0);
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(4, 5))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, start)
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Attack(ObjectId(2)), false)
        .order_at(2, P2, &[ObjectId(2)], move_to(centre(46, 5)), false)
        .run_until(2);
    let mut last_seen = start;
    while is_visible(&t, P1, 2) {
        assert!(t.sim().tick() < 200, "never out of sight");
        assert!(in_sight(&t, P1, 2), "no tile under it lingers");
        last_seen = t.sim().object(ObjectId(2)).unwrap().pos;
        t.step();
    }
    let (lo, hi) = Shape::of(t.sim().object(ObjectId(2)).unwrap()).bounds();
    let tiles = t.sim().vision().player(P1).unwrap().box_state(lo, hi);
    assert_eq!(tiles, VisibilityState::Unexplored, "never seen");
    assert!(!lost(&t, 1));
    t.step();
    assert!(lost(&t, 1));
    assert_eq!(t.mover(1).goal, MoveGoal::Point(last_seen));
}
