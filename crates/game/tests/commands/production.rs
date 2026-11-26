//! Production from the command panel (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`, slice
//! M2-H1): Q trains a Peacekeeper and X cancels the last unit queued, by hotkey and by button, as one
//! `Command::Production` to every selected Barracks (no smart-casting, design changes 0071 and 0075: the sim orders
//! the payments, 0083, and the player hears when crystals fell short). A Train or a Cancel Production that the
//! command gate would refuse shows disabled and does nothing (0081); Unit Control never does: it is checked when
//! production starts, and the player hears when a Barracks waits for it (design change 0143).

use bevy::prelude::*;
use sc_sim::{ArmyResource, Command, ObjectKind, PlayerId, ProductionCommand, UNIT_CONTROL_CAP};
use space_crystals::command_ui::feedback::{NO_CRYSTALS_TEXT, TargetFeedback, UNIT_CONTROL_TEXT};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::InfoText;

use super::*;
use KeyCode::{KeyE, KeyQ, KeyX, ShiftLeft};

const P1: PlayerId = PlayerId(1);
const PK: ObjectKind = ObjectKind::Peacekeeper;
const TRAIN: ProductionCommand = ProductionCommand::Train(PK);
const CANCEL: ProductionCommand = ProductionCommand::CancelProduction;

fn q() -> CommandSlot {
    slot(0, 0)
}

/// The buttons the HUD shows: slot, button, look, label.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = query
        .iter(app.world())
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.label()))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The look of the button the HUD shows in `slot`.
fn look(app: &mut App, slot: CommandSlot) -> Option<ButtonLook> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query.iter(app.world()).find(|b| b.slot == slot).map(|b| b.shown.look)
}

/// How many units Barracks `id` has queued, the one in production included.
fn queued(app: &App, id: u32) -> usize {
    production_of(app, id).queue.len()
}

/// The production commands given since the first `from`.
fn given_since(app: &App, from: usize) -> Vec<(PlayerId, Vec<u32>, ProductionCommand)> {
    productions(app)[from..].to_vec()
}

/// `> ObjectInterfaceState[Barracks]` and `systems/controls/interface.md > CommandPanel > Standard Slot
/// Assignments`: a selected Barracks shows Train Peacekeeper in Q, Train Field Engineer in E (slice M5-N9; greyed: its
/// owner has no Career Campus), Cancel Production in X and Set Rally Point in C (W, the Rocketman's, stays empty until
/// it is in the game). Q pays 50 Space Crystals and queues a Peacekeeper, by hotkey
/// or by button; X takes the last one off the queue and refunds it; Shift queues neither. Cancel Production is disabled
/// while nothing is queued.
#[test]
fn q_trains_and_x_cancels_by_hotkey_and_by_button() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let mut app = production_game(0, 1000);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(selected(&app), [15]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Train(PK)), Common, "PK"),
            (slot(0, 2), c(Train(ObjectKind::FieldEngineer)), Disabled, "FEN"),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelProduction), Disabled, "Cancel"),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), Common, "Rally"),
        ]
    );

    tap(&mut app, &[], KeyQ);
    assert_eq!(productions(&app), [(P1, vec![15], TRAIN)]);
    assert_eq!((queued(&app, 15), bank(&app)), (1, 950));
    click_button(&mut app, q(), &[]);
    assert_eq!(given_since(&app, 1), [(P1, vec![15], TRAIN)]);
    assert_eq!((queued(&app, 15), bank(&app)), (2, 900));
    assert_eq!(look(&mut app, CommandSlot::CANCEL_PRODUCTION), Some(Common));

    tap(&mut app, &[], KeyX);
    assert_eq!(given_since(&app, 2), [(P1, vec![15], CANCEL)]);
    assert_eq!((queued(&app, 15), bank(&app)), (1, 950));
    click_button(&mut app, CommandSlot::CANCEL_PRODUCTION, &[]);
    assert_eq!(given_since(&app, 3), [(P1, vec![15], CANCEL)]);
    assert_eq!((queued(&app, 15), bank(&app)), (0, 1000));
    assert_eq!(look(&mut app, CommandSlot::CANCEL_PRODUCTION), Some(Disabled));

    // Production isn't queued like orders: with Shift held, Q still trains at once.
    tap(&mut app, &[ShiftLeft], KeyQ);
    assert_eq!(given_since(&app, 4), [(P1, vec![15], TRAIN)]);
    assert_eq!(queued(&app, 15), 1);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), [], "no unit orders");
}

