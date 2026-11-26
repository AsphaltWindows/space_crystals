//! The command panel's commands (`systems/controls/interface.md > CommandPanel`), for units, producing
//! structures and constructors alike (plan decision T10: one command flow, not a structure UI beside the unit UI):
//! [`PanelCommand`]; what pressing each one gives the sim (`sim.rs`), and how it flows: whether it awaits a target,
//! goes to one instance, and is queued by Shift (`flow.rs`); its labels and names (`names.rs`).

mod flow;
mod names;
mod sim;

use sc_sim::{ObjectKind, RecruitKind};

use super::{BuildMenu, Planned};

/// A command of the command panel: the unit commands of `systems/controls/commands.md`, as `>
/// BasicCombatUnitInterfaceState` lists them, those of the producing structures (`factions/gdo/structures/barracks.md >
/// ObjectInterfaceState[Barracks]`, `> Standard Slot Assignments`), and those of the constructors
/// (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`, `factions/gdo/economy.md >
/// ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`), and the Colonists' Beacon, Drone, Egress Node and
/// transforming objects (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`,
/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState`, `factions/colonists/economy.md > Egress Node >
/// Egress Node ObjectInterfaceState`, `factions/colonists/economy.md > Beacon Deployment > While transforming: only
/// Cancel`; slices M3-J1 and M3-J2, design change 0124), and the Syndicate's Tunnel
/// (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`, slices M4-J2 and M4-F) and Agent
/// (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1), and the Cults' pool buildings'
/// (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`; slice M5-F2).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelCommand {
    Move,
    Reverse,
    HoldPosition,
    Attack,
    Patrol,
    AttackGround,
    Stop,
    /// `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types` (design change 0173): "The Peacekeeper's
    /// panel shows **Raise Shield**": each Peacekeeper becomes a Peacekeeper (Shielded), its shield going up from
    /// where it is, at once (`sc_sim::Ability::RiotShield`; one Firing or in Cooldown once its shot is over, design
    /// change 0172).
    RaiseShield,
    /// The same's "Peacekeeper (Shielded)'s shows **Lower Shield**": each Peacekeeper (Shielded) becomes a
    /// Peacekeeper, its shield going down from where it is, at once.
    LowerShield,
    /// `factions/colonists/units/sabre_legionnaire.md > ObjectInterfaceState`: "R: Convert to Rifle Legionnaire", and
    /// the Rifle Legionnaire's "R: Convert to Sabre Legionnaire": each form's own command, named for the form it
    /// converts to, so that in a selection of both it goes to the ActiveGroup alone
    /// (`factions/colonists/units/weapon_conversion.md > Convert Command`, design change 0198: "It goes only to the
    /// ActiveGroup"), as Raise and Lower Shield do. At once (`sc_sim::Ability::Convert`): it flips each unit's target
    /// form, and so its type, at the press (`factions/colonists/units/weapon_conversion.md > A toggle sets a target
    /// form`).
    Convert(ObjectKind),
    /// `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`: "Q: Build Peacekeeper": pays for a unit
    /// of this kind and adds it to the BuildQueue, at once (`sc_sim::ProductionCommand::Train`).
    Train(ObjectKind),
    /// `> Standard Slot Assignments > X`, "X: Cancel Production": takes the last unit queued off the BuildQueue and
    /// refunds it, at once.
    CancelProduction,
    /// `> Standard Slot Assignments > C`, "C: Set Rally Point": awaits the rally point's target
    /// (`> AwaitingTarget[SetRallyPoint]`).
    SetRallyPoint,
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter] > DefaultState`: "Q:
    /// Basic Construction" and "W: Advanced Construction" (design change 0104): open that build menu (a
    /// StateOnlyTransition).
    BuildMenu(BuildMenu),
    /// A build menu's entry of an idle constructor (`factions/gdo/structures/deployment_center.md > BuildMenu[Basic]`:
    /// "Generator: deducts 100 Space Crystals, sets CurrentConstruction = Generator"), or the Extraction Facility's "Q:
    /// Build Extraction Plate": pays for the building and starts constructing it, at once
    /// (`sc_sim::ConstructionCommand::Construct`).
    Construct(ObjectKind),
    /// "X: Cancel Construction": "refunds full cost, clears CurrentConstruction", at once.
    CancelConstruction,
    /// "X: Cancel Ready Building" (the Extraction Facility's "X: Cancel Ready Plate"): "refunds 75% cost (rounded
    /// down), clears ReadyToPlace", at once.
    CancelReady,
    /// The ready building's own slot, "Select completed building" (design change 0104), the Extraction Facility's
    /// "Q: Place Plate": enters AwaitingPlacement for it (a StateOnlyTransition), where a click places it.
    PlaceReady(ObjectKind),
    /// A Beacon's Warp In menu entry for a building of its Constructs List (`factions/colonists/economy.md > Beacon >
    /// Beacon ObjectInterfaceState`: "Choosing a building enters placement"; `factions/colonists/economy.md > Warp In >
    /// Warp In Command`): enters AwaitingPlacement of the building on the Beacon (a StateOnlyTransition), where a click
    /// warps it in (`sc_sim::ConstructionCommand::WarpIn`).
    WarpIn(ObjectKind),
    /// A Warp In menu's building, or a portal's unit, that isn't in the game yet (design change 0124: "Buildings and
    /// units whose costs are TBD show in their slot, unavailable"): it holds its slot and does nothing.
    Planned(Planned),
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > A: Deploy`: "enters AwaitingPlacement for a
    /// Beacon (StateOnlyTransition)", where a click sends one Drone of the group to deploy there (design change 0125,
    /// `sc_sim::OrderKind::Deploy`).
    Deploy,
    /// `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`: "X" Cancel Deployment and
    /// Cancel Warp In (design change 0116), the one command of an object that transforms, at once
    /// (`sc_sim::ConstructionCommand::CancelTransform`; `factions/colonists/economy.md > Beacon Deployment > Cancel
    /// Deployment`, `factions/colonists/economy.md > Warp In > Cancel Warp In`).
    CancelTransform,
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > S: Set Drop-off Location`: "enters
    /// AwaitingTarget\[SetDropOffLocation\] (StateOnlyTransition)", where a left-click on the player's own Egress Node
    /// (or refinery, none in the game yet) "sets it as the selected Drone(s)' AssignedDropOff
    /// (CommandIssuingTransition, returns to DefaultState)" (`sc_sim::Ability::SetDropOff`, slice M3-J2): every
    /// selected Drone's (`factions/colonists/units/drone.md > Right-Click Resolution`: "Right-click commands and Set
    /// Drop-off go to every selected Drone"; question Q0040 point 9). It "does not interrupt current mining/movement":
    /// their orders go on.
    SetDropOff,
    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState > F: Select Returning Drones`:
    /// "Immediate command (StateOnlyTransition). Replaces the current Selection with all Drones whose AssignedDropOff
    /// is this Egress Node and which are currently executing a Gather command" (`sc_sim::Sim::returning_drones`,
    /// slice M3-J2). "Only available if at least one such Drone exists": it shows only then
    /// (`super::Conditions::shows`).
    SelectReturningDrones,
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "**E** Eject" (design change 0257), which
    /// "enters EjectMenu (StateOnlyTransition)" (`super::InterfaceState::EjectMenu`): the unit types of the player's
    /// Tunnel Network, to eject from this Tunnel (slice M4-J2).
    EjectMenu,
    /// An EjectMenu tile (`factions/syndicate/tunnel_menus.md > EjectMenu`): "Click an enabled unit type tile: ejects
    /// one unit of that type from this Tunnel's Side A (CommandIssuingTransition)", at once
    /// (`sc_sim::TunnelCommand::Eject`): of the type, the one inside the network longest, which the click reserves
    /// ("the tile's count drops at the click"). The menu stays open for the next click: a reading, as the design
    /// names no state after the click (a rally's says "CommandIssuingTransition, returns to DefaultState") and goes
    /// back on Z, like the Cults' AugmentMenu, which "stays in this menu so further clicks queue more"
    /// (`factions/cults/augmentation.md`).
    Eject(ObjectKind),
    /// A worker's build of a structure on its site (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState >
    /// DefaultState commands`: "**A: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition)";
    /// slice M4-J1), where a left-click on a valid location "confirms placement and dispatches the Agent
    /// (CommandIssuingTransition, returns to DefaultState)": a Build order (`sc_sim::OrderKind::Build`, plan decision
    /// T31) to the one Agent of the ActiveGroup, paid as it is given (design change 0233).
    Build(ObjectKind),
    /// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState > DefaultState commands`: "Drop Off Resources —
    /// targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate
    /// side automatically (Side B for crystals, Side D for supplies)", in S (design change 0257): awaits its drop-off,
    /// where a left-click gives the recipients a Drop Off order (`sc_sim::OrderKind::DropOff`; the gate leaves out
    /// those that carry nothing). "Always visible, greyed out when Agent is not carrying resources"
    /// (`super::Conditions::unavailable`).
    DropOff,
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "**W** Expand Tunnel" (design change 0257),
    /// which "enters ExpandMenu (StateOnlyTransition)" (`super::InterfaceState::ExpandMenu`): the expansions this
    /// Tunnel builds (slice M4-F).
    ExpandMenu,
    /// An ExpandMenu entry (`factions/syndicate/tunnel_menus.md > ExpandMenu`): "Click an expansion type: begins
    /// construction if the Tunnel is not already performing an operation (enters AwaitingPlacement for that
    /// expansion)", a StateOnlyTransition to its placement in this Tunnel's Tunnel Area, where a click builds it
    /// (`sc_sim::TunnelCommand::Expand`).
    Expand(ObjectKind),
    /// The Tunnel's X, "Cancel (upgrade, construction or the current operation)" (design change 0257): `> Expansions
    /// under construction` (design change 0254): "The Tunnel's X cancels whatever operation it is running (an upgrade
    /// or an expansion)", at once (`sc_sim::TunnelCommand::Cancel`). Not an eject queue, which isn't an operation:
    /// "the queue can't be cancelled" (`factions/syndicate/tunnel_menus.md > EjectMenu`).
    CancelOperation,
    /// The Tunnel's Q (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`, design change 0257):
    /// "Upgrade Tunnel — upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula.
    /// Unavailable if already Tier 3 or if currently performing an operation", at once
    /// (`sc_sim::TunnelCommand::Upgrade`, slice M4-G), greyed with the gate's reason (`sc_sim::Sim::can_upgrade`) and
    /// priced in its hint (`sc_sim::Sim::upgrade_price`).
    UpgradeTunnel,
    /// A pool building's DefaultState "**Q** Plain, **W** Constitution, **E** Metabolism, **R** Both (each enters
    /// KindMenu\[kind\])" (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`, design
    /// change 0337): a StateOnlyTransition to `super::InterfaceState::KindMenu`. "Each kind's button shows the unit
    /// with an **icon for its augments** and the count inside (unreserved); it is greyed if there are none"
    /// (`super::PanelSlot::count`, `super::Conditions::pool_count`; slice M5-F2).
    KindMenu(RecruitKind),
    /// A KindMenu\[kind\]'s production option (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`:
    /// "**Q: Train Militant**: deducts Militant's Space Crystals cost from the player, moves one Recruit from
    /// StoredRecruits into a new TrainingQueue entry"), of the Recruits of the kind, at once
    /// (`sc_sim::ProductionCommand::TrainFrom`); "Clicks stay in the menu so they can be repeated".
    TrainFrom(ObjectKind, RecruitKind),
    /// A KindMenu\[kind\]'s "**A: Release**": a free entry that lets one Recruit of the kind out unchanged once it
    /// completes, at once (`sc_sim::ProductionCommand::Release`); the menu stays open, as for a training.
    Release(RecruitKind),
    /// `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "**A: Eject All**: ejects all stored
    /// Recruits from the exit side. Only available if StoredRecruits is not empty", at once
    /// (`sc_sim::HoldCommand::EjectAll`): every unit inside that no order reserves.
    EjectAll,
    /// (M5-J1) `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**S: Assist
    /// Construction** — targeted command (StateOnlyTransition). Left-click an in-progress own building to join
    /// construction": awaits its site, where a left-click on an own site that the recipients build from outside gives
    /// them an Assist order (`sc_sim::OrderKind::Assist`, `factions/cults/economy.md > Assist Construction Command`):
    /// every selected Recruit ("both right-click and panel commands (Construct, Assist Construction, Repair) are issued
    /// to all of them").
    Assist,
    /// (M5-N8) `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**D: Repair**
    /// — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair it", and the
    /// Agent's D (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "Left-click a damaged own Tunnel or
    /// Laser Turret to repair it"): awaits its target, where a left-click on a damaged own object that a recipient
    /// repairs gives them a Repair order (`sc_sim::OrderKind::Repair`, `sc_sim::repair`), every selected Recruit as for
    /// an Assist. On a building that the selection may enter, the panel's Repair is the way to repair it, the
    /// right-click entering it (`factions/cults/units/recruit.md > Right-Click Resolution`: "Repair those with **D:
    /// Repair**").
    Repair,
}
