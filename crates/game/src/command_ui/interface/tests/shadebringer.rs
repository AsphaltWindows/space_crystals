//! (M6-SB) The Shadebringer's panel as plain data, read from a game (`factions/syndicate/units/shadebringer.md >
//! ObjectInterfaceState`: "Move-type commands like a combat unit without an attack, plus the two toggles"; `>
//! Placeholder stats`: "Toggles follow the mode-switch rule (keep orders, not queueable). Hotkeys: **R** Sludge, **F**
//! Cover"): what its key shows, and what the gate says greys it. In a game: `crates/game/tests/commands/sludge.rs`.

use bevy::math::Vec2;
use sc_sim::defs::PatchKind;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, PlayerId, SpawnAt};

use super::*;
use crate::command_ui::feedback::{NOT_YET_TEXT, TANK_EMPTY_TEXT, unavailable_text};

const SHADEBRINGER: ObjectKind = ObjectKind::Shadebringer;
const P1: PlayerId = PlayerId(1);
/// R, switching the Sludge on, and off.
const ON: PanelCommand = PanelCommand::Toggle {
    lays: PatchKind::Sludge,
    on: true,
};
const OFF: PanelCommand = PanelCommand::Toggle {
    lays: PatchKind::Sludge,
    on: false,
};

/// Player 1's two Shadebringers (1, 2) and a Guard (3) on a flat map; tick 1 has run.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(SHADEBRINGER, Some(P1), SpawnAt::Point(Vec2::new(600.0, 600.0)))
        .spawn(SHADEBRINGER, Some(P1), SpawnAt::Point(Vec2::new(600.0, 800.0)))
        .spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(900.0, 600.0)))
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

/// The Shadebringer's DefaultState is the BasicCombatUnit's without its attack commands (no A while no selected unit
/// can attack, no D), W Reverse its own (a WheeledVehicle reverses), with R its Sludge's toggle, "Sludge", and F
/// Cover's slot, greyed until slice M6-SC, "Not available yet". R switches the Sludge on at once, an ability that
/// keeps the orders and that Shift changes nothing for, to every Shadebringer of the ActiveGroup, the Shadebringer
/// being Groupable; beside a Guard, the Guard's A shows on the Shadebringers' panel too, and R is theirs alone, the
/// Guard's panel showing none.
#[test]
fn the_shadebringers_r_switches_its_sludge_on_and_f_holds_its_covers_slot() {
    use ButtonLook::{Common, Group, Unavailable};
    use PanelCommand::{Attack, HoldPosition, Move, Patrol, Planned as P, Reverse, Stop};
    let t = field();
    let one = Selection::new([(id(1), SHADEBRINGER)]);
    let start = panel_of(&t, &one);
    let c = PanelButton::Command;
    let (r, f) = (slot(0, 3), slot(1, 3));
    assert_eq!(
        shown(&start),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Common),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(ON), Common),
            (slot(1, 1), c(Patrol), Common),
            (f, c(P(Planned::Cover)), Unavailable),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!([r, f].map(|at| start.get(at).unwrap().label()), ["Sludge", "Cover"]);
    assert_eq!(
        [r, f].map(|at| start.name(at)),
        [Some("Sludge On".into()), Some("Cover".into())]
    );
    assert_eq!(why(&start, f), Some(NOT_YET_TEXT));
    assert_eq!(press(&start, f), SlotPress::Nothing);
    assert_eq!(
        press(&start, r),
        SlotPress::UseAbility(ON, Ability::Sludge { on: true })
    );
    assert!(!ON.queues() && !ON.needs_target() && !ON.instance_only());
    let two = Selection::new([(id(1), SHADEBRINGER), (id(2), SHADEBRINGER)]);
    assert_eq!(two.groups().len(), 1, "Groupable");
    assert_eq!(recipients(&two, ON), [id(1), id(2)]);
    let mut mixed = Selection::new([(id(1), SHADEBRINGER), (id(2), SHADEBRINGER), (id(3), GUARD)]);
    if mixed.active().is_some_and(|active| active.kind == GUARD) {
        assert_eq!(panel_of(&t, &mixed).get(r), None, "the Guard's panel");
        mixed.cycle(true);
    }
    assert_eq!(mixed.active().map(|active| active.kind), Some(SHADEBRINGER));
    let shown_mixed = panel_of(&t, &mixed);
    assert_eq!(
        shown_mixed.get(slot(1, 0)).map(|shown| (shown.button, shown.look)),
        Some((c(Attack), Common)),
        "the Guard's"
    );
    assert_eq!(shown_mixed.get(r).map(|shown| shown.look), Some(Group));
    assert_eq!(recipients(&mixed, ON), [id(1), id(2)]);
    mixed.cycle(true);
    assert_eq!(mixed.active().map(|active| active.kind), Some(GUARD));
    assert_eq!(panel_of(&t, &mixed).get(r), None, "the Guard's panel");
    assert_eq!(
        InterfaceTemplate::of(SHADEBRINGER),
        Some(InterfaceTemplate::BasicCombatUnit)
    );
}

/// R's key shows the way a press switches the Sludge: on while one of its recipients has it off ("the panel sends on
/// unless every unit it goes to has it on already", `sc_sim::Ability::Sludge`), and off, "Sl.Off", "Sludge Off", once
/// every one has it on, never greyed. Once the tank of the one switched on is empty, 10 s on, its toggle is off by
/// itself, and its R is greyed, "Tank empty" (`sc_sim::Reject::TankEmpty`: "a toggle can be switched on with **any
/// charge** left in its tank"); beside the other one, whose tank is full, R switches on, the other taking it.
#[test]
fn r_switches_off_once_every_shadebringer_has_its_sludge_on_and_is_greyed_with_the_tank_empty() {
    use ButtonLook::{Common, Disabled};
    let mut t = field();
    let switch_on = Command::Ability {
        units: vec![id(1)],
        ability: Ability::Sludge { on: true },
    };
    t.at(2, P1, switch_on);
    t.run_until(2);
    let r = slot(0, 3);
    let one = Selection::new([(id(1), SHADEBRINGER)]);
    let two = Selection::new([(id(1), SHADEBRINGER), (id(2), SHADEBRINGER)]);
    let on = panel_of(&t, &one);
    assert_eq!(
        on.get(r).map(|shown| (shown.button, shown.look)),
        Some((PanelButton::Command(OFF), Common))
    );
    assert_eq!(
        (on.get(r).unwrap().label(), on.name(r).as_deref()),
        ("Sl.Off", Some("Sludge Off"))
    );
    assert_eq!(press(&on, r), SlotPress::UseAbility(OFF, Ability::Sludge { on: false }));
    let half = panel_of(&t, &two);
    assert_eq!(
        half.get(r).map(|shown| shown.button),
        Some(PanelButton::Command(ON)),
        "one is off"
    );
    assert_eq!(press(&half, r), SlotPress::UseAbility(ON, Ability::Sludge { on: true }));
    t.run_until(2 + 160);
    let tanks = t.sim().object(id(1)).and_then(|o| o.tanks).expect("its tanks");
    assert!(!tanks.sludge.on, "off by itself");
    let empty = panel_of(&t, &one);
    assert_eq!(
        empty.get(r).map(|shown| (shown.button, shown.look)),
        Some((PanelButton::Command(ON), Disabled))
    );
    assert_eq!(why(&empty, r), Some(TANK_EMPTY_TEXT));
    assert_eq!(press(&empty, r), SlotPress::Nothing);
    let other = panel_of(&t, &two);
    assert_eq!(
        other.get(r).map(|shown| shown.look),
        Some(Common),
        "the other one's tank is full"
    );
}
