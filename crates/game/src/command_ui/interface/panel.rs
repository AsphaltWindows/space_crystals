//! The command panel a selection gets (`systems/controls/selection_and_hud.md > CommandPanel`): what each slot shows
//! ([`panel`](fn@panel)), what pressing it does ([`press`]) and who receives its command ([`recipients`]).

use sc_sim::{Ability, ConstructionCommand, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, Sim};

use super::{BuildMenu, Conditions, InterfaceState, PanelCommand, offers, slot_index, state_grid};
use crate::input::CommandSlot;
use crate::selection::Selection;

/// What a slot of the command panel holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelButton {
    Command(PanelCommand),
    /// `> Standard Slot Assignments > Z`: Back, to the previous state (`InterfaceState::back`): from AwaitingTarget
    /// and a build menu to DefaultState, from AwaitingPlacement to the build menu or DefaultState it came from.
    Back,
}

/// How a button looks, which also says what pressing it does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonLook {
    /// `> CommonCommands`: every selected object offers it; it goes to the whole selection.
    Common,
    /// `> GroupCommands`: only the ActiveGroup's type offers it, or it is one instance's own (a constructor's,
    /// `PanelCommand::instance_only`); it goes to the ActiveGroup.
    Group,
    /// `> AwaitingTarget`: the PendingCommand, which the panel indicates; in AwaitingPlacement, the building being
    /// placed.
    Pending,
    /// Offered, but not now ([`Conditions::allows`]): the other commands while one awaits its target (`>
    /// AwaitingTarget > Other commands while targeting`, design change 0066: unavailable until the player cancels)
    /// or while a building is placed, Attack while none of its recipients can attack, a production or construction
    /// command the command gate would refuse (Train with the BuildQueue full, or too few Space Crystals or too little
    /// Unit Control; Cancel Production with nothing queued: `factions/gdo/structures.md > Barracks >
    /// ObjectInterfaceState[Barracks]`, "Only available if"; a building to construct while the bank can't pay for it,
    /// or while the constructor constructs or holds a building ready: `> DeploymentCenter >
    /// ObjectInterfaceState[DeploymentCenter]`, "Other build options unavailable"), and a command the sim has no
    /// order for yet.
    Disabled,
    /// In the template, but this type can't (`> BasicCombatUnitInterfaceState`: the conditional Reverse and
    /// AttackGround; a build menu none of whose buildings is in the game yet).
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
/// `player`'s. Otherwise no panel is displayed, no right-click resolves and no transition starts. Never once the game
/// is over (`core/game_rules.md > End of Game`, design change 0101); a defeated player owns nothing to command.
pub fn commandable(sim: &Sim, player: PlayerId, selection: &Selection) -> bool {
    sim.outcome().is_none()
        && !selection.is_empty()
        && selection
            .ids()
            .all(|id| sim.object(id).is_some_and(|o| o.owner == Some(player)))
}

