//! Patches running out (`factions/colonists/units/drone.md > When a Gather ends`): the Drones going on at
//! another patch, or delivering and idling when no patch is left.

use super::*;

/// The Drones' grid.
fn blocked(t: &TestSim, x: i32, y: i32) -> bool {
    let grid = t.sim().nav().grid(NavClass::GroundRugged, 1).expect("the Drone's grid");
    grid.is_blocked(TilePos::new(x, y))
}

/// `core/map.md > SpaceCrystalsPatch`: "When a patch is fully depleted, it disappears from the map", and its tile
/// opens (design change 0038). Design change 0138: the last load is what is left, after the full MiningDuration.
/// Design change 0134: the Drone that mined it looks for another patch where it stands, and goes on there once it
/// has delivered.
#[test]
fn a_depleted_patch_disappears_and_its_drones_go_on_at_another() {
    // Patch 1 holds 20; patch 2 is two tiles east of it; node 3; Drone 4.
    let mut t = flat(16, 10)
        .spawn_with(patch_of(20), tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    run_to(&mut t, 400, "two loads delivered", |t| deliveries(t).len() == 2);
    assert_eq!((remaining(&t, 1), bank(&t, P1)), (4, 100 + 2 * LOAD));
    assert!(blocked(&t, 5, 2));
    let began = mines_from(&mut t, 4);
    t.run_until(began + MINING - 2);
    assert_eq!(
        (remaining(&t, 1), load(&t, 4)),
        (4, 0),
        "the full 32 frames for the last 4"
    );
    t.step();
    // The tick the patch is emptied in.
    assert!(t.sim().object(id(1)).is_none(), "the patch is gone");
    assert!(t.sim().events().contains(&SimEvent::Removed(id(1))));
    assert!(!blocked(&t, 5, 2), "its tile is open in the same tick");
    assert_eq!(load(&t, 4), 4, "a partial last load");
    assert_eq!(
        dangling_references(t.sim()),
        [],
        "its Gather names the patch only to say where it was"
    );
    // It looks round as the behaviours next run.
    t.step();
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))), "on to the other patch");
    assert_eq!(phase(&t, 4), Some(GatherPhase::ToDropOff), "after it has delivered");
    run_to(&mut t, 100, "the last load delivered", |t| deliveries(t).len() == 3);
    assert_eq!(deliveries(&t)[2].3, 4);
    assert_eq!(bank(&t, P1), 100 + 20);
    mines_from(&mut t, 4);
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))));
}

/// `> When a Gather ends` (design change 0123): with no patch left, the Gather ends, after the Drone has delivered
/// its load. Design change 0134 (its Verification: "Deplete the last patch: Drones arrive, find nothing, deliver
/// any load and idle"): the Drone on its way back to the patch walks on to where it was before it looks round.
#[test]
fn with_no_patch_left_a_drone_delivers_its_load_and_idles() {
    // Patch 1 holds 12, node 2, Drone 3 mines first (south), Drone 4 waits (north).
    let mut t = flat(16, 10)
        .spawn_with(patch_of(12), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    // Drone 3 takes 8 and delivers them; Drone 4 takes the last 4 while Drone 3 is on its way back for more.
    let gone = run_to(&mut t, 200, "the patch is mined out", |t| {
        t.sim().object(id(1)).is_none()
    });
    assert_eq!((load(&t, 3), load(&t, 4), bank(&t, P1)), (0, 4, 100 + LOAD));
    assert_eq!(dangling_references(t.sim()), []);
    t.step();
    assert_eq!(
        (order(&t, 4), phase(&t, 4)),
        (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToDropOff)),
        "it found nothing, and delivers what it carries before its Gather ends"
    );
    assert_eq!(
        (order(&t, 3), phase(&t, 3)),
        (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToPatch)),
        "on to where the patch was"
    );
    // It arrives, finds nothing, and its Gather ends: it brakes to a stop beside the empty tile.
    let ended = run_to(&mut t, 100, "Drone 3 idles", |t| order(t, 3).is_none());
    assert!(ended > gone + 1);
    let spot = TilePos::new(5, 2).center();
    assert!(
        object(&t, 3).pos.distance(spot) < 64.0,
        "beside the empty tile: {:?}",
        object(&t, 3).pos
    );
    run_to(&mut t, 200, "the last load delivered", |t| deliveries(t).len() == 2);
    t.run(2);
    assert_eq!((order(&t, 3), order(&t, 4)), (None, None), "idle");
    assert_eq!(bank(&t, P1), 100 + 12);
    assert_eq!(t.sim().returning_drones(id(2)), []);
}
