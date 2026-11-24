//! The behaviours' working state — the design's BaseBehaviorState (`systems/controls/behaviors.md >
//! BaseBehaviorState`): what a behaviour keeps from tick to tick. It is state like any other: serialized and
//! hashed, so a save loaded in the middle of a fight continues identically.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::units::Tick;

/// An enemy a unit engages on its own (`> AttackingObject` as a sub-behaviour of idle auto-targeting and of
/// AttackMove).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pursuit {
    pub target: ObjectId,
    /// The unit's current move is the approach to `target` (`MoveGoal::Object`), so the mover's status is this
    /// approach's outcome.
    pub chasing: bool,
}

/// A target a unit gave up on, and the tick until which it doesn't pick it again
/// (`orders::GIVE_UP_RETRY_TICKS`).
pub type Ignored = Option<(ObjectId, Tick)>;

/// `> BaseAutoTargeting > Active during Idle`: what a unit without an order is doing.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Auto {
    /// Scanning for ValidTarget enemies within SightRange.
    Watching { ignored: Ignored },
    /// Attacking an enemy it acquired; `origin` is the IdleOrigin, where it stood when it acquired it.
    Engaging { pursuit: Pursuit, origin: Vec2 },
    /// Beyond IdleLeashDistance: walking back to the IdleOrigin (`> MovingToLocation`, so without scanning).
    Returning,
}

impl Default for Auto {
    fn default() -> Self {
        Auto::Watching { ignored: None }
    }
}

/// `> AttackingObject`: the state of an Attack order.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackOrder {
    /// The unit's current move is the approach to the target.
    pub chasing: bool,
    /// Where the unit's owner last saw the target (`> AttackingObject` 6, design change 0034); `None` until it has.
    pub last_seen: Option<Vec2>,
    /// The target went out of the owner's sight: the unit walks to `last_seen`, and the order completes there.
    pub lost: bool,
}

/// `> MovingToObject`: the state of a Move to an object.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Approach {
    /// The unit's current move is the approach to the object.
    pub moving: bool,
    /// `> Completion`, Follow: the followed unit's position when this unit last stood within proximity of it, or
    /// stopped behind the units around it. It follows again once the unit has left proximity and that spot.
    pub parked: Option<Vec2>,
    /// Where the unit's owner last saw a followed unit (design change 0034, extended to Follow).
    pub last_seen: Option<Vec2>,
    /// The followed unit went out of the owner's sight: the unit walks to `last_seen`, and the order completes.
    pub lost: bool,
}

/// `> AttackMovingToLocation`: one leg towards a point, engaging enemies on the way. An AttackMove order is one
/// leg; a Patrol is one after another.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leg {
    /// TargetLocation: this unit's own destination.
    pub to: Vec2,
    /// PathReference: the route first planned for the leg, from where it began; the leash is measured from it.
    /// Until the path is there (`planned`), the straight line.
    pub reference: Vec<Vec2>,
    /// `reference` is the planned path.
    pub planned: bool,
    /// The unit's current move is the leg's own, to `to`, so the mover's status is the leg's outcome.
    pub marching: bool,
    /// CurrentTarget.
    pub engaged: Option<Pursuit>,
    pub ignored: Ignored,
}

/// `> Patrolling`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patrol {
    /// PatrolOrigin: where the unit was when the Patrol started executing (design change 0036).
    pub origin: Vec2,
    /// PatrolDestination: this unit's own destination.
    pub destination: Vec2,
    /// CurrentLeg: origin-to-destination (`true`) or destination-to-origin.
    pub outbound: bool,
    /// ConsecutiveFailedLegs (`> Failed legs`, design change 0037).
    pub failed_legs: u8,
    pub leg: Leg,
}

/// Where a worker is in its job (`factions/colonists/units.md > Drone > Gathering`: "pick up at X, deliver to Y";
/// `orders::gather`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatherPhase {
    /// On its way to the order's patch, or to where it was ([`Gather::spot`]).
    ToPatch,
    /// At an occupied patch with no other patch free, or as close as it got to a patch it can't reach, since tick
    /// `since` (`> Patch Occupancy (one Drone per patch)`: it "waits right at the patch it tried to mine, within
    /// MiningReach, and mines it as soon as it frees up", design change 0132; "it waits as close as it can get",
    /// 0141). It looks again every `orders::gather::RECHECK_TICKS`, in the ticks that are multiples of it.
    Waiting { since: Tick },
    /// Mining the order's patch, which it occupies: `done` of MiningDuration frames
    /// (`economy::gathering::step` counts them and takes the load).
    Mining { done: u32 },
    /// Carrying its load to the drop-off.
    ToDropOff,
    /// Unloading at [`Gather::node`]: `done` of DropOffDuration frames (`economy::gathering::step` counts them and
    /// credits the bank).
    DroppingOff { done: u32 },
    /// The load was delivered in the last tick's economy phase: the behaviour decides what comes next.
    Delivered,
}

