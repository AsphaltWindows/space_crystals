//! The command flow's rules as plain data (`systems/controls/selection_and_hud.md`): the unit commands, the
//! ObjectInterfaceState, the command panel a selection gets and who receives a command. Unit tested here; the
//! systems apply them to the local player's ControlState. What a click resolves to is [`super::resolution`].
//!
//! The Peacekeeper's Riot Shield (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`) is a
//! command of its own ([`UnitCommand::RiotShield`]), in slot R of its panel. Pressed, it raises the shields it goes
//! to, or lowers them when all of them are up or on their way up ([`Modes`]), and the button says which. While a
//! unit's shield is up it can't attack: Attack is disabled when none of its recipients can, an explicit Attack
//! leaves it out (`super::resolution::unit_target_order`), and a right-click on an enemy sends it after the enemy
//! instead (`super::resolution::order_without_attack`).
//!
//! No smart-casting (`> SelectionGroup > No smart-casting`, design changes 0071 and 0075): a panel command goes to
//! every unit it is issued to ([`recipients`]).
//!
//! While a target is awaited, the other commands are unavailable (`> AwaitingTarget > Other commands while
//! targeting`, design change 0066): the panel shows them disabled, and their hotkeys and buttons do nothing.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::defs::AttackType;
use sc_sim::{Ability, ObjectId, ObjectKind, OrderKind, PlayerId, RiotShield, Sim};

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
    /// `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`: raises or lowers the shields, at once.
    RiotShield,
}

impl UnitCommand {
    /// `> BasicCombatUnitInterfaceState > DefaultState commands`: the target commands enter AwaitingTarget;
    /// HoldPosition and Stop are issued at once, and so is the Riot Shield.
    pub fn needs_target(self) -> bool {
        !matches!(
            self,
            UnitCommand::HoldPosition | UnitCommand::Stop | UnitCommand::RiotShield
        )
    }

    /// The order of a command that needs no target.
    pub fn immediate_order(self) -> Option<OrderKind> {
        match self {
            UnitCommand::HoldPosition => Some(OrderKind::Hold),
            UnitCommand::Stop => Some(OrderKind::Stop),
            _ => None,
        }
    }

    /// The ability of a toggled command, pressed while it is `on` (all its recipients' shields up or going up):
    /// the one that turns it the other way.
    pub fn ability(self, on: bool) -> Option<Ability> {
        match self {
            UnitCommand::RiotShield => Some(Ability::RiotShield { raise: !on }),
            _ => None,
        }
    }

    /// Whether the sim has an order or an ability for it yet: Reverse and AttackGround have none
    /// (`sc_sim::OrderKind`).
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
            UnitCommand::RiotShield => "Raise Shield",
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
    AwaitingTarget {
        /// The PendingCommand.
        pending: UnitCommand,
        /// `> AwaitingTarget > Shift while targeting` (design change 0070): a Shift-click has queued the command
        /// since the target began to be awaited. Targeting then ends when Shift is released; before the first
        /// queued click a release of Shift changes nothing (the design's TBD, its proposed default: "targeting
        /// simply continues").
        queued: bool,
    },
}

impl InterfaceState {
    /// AwaitingTarget of `pending`, as a command's hotkey or button enters it: no click queued yet.
    pub const fn awaiting(pending: UnitCommand) -> Self {
        InterfaceState::AwaitingTarget { pending, queued: false }
    }

