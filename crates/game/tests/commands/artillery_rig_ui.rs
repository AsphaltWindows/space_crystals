//! The Artillery Rig's UI in the client (slice M6-GA; `factions/gdo/units/artillery_rig.md`, plan decisions T44 and
//! T87, the vehicle deploy model): the Factory's R, greyed for want of its 5 Supplies, then paid in both resources and
//! refunded (design change 0435); the mobile Rig's R, Deploy, greyed while it deploys with the progress on its bar and
//! in the InfoPanel, and the Rig, deployed 48 frames on, in its new SelectionGroup, which is active, with R Undeploy;
//! an Attack right-clicked while it deploys, held, and a Shift-Move dropped as it ends, with feedback; and the deployed
//! Rig's refusal to move, with feedback, until it undeploys. The panels as plain data are
//! `command_ui::interface::tests::artillery_rig`'s.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, Tick, TickInputs};
use space_crystals::command_ui::feedback::{CANNOT_MOVE_TEXT, DEPLOYING_TEXT, NO_SUPPLIES_TEXT, TargetFeedback};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PanelSlot};
use space_crystals::hud::command_panel::{CommandButton, CommandPanelView};

use super::*;
use KeyCode::{KeyQ, KeyR, KeyX, ShiftLeft, Tab};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const MOBILE: ObjectKind = ObjectKind::ArtilleryRig;
const DEPLOYED: ObjectKind = ObjectKind::ArtilleryRigDeployed;
const DEPLOY: PanelCommand = PanelCommand::VehicleDeploy(DEPLOYED);
const UNDEPLOY: PanelCommand = PanelCommand::VehicleDeploy(MOBILE);
/// "Deploy 48 frames (3 s), Undeploy 48 frames."
const FRAMES: Tick = 48;

// The objects of [`rig_game`], by id.
/// Player 1's Artillery Rig, facing east, and its Peacekeeper south of it.
const RIG: u32 = 1;
const PK: u32 = 2;
/// Player 2's Peacekeeper, east of the Rig, 5 grid units off edge to edge: in its sight, outside its MinRange of 3.
const ENEMY: u32 = 3;
const ENEMY_AT: Vec2 = Vec2::new(1370.0, 900.0);
/// A point of the ground in the world view, with nothing on it.
const GROUND: Vec2 = Vec2::new(1300.0, 1150.0);

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up (no fog, no auto-targeting, the
/// camera at [`VIEW_CENTER`]): player 1's Artillery Rig and Peacekeeper, player 2's Peacekeeper.
fn rig_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(MOBILE, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(1000.0, 1100.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(ENEMY_AT));
    game_of(scenario.build())
}

/// The button the HUD shows in `slot`.
fn button(app: &mut App, slot: CommandSlot) -> Option<PanelSlot> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query.iter(app.world()).find(|b| b.slot == slot).map(|b| b.shown)
}

/// The command of the button the HUD shows in `slot`, its look and its label.
fn command(app: &mut App, slot: CommandSlot) -> Option<(PanelCommand, ButtonLook, &'static str)> {
    let shown = button(app, slot)?;
    match shown.button {
        PanelButton::Command(command) => Some((command, shown.look, shown.label())),
        PanelButton::Back => None,
    }
}

/// R, Deploy or Undeploy.
fn r() -> CommandSlot {
    slot(0, 3)
}

/// What R's bar shows: the progress of the panel the HUD shows now, which moves the bar on in place
/// (`hud::command_panel`), its buttons as they were built.
fn r_bar(app: &App) -> Option<sc_sim::Progress> {
    let view = app.world().resource::<CommandPanelView>();
    view.panel.get(r()).and_then(|shown| shown.progress)
}

/// The type of object `id`, as the sim has it.
fn kind_of(app: &App, id: u32) -> ObjectKind {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).expect("in the game").kind
}

/// The sim's tick.
fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

