//! The command panel's commands (`systems/controls/interface.md > CommandPanel`), for units, producing
//! structures and constructors alike (plan decision T10: one command flow, not a structure UI beside the unit UI):
//! [`PanelCommand`]; what pressing each one gives the sim (`sim.rs`), and how it flows: whether it awaits a target,
//! goes to one instance, and is queued by Shift (`flow.rs`); its labels and names (`names.rs`).

mod flow;
mod names;
mod sim;

use sc_sim::defs::PatchKind;
use sc_sim::{AreaAbility, AugmentOrder, ObjectKind, RecruitKind, Tech};

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
/// (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`; slice M5-F2)
/// and Altar's (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`, slice M5-N1b), and Temple's
/// (`factions/cults/structures/temple.md > Placeholder stats`, slice M5-N3), and the Disciple's spell
/// (`factions/cults/units/disciple.md > Placeholder stats`, slice M5-N4b), and the Shrine's casts
/// (`factions/cults/structures/shrine.md > Placeholder stats`, slice M5-N5), and the Rocketman's Target Lock
/// (`factions/gdo/units/rocketman.md > Placeholder stats`, slice M5-K), and the Drone Operator's Launch Drone and Build
/// Drone (`factions/gdo/units/drone_operator.md > Placeholder stats`, slice M6-GD), and the Hornet's Return to Base
/// (`factions/gdo/units/hornet.md > AttackAttributes`, slice M6-GH1), and the Shadebringer's toggles
/// (`factions/syndicate/units/shadebringer.md > Placeholder stats`, slice M6-SB), the Subterranean Transport's Unload
/// All (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`, slice M6-ST1), and the Supply
/// Chopper's supply run and the Supply Tower's Schedule Deliveries (`factions/gdo/units/supply_chopper.md >
/// ObjectInterfaceState[SupplyChopper]`, `factions/gdo/economy.md > SupplyTower > ObjectInterfaceState[SupplyTower]`).
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
    /// "Q: Place Plate": enters AwaitingPlacement for it (a StateOnlyTransition), where a click places it. The
    /// Extraction Facility's shows while its plate is constructed too, greyed, with the construction's progress
    /// (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`, design change 0392:
    /// "greyed out, with a running progress bar").
    PlaceReady(ObjectKind),
    /// The Deployment Center's DefaultState "E: Place" (`factions/gdo/structures/deployment_center.md >
    /// ObjectInterfaceState[DeploymentCenter] > DefaultState commands`, design change 0388): the building it constructs
    /// or holds ready, by its icon, greyed with the construction's progress while it is constructed, and once it is
    /// ready entering AwaitingPlacement for it "exactly as the building's slot in the build menu does", from which
    /// Escape returns to DefaultState (`super::Placing::shortcut`). Hidden while the Deployment Center is idle.
    Place(ObjectKind),
    /// A Beacon's Warp In menu entry for a building of its Constructs List (`factions/colonists/economy.md > Beacon >
    /// Beacon ObjectInterfaceState`: "Choosing a building enters placement"; `factions/colonists/economy.md > Warp In >
    /// Warp In Command`): enters AwaitingPlacement of the building on the Beacon (a StateOnlyTransition), where a click
    /// warps it in (`sc_sim::ConstructionCommand::WarpIn`).
    WarpIn(ObjectKind),
    /// A Warp In menu's building, or a portal's unit, that isn't in the game yet (design change 0124: "Buildings and
    /// units whose costs are TBD show in their slot, unavailable"): it holds its slot and does nothing.
    Planned(Planned),
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > W: Deploy`: "enters AwaitingPlacement for a
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
    /// DefaultState commands`: "**W: Build Tunnel** — enters AwaitingPlacement for a Tunnel (StateOnlyTransition)";
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
    /// (M5-N1b) An Altar's unit type tile (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`: "Unit
    /// type tiles (StateOnlyTransition to AugmentMenu[UnitType]) — each tile shows the count of that unit type in
    /// StoredUnits; disabled if there are none: **Q: Recruit**, **W: Militant**, **E: Gunner**, **R: Disciple**"): a
    /// StateOnlyTransition to `super::InterfaceState::AugmentMenu` (`super::PanelSlot::count`,
    /// `super::Conditions::stored_count`).
    AugmentMenu(ObjectKind),
    /// (M5-N1b) An AugmentMenu\[UnitType\]'s AugmentationState tile (`factions/cults/augmentation.md > Altar >
    /// ObjectInterfaceState[Altar] > AugmentMenu[UnitType]`: "StateOnlyTransition to AugmentMenu[UnitType, State].
    /// Each tile shows the count of unreserved units of that type and state in StoredUnits; disabled if there are
    /// none"): **Q: Unaugmented**, **W: Enhanced Constitution only**, **E: Improved Metabolism only**, "**R: Fully
    /// augmented** — enabled when there are any (only Release is possible)"; a StateOnlyTransition to
    /// `super::InterfaceState::AugmentStateMenu`.
    AugmentState(ObjectKind, RecruitKind),
    /// (M5-N1b) An AugmentMenu\[UnitType, State\]'s order (`factions/cults/augmentation.md > Altar >
    /// ObjectInterfaceState[Altar] > AugmentMenu[UnitType, State]`: "Each click queues one order
    /// (CommandIssuingTransition) and stays in this menu so further clicks queue more"), for a unit of the type and the
    /// state, at once (`sc_sim::ProductionCommand::Augment`).
    Augment(ObjectKind, RecruitKind, AugmentOrder),
    /// (M5-N1b) An AugmentMenu\[UnitType, State\]'s "In every state: **A: Release** (free no-op, 4 frames,
    /// placeholder; the unit walks out unchanged)" (design change 0337), of a unit of the type and the state, at once
    /// (`sc_sim::ProductionCommand::ReleaseOf`); the menu stays open, as for an order.
    ReleaseOf(ObjectKind, RecruitKind),
    /// (M5-N3) A Temple's "**Q** Enhanced Constitution (next level), **W** Improved Metabolism (next level)"
    /// (`factions/cults/structures/temple.md > Placeholder stats`): pays for the technology and adds it to the research
    /// queue, at once (`sc_sim::ProductionCommand::Research`; `core/tech_and_research.md > Research > Shared Rules` 2:
    /// "The cost is deducted when the technology is queued"). Its slot holds its line's Level 1, which the panel shows
    /// as the line's next level (`super::Conditions::key`).
    Research(Tech),
    /// (M5-N3) The key of a line whose next level is queued or in progress, at any of the player's structures
    /// (`factions/cults/structures/temple.md > Placeholder stats`: "a Level 2 can't be queued until its Level 1 has
    /// **completed** (its key shows "In progress" meanwhile)"): greyed, as the gate refuses the level again
    /// (`sc_sim::Reject::ResearchUnderWay`), and saying so.
    Researching(Tech),
    /// (M5-N3) A Temple's "**A/S/D/F** release one Recruit of a kind (README > Choosing Recruits, Release and Eject
    /// All)" (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "pressing one sends
    /// **one** Recruit of that kind out of the exit at once (no queue)"), at once (`sc_sim::HoldCommand::ReleaseOne`):
    /// the one of the kind that went in first. Its button "shows the unit with an icon for its augments and the count
    /// inside [...]; it is greyed if there are none" (`super::PanelSlot::count`, `super::Conditions::pool_count`).
    ReleaseOne(RecruitKind),
    /// (M5-N4b) The Disciple's R (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
    /// BasicCombatUnitInterfaceState, plus **R: Shroud**"; "hotkey **R** (R starts the channel; Stop or another order
    /// cancels it [...])"): a Shroud order, at once, where the Disciple stands (`sc_sim::OrderKind::Shroud`), to the
    /// one Disciple of the ActiveGroup: "Groupable - false (user, 2026-11-26; Q0112: each R press starts one
    /// Disciple's channel; Tab to the next)". Greyed while the gate would refuse it, below 10 energy
    /// (`sc_sim::Reject::NotEnoughEnergy`: "needs at least **10 energy** to start").
    Shroud,
    /// (M5-N5) The Shrine's Q and W (`factions/cults/structures/shrine.md > Placeholder stats`: "**Health
    /// regeneration** (hotkey Q)", "**Energy regeneration** (hotkey W)"; "Targeting: AwaitingTarget, left-click a point
    /// in vision"): awaits its point, where a left-click on a point the player sees now casts the ability there
    /// (`sc_sim::Command::Cast`, plan decision T76), from the one Shrine of the ActiveGroup: "Groupable **false** (user,
    /// 2026-11-26; Q0118: each press casts from one Shrine)". Greyed while the gate would refuse it wherever it is cast,
    /// below its cost in energy (`sc_sim::Sim::can_cast`: `sc_sim::Reject::NotEnoughEnergy`); a click on a point out of
    /// vision is an invalid target (`crate::command_ui::resolution::target_resolution`).
    Cast(AreaAbility),
    /// (M5-K) The Rocketman's R (`factions/gdo/units/rocketman.md > Placeholder stats`: "Panel:
    /// BasicCombatUnitInterfaceState, plus **R: Target Lock** (targeted: left-click an enemy unit or structure)"):
    /// awaits its target, where a left-click on an enemy unit or structure that the player sees now gives a Target Lock
    /// order on it (`sc_sim::OrderKind::TargetLock`, `crate::command_ui::resolution::target_order`) to every selected
    /// Rocketman, the Rocketman being Groupable ("Groupable true"). Shift queues it, as any unit order.
    TargetLock,
    /// (M6-GD) The Drone Operator's R (`factions/gdo/units/drone_operator.md > Placeholder stats`: "**Launch Drone**
    /// (hotkey **R**)"; `> Launching and piloting`: "The Drone Operator launches a drone from its stock"): a Launch
    /// order, at once, where the operator stands (`sc_sim::OrderKind::Launch`, plan decision T91: "untargeted and
    /// queueable"), to every operator of the ActiveGroup, the operator being Groupable ("Groupable true"): each puts
    /// its drone over itself, and the selection stays as it is ("**Launching doesn't change the selection**"). Shift
    /// queues it, as any unit order. Greyed while the gate would refuse it: its drone out ("Only **one drone can be
    /// active at a time** per operator", `sc_sim::Reject::DroneOut`), or none in stock (`sc_sim::Reject::StockEmpty`).
    Launch,
    /// (M6-GD) The Drone Operator's F, "**Build Drone** (hotkey **F**): 50 Space Crystals, 160 frames (10 s)": the
    /// purchase of one more of its stock's item, this kind (`sc_sim::defs::StockDef::item`, the drone), at once
    /// (`sc_sim::Ability::Buy`, plan decision T89), which keeps the operator's orders, as a Riot Shield does, so Shift
    /// changes nothing; to every operator of the ActiveGroup, each that may buy one paying at the press while the
    /// bank allows. Greyed while the gate would refuse it: "**Not while a drone is out**" (design change 0421,
    /// `sc_sim::Reject::DroneOut`), the stock full (`sc_sim::Reject::StockFull`), a purchase under way
    /// (`sc_sim::Reject::PurchaseUnderWay`), too few Space Crystals; with the purchase's progress on its bar while it
    /// runs (`super::PanelSlot::progress`), and its price in its hint (`super::Conditions::price`).
    Buy(ObjectKind),
    /// (M6-GH1) An aircraft's F (`factions/gdo/units/hornet.md > AttackAttributes`: "Panel: BasicCombatUnitInterfaceState
    /// plus **F: Return to Base**"; `systems/mechanics/aircraft_landing.md > Assigned base`: "**Return to Base**
    /// command: sends the aircraft to its assigned Airfield to land and rearm (hotkey **F**, placeholder)"): a Return to
    /// Base order, at once (`sc_sim::OrderKind::ReturnToBase`), to every aircraft of the ActiveGroup, each flying to its
    /// own Airfield, where (M6-GH2) it lands and rearms. Shift queues it, as any unit order. Greyed while none
    /// of them has a base ("An unassigned aircraft has no base [...] Return to Base is unavailable until it is
    /// reassigned", `sc_sim::Reject::NoBase`).
    ReturnToBase,
    /// (M6-GA) A vehicle's R, which leads its type into its pair's other, the one named (`sc_sim::vehicle_deploy`, plan
    /// decisions T44 and T87; `sc_sim::defs::UnitDef::vehicle_deploy`): the Artillery Rig's "**R: Deploy** (mobile) /
    /// **R: Undeploy** (deployed)" (`factions/gdo/units/artillery_rig.md > Placeholder stats`), "Deploy" into a type
    /// that can't move and "Undeploy" into one that can (`sc_sim::vehicle_deploy::immobile`). A vehicle deploy order,
    /// at once, where each unit stands (`sc_sim::OrderKind::VehicleDeploy`), to every unit of the ActiveGroup, the Rig
    /// being Groupable ("Groupable true"). Shift queues it, as any unit order ("**Queueable**: Shift-Deploy waits until
    /// the earlier orders are done"). Greyed while the gate would refuse it: a deploy under way ("**Undeploy is only
    /// available once deployment is complete**", `sc_sim::Reject::Deploying`), with its progress on its bar
    /// (`super::PanelSlot::progress`).
    VehicleDeploy(ObjectKind),
    /// (M6-CM) The Disciple's F (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
    /// BasicCombatUnitInterfaceState, plus **R: Shroud** and **F: Mirage**"; `> Spells > Mirage`: "An **instant**
    /// spell: the Disciple creates **4 illusions** of itself"): its cast, at once (`sc_sim::Ability::Mirage`), by the one
    /// Disciple of the ActiveGroup, the Disciple being Ungroupable, as its Shroud. An ability, which Shift doesn't queue:
    /// **Placeholder (not in the design: Shift's meaning for an instant spell)**, as a Riot Shield's. Greyed while the
    /// gate would refuse it: below its 50 energy (`sc_sim::Reject::NotEnoughEnergy`: "**50 energy**"), or for an illusion
    /// (`sc_sim::Reject::IllusionsCantCast`: "**No spells**: illusions can't cast"), whose R is greyed alike.
    Mirage,
    /// (M6-SB) A tank's toggle (`factions/syndicate/units/shadebringer.md > Placeholder stats`: "Hotkeys: **R** Sludge,
    /// **F** Cover"; `> Sludge (toggle)`): the Shadebringer's R, its Sludge, switched on (`on`) or off, at once
    /// (`sc_sim::Ability::Sludge`), by every unit of the selection that has the tank, the Shadebringer being Groupable
    /// ("Groupable - true (assumed)"), each laying its trail and draining its tank (`sc_sim::patches`). Its key shows
    /// it switching off while every one of its recipients has it on already, and on otherwise
    /// (`super::Conditions::key`), as the Riot Shield's R shows the way a press turns the shield: a press means the
    /// same to every unit it goes to (`sc_sim::Ability::Sludge`'s rule). "Toggles follow the mode-switch rule (keep
    /// orders, not queueable)": an ability, which Shift doesn't queue. Greyed while the gate would refuse it: switched
    /// on, while none of its recipients has any charge left ("a toggle can be switched on with **any charge** left in
    /// its tank", `sc_sim::Reject::TankEmpty`). Its F, Cover, holds its slot until slice M6-SC (`Planned::Cover`),
    /// which gives the Shadebringer its smoke tank, and so this command with `lays` smoke in F
    /// (`super::templates::syndicate::shadebringer`).
    Toggle {
        lays: PatchKind,
        on: bool,
    },
    /// (M6-ST1) A carrier's F (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`:
    /// "**Unloading**: **F: Unload All** (above ground only; they come out around it)"; `sc_sim::carry`): an Unload All
    /// order, at once (`sc_sim::OrderKind::UnloadAll`), to every carrier of the ActiveGroup, each putting its
    /// passengers on the map round it once it is at rest. Shift queues it, as any unit order (so Shift-F after a Move
    /// unloads where the move ends). Greyed while the gate would refuse it: no one aboard
    /// (`sc_sim::Reject::NothingInside`), or underground (`sc_sim::Reject::LacksAbility`, `sc_sim::carry::loads`).
    UnloadAll,
    /// A Supply Chopper's Pick Up Supplies (`factions/gdo/units/supply_chopper.md >
    /// ObjectInterfaceState[SupplyChopper]`): awaits a Supply Delivery Station, where a left-click gives the recipients
    /// a Pick Up Supplies order (`sc_sim::OrderKind::PickUpSupplies`; `factions/gdo/units/supply_chopper.md >
    /// AwaitingTarget[PickUpSupplies] resolution`; anything else is an invalid target), to every chopper of the
    /// ActiveGroup, the chopper being Groupable. Shift queues it, as any unit order. Its "not carrying units" always
    /// holds: a chopper carries Supplies alone (`sc_sim::supply_run::Cargo`).
    PickUpSupplies,
    /// A Supply Chopper's Drop Off Supplies: awaits an own Supply Tower
    /// (`factions/gdo/units/supply_chopper.md > AwaitingTarget[DropOffSupplies] resolution`: "Left-click own
    /// SupplyTower with no attached chopper"), where a left-click gives the recipients a Drop Off Supplies order
    /// (`sc_sim::OrderKind::DropOffSupplies`), which the gate gives those that carry Supplies. "Only available if
    /// chopper is carrying supplies": greyed while none of them does (`sc_sim::Reject::NothingToDeliver`). Shift queues
    /// it.
    DropOffSupplies,
    /// A Supply Chopper's Attach to Tower: awaits an own Supply Tower (`factions/gdo/units/supply_chopper.md >
    /// AwaitingTarget[AttachToTower] resolution`: "Left-click own SupplyTower with no attached chopper"), where a
    /// left-click gives the recipients an Attach to Tower order (`sc_sim::OrderKind::AttachToTower`). Shift queues it.
    AttachToTower,
    /// A Supply Tower's S, Schedule Deliveries (`factions/gdo/economy.md > SupplyTower >
    /// ObjectInterfaceState[SupplyTower]`): awaits a Supply Delivery Station, where a left-click sets the schedule of
    /// the recipients (`sc_sim::Command::Schedule`; `factions/gdo/economy.md > SupplyTower >
    /// AwaitingTarget[ScheduleDeliveries] resolution`), every Tower of the ActiveGroup, the Tower being Groupable,
    /// those with a chopper attached taking it. Greyed while none of them has one
    /// (`sc_sim::Reject::NoChopperAttached`). A command of a structure, which Shift doesn't queue.
    ScheduleDeliveries,
}
