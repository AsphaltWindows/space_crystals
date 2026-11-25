//! The Syndicate's panels in the client (slice M4-J2): the Tunnel's letters and its EjectMenu, with the network's
//! counts (design change 0257, `factions/syndicate/tunnels.md > EjectMenu`); the Hideout's letters (design change
//! 0260) and its rallies on a Hideout and on a Tunnel (design change 0252); a control group whose units are all in
//! the network; and the Tunnel Area overlay while a Tunnel is selected.

use bevy::prelude::*;
use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::nav::TileRect;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, Rally, SpawnAt};
use space_crystals::camera::CameraView;
use space_crystals::command_ui::feedback::TargetFeedback;
use space_crystals::command_ui::indicators::{Indicator, MOVE_COLOR, RallyIndicator, rally_indicators};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand, Planned, PointerDisplay};
use space_crystals::fog::Sight;
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::input::ControlGroup;
use space_crystals::placement::TunnelAreaOverlay;

use super::*;
use KeyCode::{ControlLeft, Digit1, Escape, KeyE, KeyQ, KeyW, KeyX, KeyZ};

const P1: PlayerId = PlayerId(1);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;

/// The objects of [`syndicate_game`], by id, and where they are drawn (the map is flat).
const TUNNEL: u32 = 1;
const HIDEOUT: u32 = 2;
const OTHER_HIDEOUT: u32 = 3;
const GUARDS: [u32; 2] = [4, 5];
const TUNNEL_AT: Vec2 = Vec2::new(1152.0, 1088.0); // tiles (16, 15) to (19, 18), Side A north
const HIDEOUT_AT: Vec2 = Vec2::new(1344.0, 1152.0); // tiles (20, 17) to (21, 18)
const OTHER_HIDEOUT_AT: Vec2 = Vec2::new(960.0, 896.0); // tiles (14, 13) to (15, 14)
/// North of the Tunnel's Side A, where a spawned Agent walks in from.
const NORTH: Vec2 = Vec2::new(1122.0, 860.0);
/// Bare ground in the world view, away from everything.
const GROUND: Vec2 = Vec2::new(2000.0, 650.0);

/// A game of player 1, the Syndicate, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: their
/// Tunnel ([`TUNNEL`]), two Hideouts in its Tier 1 area, tiles (14, 13) to (21, 20), and two Guards north of it.
fn syndicate_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(16, 15))
        .spawn(ObjectKind::Hideout, Some(P1), tile(20, 17))
        .spawn(ObjectKind::Hideout, Some(P1), tile(14, 13))
        .spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(1250.0, 800.0)))
        .spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(1320.0, 800.0)));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    app
}

/// The buttons the HUD shows: slot, button, look and count.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, Option<u32>)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = (query.iter(app.world()))
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.count))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The counts drawn on the buttons, in slot order.
fn counts_drawn(app: &mut App) -> Vec<(CommandSlot, String)> {
    let mut query = app.world_mut().query::<(&Name, &Text, &ChildOf)>();
    let mut drawn: Vec<_> = (query.iter(app.world()))
        .filter(|(name, ..)| name.as_str() == "Command button count")
        .map(|(_, text, child_of)| {
            let button = app
                .world()
                .get::<CommandButton>(child_of.parent())
                .expect("on a button");
            (button.slot, text.0.clone())
        })
        .collect();
    drawn.sort();
    drawn
}

/// The feedback message that shows.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// The ids of the units inside player 1's network, in id order.
fn inside(app: &App) -> Vec<u32> {
    let sim = app.world().resource::<SimRes>();
    let mut ids: Vec<u32> = sim.network(P1).map(|held| held.object.id.0).collect();
    ids.sort_unstable();
    ids
}

/// Whether object `id` is on the map.
fn on_map(app: &App, id: u32) -> bool {
    app.world().resource::<SimRes>().object(ObjectId(id)).is_some()
}