/// The tick that applied the last vehicle deploy order given.
fn deploy_given(app: &App) -> Tick {
    let recorded = app.world().resource::<RecordedInputs>();
    let deploys = |inputs: &TickInputs| {
        (inputs.commands.iter()).any(|c| {
            matches!(
                c.command,
                Command::Order {
                    kind: OrderKind::VehicleDeploy { .. },
                    ..
                }
            )
        })
    };
    let index = recorded.ticks.iter().rposition(deploys).expect("a deploy given");
    index as Tick + 1
}

/// Runs a tick at a time until object `id` is of type `kind`, and returns that tick.
fn until_kind(app: &mut App, id: u32, kind: ObjectKind) -> Tick {
    for _ in 0..4 * FRAMES {
        if kind_of(app, id) == kind {
            return tick(app);
        }
        app.update();
    }
    panic!("#{id} never became {kind:?}");
}

/// The local player's SelectionGroups, in their order: type, objects, and whether it is the ActiveGroup.
fn groups(app: &App) -> Vec<(ObjectKind, Vec<u32>, bool)> {
    let states = app.world().resource::<ControlStates>();
    let selection = states.selection(local_player(app));
    let active = selection.active();
    (selection.groups().into_iter())
        .map(|g| {
            (
                g.kind(),
                g.instances.iter().map(|id| id.0).collect(),
                Some(g.key) == active,
            )
        })
        .collect()
}

/// The type of the ActiveGroup.
fn active_type(app: &App) -> Option<ObjectKind> {
    (groups(app).into_iter()).find_map(|(kind, _, active)| active.then_some(kind))
}

/// The feedback message that shows, and the tick of its cause.
fn said(app: &App) -> Option<(&'static str, Tick)> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|since| (feedback.text, since))
}

/// Every line of every text the HUD shows.
fn lines(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<&Text>();
    (query.iter(app.world()))
        .flat_map(|t| t.0.lines().map(str::to_string).collect::<Vec<_>>())
        .collect()
}

/// `factions/gdo/units/artillery_rig.md > Modes`, the group switch ("**Two unit types** [...], with separate
/// SelectionGroups"; "When the switch makes a group disappear, the group the units joined becomes active"): with the Rig
/// and a Peacekeeper selected and the Rig's group active, its R, "Deploy", a group command, gives it a vehicle deploy
/// order, unqueued. It stays a mobile Rig for the deploy's 48 frames and is the deployed one from the 48th tick after
/// the one that applied the order; its group gone, the group it joined is active, not the Peacekeeper's, and R is
/// Undeploy, "Undep.".
#[test]
fn r_deploys_the_rig_and_the_group_it_joins_becomes_active_with_r_undeploy() {
    let mut app = rig_game();
    select_units(&mut app, &[RIG, PK]);
    if active_type(&app) != Some(MOBILE) {
        tap(&mut app, &[], Tab);
    }
    assert_eq!(active_type(&app), Some(MOBILE));
    assert_eq!(command(&mut app, r()), Some((DEPLOY, ButtonLook::Group, "Deploy")));
    tap(&mut app, &[], KeyR);
    let deploy = OrderKind::VehicleDeploy { into: DEPLOYED };
    assert_eq!(orders(&app), [given(&[RIG], deploy, false)]);
    let given_in = deploy_given(&app);
    let deployed = until_kind(&mut app, RIG, DEPLOYED);
    assert_eq!(deployed, given_in + FRAMES, "48 frames");
    app.update();
    assert_eq!(selected(&app), [RIG, PK]);
    assert_eq!(
        active_type(&app),
        Some(DEPLOYED),
        "the group it joined: {:?}",
        groups(&app)
    );
    assert_eq!(command(&mut app, r()), Some((UNDEPLOY, ButtonLook::Group, "Undep.")));
}

