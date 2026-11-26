//! Which Storage a Recruit delivers to (design changes 0345 and 0375, answering Q0102 and Q0136;
//! `factions/cults/units/recruit.md > Gathering`, Which Storage): "at **every return** the Recruit takes its load to
//! the own completed Storage **nearest to the Recruit itself** (edge to edge) **when it finishes mining**, so a new
//! Storage is used at once (user, 2026-11-27; Q0136; replaces "nearest from its patch"). **Tie**: the Storage with the
//! **earlier id** (built first). If the Storage is lost **during** a return, the replacement is again the one nearest
//! **the Recruit** (the shared drop-off rule, 0170). With **no** completed Storage, the Recruit **idles holding its
//! load**."

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

/// Design change 0375's field: 8 patches side by side (ids 1 to 8) on tiles (4, 4) to (11, 4), four 2-tile faces, and
/// a Storage under each face a 1-tile gap south of it, on tiles (x, 6) to (x + 1, 7) for x = 4, 6, 8 and 10 (ids 9 to
/// 12, from the west; or from the east with `from_the_east`). Between the patches and the Storages lies a 1-tile lane,
/// tile row 5.
fn adjacent_faces(from_the_east: bool) -> ScenarioBuilder {
    let mut builder = flat(24, 20);
    for x in 4..12 {
        builder = builder.spawn_with(patch_of(1_000_000), tile(x, 4));
    }
    let mut columns = [4, 6, 8, 10];
    if from_the_east {
        columns.reverse();
    }
    for x in columns {
        builder = builder.spawn(STORAGE, Some(P1), tile(x, 6));
    }
    builder
}

/// A point in [`adjacent_faces`]'s lane, `x` grid units from the map's west edge, where a Recruit is within its
/// MiningReach of the patch above it.
fn in_the_lane(x: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x * 64.0, 334.0))
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

/// "Nearest to the Recruit itself (edge to edge) when it finishes mining": the Recruit mines at the patch's east face,
/// nearer to Storage 3, and delivers there, although Storage 2 is the nearer to the patch (design change 0345's
/// "edge to edge from its patch", which design change 0375 replaces).
#[test]
fn the_storage_nearest_to_the_recruit_edge_to_edge_takes_the_load() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    let me = Shape::of(object(&t, 4));
    let (west, east) = (Shape::of(object(&t, 2)), Shape::of(object(&t, 3)));
    assert!(me.gap(&east) < me.gap(&west), "the Recruit is nearer to Storage 3");
    let patch = Shape::of(object(&t, 1));
    assert_eq!(
        (patch.gap(&west), patch.gap(&east)),
        (128.0, 192.0),
        "the patch is nearer to Storage 2"
    );
    run_to(&mut t, 300, "two loads are in", |t| deliveries(t).len() == 2);
    let at: Vec<u32> = deliveries(&t).iter().map(|d| d.2).collect();
    assert_eq!(at, [3, 3]);
}

/// "Edge to edge", not centre to centre: the Recruit mines at the patch's south face, over Storage 2, two tiles below
/// the patch, and takes its load to Storage 3, south-east of it beside the lane, whose edge is the nearer although its
/// centre is the farther.
#[test]
fn the_nearest_storage_is_by_its_edge_not_by_its_centre() {
    let mut t = flat(24, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(10, 11))
        .spawn(STORAGE, Some(P1), tile(12, 10))
        .spawn(RECRUIT, Some(P1), centre(10, 9))
        .start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    let me = Shape::of(object(&t, 4));
    let (below, beside) = (Shape::of(object(&t, 2)), Shape::of(object(&t, 3)));
    assert!(me.gap(&beside) < me.gap(&below), "Storage 3's edge is the nearer");
    let centre = |s: &Shape| s.centre().distance(me.centre());
    assert!(centre(&below) < centre(&beside), "Storage 2's centre is the nearer");
    run_to(&mut t, 200, "the load is in", |t| deliveries(t).len() == 1);
    assert_eq!(deliveries(&t)[0].2, 3);
}

