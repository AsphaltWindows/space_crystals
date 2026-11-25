//! The behaviours' working state — the design's BaseBehaviorState (`systems/controls/behaviors.md >
//! BaseBehaviorState`): what a behaviour keeps from tick to tick. It is state like any other: serialized and
//! hashed, so a save loaded in the middle of a fight continues identically ([`hash`]).

mod hash;

use glam::Vec2;
use serde::{Deserialize, Serialize};

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
///
/// The fields M3-R and M3-L2 added (`remembered`, `on_hold`, `backing_off`, `too_close`) are left out of files while
/// they are unset, and out of the hash ([`hash`]), so that states saved before they existed still load.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackOrder {
    /// The unit's current move is the approach to the target.
    pub chasing: bool,
    /// The unit's owner has seen the target while the order ran. From then on, the target no longer visible to it
    /// ends the order at once (`> AttackingObject` 6, design change 0200); until then, a target it remembers is
    /// walked to (design change 0139, `remembered`). Optional in files (slice M3-L3).
    #[serde(default)]
    pub seen: bool,
    /// Design change 0139: the owner doesn't see the target but remembers it (a structure), and the unit's current
    /// move is its walk to the target's last-known position. It engages nothing meanwhile, and engages the target
    /// again once the owner sees it.
    #[serde(default, skip_serializing_if = "is_false")]
    pub remembered: bool,
    /// Design change 0150: the unit may not attack now (its Riot Shield is up), so the order is on hold: the unit
    /// follows the target as a Move to it would (`orders::travel::approach`), with this state, and attacks again once
    /// it may.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_hold: Option<Approach>,
    /// Design change 0182 (`> AttackingObject > Target inside MinRange`): the target is inside the unit's MinRange,
    /// and the unit's current move is its back-off from it (`orders::engage`), given when the target stood here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backing_off: Option<Vec2>,
    /// Design change 0182: the target is inside the unit's MinRange and the unit can't move to back off, so it can't
    /// carry the order out. The step right after phase 5a ends the order, with the "Target too close" feedback
    /// (`Sim::step_too_close`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub too_close: bool,
}

/// `> MovingToObject`: the state of a Move to an object. A followed unit that goes out of the owner's sight ends it
/// at once (design change 0200), so nothing of where it was is kept.
///
/// Its `remembered` (slice M3-R) is left out of files while unset, and out of the hash, as [`AttackOrder`]'s.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Approach {
    /// The unit's current move is the approach to the object.
    pub moving: bool,
    /// `> Completion`, Follow: the followed unit's position when this unit last stood within proximity of it, or
    /// stopped behind the units around it. It follows again once the unit has left proximity and that spot.
    pub parked: Option<Vec2>,
    /// Design change 0139: the owner doesn't see the object but remembers it (a structure or a resource), and the
    /// unit's current move is its walk to the object's last-known position.
    #[serde(default, skip_serializing_if = "is_false")]
    pub remembered: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// `> AttackMovingToLocation`: one leg towards a point, engaging enemies on the way. An AttackMove order is one
/// leg; a Patrol is one after another.
///
/// The fields M3-L2 and M3-L4 added (`walking`, `resumed`, `rejected`, `chase`) are left out of files while they are
/// unset, and out of the hash ([`hash`]), as [`AttackOrder`]'s.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leg {
    /// TargetLocation: this unit's own destination.
    pub to: Vec2,
    /// PathReference (design change 0180): the route planned when the leg began, from where it began; the leash is
    /// measured from it. Until the path is there (`planned`), the straight line. Kept after a fight (the walking path
    /// is planned again, `walking`), and replaced by the new path when the leg's own move re-routes, a detour.
    pub reference: Vec<Vec2>,
    /// `reference` is the planned path.
    pub planned: bool,
    /// The unit's current move is the leg's own, to `to`, so the mover's status is the leg's outcome.
    pub marching: bool,
    /// CurrentTarget.
    pub engaged: Option<Pursuit>,
    /// Design change 0180: the path of the leg's own move while that isn't `reference`: the one planned after a fight,
    /// which doesn't replace the reference. Empty while the move's path is the reference, or has yet to come.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub walking: Vec<Vec2>,
    /// The leg's own move was given again after a fight, and its path is yet to come: it will be `walking`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resumed: bool,
    /// Design change 0230, the reach rule (`orders::reach`): the enemies judged out of the unit's reach on this leg,
    /// by ascending id, at most `orders::reach::MAX_REJECTED`. Not a memory of them: each stands only while nothing
    /// its judgement depends on has changed, and saves judging the enemy again meanwhile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected: Vec<Judged>,
    /// The judgement by which the unit goes after `engaged`, out of its weapon range: it is judged again once it
    /// no longer stands (`orders::travel::march`). `None` while the unit has only fought it within range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chase: Option<Judged>,
}

/// Design change 0230: what a judgement of an enemy by AttackMove's reach rule depended on, besides the leg's
/// PathReference (`orders::reach`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Judged {
    pub target: ObjectId,
    /// Where the enemy stood.
    pub at: Vec2,
    /// The version of what the unit's owner knew of the obstacles (`PlayerKnowledge::version`; the sim's
    /// `nav_version` for a unit without an owner).
    pub knowledge: u32,
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
    /// On its way to the order's patch, or to its spot ([`Gather::spot`]).
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
    /// Where the order's patch is, its centre, while the worker can't go by the patch itself: out of its owner's
    /// sight, where the owner remembers it (design change 0139), and once it has left the game, where it was (design
    /// change 0134, `factions/colonists/units.md > Patch Occupancy > A patch that runs out`). Gone to the owner, the
    /// patch is "treated like an occupied one": the worker heads for its spot and looks for another patch there.
    /// Meanwhile the order may name a patch that is gone (`orders::Order::outlives_target`). `None` while the owner
    /// sees the patch. Optional in files.
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

/// The state of an Enter order (`orders::enter`, `crate::network`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enter {
    /// The unit's walk to the entry path is over, wherever it ended: the step right after phase 5a takes the unit
    /// into the network if it got there, and ends the order if it didn't (`Sim::step_entries`).
    pub walked: bool,
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
    /// A unit on its way into a Tunnel (`systems/controls/commands.md > Enter`).
    Enter(Enter),
}

impl Leg {
    /// The object ids the leg refers to.
    fn targets(&self) -> impl Iterator<Item = ObjectId> + '_ {
        let rejected = self.rejected.iter().map(|judged| judged.target);
        let chase = self.chase.iter().map(|judged| judged.target);
        (self.engaged.iter().map(|p| p.target)).chain(rejected).chain(chase)
    }

    fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        if self.engaged.is_some_and(|p| is_gone(p.target)) {
            self.engaged = None;
        }
        self.rejected.retain(|judged| !is_gone(judged.target));
        if self.chase.is_some_and(|judged| is_gone(judged.target)) {
            self.chase = None;
        }
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
            Behaviour::Deploy(_) | Behaviour::Enter(_) => Vec::new(),
        }
    }

    /// The removal path (`Sim::remove_objects`): drops every reference to an object that is gone. The behaviour
    /// carries on without it in its next tick.
    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        match self {
            Behaviour::Plain | Behaviour::Attack(_) | Behaviour::Approach(_) | Behaviour::Deploy(_) => {}
            Behaviour::Enter(_) => {}
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