/// What the feedback message says, while one shows.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// No smart-casting (design changes 0071 and 0075): with three Barracks selected, Q and X each go to all of them as
/// one command, by hotkey and by button. With Space Crystals for two Peacekeepers only, Q still goes to all three,
/// the sim queues the unit at the two with the least work queued, ties by id, and the player is told the crystals
/// fell short (0083's first verification and its feedback).
#[test]
fn q_and_x_go_to_every_selected_barracks() {
    let mut app = production_game(0, 1000);
    select_barracks(&mut app, &BARRACKS);
    let queues = |app: &App| BARRACKS.map(|id| queued(app, id));
    tap(&mut app, &[], KeyQ);
    assert_eq!(productions(&app), [(P1, BARRACKS.to_vec(), TRAIN)]);
    assert_eq!((queues(&app), bank(&app)), ([1, 1, 1], 850));
    click_button(&mut app, q(), &[]);
    assert_eq!(given_since(&app, 1), [(P1, BARRACKS.to_vec(), TRAIN)]);
    assert_eq!(queues(&app), [2, 2, 2]);
    assert_eq!(feedback(&app), None);

    set_crystals(&mut app, 100);
    tap(&mut app, &[], KeyQ);
    assert_eq!(given_since(&app, 2), [(P1, BARRACKS.to_vec(), TRAIN)]);
    assert_eq!((queues(&app), bank(&app)), ([3, 3, 2], 0));
    assert_eq!(feedback(&app), Some(NO_CRYSTALS_TEXT));
    let mut texts = app.world_mut().query::<&Text>();
    assert!(texts.iter(app.world()).any(|t| t.0 == NO_CRYSTALS_TEXT));

    tap(&mut app, &[], KeyX);
    assert_eq!(given_since(&app, 3), [(P1, BARRACKS.to_vec(), CANCEL)]);
    assert_eq!((queues(&app), bank(&app)), ([2, 2, 1], 150));
}

/// `> ObjectInterfaceState[Barracks]`: "Only available if BuildQueue has fewer than 20 entries and player has
/// sufficient Space Crystals" (design change 0081: the 20 include the unit in production); "X: [...] Only available
/// if BuildQueue is not empty". A disabled Q or X sends nothing, by hotkey or by button. The plan's acceptance: "Q is
/// greyed out below 50 SC".
#[test]
fn q_is_disabled_without_the_crystals_or_with_the_queue_full() {
    use ButtonLook::{Common, Disabled};
    let mut app = production_game(0, 49);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(look(&mut app, q()), Some(Disabled), "below 50 Space Crystals");
    assert_eq!(
        look(&mut app, CommandSlot::CANCEL_PRODUCTION),
        Some(Disabled),
        "nothing queued"
    );
    tap(&mut app, &[], KeyQ);
    click_button(&mut app, q(), &[]);
    tap(&mut app, &[], KeyX);
    click_button(&mut app, CommandSlot::CANCEL_PRODUCTION, &[]);
    assert_eq!(productions(&app), []);

    set_crystals(&mut app, 50);
    assert_eq!(look(&mut app, q()), Some(Common));
    tap(&mut app, &[], KeyQ);
    assert_eq!((queued(&app, 15), bank(&app)), (1, 0));
    assert_eq!(look(&mut app, q()), Some(Disabled), "spent");
    assert_eq!(look(&mut app, CommandSlot::CANCEL_PRODUCTION), Some(Common));

    set_crystals(&mut app, 2000);
    for _ in 0..19 {
        submit(
            &mut app,
            P1,
            Command::Production {
                objects: vec![ObjectId(15)],
                command: TRAIN,
            },
        );
    }
    app.update();
    assert_eq!(queued(&app, 15), 20);
    assert_eq!(look(&mut app, q()), Some(Disabled), "the queue is full");
    let sent = productions(&app).len();
    tap(&mut app, &[], KeyQ);
    assert_eq!(productions(&app).len(), sent);
    tap(&mut app, &[], KeyX);
    assert_eq!(queued(&app, 15), 19);
    assert_eq!(look(&mut app, q()), Some(Common));
}