    /// The PendingCommand, while a target is awaited.
    pub fn pending(self) -> Option<UnitCommand> {
        match self {
            InterfaceState::DefaultState => None,
            InterfaceState::AwaitingTarget { pending, .. } => Some(pending),
        }
    }
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

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "TBD: hotkey". Placeholder: slot R, the first slot the
/// BasicCombatUnit grid leaves free, where the Colonists' Legionnaires have their own command
/// (`factions/colonists/units.md > Sabre Legionnaire > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert").
const RIOT_SHIELD_SLOT: usize = 3;

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

/// `kind`'s commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own commands in slots
/// the template leaves free (a unit that carries a Riot Shield: [`UnitCommand::RiotShield`] in R).
pub fn grid(kind: ObjectKind) -> [Option<UnitCommand>; 12] {
    let Some(template) = InterfaceTemplate::of(kind) else {
        return [None; 12];
    };
    let mut grid = *template.grid();
    if kind.def().unit.as_ref().is_some_and(|unit| unit.riot_shield.is_some()) {
        grid[RIOT_SHIELD_SLOT] = Some(UnitCommand::RiotShield);
    }
    grid
}

/// Whether objects of `kind` offer `command`: their grid has it ([`grid`]), and a conditional one only when the
/// type can (`> BasicCombatUnitInterfaceState`: Reverse with the UnitBase's CanReverse, AttackGround with the
/// AttackType's CanTargetGround).
pub fn offers(kind: ObjectKind, command: UnitCommand) -> bool {
    if !grid(kind).contains(&Some(command)) {
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
    /// Offered, but not now: the other commands while one awaits its target (`> AwaitingTarget > Other commands
    /// while targeting`, design change 0066: unavailable until the player cancels), Attack while none of its
    /// recipients can attack, and a command the sim has no order for yet.
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
    /// A toggled command that is on: the Riot Shields it goes to are all up or on their way up ([`Modes`]).
    pub on: bool,
}

impl PanelSlot {
    /// The button's label: a placeholder for its icon (Q0001). A toggle names what pressing it does (placeholder:
    /// the design doesn't say how a toggled ability shows).
    pub fn label(&self) -> &'static str {
        match self.button {
            PanelButton::Command(UnitCommand::RiotShield) if self.on => "Lower Shield",
            PanelButton::Command(command) => command.label(),
            PanelButton::Back => "Back",
        }
    }
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

/// What the selected units' modes let them do now: their Riot Shields (`factions/gdo/units.md > Peacekeeper > Riot
/// Shield`), by id, as the sim has them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    shields: BTreeMap<ObjectId, RiotShield>,
}

impl Modes {
    /// The modes of `selection`'s objects in `sim`.
    pub fn of(sim: &Sim, selection: &Selection) -> Self {
        selection
            .ids()
            .filter_map(|id| Some((id, sim.object(id)?.riot_shield?)))
            .collect()
    }

    /// Whether `id` may attack now: not while its Riot Shield is up, nor on its way up or down
    /// (`RiotShield::allows_attack`).
    pub fn can_attack(&self, id: ObjectId) -> bool {
        self.shields.get(&id).is_none_or(|shield| shield.allows_attack())
    }

