//! The Drone Operator's UI in the client (slice M6-GD; `factions/gdo/units/drone_operator.md`, design changes 0421 and
//! 0441): the Barracks' R, greyed for want of a Supply with a feedback line that names Supplies, and paid in both
//! resources when pressed; the operator's R, which launches its drone over it, the selection staying as it is; its F,
//! greyed while the drone is out; and a right-click that would move it while its drone is out, refused with feedback.
//! The panels as plain data are `command_ui::interface::tests::drone_operator`'s.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Ability, Faction};
use space_crystals::command_ui::feedback::{DRONE_OUT_TEXT, NO_SUPPLIES_TEXT, TargetFeedback};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;

use super::*;
use KeyCode::{KeyF, KeyR, KeyX};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// The objects of [`operator_game`], by id.
/// Player 1's Drone Operator, and its Barracks and Career Campus.
const OPERATOR: u32 = 1;
const BARRACKS: u32 = 2;
/// The drone the operator launches, the game's next object after player 2's Peacekeeper (4).
const DRONE: u32 = 5;
/// The centre of the Barracks' footprint, tiles (30, 12) to (32, 13).
const BARRACKS_AT: Vec2 = Vec2::new(2016.0, 832.0);
/// A point of the ground in the world view, with nothing on it.
const GROUND: Vec2 = Vec2::new(1300.0, 1100.0);

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up (no fog, no auto-targeting, the
/// camera at [`VIEW_CENTER`]): player 1's Drone Operator, its Barracks and a Career Campus beside it; player 2's
/// Peacekeeper far off.
fn operator_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::DroneOperator, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(TilePos::new(30, 12)))
        .spawn(ObjectKind::CareerCampus, Some(P1), SpawnAt::Tile(TilePos::new(36, 12)))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(3800.0, 2900.0));
    game_of(scenario.build())
}

/// Player 1's bank becomes `crystals` Space Crystals and `supplies` Supplies (a debug command), a frame later.
fn set_bank(app: &mut App, crystals: u32, supplies: u32) {
    let set = DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(app, P1, Command::Debug(set));
    app.update();
}

/// Player 1's Space Crystals and Supplies.
fn resources(app: &App) -> (u32, u32) {
    let sim = app.world().resource::<SimRes>();
    let player = sim.player(P1).expect("in the game");
    (player.crystals, player.supplies)
}

/// The button the HUD shows in `slot`: its command, its look and its label.
fn shown_in(app: &mut App, slot: CommandSlot) -> Option<(PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown = query.iter(app.world()).filter(|b| b.slot == slot);
    shown.next().map(|b| (b.shown.button, b.shown.look, b.shown.label()))
}

/// The feedback message that shows, and the tick it came in.
fn said(app: &App) -> Option<(&'static str, sc_sim::Tick)> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|since| (feedback.text, since))
}

/// `factions/gdo/units/drone_operator.md > Placeholder stats`: "trained at the Barracks on **R** (Q0155) [...] Cost:
/// Barracks, 100 Space Crystals **+ 1 Supply**"; design change 0441: "greyed without it, with a feedback line that
/// names Supplies". With a Career Campus and 1000 Space Crystals but no Supply, the Barracks' R is greyed, says "Not
/// enough Supplies" when pressed and sends nothing. Given a Supply it trains: R sends the Train, which takes 100 Space
/// Crystals and the Supply, and R is greyed again; X cancels it, refunding both.
#[test]
fn the_barracks_r_trains_a_drone_operator_for_a_supply() {
    use ButtonLook::{Common, Disabled};
    let mut app = operator_game();
    set_bank(&mut app, 1000, 0);
    click(&mut app, BARRACKS_AT, &[]);
    assert_eq!(selected(&app), [BARRACKS]);
    let (r, train) = (slot(0, 3), PanelCommand::Train(ObjectKind::DroneOperator));
    let c = PanelButton::Command;
    assert_eq!(shown_in(&mut app, r), Some((c(train), Disabled, "DO")));
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app).map(|(text, _)| text), Some(NO_SUPPLIES_TEXT));
    assert_eq!(productions(&app), []);

    set_bank(&mut app, 1000, 1);
    assert_eq!(shown_in(&mut app, r), Some((c(train), Common, "DO")));
    tap(&mut app, &[], KeyR);
    let sent = ProductionCommand::Train(ObjectKind::DroneOperator);
    assert_eq!(productions(&app), [(P1, vec![BARRACKS], sent)]);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (900, 0), "100 Space Crystals and 1 Supply");
    assert_eq!(production_of(&app, BARRACKS).queue.len(), 1);
    assert_eq!(shown_in(&mut app, r), Some((c(train), Disabled, "DO")));

    tap(&mut app, &[], KeyX);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (1000, 1), "both refunded");
    assert_eq!(production_of(&app, BARRACKS).queue.len(), 0);
}

