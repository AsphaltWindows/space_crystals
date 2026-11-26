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
/// resources". A Recruit keeps its load through another order (`factions/colonists/units/drone.md > Other orders keep
/// the load`), and a Drop Off order on a Storage is only a delivery, to that Storage, whichever is nearer.
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
/// patch that runs out`, `factions/colonists/units/drone.md > When a Gather ends`): the Recruit that takes its last
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
