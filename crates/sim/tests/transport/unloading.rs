//! Unload All (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Unloading**: **F: Unload
//! All** (above ground only; they come out around it)"; `sc_sim::carry`, `orders::unload`): an order, untargeted and
//! queueable, that brings the Transport to rest and then puts every passenger back on the map round it, on the rings
//! round its shape, none overlapping it or another (`TestSim`'s no-overlap invariant I1 checks every tick), each idle
//! and facing as the Transport does. **Placeholder (not in the design: where round it each comes out, and that the
//! carrier first comes to rest)**. A pocket with too little room for them is `sim/holding/carriers/tests.rs`'s.

use super::*;

/// A Guard's ring round a carrier, as `Sim::unload` spaces them: the circle round its centre that holds its Silhouette
/// however it lies, and `Holding::unload_gap_su`, 4 su, between it and the shape inside.
const GUARD_RADIUS: f32 = 25.455_845;
const GAP: f32 = 4.0;

/// Where unit `id` stands in `carrier`'s frame: along its facing, and across it to its left.
fn in_frame(t: &TestSim, carrier: ObjectId, id: ObjectId) -> Vec2 {
    let c = object(t, carrier);
    let ahead = sc_sim::math::facing_vector(c.facing);
    let across = Vec2::new(-ahead.y, ahead.x);
    let d = object(t, id).pos - c.pos;
    Vec2::new(d.dot(ahead), d.dot(across))
}

/// The plan's acceptance: "F gives a ring with no overlap". Four Guards aboard a Transport standing still: in the tick
/// of its Unload All, all four come out, longest aboard first, at the four corners of the nearest ring, a rectangle
/// a Guard's radius and the gap clear of the Transport's shape (65.5 × 53.5 su from its centre, along and across its
/// facing), so that none overlaps it or another. Each is idle, facing as the Transport does; the Transport's order is
/// over, its room 4 again, and the Tunnel Space in use the same.
#[test]
fn four_guards_come_out_on_the_nearest_ring_with_no_overlap() {
    let mut t = transport_and_guards(4);
    let transport = id(1);
    let guards: Vec<ObjectId> = (2..6).map(id).collect();
    board(&mut t, &guards, transport);
    let order_aboard = aboard(&t, transport);
    let space = space_used(&t, P1);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[transport], OrderKind::UnloadAll));
    t.run_until(tick);
    let out: Vec<ObjectId> = unloaded(&t, transport).into_iter().map(|(_, unit)| unit).collect();
    assert_eq!(
        out, order_aboard,
        "all out in the tick of the order, longest aboard first"
    );
    assert!(unloaded(&t, transport).iter().all(|&(at, _)| at == tick));
    let corner = HALF + Vec2::splat(GUARD_RADIUS + GAP);
    let mut corners: Vec<(i32, i32)> = Vec::new();
    for &guard in &guards {
        let p = in_frame(&t, transport, guard);
        assert!(
            (p.x.abs() - corner.x).abs() < 0.01 && (p.y.abs() - corner.y).abs() < 0.01,
            "{guard:?} at {p} in the Transport's frame: on a corner of the nearest ring"
        );
        corners.push((p.x.signum() as i32, p.y.signum() as i32));
        assert!(is_idle(&t, guard), "{guard:?} idles");
        assert_eq!(object(&t, guard).facing, object(&t, transport).facing);
    }
    corners.sort_unstable();
    assert_eq!(corners, [(-1, -1), (-1, 1), (1, -1), (1, 1)], "one on each corner");
    let (a, b) = (object(&t, guards[0]).pos, object(&t, guards[1]).pos);
    assert!(a.distance(b) >= 2.0 * GUARD_HALF, "apart");
    assert!(aboard(&t, transport).is_empty());
    assert!(is_idle(&t, transport), "its order is over");
    assert_eq!(t.sim().room(transport), CAPACITY as u32);
    assert_eq!(space_used(&t, P1), space);
}

