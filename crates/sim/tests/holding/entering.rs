//! Entering an Armory: M4-C's Enter order and entry path, the doorway at the middle of side B (design change 0347
//! point 1, `core/entities.md > Entrance and exit (production and research buildings)`: "The entrance is a dedicated
//! doorway at the middle of the entrance side (like the Exit Doorway): units walk to it and go in there"), up to its
//! capacity (`factions/cults/structures/armory.md > InternalRecruitCapacity - 10`), once it is built (0347 point 4,
//! `factions/cults/structures/README.md > Entering pool buildings`: "a building under construction **can't be
//! entered**; a right-click on it Assists").

use sc_sim::{BuildingOrientation, Target};

use super::*;

/// The plan's acceptance (slice M5-F1): "Ten Recruits right-clicking an Armory go in one after another, with no jam
/// (timed)." A block of ten, two rows of five 32 su apart, about three grid units east of the entrance, ordered in
/// together: each goes in as soon as it gets to the entry path, as into a Tunnel
/// (`factions/syndicate/tunnels.md > Tunnel Network > Walking in`: "there is no fixed hold time per unit"), one after
/// another, the first after its walk of some 180 su (45 ticks at 4 su a tick), and the ten within 4.5 s of it
/// (measured: ticks 45 to 107); none gives up.
#[test]
fn ten_recruits_file_in_one_after_another() {
    let mut t = base().start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    while inside(&t, A1).len() < 10 {
        let tick = t.sim().tick();
        assert!(
            tick < 2 + 128,
            "{} are in at tick {tick}\n{}",
            inside(&t, A1).len(),
            t.ascii(Some(ENTRY))
        );
        step_checked(&mut t);
    }
    let went_in = entries(&t);
    let mut order: Vec<ObjectId> = went_in.iter().map(|&(_, unit)| unit).collect();
    order.sort_unstable();
    assert_eq!(order, recruits, "each once");
    let ticks: Vec<Tick> = went_in.iter().map(|&(tick, _)| tick).collect();
    assert!(ticks[0] <= 2 + 50, "the first after its walk: {ticks:?}");
    let gaps: Vec<Tick> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    println!("in in ticks {ticks:?} (gaps {gaps:?})");
    // A single file of Recruits (24 su, 4 su a tick) goes in at best one every 6 ticks: on average no slower than one
    // every 8, and no Recruit held up for more than a second.
    assert!(ticks[9] - ticks[0] <= 9 * 8, "one after another: {gaps:?}");
    assert!(gaps.iter().all(|&gap| gap <= 16), "no jam: {gaps:?}");
    assert_eq!(failed(&t), []);
    // Inside, in the order they went in; the pool is full.
    let in_order: Vec<ObjectId> = went_in.iter().map(|&(_, unit)| unit).collect();
    assert_eq!(inside(&t, A1), in_order);
    assert_eq!(t.sim().room(A1), 0);
    assert!(recruits.iter().all(|&r| t.sim().object(r).is_none()), "off the map");
}

/// Recruits on every side of the Armory walk round it to side B and go in, ten of them, none giving up.
#[test]
fn recruits_from_every_side_go_round_to_the_entrance() {
    let mut t = base().start();
    t.without_auto_targeting();
    let spots = [
        (700.0, 704.0),
        (690.0, 600.0),
        (690.0, 810.0),
        (864.0, 590.0),
        (864.0, 820.0),
        (800.0, 560.0),
        (930.0, 840.0),
        (1040.0, 600.0),
        (1040.0, 800.0),
        (1100.0, 704.0),
    ];
    for (x, y) in spots {
        t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(x, y));
    }
    let recruits: Vec<ObjectId> = (2..12).map(id).collect();
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    while inside(&t, A1).len() < 10 {
        let tick = t.sim().tick();
        assert!(
            tick < 2 + 240,
            "{} are in at tick {tick}\n{}",
            inside(&t, A1).len(),
            t.ascii(Some(ENTRY))
        );
        step_checked(&mut t);
    }
    assert_eq!(failed(&t), []);
}

