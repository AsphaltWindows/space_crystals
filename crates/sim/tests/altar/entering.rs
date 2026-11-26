//! Going in (`factions/cults/augmentation.md > Altar > Entry`: "Any Cults infantry (Recruit, Militant, Gunner, Disciple)
//! can enter the Altar to join its StoredUnits pool, up to InternalUnitCapacity. Right-clicking an own Altar with Cults
//! infantry selected issues Enter Altar."; `systems/controls/commands.md > Enter`: "**Cults — Altar**: any Cults
//! infantry (Recruit, Militant, Gunner, Disciple)"), at the entrance, side B ("EntranceSide / ExitSide - B / C"), as at
//! every entrance (`core/entities.md > Entrance and exit (production and research buildings)`: "units walk to it and go
//! in there").

use super::*;

/// The middle of [`base`]'s Altar's entrance side, B.
const ENTRANCE: Vec2 = Vec2::new(960.0, 704.0);

/// The four types go in, from the far side of the Altar: each walks round to the entrance, in the middle of side B,
/// and goes in there, its box against the side within the entry path's 8 su of it. The pool counts one Plain unit of
/// each type.
#[test]
fn the_four_infantry_types_go_in_at_the_entrance_on_side_b() {
    let mut t = base().start();
    t.without_auto_targeting();
    let kinds = [RECRUIT, MILITANT, GUNNER, DISCIPLE];
    let units = block(&mut t, 1, &kinds, 5, Vec2::new(500.0, 704.0));
    run_checked(&mut t, 1);
    let mut went_in = go_in(&mut t, &units);
    went_in.sort();
    assert_eq!(went_in, units, "all four types");
    for &unit in &units {
        let held = t.sim().held_unit(unit).expect("inside");
        let (at, half) = (held.object.pos, held.object.def().half_extents().unwrap());
        assert!(
            at.x - half.x >= ENTRANCE.x
                && at.x - half.x - ENTRANCE.x <= 8.0
                && (at.y - ENTRANCE.y).abs() - half.y <= 8.0,
            "{unit:?} went in at {at}, not at the entrance"
        );
    }
    for kind in kinds {
        assert_eq!(t.sim().pool_counts_of(A1, kind), [1, 0, 0, 0], "{kind:?}");
    }
}

/// `factions/cults/augmentation.md > Altar > Entry`: "Right-clicking an own Altar with Cults infantry selected issues
/// Enter Altar" (`systems/controls/interface.md > RightClickResolution`; `factions/cults/units/recruit.md`: "Own Altar
/// → Enter Altar"): the right-click of each of the four types on its own Altar is an Enter, by the sim's one rule
/// (`RULES`'s Enter: an own building in operation that takes the type in). Another type (a GDO Peacekeeper, in this
/// debug game) moves to it. On an enemy's Altar the Recruit, the Militant and the Gunner attack it, and the Disciple,
/// which has no attack, moves to it.
#[test]
fn a_right_click_on_an_own_altar_enters_it() {
    let scenario = base().spawn(ALTAR, Some(P2), tile(20, 10)); // 2
    let kinds = [RECRUIT, MILITANT, GUNNER, DISCIPLE, ObjectKind::Peacekeeper];
    // Beside the enemy's Altar (x 1280..1472, y 640..768), which they see.
    let mut t = scenario.start();
    t.without_auto_targeting();
    let units = block(&mut t, 1, &kinds, 5, Vec2::new(1200.0, 860.0));
    run_checked(&mut t, 1);
    let right_click = |unit, target| t.sim().default_order(unit, Target::Object(target));
    for &unit in &units[..4] {
        assert_eq!(right_click(unit, A1), Some(OrderKind::Enter(A1)), "{unit:?}");
    }
    let enemy = id(2);
    for &unit in &units[..3] {
        assert_eq!(right_click(unit, enemy), Some(OrderKind::Attack(enemy)), "{unit:?}");
    }
    assert_eq!(
        right_click(units[3], enemy),
        Some(OrderKind::Move(Target::Object(enemy)))
    );
    assert_eq!(right_click(units[4], A1), Some(OrderKind::Move(Target::Object(A1))));
}

/// Design change 0325 (`factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own building or vehicle
/// (completed) → Repair, **unless the building can be entered** by the selection: then **Enter** wins"): player 2's
/// Militant damages player 1's Altar. The Recruits repair it (the gate takes a Recruit's Repair on it), yet the
/// right-click of each of the four types on it is an Enter, the Recruit's too.
#[test]
fn a_damaged_altar_is_entered_on_a_right_click_not_repaired() {
    let mut t = base().start();
    t.without_auto_targeting();
    run_checked(&mut t, 1);
    let full = t.sim().object(A1).and_then(|o| o.hp);
    let now = t.sim().tick() + 1;
    t.spawn_unit_at(now, MILITANT, P2, Vec2::new(ENTRANCE.x + 64.0, ENTRANCE.y));
    run_checked(&mut t, now);
    let raider = (t.sim().objects())
        .find(|o| o.owner == Some(P2))
        .expect("the raider")
        .id;
    t.order_at(now + 1, P2, &[raider], OrderKind::Attack(A1), false);
    while t.sim().object(A1).and_then(|o| o.hp) == full {
        assert!(t.sim().tick() < now + 100, "the raider hits it");
        step_checked(&mut t);
    }
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, raider);
    let units = block(
        &mut t,
        gone,
        &[RECRUIT, MILITANT, GUNNER, DISCIPLE],
        5,
        Vec2::new(500.0, 900.0),
    );
    run_checked(&mut t, gone + 1);
    assert!(t.sim().object(A1).and_then(|o| o.hp) < full, "damaged");
    let repair = Command::Order {
        units: vec![units[0]],
        kind: OrderKind::Repair(A1),
        queue: false,
    };
    assert_eq!(validate(&t, repair), Ok(()), "the Recruits repair it");
    for &unit in &units {
        let order = t.sim().default_order(unit, Target::Object(A1));
        assert_eq!(order, Some(OrderKind::Enter(A1)), "{unit:?}");
    }
}
