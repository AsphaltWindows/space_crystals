//! The Recruit's rules on the one Gather job (`factions/cults/units/recruit.md > Gathering` and
//! `factions/cults/units/recruit.md > Right-Click Resolution:`; `factions/cults/structures/storage.md`; design change
//! 0106).

use super::*;

/// `factions/cults/units/recruit.md > Gathering`, Patch sharing: "any number of Recruits may mine the same Space Crystals patch at the same time
/// (unlike Drones)"; `factions/cults/structures/storage.md`: "Multiple Recruits can drop off Space Crystals
/// simultaneously." Two Recruits side by side in the gap between a Storage and two patches: on one patch they mine
/// it together, and on a patch each they unload at the Storage in the same ticks. The patch has no occupancy, and
/// the Storage no line. (On a patch each they start on a tile's centre, and their stops fall exactly on the two
/// reaches: the design's round trip of 22 frames, 1.45 Space Crystals per second each.)
#[test]
fn recruits_mine_one_patch_together_and_unload_together() {
    // Storage 1; patches 2 and 3 a tile south of its face; Recruits 4 and 5 between, each in front of a patch.
    let lanes = || {
        flat(24, 20)
            .spawn(STORAGE, Some(P1), tile(10, 6))
            .spawn_with(patch_of(1_000_000), tile(10, 9))
            .spawn_with(patch_of(1_000_000), tile(11, 9))
            .spawn(RECRUIT, Some(P1), centre(10, 8))
            .spawn(RECRUIT, Some(P1), centre(11, 8))
            .start()
    };
    for patches in [[2, 2], [2, 3]] {
        let mut t = lanes();
        gather(&mut t, 1, &[4], patches[0]);
        gather(&mut t, 1, &[5], patches[1]);
        let (mut mined, mut unloaded) = (0, 0);
        for _ in 0..800 {
            t.step();
            mined += u32::from(is_mining(&t, 4) && is_mining(&t, 5));
            unloaded += u32::from(is_unloading(&t, 4) && is_unloading(&t, 5));
            for n in [4, 5] {
                let waits = matches!(
                    phase(&t, n),
                    Some(GatherPhase::Waiting { .. } | GatherPhase::AtHatch { .. })
                );
                assert!(!waits, "tick {}: Recruit {n} waits", t.sim().tick());
            }
        }
        if patches[0] == patches[1] {
            assert!(mined > 0, "both mine patch 2 in one tick");
        } else {
            assert!(
                unloaded > 30,
                "both unload at the Storage in the same ticks: {unloaded}"
            );
        }
        let each = income(&t, SETTLED, 800) / 2.0;
        assert!(each > 0.95 * 32.0 / 24.0, "{patches:?}: {each} per second each");
        if patches[0] != patches[1] {
            assert!((each - 32.0 / 22.0).abs() < 0.01, "the formula's 22 frames: {each}");
        }
    }
}

/// `factions/colonists/units/drone.md > Gathering Drones pass through each other`: "**Drones only.** Other factions'
/// workers (e.g. Cults Recruits) keep normal collision while mining" (design change 0106: "Recruits mining a patch
/// still bump into each other"). Five Recruits on one face never stand inside each other.
#[test]
fn gathering_recruits_keep_their_collision() {
    let (mut t, recruits) = face_at_work(5, 1, 2, 0);
    for _ in 0..1600 {
        t.step();
        for (i, &a) in recruits.iter().enumerate() {
            for &b in &recruits[i + 1..] {
                let apart = (t.pos(a) - t.pos(b)).abs().max_element();
                assert!(
                    apart > 24.0 - 0.5,
                    "tick {}: Recruits {a} and {b} overlap, {apart} su apart\n{}",
                    t.sim().tick(),
                    t.ascii(None)
                );
            }
        }
    }
    assert!(deliveries(&t).len() > 100, "and they work");
}

