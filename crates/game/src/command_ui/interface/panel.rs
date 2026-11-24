//! The command panel a selection gets (`systems/controls/selection_and_hud.md > CommandPanel`): what each slot shows
//! ([`panel`](fn@panel)), what pressing it does ([`press`]) and who receives its command ([`recipients`]).

use std::collections::BTreeSet;

use sc_sim::{Ability, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, Sim};

use super::{Conditions, InterfaceState, PanelCommand, grid, offers, slot_index};
use crate::input::CommandSlot;
use crate::selection::Selection;

/// What a slot of the command panel holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelButton {
    Command(PanelCommand),
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
    /// Offered, but not now ([`Conditions::allows`]): the other commands while one awaits its target (`>
    /// AwaitingTarget > Other commands while targeting`, design change 0066: unavailable until the player cancels),
    /// Attack while none of its recipients can attack, a production command the command gate would refuse (Train
    /// with the BuildQueue full, or too few Space Crystals or too little Unit Control; Cancel Production with
    /// nothing queued: `factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`, "Only available
    /// if"), and a command the sim has no order for yet.
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
    /// A toggled command that is on: the Riot Shields it goes to are all up or on their way up
    /// ([`super::Modes`]).
    pub on: bool,
}

impl PanelSlot {
    /// The button's label: a placeholder for its icon (Q0001). A toggle names what pressing it does (placeholder:
    /// the design doesn't say how a toggled ability shows).
    pub fn label(&self) -> &'static str {
        match self.button {
            PanelButton::Command(PanelCommand::RiotShield) if self.on => "Lower Shield",
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

/// `> CommandPanel`: the commands of the ActiveGroup's ObjectInterfaceState in their slots, for a commandable
/// selection ([`commandable`]). A command every selected object offers is common, one only the ActiveGroup's type
/// offers a group command; in AwaitingTarget the PendingCommand is marked, the others wait, and Z is Back
/// (`> Standard Slot Assignments`). What the game allows now ([`Conditions`]) disables a command it doesn't let go
/// to its recipients (Attack with every shield up, a Train or a Cancel Production the gate would refuse), and turns
/// the Riot Shield on.
pub fn panel(selection: &Selection, interface: InterfaceState, conditions: &Conditions) -> Panel {
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
                InterfaceState::DefaultState if !command.has_sim_command() => ButtonLook::Disabled,
                InterfaceState::DefaultState if !conditions.allows(selection, command) => ButtonLook::Disabled,
                InterfaceState::DefaultState if kinds.iter().all(|&kind| offers(kind, command)) => ButtonLook::Common,
                InterfaceState::DefaultState => ButtonLook::Group,
            }
        };
        let button = PanelButton::Command(command);
        let on = command.ability(false).is_some() && conditions.modes.shields_up(&recipients(selection, command));
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
    /// A StateOnlyTransition to AwaitingTarget\[command\].
    AwaitTarget(PanelCommand),
    /// A CommandIssuingTransition: the command's order, at once.
    Issue(PanelCommand, OrderKind),
    /// A CommandIssuingTransition: the command's ability, at once (`sc_sim::Command::Ability`).
    UseAbility(PanelCommand, Ability),
    /// A CommandIssuingTransition: the command's production command, at once (`sc_sim::Command::Production`).
    Produce(PanelCommand, ProductionCommand),
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
        }) => {
            if command.needs_target() {
                SlotPress::AwaitTarget(command)
            } else if let Some(kind) = command.immediate_order() {
                SlotPress::Issue(command, kind)
            } else if let Some(ability) = command.ability(on) {
                SlotPress::UseAbility(command, ability)
            } else if let Some(production) = command.production() {
                SlotPress::Produce(command, production)
            } else {
                SlotPress::Nothing
            }
        }
        Some(PanelSlot {
            button: PanelButton::Back,
            ..
        }) => SlotPress::Back,
        _ => SlotPress::Nothing,
    }
}

/// `> CommandPanel`: who a panel command goes to: "Clicking a common command issues it to all selected objects.
/// Clicking a group-specific command issues it only to objects in the ActiveGroup." `> SelectionGroup > No
/// smart-casting` (design changes 0071 and 0075): every one of them, the whole Selection or the whole ActiveGroup,
/// never a single member picked out; only a unit that can't carry the command out is left out of it (a raised Riot
/// Shield under an explicit Attack: `super::super::resolution::unit_target_order`). Every panel command, by hotkey
/// or button, goes to these objects: a Train to every Barracks of the ActiveGroup, which the sim serves least work
/// first as far as the player can pay (design change 0083).
pub fn recipients(selection: &Selection, command: PanelCommand) -> Vec<ObjectId> {
    let common = selection.members().all(|(_, kind)| offers(kind, command));
    selection
        .members()
        .filter(|&(id, kind)| offers(kind, command) && (common || selection.in_active_group(id)))
        .map(|(id, _)| id)
        .collect()
}