/// `units` walk into the Tunnel (an Enter order).
fn enter(app: &mut App, units: &[u32]) {
    let enter = Command::Order {
        units: units.iter().map(|&id| ObjectId(id)).collect(),
        kind: OrderKind::Enter(ObjectId(TUNNEL)),
        queue: false,
    };
    submit(app, P1, enter);
}

/// Runs frames until `done`, `frames` at most.
fn until(app: &mut App, frames: usize, done: impl Fn(&App) -> bool) {
    for _ in 0..frames {
        if done(app) {
            return;
        }
        app.update();
    }
    assert!(done(app), "not within {frames} frames");
}

/// The rally point of producer `id`.
fn rally(app: &App, id: u32) -> Option<Rally> {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id))?.production.as_ref()?.rally
}

/// The rally markers shown.
fn rally_markers(app: &App) -> Vec<RallyIndicator> {
    let sim = app.world().resource::<SimRes>();
    let states = app.world().resource::<ControlStates>();
    rally_indicators(sim, &Sight::new(sim, local_player(app), false), states)
}

/// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257): Q Upgrade Tunnel, greyed while
/// the bank can't pay it (slice M4-G: `tunnel_upgrade`), W Expand Tunnel, which opens the ExpandMenu (slice M4-F:
/// `expansions`), E Eject, X Cancel, greyed while there is no operation to cancel. A greyed button says why. E opens the EjectMenu, whose tiles
/// are greyed while the network has none of their type, each saying so; Z goes back, as Escape and a right-click do.
/// While the Tunnel is selected its area shows.
#[test]
fn the_tunnel_has_its_letters_and_e_opens_the_eject_menu() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelOperation, Eject, EjectMenu, ExpandMenu, UpgradeTunnel};
    let mut app = syndicate_game();
    let overlay = |app: &App| {
        let overlay = app.world().resource::<TunnelAreaOverlay>();
        (
            overlay.visible,
            overlay.shown.iter().map(|known| known.area).collect::<Vec<_>>(),
        )
    };
    assert_eq!(overlay(&app), (false, vec![]));
    click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(selected(&app), [TUNNEL]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(UpgradeTunnel), Disabled, None),
            (slot(0, 1), c(ExpandMenu), Common, None),
            (slot(0, 2), c(EjectMenu), Common, None),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelOperation), Disabled, None),
        ]
    );
    let area = TileRect::new(TilePos::new(14, 13), 8, 8);
    assert_eq!(overlay(&app), (true, vec![area]));
    for (key, says) in [(KeyQ, "Not enough Supplies"), (KeyX, "Nothing to cancel")] {
        tap(&mut app, &[], key);
        assert_eq!(feedback(&app), Some(says), "{key:?}");
        assert_eq!(interface(&app), InterfaceState::DefaultState);
    }
    assert!(orders(&app).is_empty());

    tap(&mut app, &[], KeyE);
    assert_eq!(interface(&app), InterfaceState::EjectMenu);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Eject(AGENT)), Disabled, Some(0)),
            (slot(0, 1), c(Eject(GUARD)), Disabled, Some(0)),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    assert_eq!(
        counts_drawn(&mut app),
        [(slot(0, 0), "0".into()), (slot(0, 1), "0".into())]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some("None in the Tunnel Network"));
    assert_eq!(interface(&app), InterfaceState::EjectMenu);
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    // Escape and a right-click go back too.
    tap(&mut app, &[], KeyE);
    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyE);
    right_click(&mut app, GROUND, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(selected(&app), [TUNNEL]);

    click(&mut app, GROUND, &[]);
    assert!(selected(&app).is_empty());
    assert!(!overlay(&app).0, "the overlay hides with the Tunnel's selection");
}