/// Slice M5-F1's QA run, kept as it went: eleven Recruits Enter an Armory, ten go in and the eleventh's Enter fails
/// for want of room; Eject All lets the ten out of side C, and all eleven are ordered in again, so that ten walk round
/// the Armory in a crowd and along its wall to side B. Ten go in again, and again only the last one's Enter fails, for
/// want of room. With the walk ending against the wall, as into a Tunnel, two didn't: one, pressed along the wall,
/// slid ever more slowly towards a spot it couldn't get round the corner to, the one behind it waited, and both gave
/// up (`Holding::entry_standoff_su`).
#[test]
fn ejected_recruits_go_round_to_side_b_and_back_in() {
    // The QA run's game: a 32 × 24 map, the Armory at tile (7, 5) (x 448..640, y 320..448: side B is x = 640), a
    // Storage, and eleven Recruits east of side B, all spawned in tick 1.
    let mut t = cults(&flat(32, 24)).start();
    for (kind, at) in [(ARMORY, tile(7, 5)), (ObjectKind::Storage, tile(2, 9))] {
        let orientation = Default::default();
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind,
                owner: Some(P1),
                at,
                orientation,
            },
        );
    }
    for x in [780.0, 812.0] {
        for y in [320.0, 352.0, 384.0, 416.0, 448.0] {
            t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(x, y));
        }
    }
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(844.0, 384.0));
    let recruits: Vec<ObjectId> = (3..14).map(id).collect();
    t.at(2, P1, enter(&recruits, A1));
    t.at(130, P1, eject_all(&[A1]));
    t.at(260, P1, enter(&recruits, A1));
    run_checked(&mut t, 129);
    assert_eq!((inside(&t, A1).len(), failed(&t).len()), (10, 1));
    run_checked(&mut t, 259);
    assert_eq!(inside(&t, A1), [], "all ejected");
    run_checked(&mut t, 520);
    assert_eq!(
        inside(&t, A1).len(),
        10,
        "{:?}\n{}",
        failed(&t),
        t.ascii(Some(Vec2::new(640.0, 384.0)))
    );
    let [(_, true), (_, true)] = failed(&t)[..] else {
        panic!("only for want of room: {:?}", failed(&t));
    };
}

/// The plan's acceptance: "The 11th Recruit's Enter fails, with feedback." Design change 0347 point 2
/// (`factions/cults/structures/README.md > Entering pool buildings`): "a unit arriving when the building is full has
/// its Enter **end** and idles beside the building (the normal Failure rule)": the one that gets to the entry path with
/// ten inside stays out, idle, by the entrance, and its owner is told (`SimEvent::EnterFailed` with `full`, the
/// client's feedback line). Design change 0347 point 5: the capacity counts every unit inside (`tests/armory/`: the
/// reserved ones too).
#[test]
fn the_eleventh_recruits_enter_fails_with_feedback() {
    let mut t = base().start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 11, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    while failed(&t).is_empty() {
        let tick = t.sim().tick();
        assert!(tick < 2 + 160, "{} are in at tick {tick}", inside(&t, A1).len());
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, A1).len(), 10);
    let [(eleventh, full)] = failed(&t)[..] else {
        panic!("one failure: {:?}", failed(&t));
    };
    assert!(full, "for want of room");
    assert!(!inside(&t, A1).contains(&eleventh));
    let o = t.sim().object(eleventh).expect("on the map");
    assert!(o.orders.as_ref().unwrap().is_idle());
    let stand = ENTRY + Vec2::new(HALF.x, 0.0);
    assert!(o.pos.distance(stand) < 24.0, "by the entrance: {}", o.pos);
    // Nothing more happens to it: it idles there.
    run_for(&mut t, 32);
    assert_eq!((failed(&t).len(), inside(&t, A1).len()), (1, 10));
    assert!(t.sim().object(eleventh).is_some());
}

