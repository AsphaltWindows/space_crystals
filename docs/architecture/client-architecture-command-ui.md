# Client architecture — Commands UI and the panels

Read this when you touch the command panel, its rules and systems, or a unit's or building's panel.

**Commands UI** (`command_ui.rs`, `command_ui/`; `systems/controls/interface.md > ObjectInterfaceState`
to `systems/controls/hud.md > CommandIndicators`, `systems/controls/commands.md > Unit Command`): how the
local player orders their units, their producing structures and their constructors, and the Colonists' Beacons and
Drones, through one command flow (plan decision T10, slices M2-H1, M2-H2 and M3-J1).
- **Rules** (pure and unit tested): `command_ui/interface.rs` and its children (`commands.rs`, `templates.rs` with
  its `production.rs`, `state.rs` and `offers.rs` and an object's own panel in `templates/<faction>/<object>.rs`,
  `conditions.rs` with its probes of the gate in `conditions/probes.rs`, `panel.rs`) have the `PanelCommand`s (the units' and the producers' commands), the `InterfaceState`
  (DefaultState, or AwaitingTarget of a command, `queued` once a Shift-click has queued it: design change 0070) in
  each `ControlState`, the ActiveGroup's `InterfaceTemplate` (the Peacekeeper's and the Guard's BasicCombatUnit grid;
  every producer's Production grid, built from its `ProductionDef` (plan decision T23): its units' Train from Q on,
  Cancel Production in X, Set Rally Point in C, so the Barracks trains the Peacekeeper in Q, the Robo Portal the
  Drone and the Bio Portal the Sabre Legionnaire, design change 0124; a unit its design gives a slot of its own
  takes it (`PanelTemplate::trains`, placed before the others fill Q, W and on: the Barracks' Field Engineer in E,
  greyed "Requires a Career Campus" until one stands, slice M5-N9; the Rocketman in W, greyed "Requires an Arsenal"
  until one stands, slice M5-K), while the portals' units not in the game yet hold their slots unavailable, 0124's
  "W Scout"
  and "W Lancer, E psionic unit", as `PanelCommand::Planned`, slice M3-J2),
  the `Panel` a selection gets (common, group, pending, disabled and unavailable buttons; Back in Z while a target is
  awaited) and who receives a command (`recipients`: a common one the whole selection, a group one the whole
  ActiveGroup, by hotkey or button; never some of them, no smart-casting, 0071, apart from the units that can't carry
  it out, which the resolution leaves out; Attack every selected unit, common whichever group is in focus,
  `to_every_unit`: design change 0397, "Attack goes to everyone"). A type's grid is its template's plus its own
  commands (`grid`: in R,
  Raise Shield for the Peacekeeper and Lower Shield for the Peacekeeper (Shielded), design changes 0173 and 0174;
  Convert to Rifle Legionnaire for the Sabre and Convert to Sabre Legionnaire for the Rifle, `PanelCommand::Convert`
  of the target form, design change 0198: "It goes only to the ActiveGroup"; each its type's own, so in a selection
  of both it goes to the ActiveGroup). **A is the selection's** (`attack_slot`, in `state_grid` and on a site's or
  a transforming object's panel; design changes 0397 to 0399: "A: Attack, on every unit", "A is shown whenever at
  least one selected group can attack"): Attack while one of the selected units can attack (`Conditions::attacks`),
  whichever group is in focus, and empty while none can, so that the Peacekeeper (Shielded)'s panel (0173: "no
  Attack"), the Disciple's and the unarmed workers' show none on their own. A building's own A stays (a pool
  building's Eject All), and a building's empty A shows the selection's Attack (a reading: the design names units).
  An Attack on an object sends the units that can't attack to it, a Move on it (0397 and 0398, where 0075 left the
  shielded units out), and (slice M6-S6a) those whose weapon can't hit it (0461), and an AttackMove takes them along
  (0076). A toggle's press switches its units' type, and group, at once (`ControlState::toggle`, by
  `Ability::switched_kind`, before the sim's tick has it; design change 0198): the group they join becomes the
  ActiveGroup, in its DefaultState (design change 0199), so the next press goes to every unit the last one reached.
  `Modes` holds the selected units' modes: whether a unit takes an Attack, by its type from the press (design change
  0202, `riot_shield::takes_attack_orders`: a Peacekeeper whose shield is on its way down takes it and attacks once the
  shield is down), and (slice M5-N4a) the selected units whose type takes none at all (`Modes::held`,
  `combat::takes_attack_orders`: without a weapon, the Disciple, or Shielded), so that an explicit Attack sends them
  to its target instead (`unit_target_order`) and A shows while one selected unit takes one (were it pressed with
  none, it would say "Lower the Riot Shield first" while a recipient is shielded, else "Not available for this unit",
  `WhyUnavailable::NotForThisType`); (slice M6-GH1) and the selected aircraft's bases and those out of ammo
  (`Modes::aircraft`, `Modes::empty`), which count as held for A and are refused an Attack and an Attack Ground
  ("Out of ammo"). An Attack sent in the frame of a Lower Shield, before the sim's tick, goes too:
  the gate refuses it only because the unit is still a Peacekeeper (Shielded), and the tick applies the Lower first
  (`OrderOutbox::give`, `attack_once_toggled`). `Conditions` holds the `Modes` and what
  the command gate refuses: a production command is disabled exactly when `Sim::validate` refuses the
  `Command::Production` pressing it would send (the gate's own predicates, no copy of them: crystals, a full queue,
  nothing to cancel; 0081; never the army cap, which is checked when production starts, 0143); a toggle never is
  (design change 0198: a press is never refused). `command_ui/resolution.rs`
  has what a click resolves to, and `resolution/pointer.rs` the `PointerDisplay` that previews it (split off in slice
  M5-J1); `interface/commands/names.rs` has the commands' labels and names, `commands/flow.rs` how each flows
  (whether it needs a target, belongs to one instance, queues) and `commands/sim.rs` what it gives the sim (split
  off in slice M5-N1b), beside `commands.rs`, the commands. A `MapTarget` is what is under the cursor
  (`CursorTarget`, from `Selectables::target_at`: a remembered record too, 0139), with the ground point there and,
  for an object, the target as the local player knows it (`aimed`, the sim's `Sim::aimed`). A right-click resolves
  per unit (design changes 0062, 0063), whichever group is active, by the sim's one rule (`unit_right_click` calls
  `sc_sim::orders::resolution::resolve`, the rally point's rule too, design change 0136) on `UnitCapabilities` (the
  unit's template and the sim's `Abilities`: its attack's TargetDomain, whether its type takes an Attack, 0202, whether it is
  a worker and carries a load): an enemy it can attack gives Attack, a patch Gather for a Drone, anything else a Move
  to it, which follows a unit (a shielded Peacekeeper follows an enemy, 0074; a right-click on a selected unit makes
  it stop and the others follow it, 0168); the ground a Move. A Drone (`InterfaceTemplate::Drone`) resolves by the
  same rule (see Gathering below); the Beacon and the Egress Node have no right-click, nor has an object that
  transforms (no template, no abilities: slice M3-J1); objects without commands do nothing. A producer's right-click
  sets its rally point
  (`object_right_click` gives a `Resolved::Rally` beside the units' `Resolved::Order`s, so a mixed selection does
  both, 0062; `rally_target`: the object under the cursor, else the ground point; the producer itself clears it,
  0086); Set Rally Point's targeting click resolves the same way (`target_resolution`), and the pointer shows Move
  for a rally point. `group_orders` makes one order per distinct order, so units
  sent to one location go as one group (the magic box). A left-click while a target is awaited gives `target_order`:
  Attack takes any destructible object, own and neutral ones too (force-attack, 0059, 0064), never an indestructible
  one; a right-click never attacks them. `unit_target_order` is the targeting click's rule for the units that can't
  attack, per unit: an explicit Attack on an object is a Move on it for them, a Peacekeeper (Shielded), a Disciple or
  a worker without a weapon (design change 0398: "on a target they move to it (as a Move on that object)", where 0075
  left the shielded units on their orders; 0202), and (slice M6-S6a) a unit whose weapon can't hit the target's domain
  (design change 0461, answering Q0186: "it **moves to the target (follows it) with the group**"; the gate's check,
  `UnitCapabilities::attacks`, `sc_sim::combat::can_hit`, on the domain `Sim::aimed` gives, so that a Twister follows a
  Hornet down on its Airfield, a ground target, slice M6-GH2). `target_orders` gives each recipient its order: none to a
  unit that can't move, which follows nothing (a deployed Artillery Rig, or one deploying into one: the gate would
  refuse it the Move, `CannotMove`), and none at all for an Attack none of them can carry out, an invalid target (0461:
  "If **no** selected unit can attack the target, the click is an **invalid target**"; an aircraft out of ammo counts
  among the units that can't attack, `Modes::empty`, as the gate's `OutOfAmmo` has it); an AttackMove goes to every
  unit, shielded and unarmed ones too, which the sim walks on without engaging (0076, `systems/controls/commands.md >
  AttackMove > Units that can't attack`: the sim's gate drops the units that can't attack from Attack only). The pointer
  previews the set of per-unit right-click actions: one shows its pointer, several show `Mixed` (0063); while a target
  is awaited, the pending command's look over a target it can use, else Inactive. In a menu (a build menu, the
  EjectMenu and the ExpandMenu, a KindMenu, the AugmentMenus) the right-click is DefaultState's, previewed as there,
  and the menu stays open; in AwaitingTarget and AwaitingPlacement it goes straight back to DefaultState, a building
  ready to place staying ready (design change 0400, `right_click_orders`). Nothing is commandable unless the whole
  selection is the player's, nor once the game is over (`commandable`).