/// `factions/cults/units/recruit.md > Right-Click Resolution:`: "Space Crystal patch → Gather", "Own Storage (carrying Space Crystals) → Drop off
/// resources". A Recruit keeps its load through another order (`systems/mechanics/gathering.md > Other orders keep the
/// load`), and a Drop Off order on a Storage is only a delivery, to that Storage, whichever is nearer.
#[test]
fn a_drop_off_order_delivers_the_load_at_its_storage() {
    // Patch 1; Storage 2 a tile south of it; Storage 3 far to the east; Recruit 4.
    let mut t = flat(24, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(STORAGE, Some(P1), tile(18, 8))
        .spawn(RECRUIT, Some(P1), centre(12, 9))
        .start();
    let resolve = |t: &TestSim, target: u32| t.sim().default_order(id(4), Target::Object(id(target)));
    assert_eq!(resolve(&t, 1), Some(OrderKind::Gather(id(1))));
    assert_ne!(resolve(&t, 3), Some(OrderKind::DropOff(id(3))), "nothing to drop off");
    assert_eq!(
        t.sim().validate(&order_of(&[4], OrderKind::DropOff(id(3)))),
        Err(Reject::NothingToDeliver)
    );
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    let now = t.sim().tick();
    let away = OrderKind::Move(Target::Point(TilePos::new(14, 5).center()));
    t.order_at(now + 1, P1, &[id(4)], away, false);
    run_to(&mut t, 300, "it has moved away", |t| order(t, 4).is_none());
    assert_eq!((load(&t, 4), bank(&t)), (LOAD, 0), "it keeps its load, idle");
    assert_eq!(resolve(&t, 3), Some(OrderKind::DropOff(id(3))));
    assert_eq!(
        t.sim().validate(&order_of(&[4], OrderKind::DropOff(id(1)))),
        Err(Reject::NotDropOff(id(1)))
    );
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::DropOff(id(3)), false);
    let unloading = run_to(&mut t, 300, "it unloads", |t| is_unloading(t, 4));
    assert!(Shape::of(object(&t, 4)).gap(&Shape::of(object(&t, 3))) <= REACH);
    t.run(DROP_OFF);
    assert_eq!(deliveries(&t), [(unloading + DROP_OFF - 1, 4, 3, LOAD)]);
    assert_eq!(
        (order(&t, 4), load(&t, 4), bank(&t)),
        (None, 0, LOAD),
        "only a delivery"
    );
}

/// The Recruit has no Set Drop-off command (its sections name none; design changes 0345 and 0375: the Storage nearest
/// to it at every return): the gate refuses the ability.
#[test]
fn a_recruit_has_no_set_drop_off() {
    let t = face(1, 1, 1, 0);
    let ability = Ability::SetDropOff(id(1));
    let command = Command::Ability {
        units: vec![id(3)],
        ability,
    };
    let set = PlayerCommand { player: P1, command };
    assert_eq!(t.sim().validate(&set), Err(Reject::LacksAbility));
}

/// The Drone's job rules hold for a patch that runs out (`factions/colonists/units/drone.md > Patch Occupancy > A
/// patch that runs out`, `systems/mechanics/gathering.md > When a Gather ends`): the Recruit that takes its last
/// load goes on at the nearest other patch within its search radius (a placeholder: the other workers' 6 grid
/// units), and with none left it delivers and idles.
#[test]
fn a_recruit_goes_on_at_another_patch_when_its_patch_runs_out_and_idles_when_none_is_left() {
    // Patch 1 holds 5, patch 2 two tiles east of it holds 4; Storage 3 a tile south of them; Recruit 4.
    let mut t = flat(24, 20)
        .spawn_with(patch_of(5), tile(10, 8))
        .spawn_with(patch_of(4), tile(12, 8))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(RECRUIT, Some(P1), centre(10, 9))
        .start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 400, "patch 1 runs out", |t| t.sim().object(id(1)).is_none());
    assert_eq!(load(&t, 4), 1, "the last load is what is left");
    t.step();
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))), "on to the other patch");
    run_to(&mut t, 600, "patch 2 runs out", |t| t.sim().object(id(2)).is_none());
    run_to(&mut t, 200, "it idles", |t| order(t, 4).is_none());
    let amounts: Vec<u32> = deliveries(&t).iter().map(|d| d.3).collect();
    assert_eq!(amounts, [2, 2, 1, 2, 2], "its last load is delivered first");
    assert_eq!((load(&t, 4), bank(&t)), (0, 9));
}

