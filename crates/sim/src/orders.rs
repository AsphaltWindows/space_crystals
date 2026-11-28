//! The `Orders` capability: a unit's command queue and the behaviour of its current order —
//! `systems/controls/commands.md > Unit Command`, `systems/controls/behaviors.md`, `docs/movement.md > 3`,
//! `docs/movement.md > 4`.
//!
//! A player's `Command::Order` (phase 4) only writes queues ([`Orders::give`]) and decides where each unit of a
//! group goes ([`destinations`], the magic box); it never moves anything. Phase 5a ([`update`]) completes orders,
//! starts the next one and runs the behaviours, which write `Mover::goal` and read `Mover::status`.
//!
//! **Behaviours**, each one tick at a time, on the snapshot of the game that [`world`] takes when the phase
//! begins:
//! - [`engage`]: AttackingObject (the approach, the chase and its ends), idle auto-targeting with its leash, and
//!   Hold Position;
//! - [`travel`]: MovingToObject (Follow for a unit), AttackMovingToLocation and Patrolling;
//! - Move to a point, and Stop, need only the mover's status;
//! - [`gather`]: a worker's job, the Gather and Drop Off orders of `factions/colonists/units/drone.md`;
//! - [`build`]: a worker's walk to the site of what it builds, a Drone's Beacon (`crate::deploy`) or an Agent's Tunnel
//!   (`crate::construction::site`), or a Recruit's walk to its own spot beside a group's site (slice M5-E);
//! - [`assist`]: a Recruit building a site from outside, from its own spot beside it (slice M5-E,
//!   `crate::spot_work`);
//! - [`repair`]: a repairer at a damaged own building, from its own spot beside it, or with a damaged own vehicle,
//!   which it follows (slice M5-N8, `crate::repair`);
//! - [`enter`]: a unit's walk into the entry path of the structure being entered, a Tunnel or a building that holds
//!   units (`crate::network`, `crate::holding`), or (M6-ST1) up to the carrier it boards (`crate::carry`);
//! - [`attack_ground`]: AttackingLocation, an Attack Ground's walk into range and its shots, until another order
//!   (slice M4-O; design change 0293);
//! - [`shroud`]: a Disciple channelling Shroud where it stands, its energy drained each tick (slice M5-N4b); the
//!   phase lists the units that channel as it ends, for their clouds (`crate::shroud`);
//! - [`target_lock`]: a Rocketman locking onto its target and firing at it where it stands, until the lock breaks
//!   (slice M5-K; the lock itself is phase 5e's, `crate::combat::target_lock`);
//! - [`launch`]: a Drone Operator stopping to launch its drone, which the step right after the phase does (slice
//!   M6-GD, `crate::tether`);
//! - [`air_base`]: an aircraft flying to its Airfield, a new one it was reassigned to or the one it returns to (slice
//!   M6-GH1, `crate::aircraft`);
//! - [`vehicle_deploy`]: a vehicle standing while it deploys into its pair's other type, which the step right before
//!   the phase counts and completes (slice M6-GA, `crate::vehicle_deploy`);
//! - [`unload`]: a carrier stopping to unload its passengers, which a step of the economy phase does once it is at
//!   rest (slice M6-ST1, `crate::carry`);
//! - [`supply_run`]: a Supply Chopper flying to a station's or a tower's spot, which the step right after the phase
//!   lands it on (`crate::supply_run`).
//!
//! What a unit does by default against a target, a right-click's and a rally point's order, is [`resolution`]'s.
//!
//! Their working state ([`state`], the design's BaseBehaviorState) is part of the order, or of the unit while it
//! has none ([`Orders::auto`]).

mod aim;
mod air_base;
mod assist;
mod attack_ground;
pub mod build;
pub(crate) mod destinations;
mod engage;
mod enter;
pub(crate) mod gather;
mod hash;
pub(crate) mod launch;
pub(crate) mod reach;
mod repair;
pub mod resolution;
mod run;
pub(crate) mod shroud;
pub mod state;
mod supply_run;
mod target_lock;
mod travel;
mod turret;
pub(crate) mod unload;
pub(crate) mod vehicle_deploy;
pub(crate) mod world;

