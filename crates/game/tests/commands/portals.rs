//! The Colonists' portals from the command panel (slice M3-D, plan decision T23): a selected Robo Portal or Bio
//! Portal shows the production panel built from its `ProductionDef`, as the Barracks does (`production`, `rally`):
//! Q trains its unit (the Drone, the Sabre Legionnaire), X cancels the last one queued, and C or a right-click sets
//! its rally point. A Train the command gate refuses for the Beacon Capacity shows disabled, and available again
//! once there is room (design change 0112).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, DebugCommand, Faction, ObjectId, ObjectKind, PlayerId, ProductionCommand, Rally, SpawnAt};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;

use super::*;
use KeyCode::{KeyC, KeyQ, KeyX};

const P1: PlayerId = PlayerId(1);

/// Player 1's portals in [`portal_game`], by id, and the centres of their footprints, where they are drawn (the map
/// is flat).
const ROBO_PORTAL: u32 = 1;
const BIO_PORTAL: u32 = 2;
const ROBO_PORTAL_AT: Vec2 = Vec2::new(1152.0, 896.0); // tiles (16, 12) to (19, 15)
const BIO_PORTAL_AT: Vec2 = Vec2::new(1632.0, 864.0); // tiles (24, 12) to (26, 14)

/// Bare ground in the world view, away from both portals.
const GROUND: Vec2 = Vec2::new(2000.0, 650.0);

/// A game of player 1, the Colonists, with a Robo Portal and a Bio Portal in the world view (5 Beacon Capacity
/// each), `drones` Drones off screen, which take it up, and `crystals` Space Crystals, set up as [`game`] does.
fn portal_game(drones: usize, crystals: u32) -> App {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(ObjectKind::BioPortal, Some(P1), SpawnAt::Tile(TilePos::new(24, 12)));
    for i in 0..drones {
        let at = Vec2::new(100.0 + 40.0 * i as f32, 2900.0);
        builder = builder.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(at));
    }
    let mut app = game_of(builder.build());
    set_crystals(&mut app, crystals);
    app
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

/// `factions/colonists/structures.md > Robo Portal > Produces` ("Drone: 50 Space Crystals") and `> Bio Portal >
/// Produces` ("Sabre Legionnaire: 100 Space Crystals"), in the production panel (plan decision T23: Train in Q, X
/// Cancel, C Rally): Q queues the unit and pays, X takes it off the queue and refunds it, C then a click sets the
/// rally point, and so does a right-click (`> Production Building Default Right-Click`).
#[test]
fn a_portal_trains_in_q_cancels_in_x_and_rallies_in_c() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let cases = [
        (ROBO_PORTAL, ROBO_PORTAL_AT, ObjectKind::Drone, "Train DR", 50),
        (BIO_PORTAL, BIO_PORTAL_AT, ObjectKind::SabreLegionnaire, "Train SL", 100),
    ];
    for (portal, at, kind, label, cost) in cases {
        let mut app = portal_game(0, 1000);
        click(&mut app, at, &[]);
        assert_eq!(selected(&app), [portal]);
        let c = PanelButton::Command;
        assert_eq!(
            buttons(&mut app),
            [
                (slot(0, 0), c(Train(kind)), Common, label),
                (CommandSlot::CANCEL_PRODUCTION, c(CancelProduction), Disabled, "Cancel"),
                (CommandSlot::RALLY_POINT, c(SetRallyPoint), Common, "Rally"),
            ]
        );
        tap(&mut app, &[], KeyQ);
        assert_eq!(productions(&app), [(P1, vec![portal], ProductionCommand::Train(kind))]);
        assert_eq!((production_of(&app, portal).queue.len(), bank(&app)), (1, 1000 - cost));
        tap(&mut app, &[], KeyX);
        assert_eq!(
            productions(&app)[1],
            (P1, vec![portal], ProductionCommand::CancelProduction)
        );
        assert_eq!((production_of(&app, portal).queue.len(), bank(&app)), (0, 1000));

        tap(&mut app, &[], KeyC);
        click(&mut app, GROUND, &[]);
        let rally = production_of(&app, portal).rally;
        assert!(matches!(rally, Some(Rally::Point(p)) if near(p, GROUND)), "{rally:?}");
        let elsewhere = GROUND + Vec2::new(-300.0, 50.0);
        right_click(&mut app, elsewhere, &[]);
        let rally = production_of(&app, portal).rally;
        assert!(
            matches!(rally, Some(Rally::Point(p)) if near(p, elsewhere)),
            "{rally:?}"
        );
        assert_eq!(orders(&app), [], "no unit orders");
    }
}

/// Design change 0112: at its Beacon Capacity (two portals: 10, and ten Drones), a portal's Train is disabled and
/// sends nothing; once a Drone is gone there is room, and Q trains again.
#[test]
fn a_portal_at_its_beacon_capacity_trains_again_once_there_is_room() {
    let mut app = portal_game(10, 1000);
    click(&mut app, ROBO_PORTAL_AT, &[]);
    let q = |app: &mut App| buttons(app).into_iter().find(|b| b.0 == slot(0, 0)).map(|b| b.2);
    assert_eq!(q(&mut app), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(productions(&app), []);
    let drone = ObjectId(3);
    submit(&mut app, P1, Command::Debug(DebugCommand::Remove { id: drone }));
    updates(&mut app, 2);
    assert_eq!(q(&mut app), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyQ);
    let train = ProductionCommand::Train(ObjectKind::Drone);
    assert_eq!(productions(&app), [(P1, vec![ROBO_PORTAL], train)]);
    assert_eq!(production_of(&app, ROBO_PORTAL).queue.len(), 1);
}