/// The state of a Gather or a Drop Off order (`orders::gather`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gather {
    pub phase: GatherPhase,
    /// The drop-off the worker is returning to, or unloading at: its AssignedDropOff as the return leg read it
    /// (`> Assigned Drop-off`), or a Drop Off order's own.
    pub node: Option<ObjectId>,
    /// The unit's current move is this phase's leg, so the mover's status is the leg's outcome.
    pub moving: bool,
    /// The leg to the drop-off ended short of it: the next one begins no sooner than this tick
    /// (`orders::gather::LEG_RETRY_TICKS`). 0 while no leg has.
    pub retry_at: Tick,
    /// Design change 0134 (`factions/colonists/units.md > Patch Occupancy > A patch that runs out`): where the
    /// order's patch was, its centre, once the patch has left the game. The worker heads there and looks for
    /// another patch when it gets there. Meanwhile the order names a patch that is gone, the one order that may
    /// (`orders::ActiveOrder::live_target`). `None` while the patch is there. Optional in files.
    #[serde(default)]
    pub spot: Option<Vec2>,
    /// Design change 0123 (`> When a Gather ends`): the worker found no patch left within its search radius. It
    /// delivers the load it carries, and then the order is complete. Optional in files.
    #[serde(default)]
    pub ending: bool,
}

impl Gather {
    /// A job that begins in `phase`.
    pub fn new(phase: GatherPhase) -> Self {
        Self {
            phase,
            node: None,
            moving: false,
            retry_at: 0,
            spot: None,
            ending: false,
        }
    }

    /// The job goes on in `phase`, whose leg hasn't begun.
    pub fn enter(&mut self, phase: GatherPhase) {
        self.phase = phase;
        self.moving = false;
        self.retry_at = 0;
    }
}

/// The state of a Deploy order (`orders::deploy`, `crate::deploy`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deploy {
    /// The Drone's walk to its site is over (design change 0117): the step right after phase 5a starts its deployment
    /// or fails the order (`Sim::step_deployments`).
    pub arrived: bool,
}

/// The state of the current order's behaviour.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Behaviour {
    /// `> MovingToLocation`, `> StoppingBehavior`: the mover's status says it all.
    #[default]
    Plain,
    Attack(AttackOrder),
    Approach(Approach),
    AttackMove(Leg),
    Patrol(Patrol),
    /// `> HoldingPosition`: the enemy the unit engages from where it stands.
    Hold {
        target: Option<ObjectId>,
    },
    /// A worker's job, for a Gather and a Drop Off order alike (`factions/colonists/units.md > Drone > Gathering`).
    Gather(Gather),
    /// A Drone on its way to deploy (`factions/colonists/economy.md > Beacon Deployment > Deploy Command`).
    Deploy(Deploy),
}

impl Leg {
    /// The object ids the leg refers to.
    fn targets(&self) -> impl Iterator<Item = ObjectId> + '_ {
        (self.engaged.iter().map(|p| p.target)).chain(self.ignored.iter().map(|&(id, _)| id))
    }

    fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        if self.engaged.is_some_and(|p| is_gone(p.target)) {
            self.engaged = None;
        }
        forget_ignored(&mut self.ignored, is_gone);
    }
}

fn forget_ignored(ignored: &mut Ignored, is_gone: &impl Fn(ObjectId) -> bool) {
    if ignored.is_some_and(|(id, _)| is_gone(id)) {
        *ignored = None;
    }
}

impl Behaviour {
    /// The objects the state refers to, besides the order's own target.
    pub(crate) fn targets(&self) -> Vec<ObjectId> {
        match self {
            Behaviour::Plain | Behaviour::Attack(_) | Behaviour::Approach(_) => Vec::new(),
            Behaviour::AttackMove(leg) => leg.targets().collect(),
            Behaviour::Patrol(patrol) => patrol.leg.targets().collect(),
            Behaviour::Hold { target } => target.iter().copied().collect(),
            Behaviour::Gather(gather) => gather.node.iter().copied().collect(),
            Behaviour::Deploy(_) => Vec::new(),
        }
    }

    /// The removal path (`Sim::remove_objects`): drops every reference to an object that is gone. The behaviour
    /// carries on without it in its next tick.
    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        match self {
            Behaviour::Plain | Behaviour::Attack(_) | Behaviour::Approach(_) | Behaviour::Deploy(_) => {}
            Behaviour::AttackMove(leg) => leg.forget(is_gone),
            Behaviour::Patrol(patrol) => patrol.leg.forget(is_gone),
            Behaviour::Hold { target } => {
                if target.is_some_and(is_gone) {
                    *target = None;
                }
            }
            // The drop-off is gone: the return leg picks another, and an unloading starts over there
            // (`orders::gather`, `economy::gathering::step`).
            Behaviour::Gather(gather) => {
                if gather.node.is_some_and(is_gone) {
                    gather.node = None;
                    gather.moving = false;
                    gather.retry_at = 0;
                }
            }
        }
    }
}