use std::collections::VecDeque;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::scenario::BuildingOrientation;
use crate::units::{Tick, TilePos};

pub use reach::{JUDGE_PATHS_PER_TICK, MAX_REJECTED, ReachCounters};
pub(crate) use run::update;
pub use state::{Auto, Behaviour};

/// `docs/movement.md > 2`, G13: orders waiting in one unit's queue, besides the current one. A technical cap: an
/// order queued beyond it is dropped for that unit.
pub const MAX_QUEUED_ORDERS: usize = 32;

/// Units one order command may name, duplicates included. Technical: selections have no size limit
/// (`systems/controls/selection.md`), so this only bounds the work of one command.
pub const MAX_ORDER_UNITS: usize = 1024;

/// The units of one order command share it (`docs/movement.md > 3`). Allocated from `Sim::next_group`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderGroup(pub u32);

/// What a move order goes to (`systems/controls/commands.md > Move`: a location or an object).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    /// A point, space units: where the player clicked.
    Point(Vec2),
    Object(ObjectId),
}

/// The orders of `systems/controls/commands.md` that units have so far (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OrderKind {
    /// `systems/controls/commands.md > Move`.
    Move(Target),
    /// `systems/controls/commands.md > Attack`: a destructible object.
    Attack(ObjectId),
    /// `systems/controls/commands.md > AttackMove`: a point.
    AttackMove(Vec2),
    /// `systems/controls/commands.md > Patrol`: the far end of the patrol, a point.
    Patrol(Vec2),
    /// `systems/controls/commands.md > HoldPosition`.
    Hold,
    /// `systems/controls/commands.md > Stop`.
    Stop,
    /// `factions/colonists/units/drone.md > Unit Commands > Gather`: a worker gathers from a Space Crystals patch
    /// and delivers to its AssignedDropOff, again and again ([`gather`]). The patch is the one the worker heads for
    /// now: the order follows it to another when its patch is occupied, out of reach or gone (design change 0132:
    /// "its gather target follows where it actually mined"). A patch that is gone stays named while the worker
    /// heads for where it was (design change 0134, `state::Gather::spot`).
    Gather(ObjectId),
    /// `factions/colonists/units/drone.md > Unit Commands > Drop Off Resources`: a worker delivers the load it carries
    /// to this drop-off of its owner's, an Egress Node ([`gather`]).
    DropOff(ObjectId),
    /// `factions/colonists/economy.md > Beacon Deployment > Deploy Command`: a Drone walks to the site of a Beacon
    /// whose footprint's top-left tile is `origin` and deploys into it there (`crate::deploy`, [`build`]).
    Deploy { origin: TilePos },
    /// `systems/controls/commands.md > Enter`: "Order the unit to enter an own structure that can hold units", its
    /// TargetObject "the structure being entered" (design change 0319): this structure of its owner's, through its
    /// entry path ([`enter`]). A Tunnel takes the unit into the network ("The unit walks to Side A and enters the
    /// Tunnel Network", `factions/syndicate/tunnels.md > Tunnel Network > Walking in`; `crate::network`), a building
    /// that holds units into its pool (slice M5-F1, `crate::holding`).
    Enter(ObjectId),
    /// (M4-O) `systems/controls/commands.md > AttackGround`: "Attack a specific ground location", the point clicked,
    /// space units, with no snapping (plan decision T47). Only units whose attack can target the ground take it
    /// (`combat::can_target_ground`, the gate); [`attack_ground`] runs it.
    AttackGround(Vec2),
    /// `core/entities.md > Placement Validation > Worker-Built Structures` (plan decision T51, slice M4-E): a worker
    /// walks to the site of a structure of `kind` whose footprint's top-left tile is `origin`, turned by `orientation`,
    /// and builds it there (`crate::construction::site`, [`build`]): an Agent's Tunnel
    /// (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`). Its price is paid when it is given
    /// (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, design change 0233).
    Build {
        kind: ObjectKind,
        origin: TilePos,
        #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
        orientation: BuildingOrientation,
    },
    /// `factions/cults/economy.md > Assist Construction Command` (slice M5-E): "Left Click on an in-progress building";
    /// "Once the selected Recruits reach the building they join construction from outside, increasing build speed".
    /// The unit walks to its own spot beside this site's footprint and builds it while it stands within reach of the
    /// footprint's edge ([`assist`], `crate::spot_work`), until the site is complete or gone. Each Build of a group's
    /// becomes one as its site starts (plan decision T59, `sim/builds/shared.rs`).
    Assist(ObjectId),
    /// (M5-N8) `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**D: Repair**
    /// — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair it" (the Agent's
    /// D: "a damaged own Tunnel or Laser Turret"). The unit repairs this object (`crate::repair`): from its own spot
    /// beside a building's footprint, as an Assist, or within reach of a vehicle, which it follows ([`repair`]), until
    /// the object is at full HP, gone or no longer its owner's, or its owner runs out of Space Crystals for it.
    Repair(ObjectId),
    /// (M5-N4b) `factions/cults/units/disciple.md > Spells > Shroud`: "A **channeling** spell: while the Disciple
    /// channels it, its energy **drains gradually**, and it creates a **cloud**" (plan decision T76: an order, so that
    /// "a **cancel** command (Stop or any other order)" ends it, and Shift-queued orders wait behind it). The unit
    /// stands ("The Disciple **can't move while channeling**") and pays its drain each tick until another order
    /// replaces it or its energy can't pay a tick's drain ([`shroud`]); its cloud is `crate::shroud`'s. Only a unit
    /// whose type casts Shroud takes it (`UnitDef::spells`), with the energy it needs to start (the gate).
    Shroud,
    /// (M5-V2) `systems/controls/commands.md > Reverse`: "Order the unit to reverse to a target location", a point.
    /// "Only available to bases with CanReverse = true": only units that can back up take it (`UnitDef::can_reverse`,
    /// the gate). Its behaviour is `systems/controls/command_behaviors.md > ReversingToLocation`, "Identical to
    /// MovingToLocation except the plan uses Reversing locomotion instead of Moving" ([`run`]: a Move to the point whose
    /// leg drives backwards, `Mover::reverses`). A location-targeted movement order, so a group's takes the magic box
    /// as a Move's does ([`Order::slot`]; the design's list, "Move, Attack Move, Patrol", predates it).
    Reverse(Vec2),
    /// (M5-K) `factions/gdo/units/rocketman.md > Target Lock (targeted ability)`: "The player orders Target Lock on one
    /// enemy unit", or structure ("Target Lock can target structures (user, 2026-11-26) as well as units"). An order
    /// (the Director's reading, by plan decision T76's Shroud), so that "the Rocketman is given **any** order [...]
    /// **without Shift**" breaks it, and "a Shift-queued order waits behind the lock and runs when it ends" (design
    /// change 0438, answering Q0171). The unit stands and locks onto this object, then fires its homing missiles at it,
    /// until the lock breaks (`crate::combat::target_lock`), which ends the order ([`target_lock`]). Only a unit whose
    /// type has a Target Lock takes it (`UnitDef::target_lock`, the gate).
    TargetLock(ObjectId),
    /// (M6-GD) `factions/gdo/units/drone_operator.md > Launching and piloting`: "The Drone Operator launches a drone
    /// from its stock" (the operator's R; plan decision T91: an order, untargeted and queueable, so that it waits behind
    /// a Move). The unit stops, and the step right after phase 5a launches its drone over it, ending the order
    /// ([`launch`], `crate::tether`). Only a unit whose stock holds drones takes it, with one in stock and none out
    /// (the gate).
    Launch,
    /// (M6-GH1) `systems/mechanics/aircraft_landing.md > Assigned base`: "**Reassigning**: the player can move an
    /// aircraft's assignment to another Airfield, if that Airfield has room [...] Input: select the aircraft and
    /// **right-click the other Airfield**. The aircraft is reassigned and flies there to land" (plan decision T88).
    /// Applied as it is given (`Sim::give_order`): this Airfield is the aircraft's base from then on
    /// (`crate::aircraft`), and the order flies it there ([`air_base`]). Only an aircraft that rearms at an Airfield
    /// takes it, to an own Airfield in operation that has room for it (the gate).
    Rebase(ObjectId),
    /// (M6-GH1) `systems/mechanics/aircraft_landing.md > Assigned base`: "**Return to Base** command: sends the
    /// aircraft to its assigned Airfield to land and rearm (hotkey **F**, placeholder). Right-clicking its own Airfield
    /// does the same"; and the automatic return, "when an aircraft runs out of ammo, it automatically flies back to its
    /// assigned Airfield", which "**can be interrupted** by giving the aircraft any other order" (plan decision T88:
    /// pushed by itself as its ammo reaches 0, an order that any new order replaces; `crate::aircraft`). The aircraft
    /// flies to its base ([`air_base`]), and (M6-GH2) lands there and rearms (`crate::landing`). Only an aircraft with
    /// a base takes it (the gate): "An unassigned aircraft has no base [...] Return to Base is unavailable until it is
    /// reassigned".
    ReturnToBase,
    /// (M6-GA) `factions/gdo/units/artillery_rig.md > Modes`: "**Deploy is an order, not a mode switch** [...] the
    /// deploying process is what actually changes the unit: it stays the mobile type until the process completes"
    /// (plan decisions T44 and T87, `crate::vehicle_deploy`): the unit stands for its type's frames
    /// (`UnitDef::vehicle_deploy`), then becomes a unit of type `into`, its pair's other type: the Artillery Rig's
    /// Deploy into the Artillery Rig (Deployed), whose Undeploy is this order into the Artillery Rig. "Queueable":
    /// Shift queues it behind the unit's orders; "Not cancellable": once it has started, the orders given meanwhile are
    /// held behind it (`Orders::give`), and checked against the new type as it completes. Only a unit whose type
    /// deploys into `into` takes it, and not one deploying already (the gate). `into` names the type (a reading: T44's
    /// order has no payload), so that a Deploy pressed twice doesn't undeploy, and the panel's Deploy and Undeploy are
    /// two commands.
    VehicleDeploy { into: ObjectKind },
    /// (M6-ST1) `factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Unloading**: **F: Unload
    /// All** (above ground only; they come out around it)" (plan decision T94, `crate::carry`): an order, untargeted
    /// and queueable, as the Launch is (plan decision T91), so that it waits behind a Move. The carrier stops, and the
    /// economy phase's step puts its passengers on the map round it once it is at rest, ending the order
    /// ([`unload`]). Only a carrier that loads now takes it (`carry::loads`: the Transport above ground), with a
    /// passenger aboard (the gate).
    UnloadAll,
    /// `factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`: "Right-click
    /// SupplyDeliveryStation: issues PickUpSupplies command (fly to SDS, land, pick up all available supplies)"
    /// (`crate::supply_run`). The chopper flies to the station's spot and lands on it once Supplies are there or due by
    /// the time it is down; down, it takes all the Supplies no worker's pick-up has claimed, and lifts off
    /// ([`supply_run`], `economy::supply_run`). Only a Supply Chopper takes it, on a station its owner knows (the
    /// gate).
    PickUpSupplies(ObjectId),
    /// `factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`: "Right-click own
    /// SupplyTower (when carrying supplies): issues DropOffSupplies command (fly to tower, land, drop off all carried
    /// supplies, immediately lift off — unless this is the chopper's attached tower)". Only a chopper that carries
    /// Supplies takes it, on an own Supply Tower in operation with no other chopper attached (the gate).
    DropOffSupplies(ObjectId),
    /// `factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`: "Right-click own
    /// SupplyTower (when not carrying supplies): issues AttachToTower command (fly to tower, land, attach)". The
    /// chopper is attached as the order starts, evicting any other chopper down on the tower
    /// (`factions/gdo/economy.md > SupplyTower > Landing and Attachment`: "When a Supply Chopper attaches to a Supply
    /// Tower, it immediately evicts any existing landed Supply Chopper that is not the attached chopper";
    /// `Sim::step_supply_runs`), flies there and lands; the order ends once it is down. Only a Supply Chopper takes it,
    /// on an own Supply Tower in operation with no other chopper attached (the gate).
    AttachToTower(ObjectId),
}