/// `factions/syndicate/tunnels.md > EjectMenu`: each tile counts the units of its type in the network, the Agents in
/// Q and the Guards in W (design change 0257); a type whose every unit only just walked in is greyed for the transit
/// delay, and its press refused (design change 0288's Verification: "Eject a Guard right after it entered: its tile is
/// grey for 2 s"; point 1: "its tile is greyed and the click is refused; nothing is queued"); Q ejects the Agent inside
/// the longest, the count dropping at the press, and the menu stays open (plan M4-J2: "Q ejects the oldest Agent").
#[test]
fn the_eject_menu_counts_the_network_and_q_ejects_the_oldest_agent() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::Eject;
    let mut app = syndicate_game();
    let oldest = spawn(&mut app, AGENT, Some(P1), NORTH);
    enter(&mut app, &[oldest]);
    until(&mut app, 200, |app| inside(app) == [oldest]);
    updates(&mut app, 40);
    click(&mut app, TUNNEL_AT, &[]);
    tap(&mut app, &[], KeyE);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Eject(AGENT)), Common, Some(1)),
            (slot(0, 1), c(Eject(GUARD)), Disabled, Some(0)),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );

    // Another Agent and a Guard walk in: the Guard's tile counts it and is greyed while it is in transit, from the tick
    // it walked in until its transit delay is over, 2 s later; W says why and queues nothing.
    let newer = spawn(&mut app, AGENT, Some(P1), NORTH);
    enter(&mut app, &[newer, GUARDS[0]]);
    until(&mut app, 300, |app| inside(app).len() == 3);
    let walked_in = {
        let sim = app.world().resource::<SimRes>();
        let held = sim.network(P1).find(|held| held.object.id == ObjectId(GUARDS[0]));
        held.expect("inside").since
    };
    app.update();
    let (agents, guards) = (buttons(&mut app)[0], buttons(&mut app)[1]);
    assert_eq!((agents.2, agents.3), (Common, Some(2)), "the oldest Agent may leave");
    assert_eq!((guards.2, guards.3), (Disabled, Some(1)));
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Still in transit"));
    let tick = |app: &App| app.world().resource::<SimRes>().tick();
    let queued = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        let tunnel = sim.object(ObjectId(TUNNEL)).and_then(|o| o.tunnel.as_ref());
        tunnel.expect("a Tunnel").eject.len()
    };
    while tick(&app) < walked_in + TRANSIT_DELAY_TICKS {
        assert_eq!(buttons(&mut app)[1].2, Disabled, "tick {}", tick(&app));
        assert_eq!((queued(&app), inside(&app).len()), (0, 3), "nothing is queued");
        app.update();
    }
    assert_eq!(buttons(&mut app)[1].2, Common, "2 s after it walked in");

    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::EjectMenu);
    assert_eq!(buttons(&mut app)[0].3, Some(1), "the count drops at the press");
    assert_eq!(
        counts_drawn(&mut app),
        [(slot(0, 0), "1".into()), (slot(0, 1), "1".into())]
    );
    until(&mut app, 100, |app| on_map(app, oldest));
    assert_eq!(inside(&app), [GUARDS[0], newer]);
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// `factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: Q Build Agent, W Build Guard (greyed
/// "Requires an Optics Lab" by the gate while the player owns none, slice M4-K), E Build Grenadier (greyed "Requires a
/// Chemical Plant", design change 0260), X Cancel Production and C Set Rally Point. Design change 0252's verification: "Rally a Hideout on another Hideout. The
/// produced Guard comes out of the parent Tunnel and stands over the other Hideout" (an Agent here: the Guard needs an
/// Optics Lab), the rally marker over the other Hideout; "Rally on a Tunnel: the Guard stays inside".
#[test]
fn the_hideout_has_its_letters_and_rallies_on_a_hideout_or_a_tunnel() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{CancelProduction, Planned as P, SetRallyPoint, Train};
    let mut app = syndicate_game();
    click(&mut app, HIDEOUT_AT, &[]);
    assert_eq!(selected(&app), [HIDEOUT]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Train(AGENT)), Common, None),
            (slot(0, 1), c(Train(GUARD)), Disabled, None),
            (slot(0, 2), c(P(Planned::Grenadier)), Unavailable, None),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelProduction), Disabled, None),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), Common, None),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(feedback(&app), Some("Requires an Optics Lab"));
    tap(&mut app, &[], KeyE);
    assert_eq!(feedback(&app), Some("Requires a Chemical Plant"));

    // A right-click on the other Hideout rallies there: the Move pointer, then the green marker around it.
    assert_eq!(pointer_over(&mut app, OTHER_HIDEOUT_AT), Some(PointerDisplay::Move));
    right_click(&mut app, OTHER_HIDEOUT_AT, &[]);
    assert!(matches!(rally(&app, HIDEOUT), Some(Rally::Object { id, .. }) if id == ObjectId(OTHER_HIDEOUT)));
    let marker = Indicator::Object {
        id: ObjectId(OTHER_HIDEOUT),
        color: MOVE_COLOR,
    };
    let shown = rally_markers(&app);
    assert_eq!(
        shown,
        [RallyIndicator {
            producer: ObjectId(HIDEOUT),
            marker
        }]
    );
    tap(&mut app, &[], KeyQ);
    let agent = GUARDS[1] + 1;
    until(&mut app, 300, |app| on_map(app, agent));
    assert!(inside(&app).is_empty());
    until(&mut app, 400, |app| pos(app, agent) == OTHER_HIDEOUT_AT);

    // On the Tunnel, the next one stays inside.
    right_click(&mut app, TUNNEL_AT, &[]);
    assert!(matches!(rally(&app, HIDEOUT), Some(Rally::Object { id, .. }) if id == ObjectId(TUNNEL)));
    assert!(
        matches!(rally_markers(&app)[..], [RallyIndicator { marker: Indicator::Object { id, color }, .. }]
        if id == ObjectId(TUNNEL) && color == MOVE_COLOR)
    );
    tap(&mut app, &[], KeyQ);
    until(&mut app, 300, |app| inside(app) == [agent + 1]);
    updates(&mut app, 30);
    assert!(!on_map(&app, agent + 1));
}

