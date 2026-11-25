//! Entering an Armory: M4-C's Enter order and entry path, at the middle of side B (Placeholder (Q0104 point 1)), up to
//! its capacity (`factions/cults/structures/armory.md > InternalRecruitCapacity - 10`).

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
/// up (`holding::ENTRY_STANDOFF_SU`).
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

/// The plan's acceptance: "The 11th Recruit's Enter fails, with feedback." Placeholder (Q0104 point 2): "The Enter ends
/// and it idles beside the Armory (the normal Failure rule)": the one that gets to the entry path with ten inside
/// stays out, idle, by the entrance, and its owner is told (`SimEvent::EnterFailed` with `full`, the client's feedback
/// line). Placeholder (Q0104 point 5): the capacity counts the pool.
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
