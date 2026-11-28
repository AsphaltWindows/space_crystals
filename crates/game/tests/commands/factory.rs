//! The Factory in the client (slice M6-GF; `factions/gdo/structures/factory.md > Placeholder stats`: "Production as
//! the Barracks (queue 20, refunds, rally). Panel: **Q** Raptor, **W** Twister, **E** Prospector, **R** Artillery Rig
//! (greyed without Advanced Mechanics), **X** Cancel Production, **C** Set Rally Point"): its panel, Q and W training,
//! W greyed for want of a Supply (design change 0441), E not available yet (slice M6-GP), (M6-GA) R greyed without
//! Advanced Mechanics (the Artillery Rig's training is `artillery_rig_ui`'s); its default
//! rally point out of side B (design change 0446), which a right-click on it clears (0447); the Deployment Center's
//! Advanced menu constructing it once a Barracks stands, and (M6-GH2) the Airfield once an Advanced Mechanics stands;
//! and a Twister's Attack on a Peacekeeper, an invalid target
//! ("An Attack on a Peacekeeper is refused, with feedback": its machine gun hits air only), and its R and F, its mines,
//! not available yet (slice M6-GM). The panel as plain data
//! is the unit tests' (`command_ui::interface::tests`), the training, the exit and the parking the sim's
//! (`crates/sim/tests/factory`).

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{ConstructionCommand, Faction};
use space_crystals::command_ui::feedback::{
    INVALID_TARGET_TEXT, NO_SUPPLIES_TEXT, NOT_YET_TEXT, TargetFeedback, requires_text,
};
use space_crystals::command_ui::indicators::{Indicator, MOVE_COLOR, RallyIndicator};
use space_crystals::command_ui::{BuildMenu, ButtonLook, PanelButton, PanelCommand, Planned};
use space_crystals::hud::command_panel::CommandButton;

use super::indicators::drawn_vertices;
use super::rally::shown;
use super::*;
use KeyCode::{KeyA, KeyE, KeyQ, KeyR, KeyW};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RAPTOR: ObjectKind = ObjectKind::Raptor;
const TWISTER: ObjectKind = ObjectKind::Twister;

// The objects of [`factory_game`], by id.
/// Player 1's Deployment Center, on tiles (14, 8) to (17, 11), and its Factory, unturned, on tiles (20, 12) to (22,
/// 14): x 1280 to 1472, y 768 to 960, its exit, side B, to the east.
const DC: u32 = 1;
const FACTORY: u32 = 2;
const FACTORY_AT: Vec2 = Vec2::new(1376.0, 864.0);
const DC_AT: Vec2 = Vec2::new(1024.0, 640.0);
/// Player 1's Twister, south of the Factory, and player 2's Peacekeeper, east of it, in its sight.
const TWISTER_ID: u32 = 3;
const ENEMY_PK: u32 = 4;
/// The middle of side B, where a vehicle comes out, half inside, facing east.
const EXIT: Vec2 = Vec2::new(1472.0, 864.0);

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up (no fog, no auto-targeting, the
/// camera at [`VIEW_CENTER`]): player 1's Deployment Center, whose Power runs the Factory, its Factory and a Twister;
/// player 2's Peacekeeper; and a bank of `crystals` Space Crystals and `supplies` Supplies.
fn factory_game(crystals: u32, supplies: u32) -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(14, 8)),
        )
        .spawn(ObjectKind::Factory, Some(P1), SpawnAt::Tile(TilePos::new(20, 12)))
        .spawn(TWISTER, Some(P1), point(1376.0, 1100.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(1700.0, 1100.0));
    let mut app = game_of(scenario.build());
    set_bank(&mut app, crystals, supplies);
    app
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

/// The buttons the HUD shows: slot, button, look and label.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = (query.iter(app.world()))
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.label()))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The button the HUD shows in `slot`: its command and its look.
fn shown_in(app: &mut App, slot: CommandSlot) -> Option<(PanelButton, ButtonLook)> {
    (buttons(app).into_iter()).find_map(|(at, button, look, _)| (at == slot).then_some((button, look)))
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// Every construction command given so far, as the sim received them: player, constructors, command.
fn constructions(app: &App) -> Vec<(PlayerId, Vec<u32>, ConstructionCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Construction { objects, command } => {
                Some((c.player, objects.iter().map(|id| id.0).collect(), *command))
            }
            _ => None,
        })
        .collect()
}