/// "At **every return**", "so a new Storage is used at once": a Storage that appears nearer to the Recruit takes the
/// next load, with no new order, and when it goes the loads go to the nearest left. Another player's Storage is
/// nobody's drop-off but its owner's.
#[test]
fn each_return_takes_the_nearest_storage_there_is_then() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 300, "the first load is in", |t| deliveries(t).len() == 1);
    // Player 2's, a tile from the patch, to the north; then player 1's own, a tile from the patch, to the south.
    spawn_storage(&mut t, P2, 10, 5);
    run_to(&mut t, 300, "the second load is in", |t| deliveries(t).len() == 2);
    spawn_storage(&mut t, P1, 10, 10);
    run_to(&mut t, 300, "its load in hand", |t| deliveries(t).len() == 3);
    let own = t.sim().objects().last().expect("objects").id.0;
    run_to(&mut t, 300, "two more loads", |t| deliveries(t).len() == 5);
    let at: Vec<u32> = deliveries(&t).iter().map(|d| d.2).collect();
    assert_eq!(at[..2], [3, 3]);
    assert_eq!(at[3..], [own, own], "the new Storage, a tile from the patch");
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))), "the same Gather");
    // From the new Storage it comes back to the patch's south face, which is nearer to Storage 2 than to Storage 3.
    remove(&mut t, own);
    run_to(&mut t, 300, "two more loads", |t| deliveries(t).len() == 7);
    let at: Vec<u32> = deliveries(&t)[5..].iter().map(|d| d.2).collect();
    assert_eq!(at, [2, 2]);
}

/// Design change 0345's Verification: "Complete a new Storage beside a busy patch: the next returns go to it." Three
/// Recruits (3 to 5) gather patch 1 for Storage 2, 3 tiles west of it, while a fourth (6) builds a Storage a tile
/// south of the patch, through the gate: the nearer to a Recruit wherever round the patch it finishes mining (design
/// change 0375). Once it is complete, every load whose return starts after that goes to the new Storage, with no new
/// order; only a return already under way then ends at Storage 2.
#[test]
fn a_storage_completed_beside_a_busy_patch_takes_the_next_returns() {
    let mut t = flat(24, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(5, 8))
        .spawn(RECRUIT, Some(P1), centre(12, 8))
        .spawn(RECRUIT, Some(P1), centre(12, 9))
        .spawn(RECRUIT, Some(P1), centre(13, 8))
        .spawn(RECRUIT, Some(P1), centre(15, 13))
        .start();
    let gatherers = [3, 4, 5];
    let funds = DebugCommand::SetResources {
        player: P1,
        crystals: 100,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, funds);
    gather(&mut t, 1, &gatherers, 1);
    run_to(&mut t, 400, "the patch is busy", |t| deliveries(t).len() >= 6);
    assert!(deliveries(&t).iter().all(|d| d.2 == 2), "Storage 2's");
    let build = OrderKind::Build {
        kind: STORAGE,
        origin: TilePos::new(10, 10),
        orientation: Default::default(),
    };
    let command = order_of(&[6], build);
    t.sim().validate(&command).expect("the gate takes the Build");
    let now = t.sim().tick();
    t.at(now + 1, P1, command.command);
    let site = |t: &TestSim| {
        (t.events().iter()).find_map(|(_, e)| match *e {
            SimEvent::SiteBuilt { site } => Some(site.0),
            _ => None,
        })
    };
    let built = run_to(&mut t, 1200, "the Storage is complete", |t| site(t).is_some());
    let site = site(&t).expect("built");
    let gap = Shape::of(object(&t, 1)).gap(&Shape::of(object(&t, site)));
    assert_eq!(gap, 64.0, "a tile from the patch");
    let under_way: Vec<u32> = (gatherers.iter().copied())
        .filter(|&n| {
            matches!(
                phase(&t, n),
                Some(GatherPhase::ToDropOff | GatherPhase::DroppingOff { .. })
            )
        })
        .collect();
    let after = |t: &TestSim, n: u32| -> Vec<u32> {
        (deliveries(t).iter())
            .filter(|d| d.0 > built && d.1 == n)
            .map(|d| d.2)
            .collect()
    };
    run_to(&mut t, 600, "three more loads each", |t| {
        gatherers.iter().all(|&n| after(t, n).len() >= 3)
    });
    for n in gatherers {
        let at = after(&t, n);
        let first = usize::from(under_way.contains(&n) && at[0] == 2);
        assert!(at[first..].iter().all(|&at| at == site), "Recruit {n}: {at:?}");
        assert_eq!(order(&t, n), Some(OrderKind::Gather(id(1))), "the same Gather");
    }
}