impl OrderKind {
    /// The object the order names, if any.
    pub fn target(self) -> Option<ObjectId> {
        match self {
            OrderKind::Move(Target::Object(id))
            | OrderKind::Attack(id)
            | OrderKind::Gather(id)
            | OrderKind::DropOff(id)
            | OrderKind::Enter(id)
            | OrderKind::Assist(id)
            | OrderKind::Repair(id)
            | OrderKind::TargetLock(id)
            | OrderKind::Rebase(id)
            | OrderKind::PickUpSupplies(id)
            | OrderKind::DropOffSupplies(id)
            | OrderKind::AttachToTower(id) => Some(id),
            OrderKind::Move(Target::Point(_)) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => None,
            OrderKind::Hold | OrderKind::Stop | OrderKind::Deploy { .. } | OrderKind::Build { .. } => None,
            OrderKind::AttackGround(_) | OrderKind::Shroud | OrderKind::Reverse(_) | OrderKind::Launch => None,
            // (M6-GH1) Its base is the aircraft's (`crate::aircraft`), not the order's.
            OrderKind::ReturnToBase => None,
            // (M6-GA) It names a type, not an object.
            OrderKind::VehicleDeploy { .. } => None,
            // (M6-ST1) Its passengers are the carrier's (`crate::carry`).
            OrderKind::UnloadAll => None,
        }
    }
}

