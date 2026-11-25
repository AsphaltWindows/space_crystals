//! The command panel a selection gets (`systems/controls/selection_and_hud.md > CommandPanel`): what each slot shows
//! ([`panel`](fn@panel)), what pressing it does ([`press`]) and who receives its command ([`recipients`]).

use sc_sim::{
    Ability, ConstructionCommand, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, Sim, TunnelCommand,
};

use super::{
    BuildMenu, Conditions, InterfaceState, PanelCommand, SITE_GRID, TRANSFORMING_GRID, WhyUnavailable, offers,
    slot_index, state_grid,
};
use crate::input::CommandSlot;
use crate::selection::{GroupType, Selection};

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
    /// AwaitingTarget > Other commands while targeting`, design change 0066: unavailable until the player cancels) or
    /// while a building is placed, Attack while none of its recipients can attack, a production or construction command
    /// the command gate would refuse (Train with the BuildQueue full, or too few Space Crystals, the army cap being
    /// checked when production starts, design change 0143; Cancel Production with nothing queued:
    /// `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`, "Only available if"; a building to
    /// construct while the bank can't pay for it, or while the constructor constructs or holds a building ready:
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`, "Other build options
    /// unavailable"; a Convert none of whose recipients may convert, all of them converting or waiting to:
    /// `factions/colonists/units/weapon_conversion.md > Eligibility`, design change 0153; a Warp In while the Beacon
    /// deploys or the bank can't pay for it), and a command the sim has no order for yet.
    Disabled,
    /// In the template, but this type can't (`> BasicCombatUnitInterfaceState`: the conditional Reverse and
    /// AttackGround; a build menu none of whose buildings is in the game yet; a building of the Beacon's Warp In
    /// menus or a unit of a portal's panel that isn't in the game yet, design change 0124).
    Unavailable,
    /// The Back button.
    Back,
}

/// A slot's button and its look.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelSlot {
    pub button: PanelButton,
    pub look: ButtonLook,
    /// Why a disabled or unavailable button is greyed out, which pressing it says (design change 0162); none for
    /// the others.
    pub why: Option<WhyUnavailable>,
    /// An EjectMenu tile's count (`factions/syndicate/tunnels.md > EjectMenu`: "Each tile shows the unit type icon and
    /// a count of how many of that type are in the network"): those not reserved on an eject queue
    /// ([`Conditions::network_count`]); none on the other buttons.
    pub count: Option<u32>,
    /// What a Build costs now, in Supplies ([`Conditions::price`], design change 0233: "the price shown on the Build
    /// button is the next Tunnel's"), and a Tunnel's Upgrade (slice M4-G), which the InfoPanel's hint for the button
    /// shows (`crate::hud::info_panel::hint`); none on the other buttons.
    pub price: Option<u32>,
}

impl PanelSlot {
    /// The button's label: a placeholder for its icon (Q0001).
    pub fn label(&self) -> &'static str {
        match self.button {
            PanelButton::Command(command) => command.label(),
            PanelButton::Back => "Back",
        }
    }

    /// The button's full name (`PanelCommand::name`), which its label may shorten; on a panel, named after the
    /// ActiveGroup's type where the design does so ([`Panel::name`]).
    pub fn name(&self) -> String {
        match self.button {
            PanelButton::Command(command) => command.name(),
            PanelButton::Back => "Back".into(),
        }
    }
}

/// The command panel (`> CommandPanel > Grid Layout`): what each of the 12 slots shows, if anything, for the
/// ActiveGroup of a type.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Panel {
    slots: [Option<PanelSlot>; 12],
    /// The type of the ActiveGroup whose commands it shows, which names some of them ([`Panel::name`]).
    active: Option<GroupType>,
}

impl Panel {
    pub fn get(&self, slot: CommandSlot) -> Option<PanelSlot> {
        self.slots[slot_index(slot)]
    }

    /// The full name of `slot`'s button on this panel, which the InfoPanel's hint for the button under the pointer
    /// shows (`crate::hud::info_panel::hint`): its command's as the ActiveGroup's type names it
    /// (`PanelCommand::name_on`: a Beacon's menus are its Warp In menus), or Back's. `None` for an empty slot.
    pub fn name(&self, slot: CommandSlot) -> Option<String> {
        let shown = self.get(slot)?;
        Some(match (shown.button, self.active) {
            (PanelButton::Command(command), Some(active)) => command.name_on(active.kind),
            _ => shown.name(),
        })
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
/// the menu that is open), or only X Cancel for the group of a type's unfinished buildings ([`TRANSFORMING_GRID`],
/// design changes 0116 and 0185: under construction or transformation, they form a group of their own, "with its own
/// panel"). A common command ([`is_common`]) and a group command look different; in AwaitingTarget the
/// PendingCommand is marked and the others wait, in AwaitingPlacement the building being placed is marked
/// (`super::Placing::command`) and the others wait, and in every state but DefaultState Z is Back (`> Standard Slot
/// Assignments`). What the game allows now ([`Conditions`]) disables a command it doesn't let go to its recipients
/// (Attack while none of them may attack, a Train, a Cancel Production, a constructor's command, a Warp In or a
/// Convert the gate would refuse). Each greyed-out button says why ([`PanelSlot::why`]). The
/// Egress Node's F shows only while it has Drones to select ([`Conditions::shows`]). The Tunnel's EjectMenu works as a
/// build menu does, its tiles greyed while the gate would refuse their Eject, each with its count, and so does its
/// ExpandMenu, its entries greyed while the gate would refuse their Expand (`sc_sim::Sim::can_expand`). A site under
/// construction shows X Cancel Construction alone ([`SITE_GRID`], design change 0237), whatever its type (an
/// expansion's too), and an Agent's Build its price ([`PanelSlot::price`]).
pub fn panel(selection: &Selection, interface: InterfaceState, conditions: &Conditions) -> Panel {
    let mut panel = Panel::default();
    let Some(active) = selection.active() else {
        return panel;
    };
    panel.active = Some(active.group_type());
    let grid = if active.unfinished && conditions.site {
        SITE_GRID
    } else if active.unfinished {
        TRANSFORMING_GRID
    } else {
        state_grid(active.kind, interface, conditions.constructor)
    };
    for (slot, command) in CommandSlot::all().zip(grid) {
        let Some(command) = command.filter(|&command| conditions.shows(command)) else {
            continue;
        };
        let (look, why) = if !offers(active.group_type(), command) {
            (ButtonLook::Unavailable, Some(not_offered(command, conditions)))
        } else {
            match interface {
                InterfaceState::AwaitingTarget { pending, .. } if pending == command => (ButtonLook::Pending, None),
                InterfaceState::AwaitingPlacement(placing) if command == placing.command() => {
                    (ButtonLook::Pending, None)
                }
                InterfaceState::AwaitingTarget { .. } | InterfaceState::AwaitingPlacement(_) => {
                    (ButtonLook::Disabled, Some(WhyUnavailable::Waiting))
                }
                InterfaceState::DefaultState
                | InterfaceState::BuildMenu(_)
                | InterfaceState::EjectMenu
                | InterfaceState::ExpandMenu => {
                    let why = (!command.has_sim_command())
                        .then_some(WhyUnavailable::NotYet)
                        .or_else(|| conditions.unavailable(selection, command));
                    if why.is_some() {
                        (ButtonLook::Disabled, why)
                    } else if is_common(selection, command) {
                        (ButtonLook::Common, None)
                    } else {
                        (ButtonLook::Group, None)
                    }
                }
            }
        };
        let button = PanelButton::Command(command);
        let count = match command {
            PanelCommand::Eject(kind) => Some(conditions.network_count(kind)),
            _ => None,
        };
        panel.set(
            slot,
            PanelSlot {
                button,
                look,
                why,
                count,
                price: conditions.price(command),
            },
        );
    }
    if interface != InterfaceState::DefaultState {
        let back = PanelSlot {
            button: PanelButton::Back,
            look: ButtonLook::Back,
            why: None,
            count: None,
            price: None,
        };
        panel.set(CommandSlot::BACK, back);
    }
    panel
}