/// `> CommandPanel`: the commands of the ActiveGroup's ObjectInterfaceState in their slots, for a commandable
/// selection ([`commandable`]): those of its state ([`state_grid`]: a constructor's follow what it constructs and
/// the menu that is open). A common command ([`is_common`]) and a group command look different; in AwaitingTarget the
/// PendingCommand is marked and the others wait, in AwaitingPlacement the building being placed is marked and the
/// others wait, and in every state but DefaultState Z is Back (`> Standard Slot Assignments`). What the game allows
/// now ([`Conditions`]) disables a command it doesn't let go to its recipients (Attack with every shield up, a Train,
/// a Cancel Production or a constructor's command the gate would refuse), and turns the Riot Shield on.
pub fn panel(selection: &Selection, interface: InterfaceState, conditions: &Conditions) -> Panel {
    let mut panel = Panel::default();
    let Some(active) = selection.active() else {
        return panel;
    };
    for (slot, command) in CommandSlot::all().zip(state_grid(active.kind, interface, conditions.constructor)) {
        let Some(command) = command else {
            continue;
        };
        let look = if !offers(active.kind, command) {
            ButtonLook::Unavailable
        } else {
            match interface {
                InterfaceState::AwaitingTarget { pending, .. } if pending == command => ButtonLook::Pending,
                InterfaceState::AwaitingPlacement(placing) if command == PanelCommand::PlaceReady(placing.kind) => {
                    ButtonLook::Pending
                }
                InterfaceState::AwaitingTarget { .. } | InterfaceState::AwaitingPlacement(_) => ButtonLook::Disabled,
                InterfaceState::DefaultState | InterfaceState::BuildMenu(_) => {
                    if !command.has_sim_command() || !conditions.allows(selection, command) {
                        ButtonLook::Disabled
                    } else if is_common(selection, command) {
                        ButtonLook::Common
                    } else {
                        ButtonLook::Group
                    }
                }
            }
        };
        let button = PanelButton::Command(command);
        let on = command.ability(false).is_some() && conditions.modes.shields_up(&recipients(selection, command));
        panel.set(slot, PanelSlot { button, look, on });
    }
    if interface != InterfaceState::DefaultState {
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
    /// A CommandIssuingTransition: the command's construction command, at once (`sc_sim::Command::Construction`),
    /// which returns to DefaultState.
    Construct(PanelCommand, ConstructionCommand),
    /// A StateOnlyTransition to the build menu.
    OpenMenu(BuildMenu),
    /// A StateOnlyTransition to AwaitingPlacement of the ready building, a `kind`.
    AwaitPlacement(ObjectKind),
    /// Back to the previous state (`InterfaceState::back`).
    Back,
    /// Nothing: an empty slot, or a button that can't be pressed now.
    Nothing,
}

/// `> DefaultState`: "Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate
/// commands"; a constructor's enter its build menus and AwaitingPlacement or issue construction commands;
/// `> Standard Slot Assignments > Z`: Back.
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
            } else if let Some(construction) = command.construction() {
                SlotPress::Construct(command, construction)
            } else if let PanelCommand::BuildMenu(menu) = command {
                SlotPress::OpenMenu(menu)
            } else if let PanelCommand::PlaceReady(kind) = command {
                SlotPress::AwaitPlacement(kind)
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

/// `> CommandPanel`: whether `command` is a common command of `selection`, "available to every object in Selection"
/// and issued to all of them: every selected object offers it, and it isn't one instance's own
/// (`PanelCommand::instance_only`), unless the ActiveGroup is all there is to it. Otherwise it is a group command.
pub fn is_common(selection: &Selection, command: PanelCommand) -> bool {
    selection
        .members()
        .all(|(id, kind)| offers(kind, command) && (!command.instance_only() || selection.in_active_group(id)))
}

/// `> CommandPanel`: who a panel command goes to: "Clicking a common command issues it to all selected objects.
/// Clicking a group-specific command issues it only to objects in the ActiveGroup." `> SelectionGroup > No
/// smart-casting` (design changes 0071 and 0075): every one of them, the whole Selection or the whole ActiveGroup,
/// never a single member picked out; only a unit that can't carry the command out is left out of it (a raised Riot
/// Shield under an explicit Attack: `super::super::resolution::unit_target_order`). Every panel command, by hotkey
/// or button, goes to these objects: a Train to every Barracks of the ActiveGroup, which the sim serves least work
/// first as far as the player can pay (design change 0083). A constructor's command goes to the ActiveGroup, the one
/// Deployment Center or Extraction Facility whose state the panel shows ([`is_common`]).
pub fn recipients(selection: &Selection, command: PanelCommand) -> Vec<ObjectId> {
    let common = is_common(selection, command);
    selection
        .members()
        .filter(|&(id, kind)| offers(kind, command) && (common || selection.in_active_group(id)))
        .map(|(id, _)| id)
        .collect()
}