- **Systems** (`CommandUiSystems`): `Validate` (in `SyncSim`, after the selection's sync) ends an AwaitingTarget that
  no longer holds. `Input` (in `Actions`, after the selection's input) turns Esc, the command hotkeys
  (`InputAction::CommandSlot`), clicks on the panel (found by `CommandGrid`), left-clicks on the map while a target is
  awaited and right-clicks into state changes and orders. An order is a `Command::Order` in `PendingCommands` naming
  only the player's own units, given only if `Sim::validate` takes it, queued while `Queue` (Shift) is held; each one
  is also an `OrderIssued` message. A click's per-unit orders go out grouped (`OrderOutbox::give_each`); in a
  targeting click each recipient's order is `unit_target_order`'s. An ability is a `Command::Ability`, validated the
  same way, never queued, and no `OrderIssued`. A producer's command is one `Command::Production` to every recipient
  (`OrderOutbox::produce`: the player's own producers, validated the same way), never queued, no `OrderIssued`: no
  smart-casting, the sim decides which producers a Train reaches and pays for (0083); a Shift-click while Set Rally
  Point awaits its target sets the rally point and ends targeting like a plain click (a placeholder reading: a rally
  point isn't queued). While a target is awaited (`systems/controls/interface.md > AwaitingTarget`, design
  changes 0065 to 0070 and 0073): the other commands do nothing (`press`; the panel shows them disabled); a click counts
  at the press point, as the frame of the press drew the objects (`TargetPress`, recorded when the Select button goes
  down on the map, used when it comes up, `Pointer::released`); the press is a click while the pointer stayed within
  `input.targeting_click_allowance_px` of where it went down (16, a placeholder: more than the normal
  `input.drag_threshold_px`, which holds everywhere else), and beyond it a drag, which does nothing (the selection's
  input ignores it, no box is drawn, the target is still awaited); a click that gives nothing (an invalid target, or an
  order the sim rejects) keeps targeting and sends an `InvalidTarget` message, Shift held or not. A click without Shift
  returns to DefaultState. A Shift-click (`Queue` held as the button went down, where the click counts) queues its
  order, one more in each unit's queue, and keeps the target awaited, marked `queued`; letting go of Shift then ends
  targeting (`end_queued_targeting`, first in `Input`, after a targeting press still held has had its click), while
  before the first queued click it changes nothing (0070's TBD, its proposed default). Esc, a right-click and Back
  cancel, Shift held or not. The player's changes of the selection or the ActiveGroup cancel targeting
  (`LocalControl::update`), and so does every control group recall, of the same selection too (0078); removals by
  the game (deaths) don't, unless the whole ActiveGroup is gone (`ControlState::retain`).
- **Construction** (slice M2-H2; `factions/gdo/structures/deployment_center.md >
  ObjectInterfaceState[DeploymentCenter]`, design change 0104; `factions/gdo/economy.md > ExtractionFacility >
  ObjectInterfaceState[ExtractionFacility]`): the constructors' commands are `PanelCommand`s too: `BuildMenu(menu)` (the
  Deployment Center's Q Basic and W Advanced; since slice M6-GK every building of its Constructs holds its slot, those
  it doesn't construct yet as `PanelCommand::Planned`, greyed "Not available yet", `deployment_center::menu`),
  `Construct(kind)`, `CancelConstruction` and `CancelReady` (X), and `PlaceReady(kind)`, "Select completed building",
  which takes its building's slot once it is ready. `InterfaceState` adds `BuildMenu(menu)` and
  `AwaitingPlacement(Placing { builder, kind, orientation, shortcut })`, the builder a constructor's
  `Builder::Constructor(id)`. A constructor's panel follows its state (`state_grid`, from `Conditions::constructor`, the
  ActiveGroup's one instance's `Constructor`), and its commands are that instance's own (`PanelCommand::instance_only`):
  they go to the active constructor only, never to the other selected ones. A construction command is one
  `Command::Construction` (`OrderOutbox::construct`, validated like the others), and Construct and the Cancels return to
  DefaultState. Back and Esc go back one state (`InterfaceState::back`): from AwaitingPlacement to the Deployment
  Center's build menu (its DefaultState from E's) or the Extraction Facility's DefaultState, from a build menu to
  DefaultState; a right-click leaves a placement for DefaultState, and a build menu open (0400). While placing, the
  placement keys turn the ghost (`turn_ghost`, see Input), and a left-click on
  the map places it if `Sim::placement_check` allows it now: one Place at the footprint centred under the cursor
  (`Placing::origin_under`), as turned, and back to DefaultState; a Shift-click the same (0071); elsewhere nothing is
  sent, placing goes on, and an `InvalidPlacement` message says why. A Place the sim refuses as it applies it
  (`SimEvent::PlacementRefused`) leaves the building ready and the player in DefaultState. A build menu leaves the map's
  clicks to the selection; AwaitingPlacement takes them (`InterfaceState::takes_map_clicks`). `Validate` ends a build
  menu the ActiveGroup doesn't have, a placement whose constructor isn't active or holds the building no more, and both
  when the selection isn't commandable. **Place buttons** (design changes 0388 and 0392): the Deployment Center's
  DefaultState shows `Place(kind)` in E (its template's `PLACE_SLOT`, labelled with the building's tag for its icon)
  while it constructs or holds a building ready, and the Extraction Facility's Q is `PlaceReady(plate)` from the start
  of the plate's construction; both are greyed until the building is ready (`WhyUnavailable::UnderConstruction`) and
  carry the construction's progress meanwhile (`PanelSlot::progress`, a `sc_sim::Progress` of the sim's
  `Constructor::Constructing` progress over the recipe's frames, never kept by the client), which the HUD draws as a bar
  (`hud/progress_bar.rs`, the one bar piece; `hud/command_panel.rs` moves it on in place while nothing else of the panel
  changes, `Panel::same_but_progress`). E enters the placement as the menu's slot does, marked `Placing::shortcut`: the
  panel shows DefaultState meanwhile, E lit, and Back, Esc and a right-click return there. The build menus keep their
  slot of the building.
- **Warp In and Deploy** (slice M3-J1; `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`,
  `factions/colonists/units/drone.md > Drone ObjectInterfaceState`; design changes 0116, 0119 to 0122, 0124, 0125):
  `command_ui/interface/templates/colonists/` has the Colonists' panels, one file per object (the portals' have
  their planned units, `PanelTemplate::planned`). The Beacon's (`InterfaceTemplate::Beacon`)
  opens Warp In (Basic) in Q and Warp In (Advanced) in W, `BuildMenu`s whose entries are its own (`menu_grid`): a
  `WarpIn(kind)` for each building of its Constructs List (`transform::warp_in_recipes`), and a `Planned(..)` for
  each building not in the game yet, which holds its slot unavailable. A `WarpIn` enters `AwaitingPlacement` with
  `Builder::Beacon(id)`, for the Beacon Tabbed to (Beacons are Ungroupable, 0122; `instance_only`): the ghost is
  `transform::warp_in_footprint` of the Beacon's placement (`Placing::origin`), turned by the placement keys about
  its centre, never moved by the mouse (`Placing::follows_cursor`: the pointer shows); a left-click anywhere sends
  `ConstructionCommand::WarpIn` if `Sim::warp_in_check` allows it (`OrderOutbox::try_construct`; a gate refusal,
  the bank short since the entry was pressed, says why and placement goes on), and Back returns to its menu. A
  menu entry is greyed out by the gate's answer to its Warp In unturned, `Unplaceable` left out: the ghost shows
  where it can't stand. The Drone's grid (`InterfaceTemplate::Drone`) is Q Move, W Deploy (design change 0398), S Set
  Drop-off, X Stop (S and the right-click: Gathering below); W enters `AwaitingPlacement` with
  `Builder::Drones { queued }`: the ghost
  under the cursor, checked by `Sim::deploy_check` for the Drone the click would go to (`deploy_recipient`: a plain
  click's the ActiveGroup's first Drone in portrait order, whose orders the Deploy replaces; a Shift-click's the
  first with no Deploy, among `Sim::pending_builds` and the Deploys still in `PendingCommands`, `deploying`). A
  click gives that one Drone `OrderKind::Deploy { origin }`, queued with Shift; Shift-placement stays (`queued:
  true`) while another Drone of the group has none, and letting go of Shift ends it (`end_queued_placement`).
  Whatever transforms (`Object::is_transforming`) is unfinished, in its type's unfinished group (design change
  0185, `GroupType`): that group's key gives the panel `TRANSFORMING_GRID`, X `CancelTransform` alone
  (`ConstructionCommand::CancelTransform`, to the ActiveGroup), as `offers` and `commands` of an unfinished
  `GroupType` give the Cancel and nothing else, so the finished group's commands never go to it and `still_valid`
  ends a transforming Beacon's menu or placement; `Modes::transforming` gives the object no template in the
  resolution (no right-click). Since slice M5-S6 a portal warping in adds C Set Rally Point, as every unfinished
  building of a type with a rally point does (`unfinished_grid`, below). A Drone that deploys stays selected as its Beacon in a group of its
  own (`ControlState::set_type`), and a Drones' placement ends once the ActiveGroup is no longer Drones. A greyed-out button (design change 0162) keeps its reason (`PanelSlot::why`, a
  `WhyUnavailable`: the gate's `Reject`, the shields, a command awaiting its target, the type, not in the game
  yet); pressing it, by hotkey or click, issues nothing and sends `CommandRefused`, whose message
  (`feedback::unavailable_text`) the feedback band shows.