/// Held units stay in their control groups (design change 0249: "a recall selects only the members on the map").
/// A group whose every unit is inside recalls as an empty group does (`ControlState::recall_group`, placeholder rule):
/// nothing changes, the selection, its ObjectInterfaceState and, on a double tap, the camera. Ejected, a unit is
/// recalled again.
#[test]
fn a_group_all_inside_the_network_recalls_nothing() {
    let mut app = syndicate_game();
    select_units(&mut app, &GUARDS);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    enter(&mut app, &GUARDS);
    until(&mut app, 300, |app| inside(app) == GUARDS);
    app.update();
    assert!(selected(&app).is_empty());
    let group = ControlGroup::new(1).unwrap();
    let members = |app: &App| {
        let states = app.world().resource::<ControlStates>();
        let ids = states.of(local_player(app)).groups.get(group).iter();
        ids.map(|id| id.0).collect::<Vec<u32>>()
    };
    assert_eq!(members(&app), GUARDS);

    click(&mut app, TUNNEL_AT, &[]);
    tap(&mut app, &[], KeyE);
    let camera = app.world().resource::<CameraView>().center;
    tap(&mut app, &[], Digit1);
    assert_eq!(
        (selected(&app), interface(&app)),
        (vec![TUNNEL], InterfaceState::EjectMenu)
    );
    tap(&mut app, &[], Digit1);
    assert_eq!(
        app.world().resource::<CameraView>().center,
        camera,
        "a double tap doesn't move the camera"
    );
    assert_eq!(
        (selected(&app), interface(&app)),
        (vec![TUNNEL], InterfaceState::EjectMenu)
    );

    // A Guard comes out: the recall selects it.
    updates(&mut app, 40);
    tap(&mut app, &[], KeyW);
    until(&mut app, 100, |app| inside(app).len() == 1);
    pause(&mut app);
    tap(&mut app, &[], Digit1);
    let out: Vec<u32> = GUARDS.into_iter().filter(|&id| on_map(&app, id)).collect();
    assert_eq!(selected(&app), out);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(members(&app), GUARDS);
}