/// A Storage lost while the Recruit is on its way to it: it turns to the nearest other at once, as every worker does
/// (design change 0170, `Sim::reassign_drop_offs`). With none left it idles holding its load (design change 0345).
#[test]
fn a_storage_lost_on_the_way_is_replaced_and_with_none_left_the_recruit_idles_with_its_load() {
    let mut t = two_storages().start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "on its way back", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && object(t, 4).worker.unwrap().drop_off == Some(id(3))
    });
    remove(&mut t, 3);
    run_to(&mut t, 300, "the load is in", |t| deliveries(t).len() == 1);
    assert_eq!(deliveries(&t)[0].2, 2, "the other Storage");
    run_to(&mut t, 200, "on its way back", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff)
    });
    remove(&mut t, 2);
    run_to(&mut t, 100, "it idles", |t| order(t, 4).is_none());
    assert_eq!((load(&t, 4), bank(&t)), (LOAD, LOAD), "holding its load");
    t.run(64);
    assert_eq!((order(&t, 4), load(&t, 4)), (None, LOAD));
}

/// Design change 0375's Verification: "On 8 adjacent patches with one Storage per face, Recruits on the end patch of a
/// face deliver to their own face's Storage, not the neighbouring one." [`adjacent_faces`], with a Recruit in the lane
/// in front of each patch (13 to 20, from the west), each patch the end of its face. Patches 3, 5 and 7 lie as near
/// the Storage of the face west of theirs as their own, edge to edge from the patch (a tile from both), and by design
/// change 0345's rule, which 0375 replaces, their loads went to that one, the earlier. From the Recruit, its own
/// face's Storage is the nearer: the change entry's "A Recruit mining an end patch from its own face's side is nearer
/// that face's Storage, so it stays with its face".
#[test]
fn on_adjacent_patches_a_recruit_on_the_end_patch_of_a_face_delivers_to_its_own_face() {
    let mut builder = adjacent_faces(false);
    for n in 0..8 {
        builder = builder.spawn(RECRUIT, Some(P1), in_the_lane(4.5 + n as f32));
    }
    let mut t = builder.start();
    for n in 0..8 {
        gather(&mut t, 1, &[13 + n], 1 + n);
    }
    t.run(60 * 16);
    for n in 0..8 {
        let at: Vec<u32> = (deliveries(&t).iter()).filter(|d| d.1 == 13 + n).map(|d| d.2).collect();
        let own = 9 + n / 2;
        assert!(
            at.len() >= 30 && at.iter().all(|&at| at == own),
            "Recruit {} on patch {}: {at:?}",
            13 + n,
            1 + n
        );
    }
}

/// "**Tie**: the Storage with the **earlier id** (built first)". On [`adjacent_faces`], a Recruit that mines patch 3,
/// the west end of face B, from the lane with its Silhouette reaching over face A's Storage is as near to it as to
/// face B's, its own: the gap to each is the lane's alone, edge to edge. It takes its load to the earlier of the two:
/// face A's with the Storages numbered from the west, face B's numbered from the east. So a Recruit that comes along
/// the lane from the face to the west and stops as it comes within MiningReach, short of the middle of its patch's
/// face, may not stay with its own face: the Verification above holds for a Recruit that mines in front of its own
/// face's Storage.
#[test]
fn a_recruit_as_near_two_storages_takes_its_load_to_the_earlier() {
    // Faces A's and B's Storages, and the earlier of the two.
    for (from_the_east, [a, b], earlier) in [(false, [9, 10], 9), (true, [12, 11], 11)] {
        let mut t = adjacent_faces(from_the_east)
            .spawn(RECRUIT, Some(P1), in_the_lane(6.1))
            .start();
        gather(&mut t, 1, &[13], 3);
        run_to(&mut t, 50, "it mines patch 3", |t| is_mining(t, 13));
        let me = Shape::of(object(&t, 13));
        let gap = |n: u32| me.gap(&Shape::of(object(&t, n)));
        assert_eq!(gap(a), gap(b), "as near to both");
        run_to(&mut t, 100, "the load is in", |t| deliveries(t).len() == 1);
        assert_eq!(deliveries(&t)[0].2, earlier, "numbered from the east: {from_the_east}");
    }
}

/// Design change 0345: "With none, it idles holding its load." It mines its load first: the return finds no
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
