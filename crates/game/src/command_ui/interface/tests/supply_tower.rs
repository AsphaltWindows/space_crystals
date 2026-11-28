//! The Supply Tower's panel and the Supply Chopper's as plain data, read from a game (`factions/gdo/economy.md >
//! SupplyTower > ObjectInterfaceState[SupplyTower]`, `factions/gdo/units/supply_chopper.md >
//! ObjectInterfaceState[SupplyChopper]`): what the gate says greys them.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, PlayerId, SpawnAt, TilePos, supply_run};

use super::*;
use crate::command_ui::feedback::{NO_CHOPPER_TEXT, NOTHING_CARRIED_TEXT, unavailable_text};

const TOWER: ObjectKind = ObjectKind::SupplyTower;
const CHOPPER: ObjectKind = ObjectKind::SupplyChopper;
const P1: PlayerId = PlayerId(1);
const SCHEDULE: PanelCommand = PanelCommand::ScheduleDeliveries;

/// Player 1's Supply Tower (1) on tiles (4, 4) to (6, 6), which the map places without a chopper, and a Supply
/// Chopper (2) east of it, in the air; tick 1 has run, and with `attach` the chopper's Attach to Tower in tick 2 too,
/// which attaches it as the order starts.
fn field(attach: bool) -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(32); 24].join("\n"))
        .faction(P1, Faction::Gdo)
        .spawn(TOWER, Some(P1), SpawnAt::Tile(TilePos::new(4, 4)))
        .spawn(CHOPPER, Some(P1), SpawnAt::Point(Vec2::new(900.0, 400.0)))
        .start();
    t.run_until(1);
    if attach {
        let order = Command::Order {
            units: vec![id(2)],
            kind: OrderKind::AttachToTower(id(1)),
            queue: false,
        };
        t.at(2, P1, order);
        t.run_until(2);
        let chopper = t.sim().object(id(2)).unwrap();
        assert_eq!(supply_run::attached_to(chopper), Some(id(1)));
    }
    t
}

/// `selection`'s DefaultState panel in `t`.
fn panel_of(t: &TestSim, selection: &Selection) -> Panel {
    let conditions = Conditions::of(t.sim(), P1, selection);
    panel(selection, InterfaceState::DefaultState, &conditions)
}

/// What the button of `slot` looks like, and what it says if it is greyed.
fn state(panel: &Panel, slot: CommandSlot) -> (Option<ButtonLook>, Option<&'static str>) {
    let shown = panel.get(slot);
    (
        shown.map(|shown| shown.look),
        shown.and_then(|shown| shown.why).map(unavailable_text),
    )
}

/// The Tower's DefaultState: Q trains a Supply Chopper, "SCR", at once; S Schedule Deliveries, "Sched.", which awaits
/// its station, greyed "No chopper attached" until a chopper is attached to the Tower; X Cancel Production (greyed
/// with nothing queued) and C Set Rally Point, as on every producer. S goes to every Tower of the ActiveGroup (the
/// Tower is Groupable), and Shift doesn't queue it.
#[test]
fn the_supply_towers_s_schedules_deliveries_once_a_chopper_is_attached() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let one = Selection::new([(id(1), TOWER)]);
    let alone = panel_of(&field(false), &one);
    let c = PanelButton::Command;
    let (q, s, x, r) = (
        slot(0, 0),
        slot(1, 1),
        CommandSlot::CANCEL_PRODUCTION,
        CommandSlot::RALLY_POINT,
    );
    assert_eq!(
        shown(&alone),
        [
            (q, c(Train(CHOPPER)), Common),
            (s, c(SCHEDULE), Disabled),
            (x, c(CancelProduction), Disabled),
            (r, c(SetRallyPoint), Common),
        ]
    );
    assert_eq!([q, s].map(|at| alone.get(at).unwrap().label()), ["SCR", "Sched."]);
    assert_eq!(
        [q, s].map(|at| alone.name(at)),
        [Some("Train Supply Chopper".into()), Some("Schedule Deliveries".into())]
    );
    assert_eq!(state(&alone, s), (Some(Disabled), Some(NO_CHOPPER_TEXT)));
    assert_eq!(press(&alone, s), SlotPress::Nothing, "greyed");
    assert_eq!(
        press(&alone, q),
        SlotPress::Produce(Train(CHOPPER), ProductionCommand::Train(CHOPPER))
    );

    let attached = panel_of(&field(true), &one);
    assert_eq!(state(&attached, s), (Some(Common), None));
    assert_eq!(press(&attached, s), SlotPress::AwaitTarget(SCHEDULE));
    assert!(SCHEDULE.needs_target() && !SCHEDULE.queues() && !SCHEDULE.instance_only());
    let two = Selection::new([(id(1), TOWER), (id(3), TOWER)]);
    assert_eq!(recipients(&two, SCHEDULE), [id(1), id(3)]);
}

