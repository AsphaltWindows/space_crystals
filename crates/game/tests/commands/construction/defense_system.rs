//! (M6-GR) The Defense System's panel (`factions/gdo/structures/defense_system.md > Placeholder stats`, design change
//! 0392: "**Works like the Extraction Facility** [...] one slot, Q, no build menu"): Q builds a Missile Turret, greyed
//! with its progress bar while it is constructed, X cancels it with a full refund, and once it is ready Q enters its
//! placement in the GDO build area. Defense Systems are Groupable ("Groupable true"): two selected are one
//! SelectionGroup, whose Q builds at both and places the first one's turret.

use super::place_button::{bar, button};
use super::*;

const TURRET: ObjectKind = ObjectKind::MissileTurret;
/// The objects spawned for these games, after [`construction_scenario`]'s: two Generators of player 1's, so that two
/// Defense Systems and two turrets leave them at full Power, then the Defense Systems.
const DS: u32 = 11;
const DS2: u32 = 12;
const DS_AT: TilePos = TilePos::new(30, 17);
const DS2_AT: TilePos = TilePos::new(27, 17);
/// Where a turret (1 × 1) may go, in the build area and in sight, and where it may not: outside the build area.
const INSIDE: TilePos = TilePos::new(33, 19);
const INSIDE2: TilePos = TilePos::new(34, 18);
const OUTSIDE: TilePos = TilePos::new(37, 14);

/// [`construction_game`] with 1000 Space Crystals, its two Generators and its Defense Systems (`count` of them),
/// spawned by debug commands.
fn defense_game(count: usize) -> App {
    let mut app = construction_game(1000);
    let spawns = [
        (ObjectKind::Generator, TilePos::new(9, 2)),
        (ObjectKind::Generator, TilePos::new(12, 2)),
        (ObjectKind::DefenseSystem, DS_AT),
        (ObjectKind::DefenseSystem, DS2_AT),
    ];
    for (kind, at) in &spawns[..2 + count] {
        let spawn = DebugCommand::Spawn {
            kind: *kind,
            owner: Some(P1),
            at: SpawnAt::Tile(*at),
            orientation: BuildingOrientation::default(),
        };
        submit(&mut app, P1, Command::Debug(spawn));
    }
    updates(&mut app, 2);
    for (n, (kind, at)) in spawns[..2 + count].iter().enumerate() {
        assert_eq!(
            structure_at(&app, *kind, *at).map(|(id, _)| id),
            Some(9 + n as u32),
            "{kind:?}"
        );
    }
    app
}

/// 0392's Verification, "the same for a Missile Turret at the Defense System", and its flow: idle, "**Q: Build
/// Missile Turret** (100 Space Crystals, starts construction)"; constructing, "**Q: Place Missile Turret**, greyed
/// out, with a **running progress bar** on the button; **X: Cancel Construction** (full refund)"; ready, "**Q: Place
/// Missile Turret** enters AwaitingPlacement (ghost within the GDO build area [...])". Pressed early, Q says why it is
/// greyed; a click outside the build area places nothing; one inside places the turret there, and the Defense System
/// is idle again.
#[test]
fn the_defense_systems_q_builds_then_places_a_missile_turret() {
    use ButtonLook::{Common, Disabled, Pending};
    use PanelCommand::{CancelConstruction, Construct, PlaceReady};
    let mut app = defense_game(1);
    click(&mut app, middle(DS_AT, (2, 2)), &[]);
    assert_eq!(selected(&app), [DS]);
    let c = PanelButton::Command;
    assert_eq!(buttons(&mut app), [(q(), c(Construct(TURRET)), Common, "MTT")]);
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(TURRET);
    assert_eq!(constructions(&app), [(P1, vec![DS], construct)]);
    updates(&mut app, 1);
    assert_eq!((bank(&app), constructor(&app, DS).current()), (900, Some(TURRET)));
    assert_eq!(button(&mut app, q()), Some((c(PlaceReady(TURRET)), Disabled, "Place")));
    assert_eq!(button(&mut app, X), Some((c(CancelConstruction), Common, "Cancel")));
    let (_, first) = bar(&mut app, q()).expect("a progress bar");
    updates(&mut app, 60);
    let (_, later) = bar(&mut app, q()).expect("a progress bar");
    assert!(later > first + 0.2, "{first} then {later}: 60 of 240 frames");
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some("Still under construction"));
    tap(&mut app, &[], KeyX);
    updates(&mut app, 1);
    assert_eq!(
        (bank(&app), constructor(&app, DS)),
        (1000, Constructor::Idle),
        "a full refund"
    );

    tap(&mut app, &[], KeyQ);
    run_until_ready(&mut app, DS);
    assert_eq!(button(&mut app, q()), Some((c(PlaceReady(TURRET)), Common, "Place")));
    assert_eq!(bar(&mut app, q()), None);
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DS), TURRET)));
    assert_eq!(button(&mut app, q()), Some((c(PlaceReady(TURRET)), Pending, "Place")));
    hover(&mut app, middle(OUTSIDE, (1, 1)));
    assert!(!ghost(&app).expect("the ghost").is_valid(), "outside the build area");
    click(&mut app, middle(OUTSIDE, (1, 1)), &[]);
    assert_eq!(constructions(&app).len(), 3, "nothing placed");
    hover(&mut app, middle(INSIDE, (1, 1)));
    assert!(ghost(&app).expect("the ghost").is_valid());
    click(&mut app, middle(INSIDE, (1, 1)), &[]);
    let placed = place(INSIDE, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DS], placed)));
    updates(&mut app, 1);
    assert!(structure_at(&app, TURRET, INSIDE).is_some());
    assert_eq!(constructor(&app, DS), Constructor::Idle);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// Two Defense Systems selected are one SelectionGroup ("Groupable true"): Q builds a turret at both, each paying,
/// and once they are ready Q places the first one's, the one whose construction the panel shows (**Placeholder (not in
/// the design: a group's placement goes to its first Defense System)**); the second still holds its turret ready.
#[test]
fn two_defense_systems_build_together_and_place_the_first_ones_turret() {
    let mut app = defense_game(2);
    click(&mut app, middle(DS_AT, (2, 2)), &[]);
    click(&mut app, middle(DS2_AT, (2, 2)), &[ShiftLeft]);
    assert_eq!(selected(&app), [DS, DS2]);
    let groups = app.world().resource::<ControlStates>().selection(P1).groups().len();
    assert_eq!(groups, 1, "Groupable");
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(TURRET);
    assert_eq!(constructions(&app), [(P1, vec![DS, DS2], construct)]);
    run_until_ready(&mut app, DS);
    assert_eq!(constructor(&app, DS2).ready(), Some(TURRET));
    assert_eq!(bank(&app), 800);
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DS), TURRET)));
    hover(&mut app, middle(INSIDE2, (1, 1)));
    click(&mut app, middle(INSIDE2, (1, 1)), &[]);
    let placed = place(INSIDE2, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DS], placed)));
    updates(&mut app, 1);
    assert!(structure_at(&app, TURRET, INSIDE2).is_some());
    assert_eq!(constructor(&app, DS), Constructor::Idle);
    assert_eq!(constructor(&app, DS2).ready(), Some(TURRET), "still ready");
}