/// One unit's order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub kind: OrderKind,
    pub group: OrderGroup,
    /// For a location order (Move to a point, AttackMove, Patrol, (M5-V2) Reverse) given to a group: this unit's own destination,
    /// its place in the group's shape (`systems/controls/commands.md > Group Destinations (Magic Box)`,
    /// [`destinations`]). `None` means the clicked point itself: a single unit, or a click inside the group's box.
    pub slot: Option<Vec2>,
}

impl Order {
    /// The point a location order sends this unit to: its own destination, else the clicked point.
    pub fn destination(&self) -> Option<Vec2> {
        match self.kind {
            OrderKind::Move(Target::Point(click))
            | OrderKind::AttackMove(click)
            | OrderKind::Patrol(click)
            | OrderKind::Reverse(click) => Some(self.slot.unwrap_or(click)),
            OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => None,
            OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => None,
            // Its spot is the spots' step's (`crate::spot_work`), not the order's; a vehicle's repairer follows it.
            OrderKind::Build { .. } | OrderKind::Assist(_) | OrderKind::Repair(_) => None,
            // The unit goes only as near the location as its attack needs (`attack_ground`).
            OrderKind::Enter(_) | OrderKind::AttackGround(_) => None,
            // (M5-N4b) A channel holds the unit where it stands; (M5-K) so does a Target Lock, and (M6-GD) a Launch.
            OrderKind::Shroud | OrderKind::TargetLock(_) | OrderKind::Launch => None,
            // (M6-GH1) The aircraft goes to its Airfield (`air_base`).
            OrderKind::Rebase(_) | OrderKind::ReturnToBase => None,
            // (M6-GA) The unit deploys where it stands (`vehicle_deploy`), and (M6-ST1) a carrier unloads (`unload`).
            OrderKind::VehicleDeploy { .. } | OrderKind::UnloadAll => None,
            // The chopper goes to its target's spot (`supply_run`).
            OrderKind::PickUpSupplies(_) | OrderKind::DropOffSupplies(_) | OrderKind::AttachToTower(_) => None,
        }
    }