/// `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only (to be trained)", an own one.
/// The gate takes an Enter on the player's Armory for the Recruits it names and ignores the others; refuses one whose
/// units are none of them Recruits, one on another player's Armory, and one on a building that holds nothing.
#[test]
fn the_gate_lets_recruits_into_their_own_armory() {
    let mut t = base()
        .spawn(ARMORY, Some(P2), tile(20, 4)) // 2
        .spawn(ObjectKind::Storage, Some(P1), tile(4, 4)) // 3
        .start();
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1100.0, 704.0)) // 4
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(1100.0, 800.0)); // 5
    run_checked(&mut t, 1);
    let (recruit, guard) = (id(4), id(5));
    assert_eq!(validate(&t, enter(&[recruit], A1)), Ok(()));
    assert_eq!(validate(&t, enter(&[recruit, guard], A1)), Ok(()));
    assert_eq!(validate(&t, enter(&[guard], A1)), Err(Reject::NotAccepted(A1)));
    assert_eq!(validate(&t, enter(&[recruit], id(2))), Err(Reject::NotOwned(id(2))));
    assert_eq!(validate(&t, enter(&[recruit], id(3))), Err(Reject::NotATunnel(id(3))));
    // The Guard keeps its own order; the Recruit goes in.
    t.order_at(2, P1, &[guard], OrderKind::Hold, false);
    t.at(3, P1, enter(&[recruit, guard], A1));
    run_checked(&mut t, 80);
    assert_eq!(inside(&t, A1), [recruit]);
    let guard_order = t.sim().object(guard).unwrap().orders.as_ref().unwrap().current.as_ref();
    assert_eq!(guard_order.map(|active| active.order.kind), Some(OrderKind::Hold));
}

/// `factions/cults/units/recruit.md > Right-Click Resolution`: "Own Armory → Enter Armory"; "Enemy unit/building →
/// Attack (melee)". Other units move to an own Armory, which takes none of them.
#[test]
fn a_right_click_on_an_own_armory_enters_it() {
    let mut t = base().spawn(ARMORY, Some(P2), tile(20, 10)).start(); // 2
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1100.0, 704.0)) // 3
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(1100.0, 800.0)); // 4
    run_checked(&mut t, 1);
    let right_click = |unit, target| t.sim().default_order(unit, sc_sim::Target::Object(target));
    assert_eq!(right_click(id(3), A1), Some(OrderKind::Enter(A1)));
    assert_eq!(right_click(id(3), id(2)), Some(OrderKind::Attack(id(2))));
    assert_eq!(
        right_click(id(4), A1),
        Some(OrderKind::Move(sc_sim::Target::Object(A1)))
    );
}

/// An Enter whose Armory is destroyed on the way ends, the unit idle where it is (`factions/syndicate/tunnels.md >
/// Tunnel Network`: "if the Tunnel a unit is heading for is destroyed [...] the Enter order ends and the unit idles",
/// one rule for both), and not for want of room.
#[test]
fn an_enter_ends_when_its_armory_goes() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, RECRUIT, P1, Vec2::new(1300.0, 704.0)); // 2
    t.at(2, P1, enter(&[id(2)], A1));
    remove_at(&mut t, 20, A1);
    run_checked(&mut t, 40);
    let o = t.sim().object(id(2)).expect("on the map");
    assert!(o.orders.as_ref().unwrap().is_idle());
    assert!(o.pos.x > 1100.0, "it stopped on its way: {}", o.pos);
    assert!(
        failed(&t).is_empty(),
        "the order went with its target: {:?}",
        failed(&t)
    );
}

/// [`flat`] with Mountain tiles at (15, 9), (16, 9), (17, 9) and (17, 10): with [`base`]'s Armory west of them, tiles
/// (15, 10) and (16, 10), against the north half of its side B, are a pocket open only to the south.
fn pocket_map() -> String {
    let mut rows: Vec<Vec<u8>> = (0..22).map(|_| vec![b'.'; 30]).collect();
    for (x, y) in [(15, 9), (16, 9), (17, 9), (17, 10)] {
        rows[y][x] = b'M';
    }
    let rows: Vec<String> = rows.into_iter().map(|row| String::from_utf8(row).unwrap()).collect();
    rows.join("\n")
}