- **Gathering** (slice M3-J2; `factions/colonists/units/drone.md > Right-Click Resolution`,
  `factions/colonists/units/drone.md > Set Drop-off Location`, `factions/colonists/economy.md > Egress Node > Egress
  Node ObjectInterfaceState`; design changes 0124, 0136, 0139, 0140, 0155): a Drone's right-click is the sim's one rule,
  per unit like every unit's: a patch (seen or remembered) gives Gather, an own drop-off a carrying Drone's `DropOff`
  and an empty one's Move to it (0140: no empty Drop Off), anything else a Move to it, a unit followed; the pointer
  previews Gather as GatherResources and Drop Off as ReturnResources ("Cursor over resource (resource gatherer
  selected)", "Cursor over drop-off point (resource gatherer carrying resources)"), so Drones that would do both show
  Mixed. S, Set Drop-off (`PanelCommand::SetDropOff`, a common command: every selected Drone's, Q0040 point 9), awaits a
  target: an own Egress Node that isn't transforming (`drop_off_target`, the `Aimed::drop_off` the gate checks) resolves
  to `Resolved::Ability`, one `Command::Ability` of `Ability::SetDropOff` that only updates the AssignedDropOff, the
  orders going on; anything else is an invalid target, and the pointer over a node shows ReturnResources (a placeholder:
  the rules have no AwaitingTarget\[SetDropOffLocation\] row). The Egress Node's grid (`InterfaceTemplate::EgressNode`)
  has F, Select Returning Drones (`PanelCommand::SelectReturningDrones`), a StateOnlyTransition
  (`SlotPress::SelectReturningDrones`, then `ControlState::replace_selection`) that replaces the selection with the
  node's `Sim::returning_drones` (`Conditions::returning`, asked for the ActiveGroup's node: `instance_only`); the
  button shows only while there are some (`Conditions::shows`). Its label, "Drones", keeps the full label size at 720p.
