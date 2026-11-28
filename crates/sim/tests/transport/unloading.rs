//! Unload All (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Unloading**: **F: Unload
//! All** (above ground only; they come out around it)"; `sc_sim::carry`, `orders::unload`): an order, untargeted and
//! queueable, that brings the Transport to rest and then puts every passenger back on the map round it, on the rings
//! round its shape, none overlapping it or another (`TestSim`'s no-overlap invariant I1 checks every tick), each idle
//! and facing as the Transport does (`systems/mechanics/carrying.md > Unloading`: "**On the move, it stops first**";
//! "the **nearest free spots** around the carrier"; "**Crowded**: passengers with no free spot **stay aboard**"). A
//! pocket with too little room for them is `sim/holding/carriers/tests.rs`'s, and through the gate
//! `a_packed_unload_leaves_the_rest_aboard_until_a_move_and_another_f`'s.

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

/// Design change 0469's Verification 3, "Pressing F while it drives makes it stop, then unload" ("**On the move, it
/// stops first**"): a loaded Transport driving east, given Unload All on its way, brakes, and its passengers come out
/// only once it is at rest, round where it stopped, short of where it was going.
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

/// A pocket of open ground on tiles (4, 2) to (6, 3), x 256..448 and y 128..256, closed by cliffs but for its way east
/// on tiles (7, 2) to (8, 3), which player 2's Generator ([`PLUG`]) fills, onto the open ground east of column 8.
const POCKET: [&str; 8] = [
    "ccccccccc.......",
    "ccccccccc.......",
    "cccc............",
    "cccc............",
    "ccccccccc.......",
    "ccccccccc.......",
    "ccccccccc.......",
    "ccccccccc.......",
];
/// Player 2's Generator in [`POCKET`]'s way out; player 2 has another in the north-east corner.
const PLUG: ObjectId = ObjectId(6);

/// Design change 0469's Verification 1, "Unload a full Transport with the ground around it packed. Only those with
/// room come out; the rest stay and can be unloaded after moving": player 1's Transport, facing east at the east end of
/// [`POCKET`], with its 4 Guards aboard. Its F lets out the two longest aboard, behind it, where the pocket has room
/// for them, and the other two stay aboard, the order over. Once the Generator is gone, it drives out onto the open
/// ground, and its next F lets the other two out round it.
#[test]
fn a_packed_unload_leaves_the_rest_aboard_until_a_move_and_another_f() {
    let mut scenario = ScenarioBuilder::new(&POCKET.join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(TRANSPORT, Some(P1), point(410.0, 192.0));
    for (x, y) in [(280.0, 150.0), (280.0, 234.0), (330.0, 150.0), (330.0, 234.0)] {
        scenario = scenario.spawn(GUARD, Some(P1), point(x, y));
    }
    let scenario = (scenario.spawn(ObjectKind::Generator, Some(P2), tile(7, 2))).spawn(
        ObjectKind::Generator,
        Some(P2),
        tile(14, 0),
    );
    let mut t = started(scenario);
    let transport = id(1);
    let guards = [id(2), id(3), id(4), id(5)];
    board(&mut t, &guards, transport);
    let first = aboard(&t, transport);
    assert_eq!(first.len(), CAPACITY, "full");

    let f = t.sim().tick() + 1;
    t.at(f, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "it unloads", |t| !unloaded(t, transport).is_empty());
    let out: Vec<ObjectId> = unloaded(&t, transport).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(out, first[..2], "the two longest aboard, where there is room");
    for &guard in &out {
        assert!(in_frame(&t, transport, guard).x < 0.0, "{guard:?} behind it");
    }
    t.run(32);
    assert_eq!(aboard(&t, transport), first[2..], "no free spot: aboard still");
    assert!(is_idle(&t, transport), "the order over");

    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: PLUG });
    let goal = Vec2::new(800.0, 192.0);
    t.at(now + 2, P1, order(&[transport], move_to(goal.x, goal.y)));
    step_until(&mut t, 400, "out on the open ground", |t| {
        object(t, transport).pos.distance(goal) < 2.0 && is_idle(t, transport)
    });
    assert_eq!(aboard(&t, transport), first[2..], "aboard on the way");
    let again = t.sim().tick() + 1;
    t.at(again, P1, order(&[transport], OrderKind::UnloadAll));
    step_until(&mut t, 60, "the rest unload", |t| aboard(t, transport).is_empty());
    let rest: Vec<ObjectId> = (unloaded(&t, transport).iter())
        .filter(|&&(tick, _)| tick >= again)
        .map(|&(_, unit)| unit)
        .collect();
    assert_eq!(rest, first[2..], "the other two, after the move");
    for &guard in &rest {
        let p = in_frame(&t, transport, guard);
        assert!(p.length() < 100.0, "{guard:?} round it: {p}");
    }
}