/// While the Rig deploys its R is greyed, its bar showing the deploy's progress, and the InfoPanel says it,
/// "Deploying: 50%" halfway (placeholder lines); R pressed meanwhile says "Still switching modes" and sends nothing
/// ("**Undeploy is only available once deployment is complete**"). Deployed, the line is gone and R Undeploy is lit.
#[test]
fn while_the_rig_deploys_r_is_greyed_and_the_info_panel_shows_the_progress() {
    let mut app = rig_game();
    select_units(&mut app, &[RIG]);
    tap(&mut app, &[], KeyR);
    let given_in = deploy_given(&app);
    while tick(&app) < given_in + FRAMES / 2 {
        app.update();
    }
    let shown = button(&mut app, r()).expect("R");
    assert_eq!(
        (shown.button, shown.look),
        (PanelButton::Command(DEPLOY), ButtonLook::Disabled)
    );
    let bar = r_bar(&app).expect("a bar");
    assert_eq!((bar.total, bar.ticks_done()), (48, 24));
    app.update();
    let progress = format!("Deploying: {}%", 25 * 100 / 48);
    assert!(lines(&mut app).contains(&progress), "{progress}: {:?}", lines(&mut app));
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app).map(|(text, _)| text), Some(DEPLOYING_TEXT));
    assert_eq!(orders(&app).len(), 1, "no second deploy");

    until_kind(&mut app, RIG, DEPLOYED);
    app.update();
    assert!(!lines(&mut app).iter().any(|line| line.starts_with("Deploying")));
    assert_eq!(command(&mut app, r()), Some((UNDEPLOY, ButtonLook::Common, "Undep.")));
}

/// `> Modes`, "Orders given while deploying": "the **last order given** [...] is **held** [...]. Shift-queued orders
/// line up behind it as usual. At that point each order is checked against the deployed unit: one it can't carry out
/// (e.g. **Move**, since a deployed unit can't move) is **dropped** with the usual unavailable-command feedback". While
/// the Rig deploys, a right-click on the enemy gives it an Attack (it is judged as the type it becomes, which attacks),
/// and a Shift right-click on the ground a Move queued behind it. As it deploys, the Move is dropped, "Cannot move while
/// deployed" said in that tick, and the Attack runs.
#[test]
fn an_attack_right_clicked_while_it_deploys_is_held_and_a_shift_move_dropped_with_feedback() {
    let mut app = rig_game();
    select_units(&mut app, &[RIG]);
    tap(&mut app, &[], KeyR);
    updates(&mut app, 2);
    right_click(&mut app, ENEMY_AT, &[]);
    right_click(&mut app, GROUND, &[ShiftLeft]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 3, "{given_orders:?}");
    assert_eq!(
        given_orders[1],
        given(&[RIG], OrderKind::Attack(ObjectId(ENEMY)), false)
    );
    assert!(
        given_orders[2].queue && matches!(given_orders[2].kind, OrderKind::Move(Target::Point(p)) if near(p, GROUND))
    );
    assert_eq!(said(&app), None);

    let deployed = until_kind(&mut app, RIG, DEPLOYED);
    app.update();
    assert_eq!(said(&app), Some((CANNOT_MOVE_TEXT, deployed)));
    let held = object_orders(&app, RIG);
    let current = held.current.as_ref().map(|active| active.order.kind);
    assert_eq!(current, Some(OrderKind::Attack(ObjectId(ENEMY))), "the Attack runs");
    assert!(held.queue.is_empty(), "the Move dropped");
}

