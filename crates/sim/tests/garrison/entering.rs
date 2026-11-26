//! Going in (`factions/cults/structures/garrison.md > Garrisoning > Who can enter`: "Recruits, Militants, Gunners, and
//! Disciples. Units enter with the Enter command (right-click own Garrison with eligible infantry selected), up to 4
//! units total"), through its one door, on side B ("SymmetryType AAAB, **the door on side B** (one door, entrance and
//! exit"), as at every entrance (`core/entities.md > Entrance and exit (production and research buildings)`: "units walk
//! to it and go in there").

use sc_sim::{BuildingOrientation, Target};

use super::*;

/// Where unit `unit`, inside, went in from: the place a unit inside keeps (`Sim::take_off_map`).
fn went_in_at(t: &TestSim, unit: ObjectId) -> Vec2 {
    t.sim().held_unit(unit).expect("inside").object.pos
}

/// The four kinds go in, from the far side of the Garrison: each walks round to the door, in the middle of side B (x =
/// 768, y = 704), and goes in there, its box against the side within the entry path's 8 su of it.
#[test]
fn recruits_militants_gunners_and_disciples_go_in_through_the_door_on_side_b() {
    let mut t = base().start();
    t.without_auto_targeting();
    let kinds = [RECRUIT, MILITANT, GUNNER, DISCIPLE];
    let units = column(&mut t, 1, &kinds, Vec2::new(1100.0, 704.0));
    t.at(2, P1, enter(&units, G1));
    run_checked(&mut t, 2);
    while inside(&t, G1).len() < 4 {
        assert!(
            t.sim().tick() < 300,
            "{:?} are in\n{}",
            inside(&t, G1),
            t.ascii(Some(DOOR))
        );
        step_checked(&mut t);
    }
    let mut went_in = inside(&t, G1);
    went_in.sort();
    assert_eq!(went_in, units, "all four kinds");
    for &unit in &units {
        let at = went_in_at(&t, unit);
        assert!(
            at.x + HALF.x <= DOOR.x && DOOR.x - (at.x + HALF.x) <= 8.0 && (at.y - DOOR.y).abs() - HALF.y <= 8.0,
            "{unit:?} went in at {at}, not at the door"
        );
    }
    assert_eq!(failed(&t), []);
}

/// "up to 4 units total" (`factions/cults/structures/garrison.md > GarrisonCapacity - 4 units`): of five Militants
/// ordered in, four go in; the fifth gets to the door with the Garrison full, and its Enter ends there, the unit idle
/// beside the building (design change 0347 point 2, `factions/cults/structures/README.md > Entering pool buildings`: "a
/// unit arriving when the building is full has its Enter **end** and idles beside the building (the normal Failure
/// rule)", F3's rule for every building that holds units).
#[test]
fn a_fifth_unit_finds_the_garrison_full_and_idles_beside_it() {
    let mut t = base().start();
    t.without_auto_targeting();
    let units = column(&mut t, 1, &[MILITANT; 5], DOOR - Vec2::new(60.0, 0.0));
    t.at(2, P1, enter(&units, G1));
    run_checked(&mut t, 2);
    while failed(&t).is_empty() {
        assert!(
            t.sim().tick() < 200,
            "{:?} are in\n{}",
            inside(&t, G1),
            t.ascii(Some(DOOR))
        );
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, G1), units[..4], "the first four, in the order they came");
    assert_eq!(failed(&t), [(units[4], true)], "for want of room");
    let fifth = t.sim().object(units[4]).expect("on the map");
    assert!(fifth.orders.as_ref().unwrap().is_idle());
    assert!(
        fifth.pos.x < DOOR.x && fifth.pos.distance(DOOR) < 80.0,
        "beside the door: {}",
        fifth.pos
    );
    let at = fifth.pos;
    run_for(&mut t, 30);
    assert_eq!(t.sim().object(units[4]).unwrap().pos, at, "it stays idle");
}

