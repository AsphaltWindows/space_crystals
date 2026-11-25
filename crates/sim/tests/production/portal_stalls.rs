//! A portal whose way out is blocked (slice M3-D2, design change 0145, `factions/colonists/structures/README.md >
//! Portals > Portal Exits (energy doorway)`, points 4 and 5): the unit that can't get out stays half out at the edge of
//! the footprint, a normal unit; the next one reaches 100% and waits inside until the first has left the doorway, and
//! is refunded if the portal is destroyed (design changes 0084 and 0085). And invariants I1 and I2 around the energy
//! doorway: negative tests, in the way of `tests/invariants_overlap.rs` (the movement step never breaks them, so they
//! put a unit where it must not be by editing a saved state, and ask the checker).

use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, ObjectId, OrderKind, PlayerCommand, Rotation, Sim, SimEvent, Target, TickInputs, Vec2,
};

use super::doors::{exit_of, waiting};
use super::portals::{
    DRONE, PORTAL, ROBO, SABRE, beacon_capacity, bio_portal, robo_portal, run_exits, set_rally, train,
};
use super::queue::{bank, crystals, queue};
use super::{P1, P2, PK, at};

/// Where the first Drone of [`ringed`] is rallied: due east.
const EAST: Vec2 = Vec2::new(1500.0, 512.0);

/// The centres of [`ringed`]'s Peacekeepers, in the order they are spawned: 30 su out from each side of the Robo
/// Portal (x 768..1024, y 384..640), so their near faces are 18 su off it, every 24 su (they touch) and the far
/// corner; the gaps they leave are 4 su.
fn ring() -> Vec<Vec2> {
    let along = |from: f32| (0..13).map(move |k| from + 24.0 * k as f32).chain([from + 316.0]);
    let mut spots = Vec::new();
    for x in along(738.0) {
        spots.extend([Vec2::new(x, 354.0), Vec2::new(x, 670.0)]);
    }
    for y in along(354.0).skip(1).take(12) {
        spots.extend([Vec2::new(738.0, y), Vec2::new(1054.0, y)]);
    }
    spots
}

/// The Peacekeepers of the ring right in front of the middle of the east side: they stop a Drone walking east.
fn east_of_the_middle() -> [ObjectId; 2] {
    let spots = ring();
    [498.0, 522.0].map(|y| {
        let index = spots.iter().position(|&p| p == Vec2::new(1054.0, y)).unwrap();
        ObjectId(4 + index as u32)
    })
}

/// The Robo Portal of `portals::robo_portal` in a ring of player 2's Peacekeepers ([`ring`]), holding their fire: a
/// Drone that walks out of its centre stops against them, whichever way it goes. Player 1 has 150 Space Crystals,
/// its rally point is [`EAST`], and it trains `n` Drones in tick 1.
fn ringed(n: usize) -> TestSim {
    let mut builder = robo_portal();
    for spot in ring() {
        builder = builder.spawn(PK, Some(P2), at(spot.x, spot.y));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, 150));
    set_rally(&mut t, 1, Target::Point(EAST));
    train(&mut t, 1, DRONE, n);
    t
}

