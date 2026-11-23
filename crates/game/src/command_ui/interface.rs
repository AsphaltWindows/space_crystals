//! The command flow's rules as plain data (`systems/controls/selection_and_hud.md`): the unit commands, the
//! ObjectInterfaceState, the command panel a selection gets and who receives a command. Unit tested here; the
//! systems apply them to the local player's ControlState. What a click resolves to is [`super::resolution`].

use std::collections::BTreeSet;

use sc_sim::defs::AttackType;
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Sim};

use crate::input::CommandSlot;
use crate::selection::Selection;

/// A command of the unit command panel (`systems/controls/commands.md`), as
/// `> BasicCombatUnitInterfaceState` lists them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitCommand {
    Move,
    Reverse,
    HoldPosition,
    Attack,
    Patrol,
    AttackGround,
    Stop,
}

impl UnitCommand {
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`: the target commands enter AwaitingTarget;
    /// HoldPosition and Stop are issued at once.
    pub fn needs_target(self) -> bool {
        !matches!(self, UnitCommand::HoldPosition | UnitCommand::Stop)
    }

    /// The order of a command that needs no target.
    pub fn immediate_order(self) -> Option<OrderKind> {
        match self {
            UnitCommand::HoldPosition => Some(OrderKind::Hold),
            UnitCommand::Stop => Some(OrderKind::Stop),
            _ => None,
        }
    }

    /// Whether the sim has an order for it yet: Reverse and AttackGround have none (`sc_sim::OrderKind`).
    pub fn has_order(self) -> bool {
        !matches!(self, UnitCommand::Reverse | UnitCommand::AttackGround)
    }

    /// The button's label: a placeholder for its icon (Q0001).
    pub fn label(self) -> &'static str {
        match self {
            UnitCommand::Move => "Move",
            UnitCommand::Reverse => "Reverse",
            UnitCommand::HoldPosition => "Hold",
            UnitCommand::Attack => "Attack",
            UnitCommand::Patrol => "Patrol",
            UnitCommand::AttackGround => "Atk Gnd",
            UnitCommand::Stop => "Stop",
        }
    }
}

/// `> ObjectInterfaceState[ObjectEnum]`: where the ActiveGroup's command flow stands. It belongs to each player's
/// ControlState (`crate::selection::ControlState::interface`), and goes back to DefaultState when the Selection or
/// the ActiveGroup changes (`crate::selection::LocalControl::update`), or when it no longer holds for the game
/// ([`still_valid`]).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InterfaceState {
    /// `> DefaultState`.
    #[default]
    DefaultState,
    /// `> AwaitingTarget[CommandType]`: the PendingCommand waits for a left-click on its target.
    AwaitingTarget(UnitCommand),
}

/// An ObjectInterfaceState template: which commands a type offers, in which slots, and how its clicks resolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterfaceTemplate {
    /// `> BasicCombatUnitInterfaceState`: the Peacekeeper's (`factions/gdo/units.md > Peacekeeper`) and the Guard's
    /// (`factions/syndicate/units.md > Guard`).
    BasicCombatUnit,
}

/// `> BasicCombatUnitInterfaceState > DefaultState commands`, row by row from the top left (`CommandSlot::all`).
#[rustfmt::skip]
const BASIC_COMBAT_UNIT_GRID: [Option<UnitCommand>; 12] = {
    use UnitCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    [
        Some(Move),   Some(Reverse), Some(HoldPosition), None,
        Some(Attack), Some(Patrol),  Some(AttackGround), None,
        None,         Some(Stop),    None,               None,
    ]
};

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState; none for types without commands (the map resources).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        match kind {
            ObjectKind::Peacekeeper | ObjectKind::Guard => Some(InterfaceTemplate::BasicCombatUnit),
            ObjectKind::CrystalPatch | ObjectKind::SupplyDeliveryStation => None,
        }
    }

    /// Its DefaultState's commands, slot by slot (`CommandSlot::all` order).
    pub fn grid(self) -> &'static [Option<UnitCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => &BASIC_COMBAT_UNIT_GRID,
        }
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType.
pub fn can_target_ground(attack_type: AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed | AttackType::DoublyDisjointed)
}

/// Whether objects of `kind` offer `command`: their template has it, and a conditional one only when the type can
/// (`> BasicCombatUnitInterfaceState`: Reverse with the UnitBase's CanReverse, AttackGround with the AttackType's
/// CanTargetGround).
pub fn offers(kind: ObjectKind, command: UnitCommand) -> bool {
    let Some(template) = InterfaceTemplate::of(kind) else {
        return false;
    };
    if !template.grid().contains(&Some(command)) {
        return false;
    }
    let Some(unit) = &kind.def().unit else {
        return false;
    };
    match command {
        UnitCommand::Reverse => unit.base_def().can_reverse,
        UnitCommand::AttackGround => unit.attack.is_some_and(|a| can_target_ground(a.attack_type)),
        _ => true,
    }
}

/// A slot's position in `CommandSlot::all` order.
pub fn slot_index(slot: CommandSlot) -> usize {
    usize::from(slot.row()) * usize::from(CommandSlot::COLUMNS) + usize::from(slot.column())
}

/// What a slot of the command panel holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelButton {
    Command(UnitCommand),
    /// `> Standard Slot Assignments > Z`: Back, from AwaitingTarget to DefaultState.
    Back,
}

/// How a button looks, which also says what pressing it does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonLook {
    /// `> CommonCommands`: every selected object offers it; it goes to the whole selection.
    Common,
    /// `> GroupCommands`: only the ActiveGroup's type offers it; it goes to the ActiveGroup.
    Group,
    /// `> AwaitingTarget`: the PendingCommand, which the panel indicates.
    Pending,
    /// Offered, but not now: the other commands while one awaits its target (placeholder: the design doesn't say
    /// whether they can be pressed then), and a command the sim has no order for yet.
    Disabled,
    /// In the template, but this type can't (`> BasicCombatUnitInterfaceState`: the conditional Reverse and
    /// AttackGround).
    Unavailable,
    /// The Back button.
    Back,
}

/// A slot's button and its look.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelSlot {
    pub button: PanelButton,
    pub look: ButtonLook,
}

/// The command panel (`> CommandPanel > Grid Layout`): what each of the 12 slots shows, if anything.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Panel {
    slots: [Option<PanelSlot>; 12],
}

impl Panel {
    pub fn get(&self, slot: CommandSlot) -> Option<PanelSlot> {
        self.slots[slot_index(slot)]
    }

    /// Every slot, row by row from the top left.
    pub fn slots(&self) -> impl Iterator<Item = (CommandSlot, Option<PanelSlot>)> + '_ {
        CommandSlot::all().map(|slot| (slot, self.get(slot)))
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    fn set(&mut self, slot: CommandSlot, shown: PanelSlot) {
        self.slots[slot_index(slot)] = Some(shown);
    }
}

/// `> CommandPanel`: whether commands can be given to `selection`: something is selected, and all of it is
/// `player`'s. Otherwise no panel is displayed, no right-click resolves and no transition starts.
pub fn commandable(sim: &Sim, player: PlayerId, selection: &Selection) -> bool {
    !selection.is_empty()
        && selection
            .ids()
            .all(|id| sim.object(id).is_some_and(|o| o.owner == Some(player)))
}

/// The ActiveGroup's template, if it has one.
pub fn active_template(selection: &Selection) -> Option<InterfaceTemplate> {
    InterfaceTemplate::of(selection.active()?.kind)
}

/// `> CommandPanel`: the commands of the ActiveGroup's ObjectInterfaceState in their slots, for a commandable
/// selection ([`commandable`]). A command every selected object offers is common, one only the ActiveGroup's type
/// offers a group command; in AwaitingTarget the PendingCommand is marked, the others wait, and Z is Back
/// (`> Standard Slot Assignments`).
pub fn panel(selection: &Selection, interface: InterfaceState) -> Panel {
    let mut panel = Panel::default();
    let (Some(active), Some(template)) = (selection.active(), active_template(selection)) else {
        return panel;
    };
    let kinds: BTreeSet<ObjectKind> = selection.members().map(|(_, kind)| kind).collect();
    for (slot, command) in CommandSlot::all().zip(template.grid()) {
        let Some(command) = *command else {
            continue;
        };
        let look = if !offers(active.kind, command) {
            ButtonLook::Unavailable
        } else {
            match interface {
                InterfaceState::AwaitingTarget(pending) if pending == command => ButtonLook::Pending,
                InterfaceState::AwaitingTarget(_) => ButtonLook::Disabled,
                InterfaceState::DefaultState if !command.has_order() => ButtonLook::Disabled,
                InterfaceState::DefaultState if kinds.iter().all(|&kind| offers(kind, command)) => ButtonLook::Common,
                InterfaceState::DefaultState => ButtonLook::Group,
            }
        };
        let button = PanelButton::Command(command);
        panel.set(slot, PanelSlot { button, look });
    }
    if matches!(interface, InterfaceState::AwaitingTarget(_)) {
        let back = PanelSlot {
            button: PanelButton::Back,
            look: ButtonLook::Back,
        };
        panel.set(CommandSlot::BACK, back);
    }
    panel
}

/// What pressing a slot (its hotkey or its button) does.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SlotPress {
    /// A StateOnlyTransition to AwaitingTarget[command].
    AwaitTarget(UnitCommand),
    /// A CommandIssuingTransition: the command's order, at once.
    Issue(UnitCommand, OrderKind),
    /// Back to DefaultState.
    Back,
    /// Nothing: an empty slot, or a button that can't be pressed now.
    Nothing,
}

/// `> DefaultState`: "Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate
/// commands"; `> Standard Slot Assignments > Z`: Back.
pub fn press(panel: &Panel, slot: CommandSlot) -> SlotPress {
    match panel.get(slot) {
        Some(PanelSlot {
            button: PanelButton::Command(command),
            look: ButtonLook::Common | ButtonLook::Group,
        }) => match command.immediate_order() {
            Some(kind) => SlotPress::Issue(command, kind),
            None => SlotPress::AwaitTarget(command),
        },
        Some(PanelSlot {
            button: PanelButton::Back,
            ..
        }) => SlotPress::Back,
        _ => SlotPress::Nothing,
    }
}

/// `> ObjectInterfaceState`: whether `interface` still holds for `selection` (validated every frame): AwaitingTarget
/// while the selection is commandable and its ActiveGroup still offers the PendingCommand.
pub fn still_valid(interface: InterfaceState, selection: &Selection, commandable: bool) -> bool {
    match interface {
        InterfaceState::DefaultState => true,
        InterfaceState::AwaitingTarget(pending) => {
            commandable
                && press(
                    &panel(selection, InterfaceState::DefaultState),
                    command_slot(selection, pending),
                ) == SlotPress::AwaitTarget(pending)
        }
    }
}

/// The slot of `command` in the ActiveGroup's grid (Back's when it has none, which no command is pressed from).
fn command_slot(selection: &Selection, command: UnitCommand) -> CommandSlot {
    active_template(selection)
        .and_then(|template| {
            CommandSlot::all()
                .zip(template.grid())
                .find(|(_, c)| **c == Some(command))
                .map(|(slot, _)| slot)
        })
        .unwrap_or(CommandSlot::BACK)
}

/// `> CommandPanel`: who a panel command goes to: "Clicking a common command issues it to all selected objects.
/// Clicking a group-specific command issues it only to objects in the ActiveGroup."
pub fn recipients(selection: &Selection, command: UnitCommand) -> Vec<ObjectId> {
    let common = selection.members().all(|(_, kind)| offers(kind, command));
    selection
        .members()
        .filter(|&(id, kind)| offers(kind, command) && (common || selection.in_active_group(id)))
        .map(|(id, _)| id)
        .collect()
}

/// The selected objects whose type offers `command`: who a right-click's order goes to. Placeholder: the design
/// says what a right-click resolves to (from the ActiveGroup), not who carries it out; every selected object able to
/// does, as a common command would.
pub fn able(selection: &Selection, command: UnitCommand) -> Vec<ObjectId> {
    selection
        .members()
        .filter(|&(_, kind)| offers(kind, command))
        .map(|(id, _)| id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PK: ObjectKind = ObjectKind::Peacekeeper;
    const GUARD: ObjectKind = ObjectKind::Guard;

    fn id(n: u32) -> ObjectId {
        ObjectId(n)
    }

    fn slot(row: u8, column: u8) -> CommandSlot {
        CommandSlot::new(row, column).unwrap()
    }

    fn shown(panel: &Panel) -> Vec<(CommandSlot, PanelButton, ButtonLook)> {
        panel
            .slots()
            .filter_map(|(slot, shown)| shown.map(|s| (slot, s.button, s.look)))
            .collect()
    }

    /// `> BasicCombatUnitInterfaceState > DefaultState commands`: the grid of a Peacekeeper and Guard selection.
    /// Neither can reverse (infantry) nor target the ground (FullyConnected), so W and D are unavailable; every other
    /// command is common to both types.
    #[test]
    fn a_mixed_infantry_selection_gets_the_basic_combat_unit_grid() {
        use ButtonLook::{Common, Unavailable};
        use UnitCommand::*;
        let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
        let panel = panel(&selection, InterfaceState::DefaultState);
        let c = PanelButton::Command;
        assert_eq!(
            shown(&panel),
            [
                (slot(0, 0), c(Move), Common),
                (slot(0, 1), c(Reverse), Unavailable),
                (slot(0, 2), c(HoldPosition), Common),
                (slot(1, 0), c(Attack), Common),
                (slot(1, 1), c(Patrol), Common),
                (slot(1, 2), c(AttackGround), Unavailable),
                (slot(2, 1), c(Stop), Common),
            ]
        );
        // `> Standard Slot Assignments`: Z, C and V stay free for a unit; X is its Stop.
        for free in [CommandSlot::BACK, CommandSlot::RALLY_POINT, slot(2, 3)] {
            assert_eq!(panel.get(free), None);
        }
        assert_eq!(press(&panel, slot(1, 0)), SlotPress::AwaitTarget(Attack));
        assert_eq!(
            press(&panel, slot(0, 2)),
            SlotPress::Issue(HoldPosition, OrderKind::Hold)
        );
        assert_eq!(press(&panel, slot(2, 1)), SlotPress::Issue(Stop, OrderKind::Stop));
        assert_eq!(press(&panel, slot(0, 1)), SlotPress::Nothing, "unavailable");
        assert_eq!(press(&panel, CommandSlot::BACK), SlotPress::Nothing, "empty");
    }

    /// `> AwaitingTarget`: the panel marks the PendingCommand, Z is Back, and the other commands wait.
    #[test]
    fn awaiting_a_target_marks_the_pending_command_and_offers_back() {
        let selection = Selection::new([(id(1), PK)]);
        let panel = panel(&selection, InterfaceState::AwaitingTarget(UnitCommand::Attack));
        let look = |slot| panel.get(slot).map(|s| s.look);
        assert_eq!(look(slot(1, 0)), Some(ButtonLook::Pending));
        assert_eq!(look(slot(0, 0)), Some(ButtonLook::Disabled));
        assert_eq!(look(slot(0, 1)), Some(ButtonLook::Unavailable));
        assert_eq!(look(CommandSlot::BACK), Some(ButtonLook::Back));
        assert_eq!(press(&panel, CommandSlot::BACK), SlotPress::Back);
        assert_eq!(press(&panel, slot(1, 0)), SlotPress::Nothing, "already pending");
        assert_eq!(press(&panel, slot(0, 2)), SlotPress::Nothing, "waits");
    }

    /// Resources have no ObjectInterfaceState: a selected patch shows no commands.
    #[test]
    fn a_resource_has_no_commands() {
        let selection = Selection::new([(id(8), ObjectKind::CrystalPatch)]);
        assert!(panel(&selection, InterfaceState::DefaultState).is_empty());
        assert!(panel(&Selection::default(), InterfaceState::DefaultState).is_empty());
        assert_eq!(active_template(&selection), None);
    }

    /// `> CommandPanel`: a common command goes to the whole selection; a command only the ActiveGroup's type offers
    /// goes to the ActiveGroup. In M1 every command a type offers is common (the selection's own objects are all
    /// infantry), so a resource stands in for a type that lacks one.
    #[test]
    fn common_commands_go_to_the_selection_and_group_commands_to_the_active_group() {
        let selection = Selection::new([(id(1), PK), (id(2), GUARD), (id(3), PK)]);
        assert_eq!(recipients(&selection, UnitCommand::HoldPosition), [id(1), id(2), id(3)]);
        let mut mixed = Selection::new([(id(1), PK), (id(3), PK), (id(8), ObjectKind::CrystalPatch)]);
        // The patch's group comes first (`ObjectKind` order): make the Peacekeepers active.
        mixed.cycle(true);
        assert_eq!(mixed.active().map(|k| k.kind), Some(PK));
        let panel = panel(&mixed, InterfaceState::DefaultState);
        assert_eq!(panel.get(slot(1, 0)).map(|s| s.look), Some(ButtonLook::Group));
        assert_eq!(recipients(&mixed, UnitCommand::Attack), [id(1), id(3)]);
        assert_eq!(able(&mixed, UnitCommand::Move), [id(1), id(3)]);
    }

    /// `> ObjectInterfaceState`: AwaitingTarget holds while its command is still offered and the selection
    /// commandable.
    #[test]
    fn awaiting_a_target_holds_while_the_active_group_offers_its_command() {
        let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
        let attack = InterfaceState::AwaitingTarget(UnitCommand::Attack);
        assert!(still_valid(attack, &selection, true));
        assert!(!still_valid(attack, &selection, false), "no longer the player's");
        let reverse = InterfaceState::AwaitingTarget(UnitCommand::Reverse);
        assert!(!still_valid(reverse, &selection, true), "infantry can't reverse");
        let patch = Selection::new([(id(8), ObjectKind::CrystalPatch)]);
        assert!(!still_valid(attack, &patch, true));
        assert!(still_valid(InterfaceState::DefaultState, &patch, false));
    }

    #[test]
    fn only_tail_and_doubly_disjointed_attacks_target_the_ground() {
        use sc_sim::defs::FullyConnectedSubtype::Ranged;
        assert!(!can_target_ground(AttackType::FullyConnected(Ranged)));
        assert!(!can_target_ground(AttackType::HeadDisjointed));
        assert!(can_target_ground(AttackType::TailDisjointed));
        assert!(can_target_ground(AttackType::DoublyDisjointed));
        assert!(!offers(PK, UnitCommand::AttackGround) && !offers(GUARD, UnitCommand::Reverse));
    }
}