/// `> Modes`: "**Deployed:** cannot move". Selected, the deployed Rig's Move is greyed, "Cannot move while deployed" when
/// pressed, and a right-click on the ground gives no order and says so. Its R, "Undep.", undeploys it: meanwhile a
/// right-click on the ground gives a Move, held, which the Rig carries out once it is mobile again, 48 frames on.
#[test]
fn the_deployed_rig_refuses_to_move_with_feedback_until_it_undeploys() {
    let mut app = rig_game();
    let at = Vec2::new(1200.0, 700.0);
    let rig = spawn(&mut app, DEPLOYED, Some(P1), at);
    select_units(&mut app, &[rig]);
    let q = button(&mut app, slot(0, 0)).expect("Q");
    assert_eq!(
        (q.button, q.look),
        (PanelButton::Command(PanelCommand::Move), ButtonLook::Disabled)
    );
    tap(&mut app, &[], KeyQ);
    let (text, pressed) = said(&app).expect("feedback");
    assert_eq!(text, CANNOT_MOVE_TEXT);
    pause(&mut app);
    right_click(&mut app, GROUND, &[]);
    assert_eq!(orders(&app), [], "no Move");
    let (text, clicked) = said(&app).expect("feedback");
    assert_eq!(text, CANNOT_MOVE_TEXT);
    assert!(clicked > pressed, "the right-click said it too");

    assert_eq!(command(&mut app, r()), Some((UNDEPLOY, ButtonLook::Common, "Undep.")));
    tap(&mut app, &[], KeyR);
    assert_eq!(
        orders(&app),
        [given(&[rig], OrderKind::VehicleDeploy { into: MOBILE }, false)]
    );
    updates(&mut app, 2);
    right_click(&mut app, GROUND, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 2, "{given_orders:?}");
    assert!(matches!(given_orders[1].kind, OrderKind::Move(Target::Point(p)) if near(p, GROUND)));
    let undeployed = until_kind(&mut app, rig, MOBILE);
    assert_eq!(undeployed, deploy_given(&app) + FRAMES);
    assert_eq!(pos(&app, rig), at, "it stood while it undeployed");
    updates(&mut app, 40);
    assert!(pos(&app, rig).distance(at) > 20.0, "it drives off: {}", pos(&app, rig));
}

// The objects of [`works_game`], by id.
/// Player 1's Factory, unturned, on tiles (20, 12) to (22, 14).
const FACTORY: u32 = 2;
const FACTORY_AT: Vec2 = Vec2::new(1376.0, 864.0);

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up: player 1's Deployment Center, its
/// Factory, its Advanced Mechanics and two Generators; player 2's Deployment Center far off; and a bank of 1000 Space
/// Crystals and `supplies` Supplies.
fn works_game(supplies: u32) -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(14, 8))
        .spawn(ObjectKind::Factory, Some(P1), tile(20, 12))
        .spawn(ObjectKind::AdvancedMechanics, Some(P1), tile(26, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 12))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(56, 40));
    let mut app = game_of(scenario.build());
    set_bank(&mut app, 1000, supplies);
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

/// Design change 0435's Verification in the client (`factions/gdo/units/artillery_rig.md > Placeholder stats`: "Cost:
/// Factory, 250 Space Crystals **+ 5 Supplies**"; "with fewer than 5 Supplies, R is greyed"): with Advanced Mechanics
/// and 4 Supplies, the Factory's R, "ARG", is greyed, says "Not enough Supplies" when pressed and sends nothing. With 5
/// it trains: R sends the Train, which takes 250 Space Crystals and the 5 Supplies, and R is greyed again; X cancels it,
/// refunding both.
#[test]
fn the_factorys_r_trains_an_artillery_rig_for_250_crystals_and_5_supplies() {
    use ButtonLook::{Common, Disabled};
    let mut app = works_game(4);
    click(&mut app, FACTORY_AT, &[]);
    assert_eq!(selected(&app), [FACTORY]);
    let train = PanelCommand::Train(MOBILE);
    assert_eq!(command(&mut app, r()), Some((train, Disabled, "ARG")));
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app).map(|(text, _)| text), Some(NO_SUPPLIES_TEXT));
    assert_eq!(productions(&app), []);

    set_bank(&mut app, 1000, 5);
    assert_eq!(command(&mut app, r()), Some((train, Common, "ARG")));
    tap(&mut app, &[], KeyR);
    let sent = ProductionCommand::Train(MOBILE);
    assert_eq!(productions(&app), [(P1, vec![FACTORY], sent)]);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (750, 0), "250 Space Crystals and 5 Supplies");
    assert_eq!(production_of(&app, FACTORY).queue.len(), 1);
    assert_eq!(command(&mut app, r()), Some((train, Disabled, "ARG")));

    tap(&mut app, &[], KeyX);
    updates(&mut app, 2);
    assert_eq!(resources(&app), (1000, 5), "both refunded");
    assert_eq!(production_of(&app, FACTORY).queue.len(), 0);
}
