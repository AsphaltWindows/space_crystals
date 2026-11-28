//! The Prism's panel as plain data, read from a game (`factions/syndicate/units/prism.md > Panel`, Q0168: "Q Move, R
//! Thrust, X Stop"): its commands, R's targeting and its greying once every charge is spent.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, OrderKind, PlayerId, SpawnAt};

use super::*;
use crate::command_ui::feedback::{NO_CHARGES_TEXT, unavailable_text};

const PRISM: ObjectKind = ObjectKind::Prism;
const P1: PlayerId = PlayerId(1);

/// Player 1's Prism (1) on a flat map; tick 1 has run.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(PRISM, Some(P1), SpawnAt::Point(Vec2::new(600.0, 600.0)))
        .start();
    t.run_until(1);
    t
}

fn panel_of(t: &TestSim) -> Panel {
    let selection = Selection::new([(id(1), PRISM)]);
    let conditions = Conditions::of(t.sim(), P1, &selection);
    panel(&selection, InterfaceState::DefaultState, &conditions)
}

/// Q Move, R Thrust and X Stop, nothing else (no A while it has no weapon); R awaits a target, and queues.
#[test]
fn the_panel_is_move_thrust_and_stop() {
    let t = field();
    let shown = panel_of(&t);
    let commands: Vec<_> = CommandSlot::all()
        .filter_map(|slot| match shown.get(slot)?.button {
            PanelButton::Command(command) => Some((slot, command)),
            _ => None,
        })
        .collect();
    assert_eq!(
        commands,
        [
            (slot(0, 0), PanelCommand::Move),
            (slot(0, 3), PanelCommand::Thrust),
            (slot(2, 1), PanelCommand::Stop)
        ]
    );
    assert_eq!(press(&shown, slot(0, 3)), SlotPress::AwaitTarget(PanelCommand::Thrust));
    assert!(PanelCommand::Thrust.needs_target() && PanelCommand::Thrust.queues());
    assert_eq!(shown.name(slot(0, 3)).as_deref(), Some("Thrust"));
    assert_eq!(
        shown.get(slot(0, 3)).map(|button| button.look),
        Some(ButtonLook::Common)
    );
}

/// A left-click on a point while Thrust awaits gives its order; none on an object.
#[test]
fn a_left_click_on_a_point_gives_the_thrust_order() {
    let point = Vec2::new(900.0, 700.0);
    assert_eq!(
        crate::command_ui::resolution::target_order(PanelCommand::Thrust, &crate::command_ui::MapTarget::ground(point)),
        Some(OrderKind::Thrust(point))
    );
}

/// Once every charge is spent R is greyed, "Every charge is spent" (`sc_sim::Reject::NoCharges`), and its press does
/// nothing.
#[test]
fn r_is_greyed_once_every_charge_is_spent() {
    let mut t = field();
    let charges = sc_sim::thrust::def_of(PRISM).expect("it thrusts").charges;
    assert!(charges > 0);
    for n in 0..charges {
        let tick = 2 + 40 * n;
        let kind = OrderKind::Thrust(Vec2::new(900.0 + 10.0 * n as f32, 700.0));
        t.at(
            tick,
            P1,
            Command::Order {
                units: vec![id(1)],
                kind,
                queue: false,
            },
        );
        t.run_until(tick);
    }
    let spent = panel_of(&t);
    assert_eq!(sc_sim::thrust::charges_left(t.sim().object(id(1)).unwrap()), 0);
    assert_eq!(
        spent.get(slot(0, 3)).map(|button| button.look),
        Some(ButtonLook::Disabled)
    );
    assert_eq!(
        spent.get(slot(0, 3)).and_then(|shown| shown.why).map(unavailable_text),
        Some(NO_CHARGES_TEXT)
    );
    assert_eq!(press(&spent, slot(0, 3)), SlotPress::Nothing);
}