/// `factions/cults/structures/garrison.md > Garrisoning`: "Units inside are not consumed and still count toward Unit
/// Control": four Militants (orphans, no Hall made them: their whole cost each, design change 0344) count the same
/// inside as on the map.
#[test]
fn units_inside_still_count_toward_unit_control() {
    let mut t = base().start();
    t.without_auto_targeting();
    let units = column(&mut t, 1, &[MILITANT; 4], DOOR - Vec2::new(60.0, 0.0));
    run_checked(&mut t, 2);
    let used = |t: &TestSim| t.sim().army_cap(P1).expect("in the game").used;
    assert_eq!(used(&t), 4);
    t.at(3, P1, enter(&units, G1));
    while inside(&t, G1).len() < 4 {
        assert!(t.sim().tick() < 200);
        step_checked(&mut t);
        assert_eq!(used(&t), 4, "tick {}", t.sim().tick());
    }
    run_for(&mut t, 10);
    assert_eq!(used(&t), 4);
}

/// `systems/controls/interface.md > RightClickResolution` (`factions/cults/units/recruit.md > Right-Click Resolution`:
/// "Own Garrison → Enter Garrison"; `factions/cults/structures/garrison.md > Garrisoning`: "right-click own Garrison
/// with eligible infantry selected"): the right-click of each of the four kinds on its own Garrison is an Enter, by the
/// sim's one rule (`RULES`'s Enter: an own building in operation that takes the kind in). Another kind (a GDO
/// Peacekeeper, in this debug game) moves to it. On an enemy's Garrison the Recruit, the Militant and the Gunner attack
/// it, and the Disciple, which has no attack, moves to it.
#[test]
fn a_right_click_on_an_own_garrison_enters_it() {
    let mut t = base().spawn(GARRISON, Some(P2), tile(20, 10)).start(); // 2
    t.without_auto_targeting();
    let kinds = [RECRUIT, MILITANT, GUNNER, DISCIPLE, ObjectKind::Peacekeeper];
    // Beside the enemy's Garrison (x 1280..1408, y 640..768), which they see.
    let units = column(&mut t, 1, &kinds, Vec2::new(1340.0, 860.0));
    run_checked(&mut t, 1);
    let right_click = |unit, target| t.sim().default_order(unit, Target::Object(target));
    for &unit in &units[..4] {
        assert_eq!(right_click(unit, G1), Some(OrderKind::Enter(G1)), "{unit:?}");
    }
    let enemy = id(2);
    for &unit in &units[..3] {
        assert_eq!(right_click(unit, enemy), Some(OrderKind::Attack(enemy)), "{unit:?}");
    }
    assert_eq!(
        right_click(units[3], enemy),
        Some(OrderKind::Move(Target::Object(enemy)))
    );
    assert_eq!(right_click(units[4], G1), Some(OrderKind::Move(Target::Object(G1))));
}

/// Design change 0347 point 4 (`factions/cults/structures/README.md > Entering pool buildings`: "a building under
/// construction **can't be entered**; a right-click on it Assists"), F3's rule for every building that holds units:
/// two Recruits build a Garrison (its TechPrerequisite, an Armory, is player 1's; `factions/cults/units/recruit.md >
/// Constructs`: "Garrison: 150 Space Crystals, 400 frames"); while it is a site the gate refuses a Militant's Enter on
/// it, and a third Recruit's right-click on it is an Assist. Once it is complete, the Militant's Enter is taken, and it
/// goes in.
#[test]
fn a_garrison_site_cant_be_entered_and_a_right_click_on_it_assists() {
    let mut t = cults(&flat(30, 22))
        .spawn(ObjectKind::Armory, Some(P1), tile(2, 2))
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
    let builders = column(&mut t, 1, &[RECRUIT, RECRUIT], Vec2::new(1100.0, 704.0)); // 2, 3
    let [recruit, militant] = column(&mut t, 1, &[RECRUIT, MILITANT], Vec2::new(1100.0, 900.0))[..] else {
        unreachable!()
    }; // 4, 5
    let build = OrderKind::Build {
        kind: GARRISON,
        origin: TilePos::new(12, 10),
        orientation: BuildingOrientation::default(),
    };
    t.order_at(2, P1, &builders, build, false);
    run_checked(&mut t, 2);
    let garrison = |t: &TestSim| {
        (t.sim().objects())
            .find(|o| o.kind == GARRISON)
            .map(|o| (o.id, o.is_site()))
    };
    while garrison(&t).is_none() {
        assert!(t.sim().tick() < 200, "the builders on their way");
        step_checked(&mut t);
    }
    let (site, true) = garrison(&t).unwrap() else {
        panic!("a site first");
    };
    assert_eq!(
        validate(&t, enter(&[militant], site)),
        Err(Reject::UnderConstruction(site))
    );
    let right_click = |t: &TestSim| t.sim().default_order(recruit, Target::Object(site));
    assert_eq!(right_click(&t), Some(OrderKind::Assist(site)));
    while garrison(&t) == Some((site, true)) {
        assert!(t.sim().tick() < 1000, "building");
        step_checked(&mut t);
        assert_eq!(inside(&t, site), []);
    }
    assert_eq!(validate(&t, enter(&[militant], site)), Ok(()));
    assert_eq!(right_click(&t), Some(OrderKind::Enter(site)));
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[militant], site));
    while inside(&t, site).is_empty() {
        assert!(t.sim().tick() < now + 300, "walking in\n{}", t.ascii(Some(DOOR)));
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, site), [militant]);
}

