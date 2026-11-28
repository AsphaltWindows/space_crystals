//! (M6-ST1) The Subterranean Transport's panel as plain data, read from a game
//! (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "Panel: Q Move, X Stop, **R: Burrow /
//! Surface**, **F: Unload All**"): what the gate says greys its F.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, PlayerId, SpawnAt};

use super::*;
use crate::command_ui::feedback::{NOTHING_INSIDE_TEXT, unavailable_text};

const TRANSPORT: ObjectKind = ObjectKind::SubterraneanTransport;
const BURROWED: ObjectKind = ObjectKind::SubterraneanTransportBurrowed;
const P1: PlayerId = PlayerId(1);
const UNLOAD: PanelCommand = PanelCommand::UnloadAll;

/// Player 1's Subterranean Transport (1), Guard (2) and burrowed Transport (3) on a flat map; tick 1 has run.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(TRANSPORT, Some(P1), SpawnAt::Point(Vec2::new(600.0, 600.0)))
        .spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(600.0, 720.0)))
        .spawn(BURROWED, Some(P1), SpawnAt::Point(Vec2::new(1000.0, 600.0)))
        .start();
    t.run_until(1);
    t
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

/// The Transport's DefaultState: Q Move, R Burrow (not available yet: slice M6-ST2's), F Unload All, "Unload", and X
/// Stop; no A while no selected unit can attack (the Transport is unarmed). Empty, its F is greyed, as the gate would
/// refuse it (`Reject::NothingInside`); with a Guard aboard, F unloads at once, an order that Shift queues, to every
/// Transport of the ActiveGroup, the Transport being Groupable. The burrowed type's R is Surface, and its F is greyed,
/// empty as it is (a loaded one's for being underground, "above ground only": `Reject::LacksAbility`).
#[test]
fn the_transports_f_unloads_and_its_r_waits_for_burrowing() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{Move, Planned as P, Stop};
    let mut t = field();
    let one = Selection::new([(id(1), TRANSPORT)]);
    let empty = panel_of(&t, &one);
    let c = PanelButton::Command;
    let (r, f) = (slot(0, 3), slot(1, 3));
    assert_eq!(
        shown(&empty),
        [
            (slot(0, 0), c(Move), Common),
            (r, c(P(Planned::Burrow)), Unavailable),
            (f, c(UNLOAD), Disabled),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(why(&empty, f), Some(NOTHING_INSIDE_TEXT));
    assert_eq!(why(&empty, r), Some("Not available yet"));
    assert_eq!([r, f].map(|at| empty.get(at).unwrap().label()), ["Burrow", "Unload"]);
    assert_eq!(
        [r, f].map(|at| empty.name(at)),
        [Some("Burrow".into()), Some("Unload All".into())]
    );
    assert_eq!(press(&empty, f), SlotPress::Nothing, "greyed");
    assert!(UNLOAD.queues() && !UNLOAD.needs_target() && !UNLOAD.instance_only());
    // A Guard boards: F unloads.
    let board = Command::Order {
        units: vec![id(2)],
        kind: OrderKind::Enter(id(1)),
        queue: false,
    };
    t.at(2, P1, board);
    t.run_until(80);
    assert_eq!(t.sim().passengers(id(1)).len(), 1, "the Guard is aboard");
    let loaded = panel_of(&t, &one);
    assert_eq!(loaded.get(f).map(|s| (s.look, s.why)), Some((Common, None)));
    assert_eq!(press(&loaded, f), SlotPress::Issue(UNLOAD, OrderKind::UnloadAll));
    let two = Selection::new([(id(1), TRANSPORT), (id(4), TRANSPORT)]);
    assert_eq!(two.groups().len(), 1, "Groupable");
    assert_eq!(recipients(&two, UNLOAD), [id(1), id(4)]);
    assert_eq!(InterfaceTemplate::of(TRANSPORT), Some(InterfaceTemplate::Transport));
    // Underground: R is Surface, and F greyed.
    let burrowed = Selection::new([(id(3), BURROWED)]);
    let under = panel_of(&t, &burrowed);
    assert_eq!(
        shown(&under),
        [
            (slot(0, 0), c(Move), Common),
            (r, c(P(Planned::Surface)), Unavailable),
            (f, c(UNLOAD), Disabled),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(why(&under, f), Some(NOTHING_INSIDE_TEXT));
    assert_eq!(under.get(r).unwrap().label(), "Surf.");
    assert_eq!(InterfaceTemplate::of(BURROWED), Some(InterfaceTemplate::Transport));
}
