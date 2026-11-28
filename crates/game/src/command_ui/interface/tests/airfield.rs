//! (M6-GH1) The Airfield's panel and the Hornet's as plain data, read from a game (`factions/gdo/structures/airfield.md >
//! Placeholder stats`: "Panel: **Q** Hornet, **W** Dragonfly (greyed without Propulsion Labs), **X** Cancel Production,
//! **C** Set Rally Point"; `factions/gdo/units/hornet.md > Placeholder stats`: "Panel: BasicCombatUnitInterfaceState
//! plus **F: Return to Base**", and (M6-GL) `factions/gdo/units/dragonfly.md`'s alike): what the gate says greys them.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, DebugCommand, Faction, PlayerId, SpawnAt, TilePos};

use super::*;
use crate::command_ui::feedback::{
    AIRFIELD_FULL_TEXT, NO_BASE_TEXT, NO_SUPPLIES_TEXT, OUT_OF_AMMO_TEXT, requires_text, unavailable_text,
};

const AIRFIELD: ObjectKind = ObjectKind::Airfield;
const HORNET: ObjectKind = ObjectKind::Hornet;
const DRAGONFLY: ObjectKind = ObjectKind::Dragonfly;
const P1: PlayerId = PlayerId(1);
const TRAIN: PanelCommand = PanelCommand::Train(HORNET);
const TRAIN_DRAGONFLY: PanelCommand = PanelCommand::Train(DRAGONFLY);
const RETURN: PanelCommand = PanelCommand::ReturnToBase;

/// Player 1's Airfield (1) on tiles (4, 4) to (6, 6), 1000 Space Crystals and `supplies` Supplies, and `hornets`
/// Hornets east of it (ids from 2), assigned to it by a Rebase in tick 2, which has run; and a Generator after them,
/// which keeps player 1 in the game without the Airfield.
fn field(supplies: u32, hornets: u32) -> TestSim {
    field_of(supplies, hornets, false)
}

/// [`field`], (M6-GL) with Propulsion Labs after the Generator if `labs`.
fn field_of(supplies: u32, hornets: u32, labs: bool) -> TestSim {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(32); 24].join("\n"))
        .faction(P1, Faction::Gdo)
        .spawn(AIRFIELD, Some(P1), SpawnAt::Tile(TilePos::new(4, 4)));
    for n in 0..hornets {
        let at = Vec2::new(900.0, 300.0 + 80.0 * n as f32);
        builder = builder.spawn(HORNET, Some(P1), SpawnAt::Point(at));
    }
    builder = builder.spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(20, 16)));
    if labs {
        builder = builder.spawn(ObjectKind::PropulsionLabs, Some(P1), SpawnAt::Tile(TilePos::new(10, 4)));
    }
    let mut t = builder.start();
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    if hornets > 0 {
        let units = (2..2 + hornets).map(id).collect();
        let rebase = Command::Order {
            units,
            kind: OrderKind::Rebase(id(1)),
            queue: false,
        };
        t.at(2, P1, rebase);
    }
    t.run_until(2);
    for n in 2..2 + hornets {
        let base = t.sim().object(id(n)).and_then(|o| o.aircraft?.base);
        assert_eq!(base, Some(id(1)), "#{n} assigned");
    }
    t
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
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

fn look(panel: &Panel, slot: CommandSlot) -> Option<ButtonLook> {
    panel.get(slot).map(|shown| shown.look)
}