impl Auto {
    pub(crate) fn targets(&self) -> Vec<ObjectId> {
        match self {
            Auto::Watching { ignored } => ignored.iter().map(|&(id, _)| id).collect(),
            Auto::Engaging { pursuit, .. } => vec![pursuit.target],
            Auto::Returning => Vec::new(),
        }
    }

    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        match self {
            Auto::Watching { ignored } => forget_ignored(ignored, is_gone),
            Auto::Engaging { pursuit, .. } if is_gone(pursuit.target) => *self = Auto::default(),
            Auto::Engaging { .. } | Auto::Returning => {}
        }
    }
}

impl StateHash for Pursuit {
    fn hash(&self, h: &mut StateHasher) {
        let Pursuit { target, chasing } = self;
        target.hash(h);
        chasing.hash(h);
    }
}

fn hash_ignored(ignored: &Ignored, h: &mut StateHasher) {
    match ignored {
        None => h.tag(0),
        Some((id, until)) => {
            h.tag(1);
            id.hash(h);
            until.hash(h);
        }
    }
}

impl StateHash for Auto {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Auto::Watching { ignored } => {
                h.tag(0);
                hash_ignored(ignored, h);
            }
            Auto::Engaging { pursuit, origin } => {
                h.tag(1);
                pursuit.hash(h);
                origin.hash(h);
            }
            Auto::Returning => h.tag(2),
        }
    }
}

impl StateHash for AttackOrder {
    fn hash(&self, h: &mut StateHasher) {
        let AttackOrder {
            chasing,
            last_seen,
            lost,
        } = self;
        chasing.hash(h);
        last_seen.hash(h);
        lost.hash(h);
    }
}

impl StateHash for Approach {
    fn hash(&self, h: &mut StateHasher) {
        let Approach {
            moving,
            parked,
            last_seen,
            lost,
        } = self;
        moving.hash(h);
        parked.hash(h);
        last_seen.hash(h);
        lost.hash(h);
    }
}

impl StateHash for Leg {
    fn hash(&self, h: &mut StateHasher) {
        let Leg {
            to,
            reference,
            planned,
            marching,
            engaged,
            ignored,
        } = self;
        to.hash(h);
        reference.hash(h);
        planned.hash(h);
        marching.hash(h);
        engaged.hash(h);
        hash_ignored(ignored, h);
    }
}

impl StateHash for Patrol {
    fn hash(&self, h: &mut StateHasher) {
        let Patrol {
            origin,
            destination,
            outbound,
            failed_legs,
            leg,
        } = self;
        origin.hash(h);
        destination.hash(h);
        outbound.hash(h);
        failed_legs.hash(h);
        leg.hash(h);
    }
}

impl StateHash for GatherPhase {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            GatherPhase::ToPatch => h.tag(0),
            GatherPhase::Waiting { since } => {
                h.tag(1);
                since.hash(h);
            }
            GatherPhase::Mining { done } => {
                h.tag(2);
                done.hash(h);
            }
            GatherPhase::ToDropOff => h.tag(3),
            GatherPhase::DroppingOff { done } => {
                h.tag(4);
                done.hash(h);
            }
            GatherPhase::Delivered => h.tag(5),
        }
    }
}

impl StateHash for Gather {
    fn hash(&self, h: &mut StateHasher) {
        let Gather {
            phase,
            node,
            moving,
            retry_at,
            spot,
            ending,
        } = self;
        phase.hash(h);
        node.hash(h);
        moving.hash(h);
        retry_at.hash(h);
        spot.hash(h);
        ending.hash(h);
    }
}

impl StateHash for Behaviour {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Behaviour::Plain => h.tag(0),
            Behaviour::Attack(state) => {
                h.tag(1);
                state.hash(h);
            }
            Behaviour::Approach(state) => {
                h.tag(2);
                state.hash(h);
            }
            Behaviour::AttackMove(leg) => {
                h.tag(3);
                leg.hash(h);
            }
            Behaviour::Patrol(patrol) => {
                h.tag(4);
                patrol.hash(h);
            }
            Behaviour::Hold { target } => {
                h.tag(5);
                target.hash(h);
            }
            Behaviour::Gather(gather) => {
                h.tag(6);
                gather.hash(h);
            }
            Behaviour::Deploy(Deploy { arrived }) => {
                h.tag(7);
                arrived.hash(h);
            }
        }
    }
}