/// Why a command the ActiveGroup's type doesn't offer is unavailable ([`offers`]): Reverse and AttackGround are the
/// type's conditional commands; anything else isn't in the game yet (a building of the Beacon's menus, a build menu
/// with none of its buildings), and a placeholder entry says why it is greyed ([`Conditions::planned`]: the Hideout's
/// Grenadier "Requires a Chemical Plant" while the player owns none).
fn not_offered(command: PanelCommand, conditions: &Conditions) -> WhyUnavailable {
    match command {
        PanelCommand::Reverse | PanelCommand::AttackGround => WhyUnavailable::NotForThisType,
        PanelCommand::Planned(planned) => conditions.planned(planned),
        _ => WhyUnavailable::NotYet,
    }
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
    /// A StateOnlyTransition to AwaitingPlacement of a `kind` that the active Beacon warps in.
    AwaitWarpIn(ObjectKind),
    /// A StateOnlyTransition to AwaitingPlacement of the Beacon one of the active Drones deploys into.
    AwaitDeploy,
    /// A StateOnlyTransition to AwaitingPlacement of a `kind` that the active Agent builds
    /// (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "A: Build Tunnel — enters AwaitingPlacement
    /// for a Tunnel (StateOnlyTransition)").
    AwaitBuild(ObjectKind),
    /// A StateOnlyTransition to the Tunnel's EjectMenu (`InterfaceState::EjectMenu`).
    OpenEjectMenu,
    /// A StateOnlyTransition to the Tunnel's ExpandMenu (`InterfaceState::ExpandMenu`).
    OpenExpandMenu,
    /// A StateOnlyTransition to AwaitingPlacement of an expansion, a `kind`, that the active Tunnel builds.
    AwaitExpansion(ObjectKind),
    /// A CommandIssuingTransition: the command's Tunnel command, at once (`sc_sim::Command::Tunnel`): an EjectMenu
    /// tile's Eject, the menu staying open; the Cancel of the Tunnel's operation.
    Tunnel(PanelCommand, TunnelCommand),
    /// The Egress Node's F, a StateOnlyTransition: the selection becomes its returning Drones
    /// (`super::Conditions::returning`).
    SelectReturningDrones,
    /// Back to the previous state (`InterfaceState::back`).
    Back,
    /// Nothing: an empty slot, or a button that can't be pressed now (a greyed-out one says why:
    /// [`PanelSlot::why`]).
    Nothing,
}