    /// Whether `ids` carry Riot Shields and all of those are up or on their way up: a toggle lowers them, and
    /// otherwise raises them all (placeholder: the design doesn't say what a toggle does to a group whose shields
    /// differ).
    pub fn shields_up(&self, ids: &[ObjectId]) -> bool {
        let mut shields = ids.iter().filter_map(|id| self.shields.get(id)).peekable();
        shields.peek().is_some() && shields.all(|shield| shield.is_going_up())
    }
}

impl FromIterator<(ObjectId, RiotShield)> for Modes {
    fn from_iter<I: IntoIterator<Item = (ObjectId, RiotShield)>>(shields: I) -> Self {
        Self {
            shields: shields.into_iter().collect(),
        }
    }
}

/// `> CommandPanel`: the commands of the ActiveGroup's ObjectInterfaceState in their slots, for a commandable
/// selection ([`commandable`]). A command every selected object offers is common, one only the ActiveGroup's type
/// offers a group command; in AwaitingTarget the PendingCommand is marked, the others wait, and Z is Back
/// (`> Standard Slot Assignments`). The units' `modes` turn the Riot Shield on, and disable Attack when none of its
/// recipients can attack (`factions/gdo/units.md > Peacekeeper > Riot Shield`: "the player must lower the shield
/// before ordering an attack").
pub fn panel(selection: &Selection, interface: InterfaceState, modes: &Modes) -> Panel {
    let mut panel = Panel::default();
    let Some(active) = selection.active() else {
        return panel;
    };
    let kinds: BTreeSet<ObjectKind> = selection.members().map(|(_, kind)| kind).collect();
    for (slot, command) in CommandSlot::all().zip(grid(active.kind)) {
        let Some(command) = command else {
            continue;
        };
        let look = if !offers(active.kind, command) {
            ButtonLook::Unavailable
        } else {
            match interface {
                InterfaceState::AwaitingTarget { pending, .. } if pending == command => ButtonLook::Pending,
                InterfaceState::AwaitingTarget { .. } => ButtonLook::Disabled,
                InterfaceState::DefaultState if !command.has_order() => ButtonLook::Disabled,
                InterfaceState::DefaultState
                    if command == UnitCommand::Attack
                        && !recipients(selection, command)
                            .into_iter()
                            .any(|id| modes.can_attack(id)) =>
                {
                    ButtonLook::Disabled
                }
                InterfaceState::DefaultState if kinds.iter().all(|&kind| offers(kind, command)) => ButtonLook::Common,
                InterfaceState::DefaultState => ButtonLook::Group,
            }
        };
        let button = PanelButton::Command(command);
        let on = command.ability(false).is_some() && modes.shields_up(&recipients(selection, command));
        panel.set(slot, PanelSlot { button, look, on });
    }
    if interface.pending().is_some() {
        let back = PanelSlot {
            button: PanelButton::Back,
            look: ButtonLook::Back,
            on: false,
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
    /// A CommandIssuingTransition: the command's ability, at once (`sc_sim::Command::Ability`).
    UseAbility(UnitCommand, Ability),
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
            on,
        }) => match (command.immediate_order(), command.ability(on)) {
            (Some(kind), _) => SlotPress::Issue(command, kind),
            (None, Some(ability)) => SlotPress::UseAbility(command, ability),
            (None, None) => SlotPress::AwaitTarget(command),
        },
        Some(PanelSlot {
            button: PanelButton::Back,
            ..
        }) => SlotPress::Back,
        _ => SlotPress::Nothing,
    }
}

/// `> ObjectInterfaceState`: whether `interface` still holds for `selection` (validated every frame): AwaitingTarget
/// while the selection is commandable and its ActiveGroup still offers the PendingCommand, which its units' `modes`
/// still allow (an Attack while some can attack).
pub fn still_valid(interface: InterfaceState, selection: &Selection, commandable: bool, modes: &Modes) -> bool {
    match interface {
        InterfaceState::DefaultState => true,
        InterfaceState::AwaitingTarget { pending, .. } => {
            commandable
                && press(
                    &panel(selection, InterfaceState::DefaultState, modes),
                    command_slot(selection, pending),
                ) == SlotPress::AwaitTarget(pending)
        }
    }
}

/// The slot of `command` in the ActiveGroup's grid (Back's when it has none, which no command is pressed from).
fn command_slot(selection: &Selection, command: UnitCommand) -> CommandSlot {
    selection
        .active()
        .and_then(|active| {
            CommandSlot::all()
                .zip(grid(active.kind))
                .find(|(_, c)| *c == Some(command))
                .map(|(slot, _)| slot)
        })
        .unwrap_or(CommandSlot::BACK)
}

