//! Design change 0296 point 3 (`factions/syndicate/structures/laser_turret.md > Pop-up rules`): "**Blocking**: an
//! **upright** turret blocks movement like any building, and so does a **raised** pop-up. A lowered pop-up never blocks
//! (units walk over it like an expansion). **Rising needs a clear footprint**: it can't rise while any unit stands over
//! it. The **owner's own idle units** over it are pushed off by the idle push ... Any other unit over it (an enemy, or
//! an own unit with an order) keeps it **underground** until the spot is clear."
//!
//! The reading built (`sc_sim::popup`): the footprint blocks from the tick the turret starts to rise until it is fully
//! lowered again.

use sc_sim::MoveStatus;

use super::*;

/// A walker's start north of the turret's tile and its goal south of it: the straight way leads across the tile.
const NORTH: Vec2 = Vec2::new(CENTRE.x, CENTRE.y - 160.0);
const SOUTH: Vec2 = Vec2::new(CENTRE.x, CENTRE.y + 160.0);

/// Unit `walker` is ordered from where it stands to [`SOUTH`] and walks until it arrives: whether its box was ever
/// over the turret's tile, and how many ticks the walk took.
fn walk_south(t: &mut TestSim, walker: ObjectId) -> (bool, u32) {
    next(t, P1, order(walker, OrderKind::Move(Target::Point(SOUTH))));
    let mut crossed = false;
    for ticks in 1..400 {
        crossed |= over_the_turret(t, walker);
        if t.status(walker.0) == MoveStatus::Arrived {
            assert_eq!(object(t, walker).pos, SOUTH);
            return (crossed, ticks);
        }
        t.step();
    }
    panic!("the walker never arrived:\n{}", t.ascii(Some(CENTRE)));
}

/// "A lowered pop-up never blocks (units walk over it like an expansion)": its tile is open on the true grids, and
/// its owner's Agent walks straight across it, 320 space units at 6 a tick. The turret stays down: its own units are
/// no targets.
#[test]
fn a_lowered_popup_is_walked_over() {
    let mut t = lowered(with_turret().spawn(AGENT, Some(P1), SpawnAt::Point(NORTH)));
    let walker = ObjectId(4);
    assert!(!blocked(&t));
    let (crossed, ticks) = walk_south(&mut t, walker);
    assert!(crossed, "straight across the tile");
    assert_eq!(ticks, 54, "320 su at 6 su a tick, the order's tick first");
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
}

/// "... and so does a **raised** pop-up": with an enemy in its range the turret is up, its tile blocked on the true
/// grids, and the same walk goes round it: the Agent's box never overlaps the tile (invariant I2 checks it every tick
/// too), and it takes longer.
#[test]
fn a_raised_popup_blocks_a_walker() {
    let mut t = lowered(with_turret().spawn(AGENT, Some(P1), SpawnAt::Point(NORTH)));
    let walker = ObjectId(4);
    // On Hold until its walk: an idle Agent would go for the Guard by itself.
    next(&mut t, P1, order(walker, OrderKind::Hold));
    next_debug(&mut t, spawn(GUARD, P2, east(4.0)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    assert!(blocked(&t), "from the tick it starts to rise");
    t.run(8);
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    let (crossed, ticks) = walk_south(&mut t, walker);
    assert!(!crossed, "round the raised turret");
    assert!(ticks > 54, "a detour: {ticks} ticks");
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })), "still up");
}

/// A walker on its way across the lowered turret's tile when an enemy comes in range: the turret waits for it, and
/// starts to rise in the tick the walker's box is off the tile. Nothing ever stands on a blocked tile (invariant I2).
#[test]
fn it_waits_for_a_walker_crossing_it() {
    let mut t = lowered(with_turret().spawn(AGENT, Some(P1), SpawnAt::Point(NORTH)));
    let walker = ObjectId(4);
    next(&mut t, P1, order(walker, OrderKind::Move(Target::Point(SOUTH))));
    while !over_the_turret(&t, walker) {
        t.step();
    }
    next_debug(&mut t, spawn(GUARD, P2, east(4.0)));
    assert!(over_the_turret(&t, walker));
    assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
    let mut waited = 0;
    while over_the_turret(&t, walker) {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert!(!blocked(&t));
        t.step();
        waited += 1;
    }
    assert!(waited > 5, "most of the tile was still ahead: {waited} ticks");
    assert_eq!(
        popup(&t, POPUP),
        Some(PopUp::Rising(frames(0, 8))),
        "in the tick it is clear"
    );
    assert!(blocked(&t));
    // The walker had an order all along: nothing pushed it off its way.
    assert_eq!(object(&t, walker).pos.x, CENTRE.x);
}

