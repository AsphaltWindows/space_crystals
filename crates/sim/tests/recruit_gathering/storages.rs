//! Which Storage a Recruit delivers to. **Placeholder (Q0102 points 1 and 2)**, the question's proposal (a): "The
//! nearest own completed Storage, edge to edge from the patch, chosen at each return. With none, the Recruit idles
//! holding its load." (`factions/cults/units/recruit.md > Gathering` says only "deliver them to Storage".)

use super::*;

/// Patch 1 on tile (10, 8); Storage 2 west of it on tiles (6, 8) to (7, 9), 2 tiles from its edge; Storage 3 east of
/// it on (14, 8) to (15, 9), 3 tiles from its edge; Recruit 4 east of the patch, so that it mines at the patch's east
/// face, nearer to Storage 3 than to Storage 2.
fn two_storages() -> ScenarioBuilder {
    flat(24, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(6, 8))
        .spawn(STORAGE, Some(P1), tile(14, 8))
        .spawn(RECRUIT, Some(P1), centre(12, 8))
}

fn spawn_storage(t: &mut TestSim, owner: PlayerId, x: i32, y: i32) {
    let now = t.sim().tick();
    let spawn = DebugCommand::Spawn {
        kind: STORAGE,
        owner: Some(owner),
        at: tile(x, y),
        orientation: Default::default(),
    };
    t.debug_at(now + 1, spawn);
}

fn remove(t: &mut TestSim, n: u32) {
    let now = t.sim().tick();
    t.debug_at(now + 1, DebugCommand::Remove { id: id(n) });
}

/// "Edge to edge from the patch": the Recruit stands east of the patch with its load, nearer to Storage 3 (by its
/// centre and by its edge), and delivers to Storage 2, the nearer one to the patch.
#[test]
fn the_storage_nearest_to_the_patch_edge_to_edge_takes_the_load() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    let me = Shape::of(object(&t, 4));
    let (west, east) = (Shape::of(object(&t, 2)), Shape::of(object(&t, 3)));
    assert!(me.gap(&east) < me.gap(&west), "the Recruit is nearer to Storage 3");
    let patch = Shape::of(object(&t, 1));
    assert_eq!((patch.gap(&west), patch.gap(&east)), (128.0, 192.0));
    run_to(&mut t, 300, "two loads are in", |t| deliveries(t).len() == 2);
    let at: Vec<u32> = deliveries(&t).iter().map(|d| d.2).collect();
    assert_eq!(at, [2, 2]);
}

/// "Chosen at each return": a Storage that appears nearer to the patch takes the next load, with no new order, and
/// when it goes the loads go to the nearest left. Another player's Storage is nobody's drop-off but its owner's.
#[test]
fn each_return_takes_the_nearest_storage_there_is_then() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 300, "the first load is in", |t| deliveries(t).len() == 1);
    // Player 2's, right beside the patch, to the north; then player 1's own, a tile from the patch, to the south.
    spawn_storage(&mut t, P2, 10, 5);
    run_to(&mut t, 300, "the second load is in", |t| deliveries(t).len() == 2);
    spawn_storage(&mut t, P1, 10, 10);
    run_to(&mut t, 300, "its load in hand", |t| deliveries(t).len() == 3);
    let own = t.sim().objects().last().expect("objects").id.0;
    run_to(&mut t, 300, "two more loads", |t| deliveries(t).len() == 5);
    let at: Vec<u32> = deliveries(&t).iter().map(|d| d.2).collect();
    assert_eq!(at[..2], [2, 2]);
    assert_eq!(at[3..], [own, own], "the new Storage, a tile from the patch");
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))), "the same Gather");
    remove(&mut t, own);
    run_to(&mut t, 300, "two more loads", |t| deliveries(t).len() == 7);
    let at: Vec<u32> = deliveries(&t)[5..].iter().map(|d| d.2).collect();
    assert_eq!(at, [2, 2]);
}

/// A Storage lost while the Recruit is on its way to it: it turns to the nearest other at once, as every worker does
/// (design change 0170, `Sim::reassign_drop_offs`). With none left it idles holding its load (Q0102 point 2).
#[test]
fn a_storage_lost_on_the_way_is_replaced_and_with_none_left_the_recruit_idles_with_its_load() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "on its way back", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && object(t, 4).worker.unwrap().drop_off == Some(id(2))
    });
    remove(&mut t, 2);
    run_to(&mut t, 300, "the load is in", |t| deliveries(t).len() == 1);
    assert_eq!(deliveries(&t)[0].2, 3, "the other Storage");
    run_to(&mut t, 200, "on its way back", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff)
    });
    remove(&mut t, 3);
    run_to(&mut t, 100, "it idles", |t| order(t, 4).is_none());
    assert_eq!((load(&t, 4), bank(&t)), (LOAD, LOAD), "holding its load");
    t.run(64);
    assert_eq!((order(&t, 4), load(&t, 4)), (None, LOAD));
}

/// Q0102 point 2: "With none, the Recruit idles holding its load." It mines its load first: the return finds no
/// Storage (the rule of `factions/colonists/units/drone.md > Drop-off gone`, design change 0135).
#[test]
fn without_a_storage_a_recruit_mines_one_load_and_idles_holding_it() {
    let mut t = flat(24, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(RECRUIT, Some(P1), centre(12, 8))
        .spawn(STORAGE, Some(P2), tile(10, 5))
        .start();
    gather(&mut t, 1, &[2], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 2) == LOAD);
    run_to(&mut t, 10, "it idles", |t| order(t, 2).is_none());
    t.run(64);
    assert_eq!((order(&t, 2), load(&t, 2), bank(&t)), (None, LOAD, 0));
    assert!(deliveries(&t).is_empty());
}