/// The Airfield's DefaultState: Q trains a Hornet, "HT", at once; (M6-GL) W a Dragonfly, "DY", greyed while the player
/// owns no Propulsion Labs ("Requires a Propulsion Labs"; it held its slot, not available, until slice M6-GL); X
/// Cancel Production (greyed with nothing queued) and C Set Rally Point, as on every producer. With 1 Supply Q is
/// greyed, "Not enough Supplies" (the Hornet's 2, design change 0435); with three Hornets assigned and one queued,
/// "Airfield full" (`factions/gdo/structures/airfield.md > Production limit`: "3 assigned + 1 queued = full"). With
/// Propulsion Labs W trains a Dragonfly at once, with its 4 Supplies (design change 0435), and is greyed with 3, "Not
/// enough Supplies".
#[test]
fn the_airfields_q_trains_a_hornet_and_w_a_dragonfly() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint};
    let one = Selection::new([(id(1), AIRFIELD)]);
    let start = panel_of(&field(2, 0), &one);
    let c = PanelButton::Command;
    let (q, w, x, r) = (
        slot(0, 0),
        slot(0, 1),
        CommandSlot::CANCEL_PRODUCTION,
        CommandSlot::RALLY_POINT,
    );
    assert_eq!(
        shown(&start),
        [
            (q, c(TRAIN), Common),
            (w, c(TRAIN_DRAGONFLY), Disabled),
            (x, c(CancelProduction), Disabled),
            (r, c(SetRallyPoint), Common),
        ]
    );
    assert_eq!(
        [q, w, x, r].map(|at| start.get(at).unwrap().label()),
        ["HT", "DY", "Cancel", "Rally"]
    );
    assert_eq!(
        [q, w].map(|at| start.name(at)),
        [Some("Train Hornet".into()), Some("Train Dragonfly".into())]
    );
    assert_eq!(why(&start, w), Some(requires_text(ObjectKind::PropulsionLabs)));
    assert_eq!(press(&start, w), SlotPress::Nothing, "greyed");
    assert_eq!(
        press(&start, q),
        SlotPress::Produce(TRAIN, ProductionCommand::Train(HORNET))
    );

    let short = panel_of(&field(1, 0), &one);
    assert_eq!(
        (look(&short, q), why(&short, q)),
        (Some(Disabled), Some(NO_SUPPLIES_TEXT))
    );

    let labs = panel_of(&field_of(4, 0, true), &one);
    assert_eq!(look(&labs, w), Some(Common));
    assert_eq!(
        press(&labs, w),
        SlotPress::Produce(TRAIN_DRAGONFLY, ProductionCommand::Train(DRAGONFLY))
    );
    let short = panel_of(&field_of(3, 0, true), &one);
    assert_eq!(
        (look(&short, w), why(&short, w)),
        (Some(Disabled), Some(NO_SUPPLIES_TEXT))
    );

    let mut t = field(10, 3);
    assert_eq!(look(&panel_of(&t, &one), q), Some(Common), "3 of 4");
    let train = Command::Production {
        objects: vec![id(1)],
        command: ProductionCommand::Train(HORNET),
    };
    next(&mut t, train);
    let full = panel_of(&t, &one);
    assert_eq!(
        (look(&full, q), why(&full, q)),
        (Some(Disabled), Some(AIRFIELD_FULL_TEXT))
    );
}

/// The Hornet's DefaultState is the BasicCombatUnit's with F Return to Base, "Base", an order at once that Shift
/// queues, to every Hornet of the ActiveGroup, the Hornet being Groupable ("Groupable true"). Unassigned, its Airfield
/// gone, F is greyed: "No Airfield assigned" (`factions/gdo/structures/airfield.md > Returning and rearming`: "Return to
/// Base is unavailable until it is reassigned").
#[test]
fn the_hornets_f_returns_it_to_base_unless_it_has_none() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    let mut t = field(0, 2);
    let one = Selection::new([(id(2), HORNET)]);
    let based = panel_of(&t, &one);
    let f = slot(1, 3);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&based),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), ButtonLook::Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Common),
            (f, c(RETURN), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(based.get(f).unwrap().label(), "Base");
    assert_eq!(based.name(f).as_deref(), Some("Return to Base"));
    assert_eq!(press(&based, f), SlotPress::Issue(RETURN, OrderKind::ReturnToBase));
    assert!(RETURN.queues() && !RETURN.needs_target() && !RETURN.instance_only());
    let two = Selection::new([(id(2), HORNET), (id(3), HORNET)]);
    assert_eq!(recipients(&two, RETURN), [id(2), id(3)]);

    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(1) }));
    let unassigned = panel_of(&t, &one);
    assert_eq!(
        (look(&unassigned, f), why(&unassigned, f)),
        (Some(Disabled), Some(NO_BASE_TEXT))
    );
    assert_eq!(press(&unassigned, f), SlotPress::Nothing, "greyed");
}

/// "Ammo: **5 missiles**, then it must reload at its assigned Airfield": a Hornet with none left can't attack, so its
/// panel shows no A alone (design change 0399, `systems/controls/interface.md > Standard Slot Assignments`: "Only when
/// **no** selected unit can attack is A not shown") and greys
/// its D, "Out of ammo", as the gate refuses both (`Reject::OutOfAmmo`). Beside a Peacekeeper A is the selection's
/// again; with missiles left both are there.
#[test]
fn a_hornet_out_of_ammo_has_no_attack() {
    use ButtonLook::{Common, Disabled};
    let (a, d) = (slot(1, 0), slot(1, 2));
    let modes = |ammo| Modes::default().with_aircraft([id(2)], Some(id(1)), ammo);
    let one = Selection::new([(id(2), HORNET)]);
    let empty = panel(&one, InterfaceState::DefaultState, &Conditions::with_modes(modes(0)));
    assert_eq!(empty.get(a), None, "no A");
    assert_eq!(
        (look(&empty, d), why(&empty, d)),
        (Some(Disabled), Some(OUT_OF_AMMO_TEXT))
    );
    let with_pk = Selection::new([(id(2), HORNET), (id(5), PK)]);
    let mixed = panel(
        &with_pk,
        InterfaceState::DefaultState,
        &Conditions::with_modes(modes(0)),
    );
    assert_eq!(look(&mixed, a), Some(Common), "the selection's Attack");
    let armed = panel(&one, InterfaceState::DefaultState, &Conditions::with_modes(modes(2)));
    assert_eq!([look(&armed, a), look(&armed, d)], [Some(Common); 2]);
}