/// 0296's verification: "An enemy Guard standing on a hidden turret's tile isn't shot by it." The turret picks it (it
/// keeps its target choice), can't rise, and stays hidden from the Guard's owner; no enemy is ever pushed. Once the
/// Guard walks off, the turret rises in the tick its tile is clear, and fires 13 ticks later.
#[test]
fn an_enemy_standing_over_it_keeps_it_down_and_isnt_shot() {
    let mut t = lowered(with_turret());
    next_debug(&mut t, spawn(GUARD, P2, CENTRE));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    for _ in 0..100 {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert!(
            object(&t, POPUP)
                .attack
                .as_ref()
                .is_some_and(|a| a.target == Some(guard))
        );
        assert!(!t.sim().is_visible(P2, object(&t, POPUP)));
        assert_eq!(object(&t, guard).pos, CENTRE, "nothing pushes an enemy");
        t.step();
    }
    assert_eq!(shots(&t, POPUP), []);
    assert_eq!(object(&t, guard).hp, Some(80.0));
    next(
        &mut t,
        P2,
        order(guard, OrderKind::Move(Target::Point(CENTRE + Vec2::new(160.0, 0.0)))),
    );
    while over_the_turret(&t, guard) {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        t.step();
    }
    let rose = t.sim().tick();
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))));
    assert!(
        t.sim().is_visible(P2, object(&t, POPUP)),
        "seen from the tick it starts to rise"
    );
    t.run_until(rose + 13);
    assert_eq!(shots(&t, POPUP), [rose + 13]);
}

/// "Any other unit over it (an enemy, or **an own unit with an order**) keeps it underground until the spot is clear":
/// player 1's Agent on Hold Position over the turret is never pushed, and the turret stays down with an enemy in its
/// range. With a Stop the Agent is idle, is pushed off, and the turret rises.
#[test]
fn an_own_unit_with_an_order_keeps_it_down() {
    let mut t = lowered(with_turret());
    next_debug(&mut t, spawn(AGENT, P1, CENTRE));
    let agent = newest(&t);
    next(&mut t, P1, order(agent, OrderKind::Hold));
    next_debug(&mut t, spawn(GUARD, P2, east(4.5)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    for _ in 0..100 {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert_eq!(object(&t, agent).pos, CENTRE, "a unit with an order is never pushed");
        t.step();
    }
    assert_eq!(shots(&t, POPUP), []);
    next(&mut t, P1, order(agent, OrderKind::Stop));
    t.run(20);
    assert!(!over_the_turret(&t, agent));
    assert!(matches!(
        popup(&t, POPUP),
        Some(PopUp::Rising(_) | PopUp::Raised { .. })
    ));
}

/// 0296's verification: "Your own idle Agent over it is nudged off when an enemy comes in range, and the turret
/// rises." An idle Agent that sees the enemy goes for it by itself (its idle behaviour), which takes it off the tile
/// too: either way it is off within a dozen ticks, and the turret rises and fires. The push alone is the next test's.
#[test]
fn its_owners_idle_agent_over_it_is_off_it_when_an_enemy_comes_in_range_and_it_rises() {
    let mut t = lowered(with_turret());
    next_debug(&mut t, spawn(AGENT, P1, CENTRE));
    let agent = newest(&t);
    t.run(3);
    assert_eq!(
        object(&t, agent).pos,
        CENTRE,
        "nothing moves it while the turret wants nothing"
    );
    assert_eq!(popup(&t, POPUP), Some(PopUp::LOWERED));
    next_debug(&mut t, spawn(GUARD, P2, east(4.5)));
    let (g, guard) = (t.sim().tick(), newest(&t));
    next(&mut t, P2, order(guard, OrderKind::Hold));
    while over_the_turret(&t, agent) {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert!(t.sim().tick() < g + 12);
        t.step();
    }
    let rose = t.sim().tick();
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))));
    t.run_until(rose + 13);
    assert_eq!(shots(&t, POPUP), [rose + 13]);
}