/// A recruit's world for the load tests: Patch 1, Storage 2 a tile south of it, Recruit 3 between.
fn worksite() -> TestSim {
    flat(40, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(RECRUIT, Some(P1), centre(12, 9))
        .start()
}

fn away() -> OrderKind {
    OrderKind::Move(Target::Point(TilePos::new(14, 5).center()))
}

/// `systems/mechanics/gathering.md > Job rules` (Interrupted mining): the load is written when mining ends, so a
/// Recruit ordered away in the middle of its 8 frames collects nothing, from itself or from the patch.
#[test]
fn a_recruit_ordered_away_mid_mining_carries_nothing() {
    let mut t = worksite();
    gather(&mut t, 1, &[3], 1);
    run_to(&mut t, 200, "it mines", |t| is_mining(t, 3));
    t.run(MINING / 2);
    assert!(is_mining(&t, 3), "half way");
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(3)], away(), false);
    run_to(&mut t, 300, "it has moved away", |t| order(t, 3).is_none());
    assert_eq!((load(&t, 3), remaining(&t, 1), bank(&t)), (0, 1_000_000, 0));
}

/// `systems/mechanics/gathering.md > Job rules` (Other orders keep the load): a loaded Recruit given a Move keeps its
/// 2 Space Crystals, and a Gather after it delivers them first, then mines.
#[test]
fn a_loaded_recruit_keeps_its_load_through_a_move_and_delivers_it_on_its_next_trip() {
    let mut t = worksite();
    gather(&mut t, 1, &[3], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(3)], away(), false);
    run_to(&mut t, 300, "it has moved away", |t| order(t, 3).is_none());
    t.run(40);
    assert_eq!((load(&t, 3), bank(&t)), (LOAD, 0), "kept through the Move");
    let now = t.sim().tick();
    gather(&mut t, now + 1, &[3], 1);
    while deliveries(&t).is_empty() {
        t.step();
        assert!(!is_mining(&t, 3), "it delivers before it mines");
        assert!(t.sim().tick() < now + 400, "it delivers");
    }
    assert_eq!(deliveries(&t)[0].3, LOAD);
    run_to(&mut t, 200, "it mines again", |t| is_mining(t, 3));
}

/// `systems/mechanics/gathering.md > Job rules` (Gather while carrying a load): a loaded Recruit ordered to another
/// patch turns round without mining, delivers its load, and then goes to that patch.
#[test]
fn a_loaded_recruit_ordered_to_gather_delivers_first() {
    let mut t = flat(40, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn_with(patch_of(1_000_000), tile(14, 8))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(RECRUIT, Some(P1), centre(12, 9))
        .start();
    gather(&mut t, 1, &[4], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 4) == LOAD);
    let now = t.sim().tick();
    gather(&mut t, now + 1, &[4], 2);
    run_to(&mut t, 200, "it unloads", |t| is_unloading(t, 4));
    assert_eq!(remaining(&t, 2), 1_000_000, "no mining before");
    t.run(DROP_OFF);
    assert_eq!(deliveries(&t).iter().map(|d| d.3).next(), Some(LOAD));
    run_to(&mut t, 300, "it mines patch 2", |t| {
        is_mining(t, 4) && order(t, 4) == Some(OrderKind::Gather(id(2)))
    });
}

/// `systems/mechanics/gathering.md > Job rules` (The Drop Off order): after the delivery the Recruit runs its next
/// queued order.
#[test]
fn a_drop_off_runs_the_next_queued_order_after_the_delivery() {
    let mut t = worksite();
    gather(&mut t, 1, &[3], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    let now = t.sim().tick();
    let point = TilePos::new(18, 5).center();
    t.order_at(now + 1, P1, &[id(3)], OrderKind::DropOff(id(2)), false);
    t.order_at(now + 1, P1, &[id(3)], OrderKind::Move(Target::Point(point)), true);
    run_to(&mut t, 300, "it delivers", |t| bank(t) == LOAD);
    run_to(&mut t, 20, "then the queue", |t| {
        order(t, 3) == Some(OrderKind::Move(Target::Point(point)))
    });
    run_to(&mut t, 300, "it has arrived", |t| order(t, 3).is_none());
    assert!(t.pos(3).distance(point) < 40.0);
    assert_eq!((load(&t, 3), bank(&t)), (0, LOAD));
}

/// `sim` with the saved object `n`'s field at `pointer` set.
fn edited(sim: &Sim, n: u32, pointer: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("saved");
    *o.pointer_mut(pointer).expect("a field") = value;
    serde_json::from_value(state).unwrap()
}

/// Recruits 1 (empty) and 2 (loaded with 2) beside Storage 3 (whole), Storage 4 (damaged) and Storage 5, a site that
/// Recruit 1 started.
fn storages_and_recruits() -> Sim {
    let mut t = flat(40, 20)
        .spawn(RECRUIT, Some(P1), centre(4, 4))
        .spawn(RECRUIT, Some(P1), centre(6, 4))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(STORAGE, Some(P1), tile(16, 10))
        .start();
    let build = OrderKind::Build {
        kind: STORAGE,
        origin: TilePos::new(24, 10),
        orientation: Default::default(),
    };
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, crystals);
    t.order_at(2, P1, &[id(1)], build, false);
    run_to(&mut t, 400, "the site stands", |t| t.sim().object(id(5)).is_some());
    assert!(object(&t, 5).construction.is_some(), "unfinished");
    let sim = edited(t.sim(), 4, "/hp", serde_json::json!(100.0));
    edited(&sim, 2, "/worker/load", serde_json::json!(LOAD))
}