/// The Supply Chopper's DefaultState (`systems/controls/interface.md > CommandPanel > Standard Slot Assignments`'s
/// letters, placeholders): Q Move, E Hold Position, R Pick Up Supplies, S Drop Off Supplies, D Attach to Tower, X
/// Stop, F free (no Unload All: the chopper carries Supplies alone); the chopper is unarmed, so A shows only beside a
/// unit that can attack.
/// R, S and D await their targets, orders that Shift queues, to every chopper of the ActiveGroup. S is greyed while
/// none of them carries Supplies, "Not carrying resources".
#[test]
fn the_supply_choppers_panel_picks_up_drops_off_and_attaches() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{AttachToTower, Attack, DropOffSupplies, HoldPosition, Move, PickUpSupplies, Stop};
    let t = field(false);
    let one = Selection::new([(id(2), CHOPPER)]);
    let empty = panel_of(&t, &one);
    let c = PanelButton::Command;
    let (r, s, d) = (slot(0, 3), slot(1, 1), slot(1, 2));
    assert_eq!(
        shown(&empty),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(PickUpSupplies), Common),
            (s, c(DropOffSupplies), Disabled),
            (d, c(AttachToTower), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(
        [r, s, d].map(|at| empty.get(at).unwrap().label()),
        ["PickUp", "Drop", "Attach"]
    );
    assert_eq!(
        [r, s, d].map(|at| empty.name(at)),
        [
            Some("Pick Up Supplies".into()),
            Some("Drop Off Supplies".into()),
            Some("Attach to Tower".into())
        ]
    );
    assert_eq!(state(&empty, s), (Some(Disabled), Some(NOTHING_CARRIED_TEXT)));
    assert_eq!(press(&empty, r), SlotPress::AwaitTarget(PickUpSupplies));
    assert_eq!(press(&empty, d), SlotPress::AwaitTarget(AttachToTower));
    for command in [PickUpSupplies, DropOffSupplies, AttachToTower] {
        assert!(
            command.needs_target() && command.queues() && !command.instance_only(),
            "{command:?}"
        );
    }
    let two = Selection::new([(id(2), CHOPPER), (id(4), CHOPPER)]);
    assert_eq!(recipients(&two, DropOffSupplies), [id(2), id(4)]);

    let modes = Modes::default().with_choppers([id(2)], true, None);
    let loaded = panel(&one, InterfaceState::DefaultState, &Conditions::with_modes(modes));
    assert_eq!(state(&loaded, s), (Some(Common), None));
    assert_eq!(press(&loaded, s), SlotPress::AwaitTarget(DropOffSupplies));

    // Beside a Peacekeeper, the choppers' panel (Tabbed to) has the selection's Attack in A.
    let mut with_pk = Selection::new([(id(2), CHOPPER), (id(5), PK)]);
    if with_pk.active().map(|key| key.kind) != Some(CHOPPER) {
        with_pk.cycle(true);
    }
    assert_eq!(with_pk.active().map(|key| key.kind), Some(CHOPPER));
    let mixed = panel(&with_pk, InterfaceState::DefaultState, &Conditions::default());
    assert_eq!(mixed.get(slot(1, 0)).map(|shown| shown.button), Some(c(Attack)));
    assert_eq!(empty.get(slot(1, 0)), None, "no A alone");
}
