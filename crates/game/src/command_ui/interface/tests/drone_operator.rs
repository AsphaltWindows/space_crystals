//! (M6-GD) The Drone Operator's panel and its drone's as plain data, read from a game (`factions/gdo/units/
//! drone_operator.md > Placeholder stats`: "**Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s)",
//! "**Launch Drone** (hotkey **R**)"; plan §GD: "the operator's panel: R Launch, and F Build Drone with a progress bar;
//! the drone's panel: Move, Attack, Stop"): what the gate says greys them. In a game:
//! `crates/game/tests/commands/drone_operator_ui.rs`.

use bevy::math::Vec2;
use sc_sim::construction::site::BuildPrice;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, DebugCommand, Faction, PlayerId, SpawnAt};

use super::*;
use crate::command_ui::feedback::{
    DRONE_OUT_TEXT, PURCHASE_UNDER_WAY_TEXT, STOCK_EMPTY_TEXT, STOCK_FULL_TEXT, unavailable_text,
};

const OPERATOR: ObjectKind = ObjectKind::DroneOperator;
const DRONE: ObjectKind = ObjectKind::OperatorDrone;
const P1: PlayerId = PlayerId(1);
const LAUNCH: PanelCommand = PanelCommand::Launch;
const BUY: PanelCommand = PanelCommand::Buy(DRONE);

/// Player 1's Drone Operator (1) and Peacekeeper (2) on a flat map, and 1000 Space Crystals; tick 1 has run.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Gdo)
        .spawn(OPERATOR, Some(P1), SpawnAt::Point(Vec2::new(600.0, 600.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(600.0, 700.0)))
        .start();
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    t.run_until(1);
    t
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
}

/// The operator's Launch Drone, sent to the sim.
fn launch() -> Command {
    Command::Order {
        units: vec![id(1)],
        kind: OrderKind::Launch,
        queue: false,
    }
}

/// `selection`'s DefaultState panel in `t`.
fn panel_of(t: &TestSim, selection: &Selection) -> Panel {
    let conditions = Conditions::of(t.sim(), P1, selection);
    panel(selection, InterfaceState::DefaultState, &conditions)
}

/// What the greyed button of `slot` says.
fn why(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot).and_then(|shown| shown.why).map(unavailable_text)
}

/// The operator's DefaultState is the BasicCombatUnit's without its attack commands (no A while no selected unit can
/// attack, no D), its W unavailable (infantry), with R Launch Drone, "Launch", and F Build Drone, "DRN", the drone it
/// makes, priced at its 50 Space Crystals. R launches at once, an order that Shift queues; F buys at once, an ability
/// that keeps the orders and that Shift changes nothing for; both go to every operator of the ActiveGroup, the operator
/// being Groupable. With its one drone in stock, F is greyed: the stock is full.
#[test]
fn the_operators_r_launches_its_drone_and_f_builds_the_next() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{HoldPosition, Move, Patrol, Reverse, Stop};
    let t = field();
    let one = Selection::new([(id(1), OPERATOR)]);
    let start = panel_of(&t, &one);
    let c = PanelButton::Command;
    let (r, f) = (slot(0, 3), slot(1, 3));
    assert_eq!(
        shown(&start),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(LAUNCH), Common),
            (slot(1, 1), c(Patrol), Common),
            (f, c(BUY), Disabled),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(why(&start, f), Some(STOCK_FULL_TEXT));
    assert_eq!([r, f].map(|at| start.get(at).unwrap().label()), ["Launch", "DRN"]);
    assert_eq!(
        [r, f].map(|at| start.name(at)),
        [Some("Launch Drone".into()), Some("Build Drone".into())]
    );
    assert_eq!(
        start.get(f).and_then(|shown| shown.price),
        Some(BuildPrice::crystals(50))
    );
    assert_eq!(start.get(f).and_then(|shown| shown.progress), None, "nothing bought");
    assert_eq!(press(&start, r), SlotPress::Issue(LAUNCH, OrderKind::Launch));
    assert!(LAUNCH.queues() && !LAUNCH.needs_target() && !LAUNCH.instance_only());
    assert!(!BUY.queues() && !BUY.needs_target() && !BUY.instance_only());
    assert_eq!(BUY.ability(), Some(Ability::Buy));
    assert_eq!(press(&start, f), SlotPress::Nothing, "greyed");
    let two = Selection::new([(id(1), OPERATOR), (id(3), OPERATOR)]);
    assert_eq!(two.groups().len(), 1, "Groupable");
    assert_eq!(recipients(&two, LAUNCH), [id(1), id(3)]);
    assert_eq!(recipients(&two, BUY), [id(1), id(3)]);
    assert_eq!(
        InterfaceTemplate::of(OPERATOR),
        Some(InterfaceTemplate::BasicCombatUnit)
    );
}