    /// The object the order names, if any.
    pub fn target(&self) -> Option<ObjectId> {
        self.kind.target()
    }

    /// The one rule for an order that names an object no longer in the game (design changes 0134 and 0139): whether
    /// this order goes on naming its target ([`Order::target`]) once the target has left the game. `state` is its
    /// behaviour's if it is the current order; `remembered`, whether its holder's owner remembers the target out of
    /// its sight (`PlayerVision::remembered_record`). The removal path (`Sim::remove_objects`), the end of the tick
    /// (`Sim::follow_targets`) and `testing::dangling_references` all go by it; every other reference to an object
    /// that is gone goes with the object.
    /// - **Remembered** (0139, `core/vision.md > Explored > Targeting remembered objects`: "Nothing about the real
    ///   state leaks"): an Attack, a Move to an object and a Gather go by what their holder's owner knows of the
    ///   target (`world::World::known`): to its last-known position, as if it stood, until the owner sees its place.
    ///   Then each ends "the way it would for a vanished target".
    /// - **A Gather on its way to a spot** (`state::Gather::spot`: where its patch is or was, as its owner last knew
    ///   it): it goes on there and looks round, whatever its owner knows now. That is how a Gather ends for a vanished
    ///   target, a patch that ran out (design change 0134) or one seen gone (0139: "Gather: the redirect rules").
    pub fn outlives_target(&self, state: Option<&Behaviour>, remembered: bool) -> bool {
        let to_spot = matches!(state, Some(Behaviour::Gather(gather)) if gather.spot.is_some());
        let by_knowledge = matches!(
            self.kind,
            OrderKind::Attack(_) | OrderKind::Move(Target::Object(_)) | OrderKind::Gather(_)
        );
        to_spot || (remembered && by_knowledge)
    }
}

