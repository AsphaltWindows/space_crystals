//! The Recruits' Repair in the client (slice M5-N8; `factions/cults/units/recruit.md > Repair`, design changes 0322,
//! 0325 and 0329): the Repair pointer over a damaged own building and the right-click that repairs it, the Recruits at
//! work on it until it is full; a damaged Armory, which a right-click enters and D repairs; (M5-S2b) a full damaged
//! Garrison, which a right-click repairs, and one with a free slot, which it enters (design change 0378); and D over a
//! building at full HP, Inactive, where a click is an invalid target.

use space_crystals::command_ui::feedback::INVALID_TARGET_TEXT;

use super::*;

/// Where the cursor points at [`recruit_game`]'s Storage (2 × 2) and Armory (3 × 2), and at bare ground.
fn storage_at() -> Vec2 {
    middle(STORAGE_AT, (2, 2))
}
fn armory_at() -> Vec2 {
    middle(ARMORY_AT, (3, 2))
}
const GROUND: Vec2 = Vec2::new(1800.0, 800.0);

/// `factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed) → Repair"
/// (design change 0325). Over player 1's Storage at half its 250 HP, the four Recruits' pointer is Repair, and a
/// right-click gives them a Repair of it. They walk to it and restore it, paid for as it is restored (`> Repair`:
/// "costs **half its price** [...] paid gradually as HP is restored"): half the Storage's 100 Space Crystals from 0 to
/// full, so 25 for its last 125 HP. They stop once it is full (design change 0329: "repairers **stop once the target is
/// at full HP**"), and the pointer over it is Move again: nothing to repair.
#[test]
fn a_right_click_on_a_damaged_storage_repairs_it() {
    let mut app = recruit_game(100);
    set_hp(&mut app, STORAGE, 125.0);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, storage_at()), Some(PointerDisplay::Repair));
    right_click(&mut app, storage_at(), &[]);
    let repair = OrderKind::Repair(ObjectId(STORAGE));
    assert_eq!(orders(&app), [given(&RECRUITS, repair, false)]);
    let mut restored = vec![hp(&app, STORAGE)];
    while restored.last() != Some(&Some(250.0)) && restored.len() < 600 {
        app.update();
        restored.push(hp(&app, STORAGE));
    }
    assert_eq!(restored.last(), Some(&Some(250.0)), "full");
    assert!(restored.windows(2).all(|w| w[0] <= w[1]), "it only goes up");
    assert_eq!(crystals(&app), 75);
    for recruit in RECRUITS {
        assert_eq!(object_orders(&app, recruit).current, None, "{recruit} stopped");
    }
    assert_eq!(pointer_over(&mut app, storage_at()), Some(PointerDisplay::Move));
}