/// "the carrier first comes to rest": a loaded Transport driving east, given Unload All on its way, brakes, and its
/// passengers come out only once it is at rest, round where it stopped, short of where it was going.
#[test]
fn a_driving_transport_comes_to_rest_before_it_unloads() {
    let mut t = transport_and_guards(2);
    let transport = id(1);
    board(&mut t, &[id(2), id(3)], transport);
    let go = t.sim().tick() + 1;
    t.at(go, P1, order(&[transport], move_to(1800.0, 600.0)));
    t.run_until(go + 30);
    assert!(t.mover(1).momentum.length() > 4.0, "at speed");
    let given = t.sim().tick() + 1;
    t.at(given, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "it unloads", |t| !unloaded(t, transport).is_empty());
    let out = t.sim().tick();
    for (tick, _) in unloaded(&t, transport) {
        assert_eq!(tick, out);
    }
    assert!(out > given, "not in the tick of the order: it was braking");
    assert_eq!(t.mover(1).momentum, Vec2::ZERO, "at rest");
    let stopped = object(&t, transport).pos;
    assert!(stopped.x < 1700.0, "short of its goal: {stopped}");
    for guard in [id(2), id(3)] {
        let p = in_frame(&t, transport, guard);
        assert!(p.length() < 100.0, "{guard:?} round where it stopped: {p}");
    }
}

/// "Queueable": a Shift-queued Unload All after a Move waits for the move to end, and unloads where it ends.
#[test]
fn shift_queues_an_unload_after_a_move() {
    let mut t = transport_and_guards(2);
    let transport = id(1);
    board(&mut t, &[id(2), id(3)], transport);
    let go = t.sim().tick() + 1;
    let goal = Vec2::new(1000.0, 800.0);
    t.at(go, P1, order(&[transport], move_to(goal.x, goal.y)));
    t.at(go, P1, queued(&[transport], OrderKind::UnloadAll));
    t.run_until(go + 5);
    assert_eq!(aboard(&t, transport).len(), 2, "not yet");
    step_until(&mut t, 400, "it unloads", |t| aboard(t, transport).is_empty());
    assert!(object(&t, transport).pos.distance(goal) < 1.0, "where the move ended");
    for guard in [id(2), id(3)] {
        assert!(object(&t, guard).pos.distance(goal) < 100.0);
    }
}

/// The gate (`Sim::check_unload`): an Unload All is refused to a Transport with no one aboard
/// ([`Reject::NothingInside`], as an Eject All with no one inside) and to a Guard ([`Reject::LacksAbility`]); given to
/// a loaded Transport with a Guard beside it, the Transport takes it and the Guard ignores it.
#[test]
fn unload_all_is_refused_with_no_one_aboard_and_to_a_unit_that_carries_none() {
    let mut t = transport_and_guards(2);
    let (transport, guard, other) = (id(1), id(2), id(3));
    assert_eq!(
        gate(&t, P1, order(&[transport], OrderKind::UnloadAll)),
        Err(Reject::NothingInside)
    );
    assert_eq!(
        gate(&t, P1, order(&[guard], OrderKind::UnloadAll)),
        Err(Reject::LacksAbility)
    );
    board(&mut t, &[guard], transport);
    assert_eq!(gate(&t, P1, order(&[transport, other], OrderKind::UnloadAll)), Ok(()));
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[other], move_to(400.0, 900.0)));
    t.at(tick + 1, P1, order(&[transport, other], OrderKind::UnloadAll));
    t.run_until(tick + 1);
    assert_eq!(unloaded(&t, transport).len(), 1);
    let kind = (object(&t, other).orders.as_ref())
        .and_then(|orders| orders.current.as_ref())
        .map(|active| active.order.kind);
    assert_eq!(kind, Some(move_to(400.0, 900.0)), "the Guard keeps its move");
}