/// The operator's R, "Launch", gives it a Launch order, unqueued; its drone comes out over it, and the selection stays
/// the operator ("Launching doesn't change the selection"). While the drone is out, a right-click on the ground gives no
/// order and says "The drone is out" ("The operator refuses Move, with feedback, until the drone is gone"), and F is
/// greyed and says so too. The drone gone, the right-click moves the operator, and F buys the next drone.
#[test]
fn r_launches_the_drone_and_the_operator_refuses_to_move_until_it_is_gone() {
    use ButtonLook::{Common, Disabled};
    let mut app = operator_game();
    set_bank(&mut app, 1000, 0);
    select_units(&mut app, &[OPERATOR]);
    let (r, f) = (slot(0, 3), slot(1, 3));
    let c = PanelButton::Command;
    let launch = PanelCommand::Launch;
    let buy = PanelCommand::Buy(ObjectKind::OperatorDrone);
    assert_eq!(shown_in(&mut app, r), Some((c(launch), Common, "Launch")));
    assert_eq!(
        shown_in(&mut app, f),
        Some((c(buy), Disabled, "DRN")),
        "the stock is full"
    );
    tap(&mut app, &[], KeyR);
    assert_eq!(orders(&app), [given(&[OPERATOR], OrderKind::Launch, false)]);
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    let drone = sim.object(ObjectId(DRONE)).expect("launched");
    assert_eq!(drone.kind, ObjectKind::OperatorDrone);
    assert_eq!(drone.pos, pos(&app, OPERATOR), "over the operator");
    assert_eq!(selected(&app), [OPERATOR], "the selection stays");
    assert_eq!(shown_in(&mut app, r), Some((c(launch), Disabled, "Launch")));
    assert_eq!(shown_in(&mut app, f), Some((c(buy), Disabled, "DRN")));

    assert_eq!(said(&app), None);
    right_click(&mut app, GROUND, &[]);
    assert_eq!(orders(&app).len(), 1, "no Move");
    let (text, first) = said(&app).expect("feedback");
    assert_eq!(text, DRONE_OUT_TEXT);
    updates(&mut app, 2);
    tap(&mut app, &[], KeyF);
    let (text, again) = said(&app).expect("feedback");
    assert_eq!(text, DRONE_OUT_TEXT);
    assert!(again > first, "F said it too");
    assert_eq!(abilities(&app), []);

    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove { id: ObjectId(DRONE) }),
    );
    updates(&mut app, 2);
    assert_eq!(shown_in(&mut app, f), Some((c(buy), Common, "DRN")), "the drone ended");
    right_click(&mut app, GROUND, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 2, "{given_orders:?}");
    assert!(matches!(given_orders[1].kind, OrderKind::Move(Target::Point(p)) if near(p, GROUND)));
    tap(&mut app, &[], KeyF);
    assert_eq!(abilities(&app), [(P1, vec![OPERATOR], Ability::Buy)]);
    updates(&mut app, 2);
    assert_eq!(resources(&app).0, 950, "50 Space Crystals");
}