/// Selected, the Factory shows the Production template of its recipes: Q Train Raptor, W Train Twister, E Train
/// Prospector, not available yet ("Not available yet" when pressed, nothing sent), R Train Artillery Rig, greyed
/// without Advanced Mechanics ("Requires an Advanced Mechanics" when pressed, nothing sent; slice M6-GA), X Cancel
/// Production, greyed with nothing to cancel, and C Set Rally Point. With no Supply W is greyed and says "Not enough
/// Supplies" when pressed, sending nothing (design change 0441); Q trains a Raptor for 100 Space Crystals. Given a
/// Supply, W trains a Twister for 125 Space Crystals and the Supply, and is greyed again.
#[test]
fn the_factory_trains_the_raptor_and_the_twister_its_e_not_available_yet_and_r_behind_advanced_mechanics() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let mut app = factory_game(1000, 0);
    click(&mut app, FACTORY_AT, &[]);
    assert_eq!(selected(&app), [FACTORY]);
    let c = PanelButton::Command;
    let planned = |planned| c(PanelCommand::Planned(planned));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Train(RAPTOR)), Common, "RR"),
            (slot(0, 1), c(Train(TWISTER)), Disabled, "TR"),
            (slot(0, 2), planned(Planned::Prospector), Unavailable, "PR"),
            (slot(0, 3), c(Train(ObjectKind::ArtilleryRig)), Disabled, "ARG"),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelProduction), Disabled, "Cancel"),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), Common, "Rally"),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(said(&app), Some(NO_SUPPLIES_TEXT));
    pause(&mut app);
    tap(&mut app, &[], KeyE);
    assert_eq!(said(&app), Some(NOT_YET_TEXT));
    pause(&mut app);
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app), Some(requires_text(ObjectKind::AdvancedMechanics)));
    assert_eq!(productions(&app), []);

    tap(&mut app, &[], KeyQ);
    assert_eq!(
        productions(&app),
        [(P1, vec![FACTORY], ProductionCommand::Train(RAPTOR))]
    );
    updates(&mut app, 2);
    assert_eq!(resources(&app), (900, 0), "a Raptor's 100");

    set_bank(&mut app, 900, 1);
    assert_eq!(shown_in(&mut app, slot(0, 1)), Some((c(Train(TWISTER)), Common)));
    tap(&mut app, &[], KeyW);
    assert_eq!(
        productions(&app).last(),
        Some(&(P1, vec![FACTORY], ProductionCommand::Train(TWISTER)))
    );
    updates(&mut app, 2);
    assert_eq!(resources(&app), (775, 0), "a Twister's 125 and its Supply");
    assert_eq!(production_of(&app, FACTORY).queue.len(), 2);
    assert_eq!(shown_in(&mut app, slot(0, 1)), Some((c(Train(TWISTER)), Disabled)));
}

/// Design change 0446's Verification, the Factory's part: "a new Factory shows its rally point 3 GU out of side B",
/// as any rally point shows while its building is selected (design change 0087): a green marker on the ground 3 grid
/// units east of the middle of side B, with its line from the building. 0447's: a right-click on the Factory itself
/// clears it, and nothing shows.
#[test]
fn a_new_factory_shows_its_rally_point_out_of_side_b_and_a_right_click_on_it_clears_it() {
    let mut app = factory_game(1000, 0);
    click(&mut app, FACTORY_AT, &[]);
    assert_eq!(selected(&app), [FACTORY]);
    let marker = RallyIndicator {
        producer: ObjectId(FACTORY),
        marker: Indicator::Location {
            at: EXIT + Vec2::new(192.0, 0.0),
            color: MOVE_COLOR,
        },
    };
    assert_eq!(shown(&app), [marker]);
    assert!(drawn_vertices(&mut app) > 0);
    right_click(&mut app, FACTORY_AT, &[]);
    let clear = ProductionCommand::SetRally(Target::Object(ObjectId(FACTORY)));
    assert_eq!(productions(&app).last(), Some(&(P1, vec![FACTORY], clear)));
    assert_eq!(production_of(&app, FACTORY).rally, None, "cleared");
    assert_eq!((shown(&app), drawn_vertices(&mut app)), (vec![], 0), "nothing shows");
}

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Factory: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Barracks". With a Factory standing but no Barracks, the
/// Deployment Center's Advanced menu has Q greyed ("Requires a Barracks") and W live, Advanced Mechanics needing a
/// Factory; with a Barracks (a debug spawn) Q constructs a Factory for 250 Space Crystals, and the menu closes.
#[test]
fn the_advanced_menus_q_constructs_a_factory_once_a_barracks_stands() {
    use ButtonLook::{Common, Disabled};
    let mut app = factory_game(1000, 0);
    click(&mut app, DC_AT, &[]);
    assert_eq!(selected(&app), [DC]);
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let construct = |kind| PanelButton::Command(PanelCommand::Construct(kind));
    assert_eq!(
        shown_in(&mut app, slot(0, 0)),
        Some((construct(ObjectKind::Factory), Disabled))
    );
    let mechanics = construct(ObjectKind::AdvancedMechanics);
    assert_eq!(
        shown_in(&mut app, slot(0, 1)),
        Some((mechanics, Common)),
        "a Factory stands"
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some("Requires a Barracks"));
    assert_eq!(constructions(&app), []);

    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Barracks,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(26, 8)),
        orientation: Default::default(),
    };
    submit(&mut app, P1, Command::Debug(spawn));
    updates(&mut app, 2);
    assert_eq!(
        shown_in(&mut app, slot(0, 0)),
        Some((construct(ObjectKind::Factory), Common))
    );
    tap(&mut app, &[], KeyQ);
    let sent = ConstructionCommand::Construct(ObjectKind::Factory);
    assert_eq!(constructions(&app), [(P1, vec![DC], sent)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (750, 0), "the Factory's 250");
}