/// 0145's Verification: "Portal surrounded by units: the first unit stops half out; the next reaches 100% and waits;
/// it appears after the first moves away." The first Drone, rallied east, finds no free way out and leaves along the
/// rally's direction all the same: it stops against the ring with its centre on the east side, its order failing
/// in time, and stays there, in the doorway. The second is finished at tick 320 and waits. Two Peacekeepers taken
/// away: the first walks on out, and the second appears at once.
#[test]
fn a_portal_surrounded_by_units_holds_its_next_unit_until_the_first_is_out() {
    let mut t = ringed(2);
    let exits = run_exits(&mut t, 399);
    assert_eq!(exits.len(), 1);
    let first = exits[0].unit;
    assert_eq!(exits[0].heading, (EAST - ROBO).normalize());
    assert_eq!(
        (t.pos(first.0), exit_of(t.sim(), first)),
        (Vec2::new(1024.0, 512.0), Some(PORTAL)),
        "half out"
    );
    let failed = t
        .events()
        .iter()
        .any(|(_, e)| matches!(e, SimEvent::MoveFailed { unit } if *unit == first));
    assert!(failed, "its order failed");
    let status = t.sim().production_status(PORTAL).unwrap();
    assert!(status.waiting && status.progress == status.total, "held at 100%");
    assert!(waiting(t.sim(), PORTAL));
    assert_eq!(queue(t.sim(), PORTAL), 1);

    for id in east_of_the_middle() {
        t.debug_at(400, DebugCommand::Remove { id });
    }
    let after = run_exits(&mut t, 440);
    assert_eq!(after.len(), 1);
    let second = after[0].unit;
    assert!(
        after[0].tick > 400 && after[0].tick <= 420,
        "at once: {}",
        after[0].tick
    );
    assert_eq!(exit_of(t.sim(), first), None, "the first is out");
    assert!(t.pos(first.0).x >= 1042.0, "{}", t.pos(first.0));
    // The second goes the same way, the first making way for it (an idle unit of its owner, `core/entities.md > Exit
    // Doorway` point 4), out through the gap and on to the rally point.
    assert_eq!(after[0].heading, exits[0].heading);
    t.run_until(700);
    assert_eq!(exit_of(t.sim(), second), None);
    assert!(t.pos(second.0).distance(EAST) < 100.0, "{}", t.pos(second.0));
}

/// Design changes 0084 and 0085 at a portal (0145, point 5: "a waiting unit is refunded if the portal is destroyed,
/// like any producer's"): the ringed portal holds its second Drone at 100%, the third queued behind it. Destroyed,
/// it refunds both, and frees the Beacon Capacity the waiting Drone took when its production started (design change
/// 0143); the 5 the portal provided go with it. The Drone half out of it is an ordinary unit where it stood.
#[test]
fn a_destroyed_portal_refunds_the_unit_waiting_at_100_percent() {
    let mut t = ringed(3);
    let exits = run_exits(&mut t, 330);
    assert_eq!(exits.len(), 1);
    assert!(waiting(t.sim(), PORTAL));
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (0, beacon_capacity(2, 15)));
    t.debug_at(331, DebugCommand::Remove { id: PORTAL });
    t.run_until(340);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (100, beacon_capacity(1, 10)));
    let refunds = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::ProductionCancelled { refund: 50, .. }))
        .count();
    assert_eq!(refunds, 2);
    let first = exits[0].unit;
    let mover = t.sim().object(first).unwrap().mover.as_ref().unwrap();
    assert_eq!(
        (t.pos(first.0), mover.exit, mover.exit_heading),
        (Vec2::new(1024.0, 512.0), None, None)
    );
    assert_eq!(dangling_references(t.sim()), []);
    // Where the portal stood is open ground now.
    t.order_at(341, P1, &[first], OrderKind::Move(Target::Point(ROBO)), false);
    t.run_until(450);
    assert_eq!(t.pos(first.0), ROBO);
}

/// `sim` as saved and loaded again, with unit `id` at `to`, having been at `from` a tick earlier, and with its
/// mover's `exit` naming `exit`, if that is given.
fn edited(sim: &Sim, id: ObjectId, from: Vec2, to: Vec2, exit: Option<ObjectId>) -> Sim {
    let mut json = serde_json::to_value(sim).unwrap();
    let objects = json["objects"].as_array_mut().unwrap();
    let object = objects.iter_mut().find(|o| o["id"] == id.0).unwrap();
    object["pos"] = serde_json::to_value(to).unwrap();
    object["prev_pos"] = serde_json::to_value(from).unwrap();
    if let Some(exit) = exit {
        object["mover"]["exit"] = serde_json::to_value(exit).unwrap();
    }
    serde_json::from_value(json).unwrap()
}

/// The inputs of a tick that teleported unit `id` to `to`.
fn teleport(id: ObjectId, to: Vec2) -> TickInputs {
    let command = Command::Debug(DebugCommand::SetPosition { id, pos: to });
    TickInputs {
        commands: vec![PlayerCommand { player: P1, command }],
    }
}