/// Design change 0378, answering Q0140 (`factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own
/// building or vehicle (completed) → Repair, **unless the building can be entered** by the selection: then **Enter**
/// wins [...]. **Except when it is full** [...]: a damaged enterable building that is **full at the moment of the
/// click** resolves to **Repair**"), its Verification: "Fill a Garrison (4 inside) and damage it. Right-click it with
/// Recruits: [...] they repair it", and "With a free slot, the same right-click enters". Player 2's Militant damages
/// player 1's Garrison, three Militants inside: with its free slot, the right-click of each kind it takes in is an
/// Enter, the Recruits' too (design change 0325). A fourth Militant fills it: the Recruits' right-click is now a
/// Repair, and they restore it to its MaxHP; the Gunner and the Disciple, whose types repair nothing, still get the
/// Enter (it would end at the door: the normal Full-building rule). Repaired, the full Garrison is the Recruits' to
/// enter again, with nothing to repair.
#[test]
fn a_full_damaged_garrison_is_repaired_on_a_right_click_and_one_with_room_entered() {
    let mut t = base().start();
    t.without_auto_targeting();
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    fill(&mut t, &[MILITANT; 3]);
    let raider = id(next_id(&t));
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P2, Vec2::new(HI.x + GU, DOOR.y));
    t.order_at(now + 1, P2, &[raider], OrderKind::Attack(G1), false);
    let max = t.sim().object(G1).and_then(|o| o.max_hp()).expect("it has HP");
    let hp = |t: &TestSim| t.sim().object(G1).and_then(|o| o.hp);
    while hp(&t) == Some(max) {
        assert!(t.sim().tick() < now + 100, "the raider hits it");
        step_checked(&mut t);
    }
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, DebugCommand::Remove { id: raider });
    let kinds = [RECRUIT, RECRUIT, GUNNER, DISCIPLE, MILITANT];
    let units = column(&mut t, gone, &kinds, Vec2::new(600.0, 900.0));
    run_checked(&mut t, gone + 1);
    assert!(hp(&t) < Some(max), "damaged");
    let right_click = |t: &TestSim, unit| t.sim().default_order(unit, Target::Object(G1));
    for &unit in &units {
        assert_eq!(
            right_click(&t, unit),
            Some(OrderKind::Enter(G1)),
            "{unit:?}: a free slot"
        );
    }
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&units[4..], G1));
    while inside(&t, G1).len() < 4 {
        assert!(t.sim().tick() < now + 300, "the fourth walks in");
        step_checked(&mut t);
    }
    let (recruits, others) = units[..4].split_at(2);
    for &unit in recruits {
        assert_eq!(right_click(&t, unit), Some(OrderKind::Repair(G1)), "{unit:?}: full");
    }
    for &unit in others {
        assert_eq!(right_click(&t, unit), Some(OrderKind::Enter(G1)), "{unit:?}");
    }
    let clicked = right_click(&t, recruits[0]).expect("an order");
    let now = t.sim().tick() + 1;
    t.order_at(now, P1, recruits, clicked, false);
    while hp(&t) < Some(max) {
        assert!(t.sim().tick() < now + 600, "the Recruits repair it");
        step_checked(&mut t);
    }
    assert_eq!(inside(&t, G1).len(), 4);
    for &unit in recruits {
        assert_eq!(right_click(&t, unit), Some(OrderKind::Enter(G1)), "{unit:?}: repaired");
    }
    assert_eq!(failed(&t), []);
}