/// Design change 0143 through the panel (`> ObjectInterfaceState[Barracks]`: Q is "Only available if BuildQueue has
/// fewer than 20 entries and player has sufficient Space Crystals (Unit Control is checked when production
/// starts)"): at the Unit Control cap (200 of 200) Q stays available, and each press queues and pays. A Barracks
/// whose Peacekeeper at the front has no room waits at 0%, and the player is told, by placeholders for 0143's
/// notification: the band names the cap as the wait begins, and the Barracks' InfoPanel says what it waits for. With
/// two Barracks selected, Q queues at both (0083 no longer considers Unit Control), and the second waits as well.
/// (The Barracks here have no power, which makes no difference to a unit without room: the HUD's figure with a unit
/// in production is the sim's, `tests/production/capacity.rs`.)
#[test]
fn q_stays_available_at_the_unit_control_cap() {
    let mut app = production_game(193, 1000);
    let unit_control = |app: &App| app.world().resource::<SimRes>().unit_control(P1);
    let waiting = |app: &App, id| {
        let sim = app.world().resource::<SimRes>();
        sim.production_status(ObjectId(id)).and_then(|status| status.blocked)
    };
    assert_eq!(unit_control(&app), UNIT_CONTROL_CAP);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(look(&mut app, q()), Some(ButtonLook::Common), "at the cap");
    tap(&mut app, &[], KeyQ);
    assert_eq!((queued(&app, 15), bank(&app)), (1, 950), "queued and paid");
    assert_eq!(waiting(&app, 15), Some(ArmyResource::UnitControl));
    assert_eq!(feedback(&app), Some(UNIT_CONTROL_TEXT), "the wait begins");
    let lines = activity(&mut app).expect("the activity area shows");
    assert_eq!(
        lines,
        "Training Peacekeeper: 0%\nWaiting for Unit Control\nQueue: 1 / 20"
    );
    assert_eq!(look(&mut app, q()), Some(ButtonLook::Common));

    select_barracks(&mut app, &[15, 16]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(given_since(&app, 1), [(P1, vec![15, 16], TRAIN)]);
    assert_eq!(([queued(&app, 15), queued(&app, 16)], bank(&app)), ([2, 1], 850));
    assert_eq!(waiting(&app, 16), Some(ArmyResource::UnitControl));
    assert_eq!(unit_control(&app), UNIT_CONTROL_CAP);
}

/// `> ObjectInterfaceState[Barracks]`: "E: Build Field Engineer [...] Only available if the player owns at least one
/// Career Campus" (design change 0328; slice M5-N9). Without one, E is greyed, says "Requires a Career Campus" when
/// pressed and sends nothing; with a Career Campus (a debug spawn) it queues a Field Engineer for 75 Space Crystals.
/// The Career Campus lost, E is greyed again, and the Field Engineer stays queued (`core/tech_and_research.md > Tech
/// Prerequisites > Unit Prerequisites`: "units already queued or in production continue and complete normally"; these
/// Barracks have no power, so it never comes out here: the sim's `field_engineer` tests have it come out).
#[test]
fn e_trains_a_field_engineer_once_a_career_campus_stands() {
    use ButtonLook::{Common, Disabled};
    const FIELD_ENGINEER: ObjectKind = ObjectKind::FieldEngineer;
    let e = slot(0, 2);
    let mut app = production_game(0, 1000);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(look(&mut app, e), Some(Disabled), "no Career Campus");
    tap(&mut app, &[], KeyE);
    assert_eq!(feedback(&app), Some("Requires a Career Campus"));
    assert_eq!(productions(&app), []);

    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::CareerCampus,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(40, 30)),
        orientation: Default::default(),
    };
    submit(&mut app, P1, Command::Debug(spawn));
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    let campus = (sim.objects()).find(|o| o.kind == ObjectKind::CareerCampus);
    let campus = campus.expect("spawned").id;
    assert_eq!(look(&mut app, e), Some(Common));
    tap(&mut app, &[], KeyE);
    let train = ProductionCommand::Train(FIELD_ENGINEER);
    assert_eq!(productions(&app), [(P1, vec![15], train)]);
    updates(&mut app, 2);
    assert_eq!((queued(&app, 15), bank(&app)), (1, 925));

    submit(&mut app, P1, Command::Debug(DebugCommand::Remove { id: campus }));
    updates(&mut app, 2);
    assert_eq!(look(&mut app, e), Some(Disabled), "greyed again");
    tap(&mut app, &[], KeyE);
    assert_eq!(feedback(&app), Some("Requires a Career Campus"));
    assert_eq!(productions(&app).len(), 1, "nothing more sent");
    assert_eq!((queued(&app, 15), bank(&app)), (1, 925), "still queued, not refunded");
}

/// The InfoPanel's activity area, while it shows (the Altar's tests' too, `super::altar`).
pub(super) fn activity(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    texts
        .iter(app.world())
        .find(|(which, _, node)| **which == InfoText::Activity && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
}