/// `> CommandPanel`: who a panel command goes to: "Clicking a common command issues it to all selected objects.
/// Clicking a group-specific command issues it only to objects in the ActiveGroup." `> SelectionGroup > No
/// smart-casting` (design changes 0071 and 0075): every one of them, the whole Selection or the whole ActiveGroup,
/// never a single member picked out; only a unit that can't carry the command out is left out of it (a raised Riot
/// Shield under an explicit Attack: `super::resolution::unit_target_order`). Every panel command, by hotkey or
/// button, goes to these units.
pub fn recipients(selection: &Selection, command: UnitCommand) -> Vec<ObjectId> {
    let common = selection.members().all(|(_, kind)| offers(kind, command));
    selection
        .members()
        .filter(|&(id, kind)| offers(kind, command) && (common || selection.in_active_group(id)))
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
    /// command is common to both types. The Peacekeepers, the ActiveGroup, add their Riot Shield in R, a group
    /// command.
    #[test]
    fn a_mixed_infantry_selection_gets_the_basic_combat_unit_grid() {
        use ButtonLook::{Common, Group, Unavailable};
        use UnitCommand::*;
        let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
        let panel = panel(&selection, InterfaceState::DefaultState, &Modes::default());
        let c = PanelButton::Command;
        assert_eq!(
            shown(&panel),
            [
                (slot(0, 0), c(Move), Common),
                (slot(0, 1), c(Reverse), Unavailable),
                (slot(0, 2), c(HoldPosition), Common),
                (slot(0, 3), c(RiotShield), Group),
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
        let panel = panel(
            &selection,
            InterfaceState::awaiting(UnitCommand::Attack),
            &Modes::default(),
        );
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
        assert!(panel(&selection, InterfaceState::DefaultState, &Modes::default()).is_empty());
        assert!(panel(&Selection::default(), InterfaceState::DefaultState, &Modes::default()).is_empty());
        assert_eq!(InterfaceTemplate::of(ObjectKind::CrystalPatch), None);
    }

    /// `> CommandPanel`: a common command goes to the whole selection; a command only the ActiveGroup's type offers
    /// goes to the ActiveGroup. A resource stands in for a type that lacks Attack; the Riot Shield, which Guards lack,
    /// is `the_riot_shield_toggles_in_slot_r`'s. A right-click leaves an object without commands alone (design change
    /// 0062: buildings aren't affected).
    #[test]
    fn common_commands_go_to_the_selection_and_group_commands_to_the_active_group() {
        use crate::command_ui::resolution::{MapTarget, capabilities, unit_right_click};
        let selection = Selection::new([(id(1), PK), (id(2), GUARD), (id(3), PK)]);
        assert_eq!(recipients(&selection, UnitCommand::HoldPosition), [id(1), id(2), id(3)]);
        let mut mixed = Selection::new([(id(1), PK), (id(3), PK), (id(8), ObjectKind::CrystalPatch)]);
        // The patch's group comes first (`ObjectKind` order): make the Peacekeepers active.
        mixed.cycle(true);
        assert_eq!(mixed.active().map(|k| k.kind), Some(PK));
        let panel = panel(&mixed, InterfaceState::DefaultState, &Modes::default());
        assert_eq!(panel.get(slot(1, 0)).map(|s| s.look), Some(ButtonLook::Group));
        assert_eq!(recipients(&mixed, UnitCommand::Attack), [id(1), id(3)]);
        let ground = MapTarget::ground(bevy::math::Vec2::new(100.0, 100.0));
        let moved: Vec<ObjectId> = capabilities(&mixed, &Modes::default())
            .into_iter()
            .filter(|(_, unit)| unit_right_click(*unit, &ground).is_some())
            .map(|(id, _)| id)
            .collect();
        assert_eq!(moved, [id(1), id(3)]);
    }

    /// `> ObjectInterfaceState`: AwaitingTarget holds while its command is still offered and the selection
    /// commandable.
    #[test]
    fn awaiting_a_target_holds_while_the_active_group_offers_its_command() {
        let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
        let attack = InterfaceState::awaiting(UnitCommand::Attack);
        assert!(still_valid(attack, &selection, true, &Modes::default()));
        assert!(
            !still_valid(attack, &selection, false, &Modes::default()),
            "no longer the player's"
        );
        let reverse = InterfaceState::awaiting(UnitCommand::Reverse);
        assert!(
            !still_valid(reverse, &selection, true, &Modes::default()),
            "infantry can't reverse"
        );
        let patch = Selection::new([(id(8), ObjectKind::CrystalPatch)]);
        assert!(!still_valid(attack, &patch, true, &Modes::default()));
        assert!(still_valid(
            InterfaceState::DefaultState,
            &patch,
            false,
            &Modes::default()
        ));
    }

    /// `factions/gdo/units.md > Peacekeeper > Riot Shield`: R raises the shields of the Peacekeepers it goes to, or
    /// lowers them when they are all up or on their way up; the button says which. Guards have no R.
    #[test]
    fn the_riot_shield_toggles_in_slot_r() {
        let r = slot(0, 3);
        let raise = |raise| SlotPress::UseAbility(UnitCommand::RiotShield, Ability::RiotShield { raise });
        let pks = Selection::new([(id(1), PK), (id(2), PK)]);
        let modes = |shields: [RiotShield; 2]| Modes::from_iter([(id(1), shields[0]), (id(2), shields[1])]);
        let down = panel(&pks, InterfaceState::DefaultState, &modes([RiotShield::Lowered; 2]));
        let shown = down.get(r).unwrap();
        assert_eq!(
            (shown.look, shown.on, shown.label()),
            (ButtonLook::Common, false, "Raise Shield")
        );
        assert_eq!(press(&down, r), raise(true));
        for going_up in [
            [RiotShield::Raised, RiotShield::Raised],
            [RiotShield::Raising { left: 2 }, RiotShield::Raised],
        ] {
            let up = panel(&pks, InterfaceState::DefaultState, &modes(going_up));
            assert_eq!(
                (up.get(r).unwrap().on, up.get(r).unwrap().label()),
                (true, "Lower Shield")
            );
            assert_eq!(press(&up, r), raise(false));
        }
        for differing in [
            [RiotShield::Raised, RiotShield::Lowered],
            [RiotShield::Raised, RiotShield::Lowering { left: 1 }],
        ] {
            let some = panel(&pks, InterfaceState::DefaultState, &modes(differing));
            assert_eq!(press(&some, r), raise(true), "{differing:?}: all of them up");
        }
        let guards = Selection::new([(id(3), GUARD)]);
        assert_eq!(
            panel(&guards, InterfaceState::DefaultState, &Modes::default()).get(r),
            None
        );
        assert!(!offers(GUARD, UnitCommand::RiotShield) && offers(PK, UnitCommand::RiotShield));
        let mixed = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
        assert_eq!(recipients(&mixed, UnitCommand::RiotShield), [id(1), id(2)]);
    }

    /// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "the player must lower the shield before ordering an
    /// attack". Attack is disabled while none of its recipients can attack, and a pending Attack ends; a Guard in the
    /// selection still can.
    #[test]
    fn attack_waits_for_a_recipient_that_can_attack() {
        let a = slot(1, 0);
        let attack = InterfaceState::awaiting(UnitCommand::Attack);
        let shielded = Modes::from_iter([(id(1), RiotShield::Raised), (id(2), RiotShield::Lowering { left: 1 })]);
        let pks = Selection::new([(id(1), PK), (id(2), PK)]);
        let panel_of = |selection: &Selection| panel(selection, InterfaceState::DefaultState, &shielded);
        assert_eq!(panel_of(&pks).get(a).map(|s| s.look), Some(ButtonLook::Disabled));
        assert_eq!(press(&panel_of(&pks), a), SlotPress::Nothing);
        assert!(!still_valid(attack, &pks, true, &shielded));
        let with_guard = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
        assert_eq!(
            press(&panel_of(&with_guard), a),
            SlotPress::AwaitTarget(UnitCommand::Attack)
        );
        assert!(still_valid(attack, &with_guard, true, &shielded));
        let one_down = Modes::from_iter([(id(1), RiotShield::Raised), (id(2), RiotShield::Lowered)]);
        assert!(one_down.can_attack(id(2)) && !one_down.can_attack(id(1)) && one_down.can_attack(id(3)));
        assert_eq!(
            panel(&pks, InterfaceState::DefaultState, &one_down)
                .get(a)
                .map(|s| s.look),
            Some(ButtonLook::Common)
        );
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