/// (M6-GH2) `factions/gdo/structures/deployment_center.md > Constructs`: "Airfield: 250 Space Crystals, 480 frames (30
/// seconds) (placeholder). Requires: player owns at least one Advanced Mechanics". With a Factory standing but no
/// Advanced Mechanics, the Advanced menu's E is greyed ("Requires an Advanced Mechanics") and sends nothing; with one
/// (a debug spawn) E constructs an Airfield for 250 Space Crystals, and the menu closes.
#[test]
fn the_advanced_menus_e_constructs_an_airfield_once_an_advanced_mechanics_stands() {
    use ButtonLook::{Common, Disabled};
    let mut app = factory_game(1000, 0);
    click(&mut app, DC_AT, &[]);
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let airfield = PanelButton::Command(PanelCommand::Construct(ObjectKind::Airfield));
    assert_eq!(shown_in(&mut app, slot(0, 2)), Some((airfield, Disabled)));
    tap(&mut app, &[], KeyE);
    assert_eq!(said(&app), Some(requires_text(ObjectKind::AdvancedMechanics)));
    assert_eq!(constructions(&app), []);

    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::AdvancedMechanics,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(26, 8)),
        orientation: Default::default(),
    };
    submit(&mut app, P1, Command::Debug(spawn));
    updates(&mut app, 2);
    assert_eq!(shown_in(&mut app, slot(0, 2)), Some((airfield, Common)));
    tap(&mut app, &[], KeyE);
    let sent = ConstructionCommand::Construct(ObjectKind::Airfield);
    assert_eq!(constructions(&app), [(P1, vec![DC], sent)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (750, 0), "the Airfield's 250");
}

/// The plan's acceptance: "An Attack on a Peacekeeper is refused, with feedback". The Twister's A awaits its target;
/// a click on player 2's Peacekeeper, which its machine gun can't hit (`TargetDomain: Air`; the gate's
/// `Reject::CannotHit`), is an invalid target: "Invalid target", no order, the target still awaited. Its right-click
/// on the Peacekeeper, a unit's on a target it can't attack, follows it.
#[test]
fn a_twisters_attack_on_a_peacekeeper_is_an_invalid_target() {
    let mut app = factory_game(1000, 0);
    select_units(&mut app, &[TWISTER_ID]);
    tap(&mut app, &[], KeyA);
    let awaiting = InterfaceState::awaiting(PanelCommand::Attack);
    assert_eq!(interface(&app), awaiting);
    let enemy_at = pos(&app, ENEMY_PK);
    click(&mut app, enemy_at, &[]);
    assert_eq!(said(&app), Some(INVALID_TARGET_TEXT));
    assert_eq!(interface(&app), awaiting, "still awaited");
    assert_eq!(orders(&app), []);

    tap(&mut app, &[], KeyCode::Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    right_click(&mut app, enemy_at, &[]);
    let follow = OrderKind::Move(Target::Object(ObjectId(ENEMY_PK)));
    assert_eq!(orders(&app), [given(&[TWISTER_ID], follow, false)]);
}

/// The Twister's panel (`factions/gdo/units/twister.md > Placeholder stats`: "Panel: BasicCombatUnitInterfaceState plus
/// **R: Lay Mine** (targeted) and **F: Buy Mine**"): its mines come with slice M6-GM, so R and F hold their slots,
/// greyed ("Lay" and "Buy"), and say "Not available yet" when pressed, sending nothing.
#[test]
fn a_twisters_r_and_f_its_mines_are_not_available_yet() {
    use ButtonLook::Unavailable;
    let mut app = factory_game(1000, 0);
    select_units(&mut app, &[TWISTER_ID]);
    let planned = |planned| PanelButton::Command(PanelCommand::Planned(planned));
    let r = (slot(0, 3), planned(Planned::LayMine), Unavailable, "Lay");
    let f = (slot(1, 3), planned(Planned::BuyMine), Unavailable, "Buy");
    let shown = buttons(&mut app);
    assert!(shown.contains(&r) && shown.contains(&f), "{shown:?}");
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app), Some(NOT_YET_TEXT));
    pause(&mut app);
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(said(&app), Some(NOT_YET_TEXT));
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