/// `factions/cults/units/recruit.md > Right-Click Resolution`: a damaged own building that the selection may enter is
/// entered: "Repair, **unless the building can be entered** by the selection: then **Enter** wins [...]. Repair those
/// with **D: Repair**" (design change 0325). Over player 1's Armory at half its 300 HP, the Recruits' pointer is Enter,
/// and a right-click sends them in; D awaits a target, the pointer is Repair over the Armory, and a click on it gives
/// the four a Repair of it, the panel back in DefaultState.
#[test]
fn a_right_click_enters_a_damaged_armory_and_d_repairs_it() {
    let mut app = recruit_game(100);
    set_hp(&mut app, ARMORY, 150.0);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, armory_at()), Some(PointerDisplay::Enter));
    right_click(&mut app, armory_at(), &[]);
    let armory = ObjectId(ARMORY);
    assert_eq!(orders(&app), [given(&RECRUITS, OrderKind::Enter(armory), false)]);
    tap(&mut app, &[], KeyD);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Repair));
    assert_eq!(pointer_over(&mut app, armory_at()), Some(PointerDisplay::Repair));
    click(&mut app, armory_at(), &[]);
    assert_eq!(orders(&app)[1..], [given(&RECRUITS, OrderKind::Repair(armory), false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// The Garrison's tiles in [`garrison_with`]: (22, 12) to (23, 13), x 1408..1536 and y 768..896, unturned, its door
/// (side B) west, in the middle of x = 1408.
const GARRISON_AT: TilePos = TilePos::new(22, 12);

/// Spawns player 1's Garrison on [`GARRISON_AT`] into [`recruit_game`] and `militants` Militants of player 1's in a row
/// west of its door, which walk in; its id, once they are all inside.
fn garrison_with(app: &mut App, militants: usize) -> u32 {
    let spawn_garrison = DebugCommand::Spawn {
        kind: ObjectKind::Garrison,
        owner: Some(P1),
        at: SpawnAt::Tile(GARRISON_AT),
        orientation: Default::default(),
    };
    submit(app, P1, Command::Debug(spawn_garrison));
    updates(app, 2);
    let garrison = app.world().resource::<SimRes>().objects().last().expect("spawned").id;
    let door = Vec2::new(1408.0, 832.0);
    let units: Vec<ObjectId> = (0..militants)
        .map(|i| {
            ObjectId(spawn(
                app,
                ObjectKind::Militant,
                Some(P1),
                door - Vec2::new(40.0 + 32.0 * i as f32, 0.0),
            ))
        })
        .collect();
    let enter = Command::Order {
        units,
        kind: OrderKind::Enter(garrison),
        queue: false,
    };
    submit(app, P1, enter);
    let inside = |app: &App| app.world().resource::<SimRes>().held_in(garrison).len();
    for _ in 0..300 {
        if inside(app) == militants {
            break;
        }
        app.update();
    }
    assert_eq!(inside(app), militants, "all in");
    garrison.0
}

/// Design change 0378's Verification through the pointer (`factions/cults/units/recruit.md > Right-Click
/// Resolution`: "a damaged enterable building that is **full at the moment of the click** resolves to **Repair** (the
/// pointer shows Repair)"): "Fill a Garrison (4 inside) and damage it. Right-click it with Recruits: the Repair pointer
/// shows, and they repair it." Four Militants in player 1's Garrison, at half its 400 HP: the four Recruits' pointer
/// over it is Repair, and a right-click gives them a Repair of it, which they carry out until it is full; then, with
/// nothing to repair, the pointer is Enter. "With a free slot, the same right-click enters": three inside, the pointer
/// is Enter and the right-click sends the Recruits in.
#[test]
fn a_right_click_repairs_a_full_damaged_garrison_and_enters_one_with_a_free_slot() {
    let at = middle(GARRISON_AT, (2, 2));
    let mut app = recruit_game(100);
    let garrison = garrison_with(&mut app, 4);
    set_hp(&mut app, garrison, 200.0);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Repair));
    right_click(&mut app, at, &[]);
    let repair = given(&RECRUITS, OrderKind::Repair(ObjectId(garrison)), false);
    assert_eq!(orders(&app).last(), Some(&repair));
    for _ in 0..600 {
        if hp(&app, garrison) == Some(400.0) {
            break;
        }
        app.update();
    }
    assert_eq!(hp(&app, garrison), Some(400.0), "repaired");
    assert_eq!(
        pointer_over(&mut app, at),
        Some(PointerDisplay::Enter),
        "nothing to repair"
    );

    let mut app = recruit_game(100);
    let garrison = garrison_with(&mut app, 3);
    set_hp(&mut app, garrison, 200.0);
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Enter));
    right_click(&mut app, at, &[]);
    let enter = given(&RECRUITS, OrderKind::Enter(ObjectId(garrison)), false);
    assert_eq!(orders(&app).last(), Some(&enter));
}

/// `systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`: while D awaits its target, the pointer
/// is Inactive over an own building at full HP, which has nothing to repair, and over the ground. A click on such a
/// building is an invalid target (design change 0065): no order, the message, and D still awaits a target.
#[test]
fn d_is_inactive_over_a_building_at_full_hp() {
    let mut app = recruit_game(100);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyD);
    let awaiting = InterfaceState::awaiting(PanelCommand::Repair);
    assert_eq!(interface(&app), awaiting);
    for at in [storage_at(), armory_at(), GROUND] {
        assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Inactive), "{at}");
    }
    click(&mut app, storage_at(), &[]);
    assert_eq!(said(&app), Some(INVALID_TARGET_TEXT));
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), awaiting);
}