/// The order a unit is carrying out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveOrder {
    pub order: Order,
    /// The tick it started.
    pub since: Tick,
    /// What its behaviour keeps from tick to tick (`systems/controls/behaviors.md > BaseBehaviorState`).
    pub state: Behaviour,
}

/// A unit's orders (`systems/controls/commands.md > Unit Command`): the current one, and the queue of those
/// that start, in order, when it completes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Orders {
    /// At most [`MAX_QUEUED_ORDERS`].
    pub queue: VecDeque<Order>,
    pub current: Option<ActiveOrder>,
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`: what the unit does on its own
    /// while it has no order. Every order that starts resets it.
    pub auto: Auto,
}

impl Orders {
    /// `systems/controls/commands.md > Unit Command`: with Shift (`queue`) the order is appended to the queue;
    /// without, it replaces the whole queue and the current order. Returns `false` if the queue was full and the
    /// order was dropped. The new order starts in phase 5a. (M6-GA) But for a vehicle deploy under way
    /// ([`Orders::deploying`]), which it can't replace (`factions/gdo/units/artillery_rig.md > Modes`: "**Not
    /// cancellable** [...] once deploying has started it always runs to completion; Stop doesn't interrupt it"): the
    /// order goes behind it, and replaces the queue, the order held before it and those queued after that included
    /// ("the **last order given** (without Shift; each new one replaces the previous) is **held**"), or with Shift
    /// joins the queue ("Shift-queued orders line up behind it as usual").
    pub fn give(&mut self, order: Order, queue: bool) -> bool {
        if !queue {
            self.queue.clear();
            if !self.deploying() {
                self.current = None;
            }
        }
        if self.queue.len() >= MAX_QUEUED_ORDERS {
            return false;
        }
        self.queue.push_back(order);
        true
    }

    /// (M6-GA) Whether its current order is a vehicle deploy under way (`crate::vehicle_deploy`): one that has started,
    /// its frames counted by its behaviour's state. A deploy queued behind other orders hasn't.
    pub fn deploying(&self) -> bool {
        (self.current.as_ref()).is_some_and(|active| {
            matches!(
                (active.order.kind, &active.state),
                (OrderKind::VehicleDeploy { .. }, Behaviour::VehicleDeploy(_))
            )
        })
    }

    /// Whether the unit has no order at all. It may still be fighting or returning by itself ([`Orders::auto`]).
    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.queue.is_empty()
    }

    /// The objects the behaviours' state refers to, besides the targets the orders themselves name: the enemies a
    /// unit engaged by itself.
    pub fn behaviour_targets(&self) -> Vec<ObjectId> {
        let mut ids = self.auto.targets();
        if let Some(active) = &self.current {
            ids.extend(active.state.targets());
        }
        ids
    }

    /// The removal path (`Sim::remove_objects`): the behaviours' state drops every reference to an object for which
    /// `is_gone` holds. Orders that name one are the removal path's own business.
    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        self.auto.forget(is_gone);
        if let Some(active) = &mut self.current {
            active.state.forget(is_gone);
        }
    }
}

#[cfg(test)]
mod tests;
