//! The map's edge (`super`): `systems/units.md > Air Collision`: "Gliders may unavoidably travel a little outside the
//! map edge while turning or circling, since they can't stop. Orders can only target points inside the map [...]. A
//! Glider outside the map edge stays visible and attackable as if it were at the nearest edge tile"; design change 0354
//! point 2: "no extra cap on overshooting the map edge"; plan decision T81: "The gate clamps orders' points into the
//! map".

use sc_sim::{Command, DebugCommand, MoveGoal, MoveStatus, PlayerCommand, Reject, VisibilityState};

use super::*;

/// A click 500 su past the west edge sends a Sparrow to the edge: the gate takes the point onto the map, to the
/// nearest point of the nearest edge tile, and the Sparrow circles that point, half of its circle past the edge.
#[test]
fn a_click_past_the_edge_sends_it_to_the_edge() {
    let mut t = quiet(flat(30, 20).spawn(SPARROW, Some(P1), point(900.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(-500.0, 640.0));
    t.step();
    assert_eq!(current(&t, 1), Some(move_to(0.0, 640.0)));
    t.run(16 * 30);
    assert_eq!(t.mover(1).orbit, Some(Vec2::new(0.0, 640.0)));
    let mut west: f32 = 0.0;
    for _ in 0..16 * 5 {
        t.step();
        west = west.min(t.pos(1).x);
        assert!(near(t.pos(1).distance(Vec2::new(0.0, 640.0)), IDLE_RADIUS, 1.0));
    }
    assert!(west < -IDLE_RADIUS + 2.0, "{west}");
}

/// Circling a point 32 su from the north edge takes a Sparrow past the edge, a third of each lap, where it counts as
/// being at the nearest edge tile (`Object::map_pos`): player 2's Militant 150 su south of the point sees it there
/// whenever it is off the map (the edge's tiles are in its sight). Then the gate takes player 2's Attack on it, given
/// while it is off the map, and the Militant's shots hit it off the map. The Sparrow is always within the Militant's
/// Range, as at the edge: the Militant never moves.
#[test]
fn past_the_edge_it_stays_visible_and_attackable_as_at_the_nearest_edge_tile() {
    let circle = Vec2::new(640.0, 32.0);
    let militant = Vec2::new(640.0, 182.0);
    let mut t = quiet(
        flat(20, 12)
            .spawn(SPARROW, Some(P1), SpawnAt::Point(circle))
            .spawn(ObjectKind::Militant, Some(P2), SpawnAt::Point(militant))
            .start(),
    );
    t.run(16 * 10);
    let (mut off, mut seen_off) = (0, 0);
    for _ in 0..16 * 10 {
        t.step();
        let sparrow = t.sim().object(ObjectId(1)).expect("the Sparrow lives");
        if !t.sim().map().contains_point(sparrow.pos) {
            off += 1;
            let at = sparrow.map_pos(t.sim().map());
            assert!(t.sim().map().contains_point(at) && at.y == 0.0, "{at}");
            if t.sim().vision().object_state(P2, sparrow) == VisibilityState::Visible {
                seen_off += 1;
            }
        }
    }
    assert!(off > 16 * 3, "off the map {off} of {} ticks", 16 * 10);
    assert_eq!(seen_off, off, "seen whenever it is off the map");
    while t.sim().map().contains_point(t.pos(1)) {
        t.step();
    }
    let attack = PlayerCommand {
        player: P2,
        command: Command::Order {
            units: vec![ObjectId(2)],
            kind: OrderKind::Attack(ObjectId(1)),
            queue: false,
        },
    };
    assert_eq!(t.sim().validate(&attack), Ok(()), "off the map at {}", t.pos(1));
    let tick = t.sim().tick() + 1;
    t.at(tick, P2, attack.command);
    let mut hit_off = 0;
    for _ in 0..16 * 10 {
        let before = hits(&t).len();
        t.step();
        let shot = hits(&t)[before..]
            .iter()
            .any(|&(_, attacker, target, _)| (attacker, target) == (2, 1));
        if shot && !t.sim().map().contains_point(t.pos(1)) {
            hit_off += 1;
        }
        if hit_off == 2 {
            break;
        }
    }
    assert_eq!(hit_off, 2, "hit off the map");
    assert_eq!(t.pos(2), militant, "the Militant never moved");
}

/// (M6-GK) A Hornet, the HoverCraft of these tests: 48 × 48, its box reaching 24 su from its centre.
const HORNET: ObjectKind = ObjectKind::Hornet;
const HORNET_HALF: f32 = 24.0;

/// (M6-GK) The gate puts a HoverCraft only where its box lies on the map (`systems/units.md > Air Collision > Map
/// edge`: "HoverCraft stay inside the map"), as it puts a ground unit only where its box is clear of the tiles and inside
/// the map: a spawn or a teleport of a Hornet whose box would hang over an edge is refused as Blocked, as a Militant's
/// is. A Glider may be put there: it may fly past the edge.
#[test]
fn a_hovercraft_is_put_only_where_its_box_lies_on_the_map() {
    assert_eq!(HORNET.def().half_extents(), Some(Vec2::splat(HORNET_HALF)));
    let t = quiet(flat(20, 12).spawn(HORNET, Some(P1), point(640.0, 384.0)).start());
    let debug = |command| PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    };
    let spawn = |kind, x, y| {
        debug(DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at: point(x, y),
            orientation: Default::default(),
        })
    };
    let teleport = |x, y| {
        debug(DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: Vec2::new(x, y),
        })
    };
    let sim = t.sim();
    // The map is 1280 × 768 su: each point is a hair over an edge for the Hornet's box, then just inside it.
    let over = [
        (23.0, 300.0),
        (1257.0, 300.0),
        (640.0, 23.0),
        (640.0, 745.0),
        (10.0, 10.0),
    ];
    for (x, y) in over {
        assert_eq!(sim.validate(&spawn(HORNET, x, y)), Err(Reject::Blocked), "({x}, {y})");
        assert_eq!(sim.validate(&teleport(x, y)), Err(Reject::Blocked), "({x}, {y})");
        assert_eq!(sim.validate(&spawn(SPARROW, x, y)), Ok(()), "a Glider at ({x}, {y})");
    }
    assert_eq!(
        sim.validate(&spawn(ObjectKind::Militant, 5.0, 300.0)),
        Err(Reject::Blocked)
    );
    let inside = [
        (24.0, 300.0),
        (1256.0, 300.0),
        (640.0, 24.0),
        (640.0, 744.0),
        (24.0, 24.0),
    ];
    for (x, y) in inside {
        assert_eq!(sim.validate(&spawn(HORNET, x, y)), Ok(()), "({x}, {y})");
        assert_eq!(sim.validate(&teleport(x, y)), Ok(()), "({x}, {y})");
    }
}