/// The push itself, with an idle unit that goes for no enemy: player 1's Drone, unarmed, on the middle of the lowered
/// turret's tile. Player 2's Guard appears in the turret's Range in tick `g`. The turret picks it in `g` and can't
/// rise; phase 5d of `g + 1` asks the Drone to step off, and it takes that one sidestep from `g + 2`: due east (x on a
/// tie, east where the centres coincide), 50 space units at its 8 a tick, to where its box touches the tile's edge,
/// which it reaches in `g + 8`. The turret starts to rise in that tick and fires 13 ticks later; the Drone stays where
/// it stepped to, idle.
#[test]
fn its_owners_idle_unit_over_it_is_pushed_off_and_it_rises() {
    let mut t = lowered(with_turret());
    next_debug(&mut t, spawn(DRONE, P1, CENTRE));
    let drone = newest(&t);
    t.run(3);
    assert_eq!(
        object(&t, drone).pos,
        CENTRE,
        "nothing moves it while the turret wants nothing"
    );
    next_debug(&mut t, spawn(GUARD, P2, east(4.5)));
    let (g, guard) = (t.sim().tick(), newest(&t));
    assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
    next(&mut t, P2, order(guard, OrderKind::Hold));
    assert_eq!(
        object(&t, drone).pos,
        CENTRE,
        "asked in this tick, it steps from the next"
    );
    let mut steps = Vec::new();
    while over_the_turret(&t, drone) {
        assert_eq!(popup(&t, POPUP), Some(PopUp::Lowered { blocked: true }));
        assert!(t.sim().tick() < g + 8);
        t.step();
        steps.push(object(&t, drone).pos.x - CENTRE.x);
    }
    assert_eq!(
        steps,
        [8.0, 16.0, 24.0, 32.0, 40.0, 48.0, 50.0],
        "just clear of the tile"
    );
    assert_eq!(t.sim().tick(), g + 8);
    assert_eq!(object(&t, drone).pos.y, CENTRE.y);
    assert_eq!(popup(&t, POPUP), Some(PopUp::Rising(frames(0, 8))));
    t.run_until(g + 8 + 13);
    assert_eq!(shots(&t, POPUP), [g + 8 + 13]);
    let o = object(&t, drone);
    assert_eq!(o.pos, CENTRE + Vec2::new(50.0, 0.0), "it stays where it stepped to");
    assert!(o.orders.as_ref().is_some_and(|orders| orders.is_idle()));
}

/// A dense idle group over the turret: the request passes on through the units in the way, as a mover's does
/// (`docs/movement.md > 6.6`), so the unit on the tile gets off though its neighbour stands where it must go.
#[test]
fn a_pushed_unit_passes_the_request_on_to_the_idle_unit_in_its_way() {
    let mut t = lowered(with_turret());
    next_debug(&mut t, spawn(DRONE, P1, CENTRE));
    let drone = newest(&t);
    // Its neighbour to the east, box against box, clear of the tile.
    next_debug(&mut t, spawn(DRONE, P1, CENTRE + Vec2::new(50.0, 0.0)));
    let neighbour = newest(&t);
    assert!(!over_the_turret(&t, neighbour));
    next_debug(&mut t, spawn(GUARD, P2, east(4.5)));
    let guard = newest(&t);
    next(&mut t, P2, order(guard, OrderKind::Hold));
    t.run(40);
    assert!(!over_the_turret(&t, drone), "{:?}", object(&t, drone).pos);
    assert!(object(&t, neighbour).pos.x > CENTRE.x + 50.0, "the neighbour made room");
    assert!(matches!(popup(&t, POPUP), Some(PopUp::Raised { .. })));
    assert!(!shots(&t, POPUP).is_empty());
}
