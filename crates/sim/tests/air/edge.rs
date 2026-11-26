//! The map's edge (`super`): `systems/units.md > Air Collision`: "Gliders may unavoidably travel a little outside the
//! map edge while turning or circling, since they can't stop. Orders can only target points inside the map [...]. A
//! Glider outside the map edge stays visible and attackable as if it were at the nearest edge tile"; design change 0354
//! point 2: "no extra cap on overshooting the map edge"; plan decision T81: "The gate clamps orders' points into the
//! map".

use sc_sim::{Command, PlayerCommand, VisibilityState};

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