/// (M6-GK) A HoverCraft's own move is held inside the map (design change 0045: "clamp HoverCraft to the map"). A Hornet
/// with its box against the east edge, facing east, is sent north. It turns 8° a frame, and its forward thrust along
/// its facing pushes it east as it turns: the drift that took one past the edge in the fuzz's seed 81. The edge holds
/// it, its box on the map every tick, and stops it as an obstacle does (design change 0035): the move's part past the
/// edge is dropped, and no momentum is left. It flies on north along the edge and arrives.
#[test]
fn a_hovercraft_turning_at_the_edge_is_held_inside_the_map() {
    let edge = 1280.0 - HORNET_HALF;
    let goal = Vec2::new(edge, 100.0);
    let mut t = quiet(flat(20, 12).spawn(HORNET, Some(P1), point(edge, 700.0)).start());
    order(&mut t, 1, P1, &[1], OrderKind::Move(Target::Point(goal)));
    let mut held = 0;
    for _ in 0..16 * 30 {
        t.step();
        let (pos, mover) = (t.pos(1), t.mover(1));
        assert!(pos.x <= edge, "its box past the edge at {pos}");
        // Its move cut at the edge: it went north alone, with no momentum left.
        if mover.momentum == Vec2::ZERO && mover.velocity != Vec2::ZERO {
            assert_eq!((pos.x, mover.velocity.x), (edge, 0.0), "{mover:?}");
            held += 1;
        }
        if mover.status == MoveStatus::Arrived {
            break;
        }
    }
    assert!(held > 0, "the edge never held it");
    assert_eq!((t.pos(1), t.mover(1).status), (goal, MoveStatus::Arrived));
}

/// (M6-GK) A HoverCraft sent to a point within half its box of the map's edge arrives where its box lies on the map,
/// Idle, without a repath and without failing. Its path ends at the nearest point at which its box lies on the map
/// (`paths::serve`), as a ground unit's ends at the nearest point it can reach; within a step of its path's end it lands
/// on it and has arrived (`movement::follow`'s arrival rule). A Hornet sent to the map's corner stops with its box in the
/// corner; sent past the east edge (the gate takes the click onto the map) or near the south edge, with its box against
/// that edge.
#[test]
fn a_hovercraft_sent_to_the_corner_or_the_edge_arrives_with_its_box_on_the_map() {
    let (right, bottom) = (1280.0 - HORNET_HALF, 768.0 - HORNET_HALF);
    let cases = [
        (Vec2::new(0.0, 0.0), Vec2::splat(HORNET_HALF)),
        (Vec2::new(1500.0, 300.0), Vec2::new(right, 300.0)),
        (Vec2::new(640.0, 760.0), Vec2::new(640.0, bottom)),
    ];
    for (asked, end) in cases {
        let mut t = quiet(flat(20, 12).spawn(HORNET, Some(P1), point(640.0, 384.0)).start());
        order(&mut t, 1, P1, &[1], move_to(asked.x, asked.y));
        t.step();
        let repaths = t.mover(1).repaths_left;
        while current(&t, 1).is_some() {
            t.step();
            assert!(t.sim().tick() < 16 * 30, "still on its way to {asked} at {}", t.pos(1));
        }
        let (unit, mover) = (t.sim().object(ObjectId(1)).unwrap(), t.mover(1));
        assert_eq!(unit.pos, end, "sent to {asked}");
        assert!(unit.orders.as_ref().unwrap().is_idle());
        assert_eq!(
            (mover.status, mover.goal, mover.repaths_left),
            (MoveStatus::Arrived, MoveGoal::None, repaths)
        );
        assert_eq!((mover.velocity, mover.momentum), (Vec2::ZERO, Vec2::ZERO), "at rest");
        let failed = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }));
        assert!(!failed, "sent to {asked}");
    }
}