/// Design change 0487's Verification: an empty Recruit's right-click on an own Storage repairs a damaged one,
/// assists an unfinished one, and walks to an undamaged complete one; a loaded Recruit's right-click on each of the
/// three delivers its load (`factions/cults/units/recruit.md > Right-Click Resolution:`, "Own Storage (carrying Space
/// Crystals) → Drop off resources"). The client resolves a right-click with this rule, unit by unit.
#[test]
fn a_right_click_on_an_own_storage_resolves_by_the_recruits_load() {
    let sim = storages_and_recruits();
    let resolve = |unit: u32, target: u32| sim.default_order(id(unit), Target::Object(id(target)));
    assert_eq!(resolve(1, 4), Some(OrderKind::Repair(id(4))), "damaged");
    assert_eq!(resolve(1, 5), Some(OrderKind::Assist(id(5))), "unfinished");
    assert_eq!(
        resolve(1, 3),
        Some(OrderKind::Move(Target::Object(id(3)))),
        "whole: it walks there"
    );
    for target in [3, 4] {
        assert_eq!(
            resolve(2, target),
            Some(OrderKind::DropOff(id(target))),
            "loaded, #{target}"
        );
    }
    // Placeholder (Q0199): an unfinished Storage takes no delivery, as the drop-off gate has it for every drop-off
    // (design changes 0116 and 0234), so a loaded Recruit assists it. 0487's Verification 4 says it delivers.
    assert_eq!(resolve(2, 5), Some(OrderKind::Assist(id(5))), "loaded, unfinished");
}

/// A group of one loaded and one empty Recruit right-clicked on a whole Storage, as the client resolves it: each
/// unit's own order. The loaded Recruit delivers its load, the empty one walks to the Storage and delivers nothing.
#[test]
fn a_mixed_group_right_clicked_on_a_storage_has_the_loaded_deliver_and_the_empty_move() {
    // Patch 1, Storage 2, Recruit 3 mines a load, Recruit 4 stays empty.
    let mut t = flat(40, 20)
        .spawn_with(patch_of(1_000_000), tile(10, 8))
        .spawn(STORAGE, Some(P1), tile(10, 10))
        .spawn(RECRUIT, Some(P1), centre(12, 9))
        .spawn(RECRUIT, Some(P1), centre(8, 9))
        .start();
    gather(&mut t, 1, &[3], 1);
    run_to(&mut t, 200, "it has its load", |t| load(t, 3) == LOAD);
    t.order_at(t.sim().tick() + 1, P1, &[id(3)], away(), false);
    run_to(&mut t, 300, "it has moved away", |t| order(t, 3).is_none());
    let sim = t.sim();
    let orders: Vec<(u32, OrderKind)> = [4, 3]
        .into_iter()
        .map(|n| (n, sim.default_order(id(n), Target::Object(id(2))).expect("an order")))
        .collect();
    assert_eq!(
        orders[0].1,
        OrderKind::Move(Target::Object(id(2))),
        "the empty one moves"
    );
    assert_eq!(orders[1].1, OrderKind::DropOff(id(2)), "the loaded one delivers");
    let now = t.sim().tick();
    for (n, kind) in orders {
        t.order_at(now + 1, P1, &[id(n)], kind, false);
    }
    run_to(&mut t, 400, "it delivers", |t| bank(t) == LOAD);
    t.run(60);
    let sent: Vec<u32> = deliveries(&t).iter().map(|d| d.1).collect();
    assert_eq!((sent, bank(&t)), (vec![3], LOAD), "the empty one delivers nothing");
}