- **The Syndicate's panels** (slice M4-J2; `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`,
  `factions/syndicate/tunnel_menus.md > EjectMenu`, `factions/syndicate/structures/hideout.md >
  ObjectInterfaceState[Hideout]`; design changes 0252, 0257, 0260): `command_ui/interface/templates/syndicate/` has
  `tunnel.rs` (`InterfaceTemplate::Tunnel`, no right-click: W `PanelCommand::ExpandMenu`, E `PanelCommand::EjectMenu`, X
  `PanelCommand::CancelOperation`; Q `PanelCommand::UpgradeTunnel`, slice M4-G: greyed by `Sim::can_upgrade` with the
  gate's reason, "Something blocks the upgrade" never saying what, its price in the hint, "Q: Upgrade Tunnel (2
  Supplies)", `Conditions::price`), `hideout.rs` (the Production grid: W
  trains the Guard, greyed "Requires an Optics Lab" by the gate until the player owns one, slice M4-K; E the
  Grenadier, greyed "Requires a Chemical Plant" alike, slice M4-U), `eject_menu.rs` and `expand_menu.rs`. **A
  placeholder entry** (`templates/planned.rs`, `Planned`) holds its slot greyed with its reason
  (`Conditions::planned`): "Requires an Optics Lab" while the player owns no
  structure of the type its unit or building will require (`Planned::requires`; `Conditions::of` keeps the types
  owned), "Not available yet" otherwise, until the slice that brings its command replaces it. An entry in the game
  asks the gate, and **a prerequisite is the gate's** (`Reject::Requires`, *Tech prerequisites*): no template names
  one. **A menu's order is the sim's data**
  (`sc_sim::defs::EJECT_MENU_ORDER`, design change 0257's "Agent, Guard, Grenadier, …"; `sc_sim::defs::EXPAND_MENU`):
  a slice that adds a unit or an expansion adds it there, not in the client. E enters
  `InterfaceState::EjectMenu`, a StateOnlyTransition (`SlotPress::OpenEjectMenu`): a `PanelCommand::Eject(kind)` tile
  per kind of that list, from Q on, every slot but Z; each is greyed by the gate's answer to its `Command::Tunnel`
  (`Reject::NotInNetwork`, `InTransit` for the transit delay, design change 0288 point 1, `NoTransit` for the tier,
  each with its message) and carries the network's count (`PanelSlot::count`, `Conditions::network_count` from
  `Sim::network_counts`, so the count drops at the press); a kind with none inside keeps its tile, 0 and greyed (design
  change 0299, answering Q0086 point 2; the order is 0285's, point 1). A tile sends `TunnelCommand::Eject` to the Tunnel
  Tabbed to (Tunnels are Ungroupable; `SlotPress::Tunnel`, `OrderOutbox::tunnel`) and the menu stays open (0300, point
  3); Z and Escape go back, not a right-click (0400). Held units leave the selection and stay in their control groups
  (`ControlState::retain_on_map`); a recall selects the members on the map, and a group with none on the map recalls as
  an empty one does: nothing changes (`ControlState::recall_group`; 0299, point 5). A Hideout's rally pointer and marker
  on an own Tunnel or Hideout are every producer's (Move, a green Object marker); the sim decides what the unit does
  (0252).
- **The ExpandMenu and an expansion's placement** (slice M4-F; design changes 0257, 0290 and 0291): W enters
  `InterfaceState::ExpandMenu` (`SlotPress::OpenExpandMenu`), whose grid is `sc_sim::defs::EXPAND_MENU` in the
  EjectMenu's tile slots (`expand_menu::grid`): a `PanelCommand::Expand(kind)` per type in the game, greyed by
  `Sim::can_expand` (`Conditions::of`, its reasons "This Tunnel is busy", "This Tunnel's tier is too low", "Requires
  a ...", "Not enough Space Crystals"), and a `Planned` placeholder per entry not in the game yet, found by name
  (`Planned::expansion`), so a slice that brings an expansion changes the sim's list only: the Laser Turret's C is its
  `Expand` since slice M4-S2, greyed by the same gate ("Requires an Optics Lab", the operation slot), and the one place
  a turret is ordered from since design change 0359 (`tests/commands/agent_ui.rs`). An entry enters `AwaitingPlacement`
  with **`Builder::Tunnel`** for the Tunnel Tabbed to (`SlotPress::AwaitExpansion`); Z goes back to the menu, where the
  panel stays meanwhile (`state_grid`). **The ghost is held within the Tunnel's area** (`Placing::origin`,
  `placing::within`: the cursor outside leaves it at the nearest spot inside; a reading of "follows the cursor within
  the Tunnel Area"), and judged by `Sim::expansion_check`, what the owner knows, in `ghost::check_now` with every
  builder's; the Tunnel's area shows meanwhile, the Tunnel being selected (`TunnelAreaOverlay`; design change 0300: "as
  built"). A click on a green ghost sends `TunnelCommand::Expand` through `OrderOutbox::try_tunnel` and ends placement,
  Shift or not (one operation, no queue); the gate's refusal sends `CommandRefused` (the bank) or `InvalidPlacement`
  (the truth) and placement goes on. X is `CancelOperation` (`TunnelCommand::Cancel`), greyed while there is none
  (`Reject::NoOperation`, "Nothing to cancel"). The expansion's site is any site: `SITE_GRID`, and the look and progress
  bar of `object_render/site.rs`. The Tunnel's InfoPanel shows its operation, "Building Hideout: 25%" or "Upgrading to
  Tier 2: 25%" (slice M4-G, placeholder wording; `activity::operation`), which its X cancels either way.
- **The Agent's panel** (slice M4-J1; `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`; design changes
  0233, 0237, 0257, 0359 and 0398): `templates/syndicate/agent.rs` (`InterfaceTemplate::Agent`) is Q Move, W
  `PanelCommand::Build(Tunnel)`, A Attack, S `PanelCommand::DropOff`, D `PanelCommand::Repair` (slice M5-N8, as the
  Recruit's D) and X Stop, with no Build of a Laser Turret (slice M4-S1's D, which 0359 took away: a turret is
  the ExpandMenu's alone); its right-click is the sim's one rule (*Gathering* above). **W Build** is an instance command
  (`SlotPress::AwaitBuild`): `AwaitingPlacement` with `Builder::Agent` for the ActiveGroup's one Agent (Agents are
  Ungroupable: Tab picks the next one). It is greyed exactly when the gate refuses an unqueued Build of that Agent's
  (`conditions::build_probe`), the placement's own refusal left out; its price (`Sim::build_price`, a `BuildPrice` in
  its resource) is `PanelSlot::price`, which the hint names ("W: Build Tunnel (2 Supplies)"; a Recruit's "W: Build
  Storage (100 Space Crystals)"). A click checks the ghost (`Sim::build_check`; a red one sends `InvalidPlacement`),
  then gives the order through `OrderOutbox::try_give`, which returns the gate's `Reject`: a refused one (the bank,
  "Already building" for a second Build queued to an Agent) sends `CommandRefused` and placement goes on; a sent one,
  queued with Shift, ends it ("each Agent takes exactly one placement"). The client counts no Build still in
  `PendingCommands` in a price or a ghost. **S** awaits an own drop-off in operation (`drop_off_target`), greyed while
  none of its recipients carries a load (`Reject::NothingToDeliver`, "Not carrying resources"). A right-click that
  enters a Tunnel shows `PointerDisplay::Enter` (`cursor::ENTER_COLOR`, a placeholder), a damaged one too (Enter wins
  over Repair, design change 0325); on a damaged own Laser Turret, Repair; a carrying Agent's on its Tunnel
  ReturnResources; on a site, Move. **A site's panel** (`templates/site.rs`, `SITE_GRID`) is X
  `CancelConstruction` alone, for any site (`Conditions::site`: `Object::is_site`, never its kind), which slice M4-F
  reuses, a turret's site among its expansions': an unfinished `GroupType` offers both Cancels (`offers::UNFINISHED`),
  and the gate takes the one that fits. (M5-S6, design change 0396) **An unfinished building of a type with a rally
  point** (`sc_sim::production::takes_rally`, *Production > Rally points*) adds C `SetRallyPoint` next to its X, a
  site's or a transform's (`site::unfinished_grid`): "Its unfinished panel adds **C: Set Rally Point**, next to
  Cancel". `offers` gives it to that type's unfinished group, a common command (the Cancels stay instance only), so
  it goes to all of the group, or to the Tabbed one of a pool building's, as on the finished one (`one_building`);
  the resolution gives the warping portal its template (`Modes::transforming`), a site has its own already, so the
  right-click sets its rally point as on the finished building, `OrderOutbox::produce` sends it, and its marker shows
  while it is selected. Tests: `interface/tests/unfinished.rs`, `interface/tests/colonists.rs`,
  `resolution/tests.rs` and `crates/game/tests/commands/recruit_ui/unfinished_rally.rs` (0396's Verification: an
  Armory being built, its C, the right-click, the marker, kept through completion, its Militant walking there).
- **The Armory's panel** (slice M5-F2; `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`, design
  changes 0337 and 0348): `templates/cults/armory.rs` (`InterfaceTemplate::PoolProducer`, the pool buildings' one
  template, built from the building's `ProductionDef` as the Production template is) is Q, W, E and R
  `PanelCommand::KindMenu(kind)`, A `EjectAll`, X `CancelProduction` and C `SetRallyPoint`. A kind button counts its
  free Recruits (`PanelSlot::count`, `Conditions::pool_count` from `Sim::pool_counts`) and is greyed at 0 (the gate's
  `Reject::NoRecruit`, "No free unit of that kind", the Altar's units being of four types, slice M5-N1b); it enters
  `InterfaceState::KindMenu(kind)` (`SlotPress::OpenKindMenu`), whose grid (`armory::kind_menu`) is a
  `PanelCommand::TrainFrom(unit, kind)` per recipe
  from Q (`ProductionCommand::TrainFrom`: the Armory's Q Militant and W Gunner, the Gunner greyed "Requires a
  Gunworks" by the gate's `Reject::Requires` while its player owns none, slice M5-G2), a unit not in the game yet in
  its slot (`PanelTemplate::planned`, none today), A `PanelCommand::Release(kind)` and Z Back; the menu stays open while
  its entries are used (`still_valid`), greyed by the gate. Eject All is a
  `Command::Holding` (`SlotPress::Hold`, `OrderOutbox::hold`), greyed by the gate ("No free unit inside"). The
  right-click sets the rally point, as a producer's (`object_right_click`). **One building at a time** (design change
  0348 point 5: the Armory is Ungroupable): every command of a pool building's panel goes to the one Tabbed to, X and C
  too (`is_common`, `holding::trains`), where a Barracks shares them with the others selected (0083); a right-click
  still rallies every selected producer, as each selected object resolves it for itself.
- **The Cathedral's and the Disciple's panels** (slice M5-N4a; `factions/cults/structures/cathedral.md > Placeholder
  stats`: "Panel like the Armory's [...]: **Q: Train Disciple**, **A: Release** in the kind menu";
  `factions/cults/units/disciple.md > Placeholder stats`: "BasicCombatUnitInterfaceState, plus **R: Shroud**"): the
  Cathedral's is the Armory's `TEMPLATE` (`kinds/cults/cathedral.rs`), its KindMenus' Q its one recipe. The Disciple's
  (`templates/cults/disciple.rs`) is the BasicCombatUnit's with Shroud in R (slice M5-N4b: `templates::grid` puts
  `PanelCommand::Shroud` in the R of a type that casts it, `UnitDef::shroud`). It is an instance command
  (`PanelCommand::instance_only`), so each press starts the channel of the one Disciple Tabbed to ("each R press
  starts one Disciple's channel; Tab to the next"), Shift queues it (Placeholder), and it is live while the gate would
  take it (`Conditions::of` asks `Sim::validate`: below 10 energy it greys, "Not enough energy"); its A is empty for
  want of a weapon, unless an armed unit is selected with it (`attack_slot`, above). The Recruit's Advanced
  Construct C is the Cathedral's Build, greyed "Requires a Temple" until one stands.
- **The Altar's panel** (slice M5-N1b; `factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`: "a
  three-stage selection: **unit type → current AugmentationState → Augmentation to apply**"): `templates/cults/altar.rs`
  (`InterfaceTemplate::Altar`) is Q, W, E and R `PanelCommand::AugmentMenu(unit)` (Recruit, Militant, Gunner,
  Disciple), A `EjectAll`, X `CancelProduction` and C `SetRallyPoint`. A type tile counts its free units
  (`Conditions::stored_count`, from `Sim::pool_counts_of`), greyed at 0, and enters
  `InterfaceState::AugmentMenu(unit)` (`SlotPress::OpenAugmentMenu`): Q to R `PanelCommand::AugmentState(unit,
  kind)`, each with its count, greyed at 0 (Fully augmented too: it enables with any, 0337), entering
  `InterfaceState::AugmentStateMenu(unit, kind)`: each order of the Altar's `ProductionDef::augment` that the kind
  fits, in its slot (Q Enhanced Constitution, W Improved Metabolism, E Both), A `PanelCommand::ReleaseOf(unit, kind)`
  and Z Back, the orders and the Release greyed by the gate. Each press queues one (`ProductionCommand::Augment`), the
  menu staying open (`still_valid`); Z goes back a stage at a time (`InterfaceState::back`). Its commands go to the
  Altar Tabbed to, as the Armory's, and its right-click sets the rally point.
- **The Garrison's panel** (slice M5-N2; `factions/cults/structures/garrison.md > Placeholder stats > Panel`: "**A:
  Eject All** (everyone inside leaves); **X: Cancel construction** while being built. Right-click resolution: none"):
  `templates/cults/garrison.rs` (`InterfaceTemplate::Garrison`) is A `EjectAll` alone, in the Armory's slot for it,
  greyed by the gate while empty (`Reject::NothingInside`); its site's panel is every site's X (`SITE_GRID`). It is
  Groupable and trains nothing, so Eject All is a common command, to every selected Garrison (`is_common`). It
  resolves no right-click (`object_right_click`: `None`, with the Beacon's and the Tunnel's); the right-click of the
  four kinds it takes in on it is the sim's Enter, under `PointerDisplay::Enter` (the Recruit's template, and the
  BasicCombatUnit one of the Militant, the Gunner and, since slice M5-N4a, the Disciple, all resolve by the sim's one
  rule). Its occupants'
  shots are drawn from where the sim says they leave, the footprint's edge (`SimEvent::Attacked::from`), in the
  style of the weapon they fire there and their owner's colour (`combat_effects::occupant_form`, from
  `Sim::held_unit`: a held attacker is neither on the map nor dead).