/// The invariant the checker, which last saw `before`, finds broken in `after`, a tick on with `inputs`. Others may
/// object to an edited state (its vision is stale): I1 and I2 are checked before them.
fn broken(before: &Sim, after: &Sim, inputs: &TickInputs) -> Option<&'static str> {
    Invariants::new(before)
        .check(after, inputs)
        .err()
        .map(|violation| violation.invariant)
}

/// Whether the checker finds I1 or I2 broken.
fn breaks_i1_or_i2(before: &Sim, after: &Sim, inputs: &TickInputs) -> bool {
    matches!(broken(before, after, inputs), Some("I1" | "I2"))
}

/// I1 (no two units of a layer overlap, but for design change 0106's exceptions, `invariants::may_overlap`): the
/// energy doorway adds no way for two units to overlap (a unit appears at the centre only where its box is clear of
/// every unit, and walks out only up to contact), so `may_overlap` takes no exception for it, and a unit in the
/// doorway that overlaps one outside breaks I1: its exception is against its own portal's tiles only.
#[test]
fn a_unit_in_an_energy_doorway_that_overlaps_another_unit_breaks_i1() {
    let mut t = ringed(1);
    let exits = run_exits(&mut t, 330);
    let (first, before) = (exits[0].unit, t.sim().clone());
    assert_eq!(t.pos(first.0), Vec2::new(1024.0, 512.0));
    // Put where it stands, it breaks nothing.
    let stands = edited(&before, first, t.pos(first.0), t.pos(first.0), None);
    assert!(!breaks_i1_or_i2(&before, &stands, &teleport(first, t.pos(first.0))));
    // 6 su farther east: still in its doorway, inside the two Peacekeepers' boxes.
    let into = Vec2::new(1030.0, 512.0);
    let after = edited(&before, first, into, into, None);
    assert_eq!(broken(&before, &after, &teleport(first, into)), Some("I1"));
}

/// I2 at an energy doorway: the unit in it only ever moves away from the portal's centre, never stands off the open
/// arc, and only a unit the portal released is in it.
#[test]
fn an_energy_doorway_unit_that_goes_back_in_or_off_the_arc_or_an_outside_unit_in_it_breaks_i2() {
    // Back towards the centre, as a move.
    let mut t = ringed(1);
    let exits = run_exits(&mut t, 330);
    let (first, before) = (exits[0].unit, t.sim().clone());
    let back = Vec2::new(1000.0, 512.0);
    let after = edited(&before, first, back, back, None);
    assert_eq!(broken(&before, &after, &TickInputs::default()), Some("I2"));
    // A Peacekeeper of the ring put into the footprint with the portal's doorway as its own.
    let outsider = ObjectId(4);
    let from = t.pos(outsider.0);
    let inside = Vec2::new(800.0, 420.0);
    let after = edited(&before, outsider, from, inside, Some(PORTAL));
    assert_eq!(broken(&before, &after, &TickInputs::default()), Some("I2"));

    // Off the arc: a Bio Portal unturned, open north and east only (x 768..960, y 384..576), its Sabre a tick out
    // of the centre, put south-west of the centre, in the footprint: through a closed side.
    let mut t = bio_portal(Rotation::R0, false, |b| b);
    t.debug_at(1, crystals(P1, 100));
    set_rally(&mut t, 1, Target::Point(Vec2::new(1500.0, 480.0)));
    train(&mut t, 1, SABRE, 1);
    let exits = run_exits(&mut t, 161);
    let (sabre, before) = (exits[0].unit, t.sim().clone());
    let centre = Vec2::new(864.0, 480.0);
    let off = centre + Vec2::new(-14.0, 20.0);
    let after = edited(&before, sabre, off, off, None);
    assert_eq!(broken(&before, &after, &teleport(sabre, off)), Some("I2"));
    // As far out on the arc: nothing broken.
    let on = centre + Vec2::new(14.0, -20.0);
    let after = edited(&before, sabre, on, on, None);
    assert!(!breaks_i1_or_i2(&before, &after, &teleport(sabre, on)));
}