/// `> DefaultState`: "Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate
/// commands"; a constructor's enter its build menus and AwaitingPlacement or issue construction commands, a
/// Beacon's its Warp In menus and a Warp In's placement, a Drone's Deploy its placement, an Egress Node's F selects
/// its Drones, a Tunnel's E its EjectMenu, whose tiles eject, its W its ExpandMenu, whose entries enter their
/// placement, and its X cancels its operation; `> Standard Slot Assignments > Z`: Back.
pub fn press(panel: &Panel, slot: CommandSlot) -> SlotPress {
    match panel.get(slot) {
        Some(PanelSlot {
            button: PanelButton::Command(command),
            look: ButtonLook::Common | ButtonLook::Group,
            ..
        }) => {
            if command.needs_target() {
                SlotPress::AwaitTarget(command)
            } else if let Some(kind) = command.immediate_order() {
                SlotPress::Issue(command, kind)
            } else if let Some(ability) = command.ability() {
                SlotPress::UseAbility(command, ability)
            } else if let Some(production) = command.production() {
                SlotPress::Produce(command, production)
            } else if let Some(construction) = command.construction() {
                SlotPress::Construct(command, construction)
            } else if let Some(tunnel) = command.tunnel() {
                SlotPress::Tunnel(command, tunnel)
            } else if command == PanelCommand::EjectMenu {
                SlotPress::OpenEjectMenu
            } else if command == PanelCommand::ExpandMenu {
                SlotPress::OpenExpandMenu
            } else if let PanelCommand::Expand(kind) = command {
                SlotPress::AwaitExpansion(kind)
            } else if let PanelCommand::BuildMenu(menu) = command {
                SlotPress::OpenMenu(menu)
            } else if let PanelCommand::PlaceReady(kind) = command {
                SlotPress::AwaitPlacement(kind)
            } else if let PanelCommand::WarpIn(kind) = command {
                SlotPress::AwaitWarpIn(kind)
            } else if command == PanelCommand::Deploy {
                SlotPress::AwaitDeploy
            } else if let PanelCommand::Build(kind) = command {
                SlotPress::AwaitBuild(kind)
            } else if command == PanelCommand::SelectReturningDrones {
                SlotPress::SelectReturningDrones
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
/// and issued to all of them: every selected object offers it (an unfinished building only its Cancel, design change
/// 0185), and it isn't one instance's own (`PanelCommand::instance_only`), unless the ActiveGroup is all there is to
/// it. Otherwise it is a group command.
pub fn is_common(selection: &Selection, command: PanelCommand) -> bool {
    selection
        .member_types()
        .all(|(id, ty)| offers(ty, command) && (!command.instance_only() || selection.in_active_group(id)))
}

/// `> CommandPanel`: who a panel command goes to: "Clicking a common command issues it to all selected objects.
/// Clicking a group-specific command issues it only to objects in the ActiveGroup." `> SelectionGroup > No
/// smart-casting` (design changes 0071 and 0075): every one of them, the whole Selection or the whole ActiveGroup,
/// never a single member picked out; only a unit that can't carry the command out is left out of it (a Peacekeeper
/// (Shielded) under an explicit Attack on an object, design change 0202: `super::super::resolution::unit_target_order`).
/// Raise Shield and Lower Shield are each one type's, and so is each form's Convert, so in a selection of both types
/// they go to the ActiveGroup (`factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types`: "the shield
/// command goes to the ActiveGroup"; design change 0198: "Only the ActiveGroup receives the press"). Every panel
/// command, by hotkey
/// or button, goes to these objects: a Train to every Barracks of the ActiveGroup, which the sim serves least work
/// first as far as the player can pay (design change 0083), but never to an unfinished one, which is in a group of
/// its own and offers its Cancel alone (design change 0185). A constructor's command goes to the ActiveGroup, the one
/// Deployment Center or Extraction Facility whose state the panel shows ([`is_common`]).
pub fn recipients(selection: &Selection, command: PanelCommand) -> Vec<ObjectId> {
    let common = is_common(selection, command);
    selection
        .member_types()
        .filter(|&(id, ty)| offers(ty, command) && (common || selection.in_active_group(id)))
        .map(|(id, _)| id)
        .collect()
}