- **The Temple's panel** (slice M5-N3; `factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced
  Constitution (next level), **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of a kind [...],
  **V: Eject All** (staff leave), **X: Cancel** (last research)"): `templates/cults/temple.rs`
  (`InterfaceTemplate::Temple`) is Q and W `PanelCommand::Research(tech)` of each line's Level 1, A, S, D and F
  `PanelCommand::ReleaseOne(kind)` (Plain, Constitution, Metabolism, Both, the kind buttons' order), X
  `CancelProduction` and V `EjectAll`. **A key shows its line's next level** as the gate answers
  (`Conditions::key`, a substitution from `Sim::validate` alone, as plan decision T10 has every button; the other is
  (slice M6-SB) a toggle's off form, *The Shadebringer's panel*):
  the first level the gate doesn't refuse as `Researched`; `PanelCommand::Researching(tech)`, labelled "InProg",
  greyed and saying "In progress", while it refuses it as `ResearchUnderWay` (0335's "its key shows "In progress"
  meanwhile"); Level 2 greyed "Requires a Cathedral" while the gate wants one; nothing once the line is completed
  (`core/tech_and_research.md > Research > Shared Rules` 4). So `Conditions::of` validates every level of both lines
  (`templates::offers` offers them all at a Temple). Q's hint prices it, "Q: Research Enhanced Constitution Level 1
  (100 Space Crystals)". A, S, D and F count the staff of their kind (`Conditions::pool_count`, `Sim::pool_counts`
  summed over the Temples selected: **Placeholder**), greyed by the gate at none (`Reject::NoRecruit`, "No free unit
  of that kind"), each press a `HoldCommand::ReleaseOne`. It is Groupable: every command goes to all the Temples
  selected (`is_common`), a Research taken by the one the sim picks. It has no rally point (no C) and resolves no
  right-click
  (`object_right_click`: `None`); a Recruit's right-click on it is the sim's Enter, under `PointerDisplay::Enter`.
  The Recruit's Advanced Construct X is its Build, greyed "Requires an Altar" until one stands.
- **The Shrine's panel and its casts** (slice M5-N5; `factions/cults/structures/shrine.md > Placeholder stats`:
  "**Health regeneration** (hotkey Q)", "**Energy regeneration** (hotkey W)"; "Targeting: AwaitingTarget, left-click
  a point in vision. **A/S/D/F** release one Recruit of a kind, **V: Eject All**"): `templates/cults/shrine.rs`
  (`InterfaceTemplate::Shrine`) is Q and W `PanelCommand::Cast(ability)`, labelled "Health" and "Energy" and named
  after the abilities, A, S, D and F `ReleaseOne(kind)` as the Temple's, and V `EjectAll`; no C (no rally point, a
  Placeholder) and no right-click of its own. A cast is greyed while the gate would refuse it but for its point
  (`Sim::can_cast`, "Not enough energy"; `conditions/probes.rs` holds the panel's probes of the gate). It is Ungroupable
  ("Groupable **false** [...] each press casts from one Shrine"): an instance command (`PanelCommand::instance_only`),
  so each press goes to the one Shrine Tabbed to. It awaits a point (AwaitingTarget; Shift casts it as a plain click
  does and ends the targeting, a Placeholder: a structure has no queue): `MapTarget::seen` is `Sim::sees_point` at
  the ground under the cursor, and a click there resolves to `Resolved::Cast(ability, ground)` whatever stands there
  ("a point": a Placeholder), sent as one `Command::Cast` (`OrderOutbox::cast`); on a point out of vision it is an
  invalid target ("Invalid target", the targeting kept). The pointer is `PointerDisplay::Cast` (`cursor::CAST_COLOR`,
  a placeholder pink) where it would cast, Inactive where not, and the indicators draw the cast's circle at the
  cursor while it is awaited (`indicators/cast.rs`: a faint disc and a ring of its radius on the terrain's surface,
  in the Cast colour over a point in vision, grey out of it; a placeholder look). The Recruit's Advanced Construct V
  is its Build, greyed "Requires a Temple" until one stands.
- **The Radar Station's and the Defense System's panels** (slice M6-GR): `templates/gdo/radar_station.rs`
  (`InterfaceTemplate::RadarStation`) is Q `PanelCommand::Cast(AreaAbility::Scan)`, "Scan", which awaits any point on
  the map (`MapTarget::on_map`; `indicators/cast.rs::takes_point` colours the cursor's 6-grid-unit circle by it),
  greyed while the station cools down (`Sim::can_cast`: `Reject::CoolingDown`, "Cooling down"). Ungroupable (design
  change 0422): each press goes to the one station Tabbed to. `templates/gdo/defense_system.rs`
  (`InterfaceTemplate::DefenseSystem`) is the Extraction Facility's flow for the Missile Turret: Q Build Missile
  Turret, then Q Place, greyed with its progress bar while constructed, X Cancel Construction or "Cancel Ready
  Turret". It is Groupable: a group's Build and Cancels go to every one of its Defense Systems, its panel follows the
  first one's construction, and its Place places the first one's turret (`systems/panel.rs`, a Placeholder). The
  Deployment Center's A and F construct them. The Missile Turret has the turret look (`Look::Turret`) and draws its
  missiles as the Rocketman's (`ShotStyle::Missile`).
- **The Factory's panel** (slice M6-GF): `templates/gdo/factory.rs`, the Production template of its recipes, the
  Raptor in Q and the Twister in W (greyed while the bank holds no Supply: "Not enough Supplies"), its Prospector and
  Artillery Rig held in E and R (`Planned::Prospector`, `Planned::ArtilleryRig`: "Not available yet"), X and C. The
  Deployment Center's Advanced menu constructs the Factory (Q), Advanced Mechanics (W) and the Industrial Powerplant
  (A), each greyed "Requires a ..." without the one before it. A Twister's A-click on a ground unit is an invalid
  target, as the gate refuses it (`Reject::CannotHit`), but beside a unit that can hit it a Move on it (design change
  0461), and its R and F hold its mines' commands, Lay Mine ("Lay") and Buy Mine ("Buy"), greyed "Not available yet"
  until slice M6-GM (`Planned::LayMine`, `Planned::BuyMine`, in the BasicCombatUnit grid by its template's `planned`).
  Tests: `crates/game/tests/commands/factory.rs`.
- **The Airfield's and the Hornet's panels** (slice M6-GH1): `templates/gdo/airfield.rs`, the Production template of
  its recipes, the Hornet in Q ("HT", greyed "Not enough Supplies" with under 2 and "Airfield full" at its limit,
  `Refusal::AirfieldFull`), its Dragonfly held in W (`Planned::Dragonfly`, "DY": "Not available yet"), X and C;
  (slice M6-GH2) the Deployment Center's Advanced menu constructs it in E ("AD"), greyed "Requires an Advanced
  Mechanics" without one. An aircraft's grid
  (`templates::grid`, a kind with ammo) adds **F Return to Base** (`PanelCommand::ReturnToBase`, "Base",
  `OrderKind::ReturnToBase` at once; Shift queues it), greyed "No Airfield assigned" without a base
  (`Reject::NoBase`). With none of the selection able to attack, an aircraft out of ammo shows no A, and its D is
  greyed "Out of ammo". Its right-click on another own Airfield is a Rebase, on its own a Return to Base, both under
  the Enter pointer (a placeholder, `PointerDisplay::of_order`); a Rebase onto a full Airfield gives no order and
  says "Airfield full" (`Reject::AirfieldFull` through `CommandRefused`). Tests: `command_ui/interface/tests/
  airfield.rs`, `command_ui/resolution/tests/airfield.rs`.
- **The Supply Tower's and the Supply Chopper's panels** (slice M6-GS1; `factions/gdo/economy.md > SupplyTower >
  ObjectInterfaceState[SupplyTower]`, `factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`):
  `templates/gdo/supply_tower.rs`, the Production template of its recipe, the Supply Chopper in Q ("SCR"), X and C,
  and (`templates::grid`, a kind with a `SupplyTowerDef`) **S Schedule Deliveries** (`PanelCommand::ScheduleDeliveries`,
  "Sched."), which awaits a station (`resolution::station_target`, under the GatherResources pointer) and sends one
  `Command::Schedule { towers, station }` for every Tower of the ActiveGroup (`OrderOutbox::schedule`; Shift doesn't
  queue it), greyed "No chopper attached" while none of them has one (`Modes::with_attached`,
  `Reject::NoChopperAttached`). A selected Tower's schedule shows as a rally point does, the Gather marker round its
  station and a line to it (`indicators/schedule.rs`). The Deployment Center's Basic menu constructs the Tower in R
  ("SPT"; its `Planned` entry gone), greyed "Requires a Radar Station" without one. `templates/gdo/supply_chopper.rs`
  (`InterfaceTemplate::SupplyChopper`; placeholder letters, from `systems/controls/interface.md > CommandPanel >
  Standard Slot Assignments`) is Q Move, E Hold Position, R Pick Up Supplies ("PickUp"), S Drop Off Supplies ("Drop"),
  D Attach to Tower ("Attach") and X Stop, F left free for slice M6-GS2's Unload All. R, S and D await their targets
  (`resolution::station_target`, `tower_target`) and give `OrderKind::PickUpSupplies`, `DropOffSupplies` and
  `AttachToTower`, which Shift queues; S is greyed "Not carrying resources" while no chopper of the ActiveGroup carries
  Supplies (`Modes::chopper`, `Reject::NothingToDeliver`). A click on a Tower another chopper is attached to is an
  invalid target (the gate's `Reject::TowerTaken`, "Another chopper is attached"), previewed Inactive
  (`resolution::tower_free`). The chopper's right-click is the sim's rule (`Rule::SupplyRun`, on `Abilities::chopper`):
  on a station a Pick Up, on an own Tower that is free (or its own) a Drop Off while it carries Supplies and else an
  Attach, on a taken one a Move; their pointers GatherResources, ReturnResources and Enter (placeholders,
  `PointerDisplay::of_order`), their markers the Gather marker, the Return Resources marker and the Move marker. Its
  carried Supplies show as pips under its HP bar (*Combat effects*). The Barracks' E, the Field Engineer, is greyed "Not
  enough Supplies" too without its Supply (design change 0435). Tests: `command_ui/interface/tests/supply_tower.rs`,
  `command_ui/resolution/tests/supply_run.rs`, `crates/game/tests/commands/construction.rs`.
- **The Prayer-mobile's panel** (slice M5-N11; `factions/cults/units/prayer_mobile.md > Placeholder stats`:
  "ObjectInterfaceState: BasicCombatUnitInterfaceState without attack commands, plus **R: Inspire** and **F:
  Suppress**"): the BasicCombatUnit grid with no D (`PanelTemplate::without_attacks`; its A the selection's, absent
  alone as an unarmed unit's), W Reverse as the Seeker's, and its spells' casts in R and F (`templates::grid`,
  `cast_slot`), labelled "Insp." and "Suppr.". A cast round its caster or ahead of it needs no target
  (`PanelCommand::needs_target` is the ability's `AbilityArea::AtPoint` alone): its press is `SlotPress::Cast`, sent
  at once as one `Command::Cast` from the one Prayer-mobile Tabbed to (Ungroupable, an instance command), its point
  the vehicle's position; greyed by `Sim::can_cast` ("Not enough energy"). The Chop Shop's KindMenus put its training
  in E (`PanelTemplate::trains`, which `cults::armory::kind_menu` honours as `production_grid` does), greyed "Requires
  a Temple"; a pool building's training names its recipe's price in its hint (`Conditions::price`, "E: Build
  Prayer-mobile (125 Space Crystals)"). Its look is `UnitShape::Megaphone` (`object_render/units.rs`: the turned
  Silhouette on four wheels, a megaphone over its front; a placeholder).
- **The Recruit's panel** (slice M5-J1; `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, design
  changes 0322, 0327, 0347 and 0398): `templates/cults/recruit.rs` (`InterfaceTemplate::Recruit`) is Q Move, W and E
  `PanelCommand::BuildMenu` (named "Basic Construct" and "Advanced Construct", `PanelTemplate::names`), A Attack, S
  `PanelCommand::Assist`, D `PanelCommand::Repair` (slice M5-N8) and X Stop. Recruits are Groupable: every
  command goes to all the selected Recruits, the one ActiveGroup (Tab stops on them once). **The Construct menus**
  (`templates/cults/construct_menu.rs`) are the sim's `defs::CONSTRUCT_MENU`, the one table of their fixed slots: a
  type in the game is its `PanelCommand::Build(kind)`, greyed by the gate (its TechPrerequisite, the bank); one not
  yet built by the Recruits (`ConstructEntry::Planned`) held its slot as a `Planned` entry, found by name, until slice
  M5-N6 made the last of them, the Chop Shop, live (greyed "Requires an Armory"), and the client has no placeholder
  for one now (`construct_menu::command`: `None`, the sim's tests ruling one out) (slice M5-G2 made the Gunworks, the
  Machine Shop and the Scrap Hangar
  `Build`s, greyed "Requires an Armory", "Requires a Chop Shop" and "Requires an Airstrip" until those stand, and
  slice M5-N2 the Garrison's D, greyed "Requires an Armory"); Z is Back. A Build enters `AwaitingPlacement` with
  **`Builder::Recruits { queued }`** (`Placing::recruits`): one ghost for the group, judged by
  `ghost::recruits_check` (`Sim::group_build_check`, the gate's own check as far as the player knows, the group left
  out of the occupants: design change 0362, a Recruit of the order on the footprint makes the site wait, so no
  warning). A click gives one `OrderKind::Build` to every
  Recruit of the group through `OrderOutbox::try_give`, paid once; a Shift-click queues it and placement stays while
  Shift is held (`end_queued_placement`), each Build its ticket's one lingering ghost. **S** awaits an own site that
  the recipients build from outside (`resolution::assist_target`) and gives `OrderKind::Assist`. The pointer shows
  `PointerDisplay::Assist` over such a site (`cursor::ASSIST_COLOR`, a placeholder), on a right-click as while S
  awaits. **D** (slice M5-N8, design changes 0322 and 0325) awaits a damaged own object that a recipient repairs
  (`resolution::repair_target`: `Aimed::repair`) and gives `OrderKind::Repair`, which the gate gives to the recipients
  that can repair it (`Reject::CantRepair`); it is never greyed, there being no target to judge before the click. The
  pointer shows `PointerDisplay::Repair` (`cursor::REPAIR_COLOR`, a placeholder) where the right-click gives a Repair,
  and while D awaits over an object that a selected type repairs (`resolution::repaired_by`,
  `sc_sim::repair::repairs`), else Inactive, where a click sends `InvalidTarget` and targeting goes on. A damaged own
  Armory that takes the Recruit in is entered by a right-click, repaired by D (Enter wins, 0325), and repaired by a
  right-click too while it is full, under the Repair pointer (slice M5-S2b, 0378; `tests/commands/recruit_ui/
  repairing.rs` plays 0378's Verification on a Garrison). Enter shows over an
  own Armory in operation that takes the Recruit in, never over its site (0347 point 4). The right-click is the sim's
  one rule (`sc_sim::orders::resolution::RULES`). The InfoPanel shows a carrying Recruit's load to its owner, as a
  Drone's ("Carrying 2 Space Crystals", `info_panel::worker_lines`).
- **The Field Engineer's panel** (slice M5-N9; `factions/gdo/units/field_engineer.md`, design changes 0328 and
  0398): `templates/gdo/field_engineer.rs` (`InterfaceTemplate::FieldEngineer`) is Q Move, W Construct, D Repair and X
  Stop, its A empty, unarmed (the selection's Attack when an armed unit is selected with it). W
  is `PanelCommand::BuildMenu(BuildMenu::Construct)`, a build menu of its own that no other type has, labelled "Constr"
  (six letters, the most a label keeps its size with) and named "Construct" in its hint; the Recruit's and the
  Deployment Center's stay Basic and Advanced. Its menu (`field_engineer::menu`, by `menu_grid`) is Q Bunker and W First
  Aid Tent, both `Planned` until slice M6-GB makes them built (their kinds are registered since slice M6-GK), and Z
  Back. D is the Recruit's Repair, and the right-click the sim's one rule
  (`object_right_click`, as the BasicCombatUnit's): Repair on a damaged own building or vehicle, under
  `PointerDisplay::Repair`. The Barracks' panel is the Production template with the Field Engineer in E
  (`templates/gdo/barracks.rs`). The Career Campus, the Deployment Center's BuildMenu[Basic] D (greyed "Requires a
  Barracks" until one stands), has no panel yet (`PanelTemplate::NONE`).
- **The Rocketman's panel** (slice M5-K; `factions/gdo/units/rocketman.md`, design change 0356): the BasicCombatUnit's
  grid with **R Target Lock** (`PanelCommand::TargetLock`, labelled "Lock"), which `templates::grid` puts in R for a
  kind with a Target Lock (`UnitDef::target_lock`). It awaits its target, a group command (the Rocketman is
  Groupable), and a left-click on an enemy unit or structure the player sees now (`MapTarget::visible`) gives
  `OrderKind::TargetLock` on it to every selected Rocketman, under the Attack's pointer; the ground, own and neutral
  objects, and a structure the player only remembers (Placeholder) are invalid targets. Shift queues it, as any unit
  order. The Barracks trains it in W, greyed "Requires an Arsenal" until one stands; the Arsenal, the Deployment
  Center's BuildMenu[Basic] S (greyed "Requires a Barracks"), has no panel (`PanelTemplate::NONE`).
- **The Drone Operator's panel** (slice M6-GD; `factions/gdo/units/drone_operator.md`, design changes 0421 and
  0441): `templates/gdo/drone_operator.rs` is the BasicCombatUnit's grid without attack commands, with **R Launch
  Drone** (`PanelCommand::Launch`, "Launch", `OrderKind::Launch` at once; Shift queues it) and **F Build Drone**
  (`PanelCommand::Buy(item)`, "DRN" by the item's tag, `Ability::Buy`, with its price in the hint), which
  `templates::grid` puts in R and F for a kind whose stock holds a tethered item. F's button shows the purchase's
  progress bar (`Conditions::purchase`); the gate's answer greys R and F ("The drone is out", "No drone to launch",
  "Already carries a drone", "Already building a drone"). While every recipient's drone is out
  (`Modes::tethered`), Move, Reverse, Patrol and Attack Ground are greyed "The drone is out", and a right-click that
  would move it gives no order and says so (`Reject::DroneOut` through `CommandRefused`); with other units selected
  they take the command. The drone's grid (`templates/gdo/operator_drone.rs`, `InterfaceTemplate::OperatorDrone`) is
  Q Move, A Attack and X Stop, and the right-click the BasicCombatUnit's. **The leash ring** (`indicators/leash.rs`,
  placeholder look): while a launched drone of the local player's, or its operator, is selected, a thin pale blue
  line on the terrain round the operator where the drone's Silhouette meets the leash edge to edge, the operator's
  and the drone's square half extents rounded by the leash (`leash::ring`), each operator's once. The drone's
  explosion is drawn as a blast's ring on the ground (`combat_effects/blasts.rs`). The Barracks trains it in R,
  greyed "Requires a Career Campus" until one stands and "Not enough Supplies" while its owner has no Supply
  (`Refusal::Supplies`; the GDO starts with 1, design change 0436, and its Supply Tower brings more: slice M6-GS1).
- **The Shadebringer's panel** (slice M6-SB; `factions/syndicate/units/shadebringer.md > ObjectInterfaceState`: "the two
  toggles", and its `Placeholder stats`: "Hotkeys: **R** Sludge, **F** Cover"): `templates/syndicate/shadebringer.rs` is
  the BasicCombatUnit's grid without attack commands, its W Reverse, and `templates::grid` puts a `PanelCommand::Toggle
  { lays, on: true }` in R or F for each of its type's tanks (`shadebringer::slot`), so that slice M6-SC's smoke tank
  brings F's toggle by itself (F holds `Planned::Cover`, "Not available yet", until then). A press sends
  `Ability::switching(lays, on)` at once (no target, no queue: the mode-switch rule). **The key shows the way a press
  switches** (`Conditions::key`, `Conditions::on`): on, "Sludge" / "Sludge On", while one of its recipients has it off,
  and off, "Sl.Off" / "Sludge Off", once every one has it on (Placeholders); greyed "Tank empty" by the gate
  (`Reject::TankEmpty`). The Vehicle Assembly trains it in W, greyed "Requires a Vault" until one stands; the Tunnel's
  ExpandMenu builds the Vault in R, greyed "Requires a Hideout"; the Vault has no panel.
- **The Prism's panel** (`factions/syndicate/units/prism.md > Panel`, Q0168: "Q Move, R Thrust, X Stop"):
  `templates/syndicate/prism.rs` is its own grid, with no A until it is armed (slice SR). **R, Thrust**
  (`PanelCommand::Thrust`) is targeted (a left-click on ground gives `OrderKind::Thrust(point)`,
  `resolution::target_order`) and queueable; its probe (`conditions/probes.rs`) asks the gate for a Thrust, so R is
  greyed "Every charge is spent" (`Reject::NoCharges`) once the charges are gone. Its right-click is the sim's one rule:
  on an own Tunnel it moves to it, a one-way unit never entering (`network::one_way`). The Aerospace Assembly trains it
  in E.