/// With its drone out the operator holds still: its Move and Patrol are greyed, "The drone is out", as the gate would
/// refuse them, and so are R (one drone at a time) and F ("Not while a drone is out"); Hold Position and Stop are not.
/// Beside a Peacekeeper, Move goes on: the Peacekeeper takes it. The drone's own panel is Q Move, A Attack and X Stop.
/// Once the drone has ended, F buys one: greyed while it is bought, "Already building a drone", with the purchase's
/// progress on its bar, R greyed meanwhile with none in stock; 160 ticks on the drone is in stock, F greyed again for
/// the full stock, its bar gone, and R launches.
#[test]
fn while_its_drone_is_out_the_operator_holds_still_and_f_waits_for_its_end() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{Attack, HoldPosition, Move, Patrol, Stop};
    let mut t = field();
    next(&mut t, launch());
    let drone = id(3);
    assert_eq!(t.sim().object(drone).map(|o| o.kind), Some(DRONE), "launched");
    let one = Selection::new([(id(1), OPERATOR)]);
    let out = panel_of(&t, &one);
    let look = |panel: &Panel, at| panel.get(at).map(|shown| shown.look);
    let (q, e, r, s, f, x) = (slot(0, 0), slot(0, 2), slot(0, 3), slot(1, 1), slot(1, 3), slot(2, 1));
    for at in [q, s, r, f] {
        assert_eq!(
            (look(&out, at), why(&out, at)),
            (Some(Disabled), Some(DRONE_OUT_TEXT)),
            "{at:?}"
        );
    }
    assert_eq!([look(&out, e), look(&out, x)], [Some(Common); 2]);
    assert_eq!(press(&out, q), SlotPress::Nothing, "Move greyed");
    let with_pk = Selection::new([(id(1), OPERATOR), (id(2), PK)]);
    assert_eq!(look(&panel_of(&t, &with_pk), q), Some(Common), "the Peacekeeper moves");

    let flown = Selection::new([(drone, DRONE)]);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&panel_of(&t, &flown)),
        [
            (q, c(Move), Common),
            (slot(1, 0), c(Attack), Common),
            (x, c(Stop), Common),
        ]
    );
    assert_eq!(InterfaceTemplate::of(DRONE), Some(InterfaceTemplate::OperatorDrone));
    assert!(!offers(DRONE, HoldPosition) && !offers(DRONE, Patrol));

    next(&mut t, Command::Debug(DebugCommand::Remove { id: drone }));
    let ended = panel_of(&t, &one);
    assert_eq!(look(&ended, f), Some(Common), "the drone ended");
    assert_eq!(press(&ended, f), SlotPress::UseAbility(BUY, Ability::Buy));
    assert_eq!(look(&ended, r), Some(Disabled));
    assert_eq!(why(&ended, r), Some(STOCK_EMPTY_TEXT));
    next(
        &mut t,
        Command::Ability {
            units: vec![id(1)],
            ability: Ability::Buy,
        },
    );
    let bought = t.sim().tick();
    t.run(80);
    let buying = panel_of(&t, &one);
    assert_eq!(look(&buying, f), Some(Disabled));
    assert_eq!(why(&buying, f), Some(PURCHASE_UNDER_WAY_TEXT));
    let bar = buying.get(f).and_then(|shown| shown.progress).expect("a bar");
    assert_eq!((bar.total, bar.ticks_done()), (160, 81));
    t.run_until(bought + 159);
    let stocked = panel_of(&t, &one);
    assert_eq!(why(&stocked, f), Some(STOCK_FULL_TEXT));
    assert_eq!(stocked.get(f).and_then(|shown| shown.progress), None);
    assert_eq!(look(&stocked, r), Some(Common));
}