/// Design change 0347 point 1 (`core/entities.md > Entrance and exit (production and research buildings)`): "units walk
/// to it and go in there", at the doorway in the middle of side B (x = 960, y = 704), not anywhere along the side. A
/// Recruit pressed against the north end of side B (its box 3 su off it, y 640..664) and walled in there, by Mountain
/// to the north and the east and by a row of its owner's Recruits on Hold to the south (which never yield,
/// `systems/units.md > Ground Collision > Who yields`), never goes in: its Enter ends, if it does, for want of a way to
/// the doorway, not of room. Once the row has gone, it walks to the doorway and goes in there, its box within the 8 su
/// of the entry spot (`network::EntryPath::reached`).
#[test]
fn a_recruit_goes_in_only_by_the_doorway_in_the_middle_of_side_b() {
    let mut t = cults(&pocket_map()).spawn(ARMORY, Some(P1), tile(12, 10)).start();
    t.without_auto_targeting();
    let walker = id(next_id(&t));
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(975.0, 652.0));
    // The row's boxes, touching, from 1 su off side B to 7 su short of the Mountain at x = 1088, y 666..690.
    let row: Vec<ObjectId> = (0..5).map(|i| id(walker.0 + 1 + i)).collect();
    for i in 0..5 {
        t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(973.0 + 24.0 * i as f32, 678.0));
    }
    t.order_at(2, P1, &row, OrderKind::Hold, false);
    t.at(2, P1, enter(&[walker], A1));
    run_checked(&mut t, 2);
    for _ in 0..300 {
        step_checked(&mut t);
        let pos = t.sim().object(walker).expect("never in").pos;
        assert!(
            pos.x - HALF.x >= 960.0 && pos.y + HALF.y <= 666.0,
            "in the pocket: {pos}"
        );
    }
    assert_eq!((entries(&t), inside(&t, A1)), (vec![], vec![]), "never in");
    let failures = failed(&t);
    assert!(
        failures.iter().all(|&failure| failure == (walker, false)),
        "{failures:?}"
    );
    let open = t.sim().tick() + 1;
    for &unit in &row {
        remove_at(&mut t, open, unit);
    }
    t.at(open + 1, P1, enter(&[walker], A1));
    run_checked(&mut t, open + 1);
    while inside(&t, A1).is_empty() {
        assert!(t.sim().tick() < open + 200, "walking in\n{}", t.ascii(Some(ENTRY)));
        step_checked(&mut t);
    }
    // Where it went in from, which a unit inside keeps (`Sim::take_off_map`): the doorway, its box within 8 su of the
    // entry spot on either axis.
    let at = t.sim().held_unit(walker).expect("inside").object.pos;
    assert!((at.y - ENTRY.y).abs() - HALF.y <= 8.0, "in the doorway: {at}");
    assert!(
        at.x - HALF.x >= ENTRY.x && at.x - HALF.x - ENTRY.x <= 8.0,
        "against side B: {at}"
    );
}

/// Design change 0347 point 4 (`factions/cults/structures/README.md > Entering pool buildings`): "a building under
/// construction **can't be entered**; a right-click on it Assists". Two Recruits build an Armory (player 1's Storage
/// is its TechPrerequisite); while it is a site the gate refuses a third Recruit's Enter on it, and its right-click on
/// it is an Assist. Once it is complete, the right-click is an Enter, and the Recruit goes in.
#[test]
fn an_armory_site_cant_be_entered_and_a_right_click_on_it_assists() {
    let mut t = cults(&flat(30, 22))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .start(); // 1
    t.without_auto_targeting();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let builders = block(&mut t, 1, 2, 2, Vec2::new(1100.0, 704.0)); // 2, 3
    let recruit = id(next_id(&t)); // 4
    t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1100.0, 900.0));
    let build = OrderKind::Build {
        kind: ARMORY,
        origin: TilePos::new(12, 10),
        orientation: BuildingOrientation::default(),
    };
    t.order_at(2, P1, &builders, build, false);
    run_checked(&mut t, 2);
    let armory = |t: &TestSim| {
        (t.sim().objects())
            .find(|o| o.kind == ARMORY)
            .map(|o| (o.id, o.is_site()))
    };
    while armory(&t).is_none() {
        assert!(t.sim().tick() < 200, "the builders on their way");
        step_checked(&mut t);
    }
    let (site, true) = armory(&t).unwrap() else {
        panic!("a site first");
    };
    assert_eq!(
        validate(&t, enter(&[recruit], site)),
        Err(Reject::UnderConstruction(site))
    );
    let right_click = |t: &TestSim| t.sim().default_order(recruit, Target::Object(site));
    assert_eq!(right_click(&t), Some(OrderKind::Assist(site)));
    while armory(&t) == Some((site, true)) {
        assert!(t.sim().tick() < 1000, "building");
        step_checked(&mut t);
        assert_eq!(inside(&t, site), []);
    }
    assert_eq!(validate(&t, enter(&[recruit], site)), Ok(()));
    assert_eq!(right_click(&t), Some(OrderKind::Enter(site)));
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[recruit], site));
    while inside(&t, site).is_empty() {
        assert!(t.sim().tick() < now + 200, "walking in");
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, site), [recruit]);
}
