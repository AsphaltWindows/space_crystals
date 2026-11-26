//! The Tunnel Network in the client (slice M4-C, `sc_sim::network`): a right-click on an own Tunnel is Enter, and a
//! unit that goes in "leaves the selection by entering a building, transport or tunnel"
//! (`systems/controls/selection.md > Selection`) and "stays in its control groups (a recall selects only the
//! members on the map)" (`factions/syndicate/tunnels.md > Tunnel Network > Walking in`, design change 0249). The
//! Tunnel's panel and its EjectMenu are slice M4-J2's: the Eject here is submitted as a command.

use bevy::prelude::{App, KeyCode, Vec2};
use sc_sim::{Command, Spawn, TunnelCommand};
use space_crystals::input::ControlGroup;

use super::*;
use KeyCode::{ControlLeft, Digit1};

/// Player 1's Tunnel of [`with_tunnel`]: on tiles (16, 15) to (19, 18), its centre at (1152, 1088) and its Side A
/// facing north.
const TUNNEL: u32 = 15;

/// [`scenario`] with a Tunnel of player 1's south of its Guards.
fn with_tunnel() -> Scenario {
    let mut scenario = scenario();
    let tunnel = Spawn::new(ObjectKind::Tunnel, Some(SlotId(1)), SpawnAt::Tile(TilePos::new(16, 15)));
    scenario.map.spawns.push(tunnel);
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// The local player's control group `n`.
fn control_group(app: &App, n: u8) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    let group = ControlGroup::new(n).unwrap();
    let ids = states.of(local_player(app)).groups.get(group).iter();
    ids.map(|id| id.0).collect()
}

/// The ids of the units inside player 1's network.
fn inside(app: &App) -> Vec<u32> {
    let sim = app.world().resource::<SimRes>();
    sim.network(PlayerId(1)).map(|held| held.object.id.0).collect()
}

#[test]
fn units_that_enter_a_tunnel_leave_the_selection_and_stay_in_their_control_groups() {
    let mut app = game_of(with_tunnel());
    // The Guards and a Peacekeeper, in control group 1.
    drag(&mut app, Vec2::new(1000.0, 760.0), Vec2::new(1400.0, 840.0));
    assert_eq!(selected(&app), [2, 3, 4]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    // A right-click on the own Tunnel: Enter, for the units its tier lets in (all infantry here).
    right_click(&mut app, Vec2::new(1152.0, 1088.0), &[]);
    let enter = OrderKind::Enter(ObjectId(TUNNEL));
    assert_eq!(orders(&app).last(), Some(&given(&[2, 3, 4], enter, false)));
    // A Guard stays out: a Stop takes its Enter away.
    let stop = Command::Order {
        units: vec![ObjectId(4)],
        kind: OrderKind::Stop,
        queue: false,
    };
    submit(&mut app, PlayerId(1), stop);
    updates(&mut app, 2);
    for _ in 0..200 {
        if inside(&app).len() == 2 {
            break;
        }
        app.update();
    }
    assert_eq!(inside(&app), [2, 3]);
    app.update();
    // Those inside left the selection; the control group keeps all three.
    assert_eq!(selected(&app), [4]);
    assert_eq!(control_group(&app, 1), [2, 3, 4]);
    // A recall selects only the member on the map.
    click(&mut app, Vec2::new(1500.0, 1200.0), &[]);
    assert!(selected(&app).is_empty());
    pause(&mut app);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [4]);
    // Ejected, a unit is in its group's recall again, with the id it had.
    updates(&mut app, 40);
    let eject = Command::Tunnel {
        tunnel: ObjectId(TUNNEL),
        command: TunnelCommand::Eject {
            kind: ObjectKind::Guard,
        },
    };
    submit(&mut app, PlayerId(1), eject);
    updates(&mut app, 30);
    assert_eq!(inside(&app), [2]);
    assert_eq!(control_group(&app, 1), [2, 3, 4]);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [3, 4]);
}
