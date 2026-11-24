//! The command gate (`ARCHITECTURE.md > Commands, events and replays`): the one check every command passes before
//! it is applied. A rejected command changes nothing and never panics the sim.
//!
//! This file holds what every command shares: the reasons for a rejection ([`Reject`]), what a passed command
//! resolved ([`Checked`]), the dispatch by command ([`Sim::check`]) and the lookups of the objects a command names.
//! Each family of commands has its own checks (plan decision T28): orders and abilities in `gate/orders.rs`, debug
//! commands in `gate/debug.rs`, production in `sim/producers.rs`, construction in `sim/construction.rs`.

mod debug;
mod orders;

use std::fmt;

use glam::Vec2;

use super::Sim;
use super::construction::Resolved;
use super::producers::ProductionPlan;
use crate::command::{Ability, Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::orders::{MAX_ORDER_UNITS, OrderKind};
use crate::placement::PlacementProblem;
use crate::scenario::SpawnProblem;

/// Why the sim ignores a command ([`Sim::validate`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The issuing player, or the player a debug command names, is not in the game.
    UnknownPlayer(PlayerId),
    /// The game is over (`core/game_rules.md > End of Game`, `Sim::outcome`): the sim accepts no command any more.
    GameOver,
    /// The issuing player is defeated, and nothing they had can be commanded again (`core/game_rules.md > On
    /// Defeat`, design change 0101); or a debug spawn would give a defeated player an object.
    Defeated(PlayerId),
    /// Debug commands are off in this game ([`super::SimConfig::debug`]).
    DebugDisabled,
    /// No object has this id: it never existed, or it is gone.
    UnknownObject(ObjectId),
    /// The object belongs to someone else, and only its owner may order it.
    NotOwned(ObjectId),
    /// The object can't be created as asked.
    Spawn(SpawnProblem),
    /// Every object id is used up.
    OutOfIds,
    /// A tile-based object keeps its placement: only units move.
    TileObject(ObjectId),
    /// The position is not finite or lies off the map.
    OffMap,
    /// None of the units an order names exists any more (or takes orders).
    NoUnits,
    /// An order names more than [`MAX_ORDER_UNITS`] units.
    TooManyUnits,
    /// The order's target can't be attacked: it is indestructible
    /// (`systems/controls/commands.md > Attack`: a destructible object).
    NotAttackable(ObjectId),
    /// The order's target isn't in the issuing player's sight (`core/vision.md > Explored Selectability`; question
    /// Q0017, point 2).
    NotVisible(ObjectId),
    /// None of the units an ability names has that ability.
    LacksAbility,
    /// None of the units an Attack order names can attack now: their Riot Shields are up
    /// (`factions/gdo/units.md > Peacekeeper > Riot Shield`: "Attack orders are not possible").
    CannotAttack,
    /// A unit's box there would overlap a blocked tile, a blocking footprint or the map's edge
    /// (`docs/movement.md > 3`).
    Blocked,
    /// A unit's box there would overlap this unit of its layer, or a blocking footprint would cover this unit
    /// (`docs/movement.md > 3`).
    Occupied(ObjectId),
    /// A footprint there would overlap this object's (`core/entities.md > Placement Validation`): only a plate
    /// shares its tile, with the uncovered patch it is placed onto (`ObjectKind::may_overlap`).
    Overlaps(ObjectId),
    /// None of the objects a production command names produces units (or exists any more).
    NoProducers,
    /// None of the producers can produce a unit of this kind (`factions/gdo/structures.md > Barracks > Produces`).
    CannotProduce(ObjectKind),
    /// The BuildQueue of every producer that could train the unit is full (design change 0081: 20 with the unit in
    /// production).
    QueueFull,
    /// The player's Space Crystals don't cover the unit, or the building to construct ("Only available if player
    /// has sufficient Space Crystals", `factions/gdo/structures.md > DeploymentCenter >
    /// ObjectInterfaceState[DeploymentCenter]`).
    InsufficientCrystals,
    /// The unit would take the player's Unit Control past the cap (`factions/gdo/overview.md > GDOResources > Unit
    /// Control`, design change 0082).
    UnitControlCap,
    /// Cancel Production: no producer has anything queued.
    NothingQueued,
    /// None of the objects a construction command names is one of the player's constructors that exists
    /// (`crate::construction`).
    NoConstructor,
    /// A Place names more than one constructor: one building is placed by one constructor (design change 0071).
    SeveralConstructors,
    /// The building isn't on the constructor's `Constructs` list (`factions/gdo/structures.md > DeploymentCenter >
    /// Constructs`, `factions/gdo/economy.md > ExtractionFacility > Constructs`).
    CannotConstruct(ObjectKind),
    /// The constructor is constructing, or holds a building ready to place: "Other build options unavailable"
    /// (`> ObjectInterfaceState[DeploymentCenter]`).
    ConstructorBusy(ObjectId),
    /// A Cancel Construction to a constructor that isn't constructing.
    NotConstructing,
    /// A Cancel Ready Building or a Place to a constructor that holds no building ready to place.
    NothingReady,
    /// The ready building can't be placed there (`core/entities.md > Placement Validation`, `crate::placement`): the
    /// first problem the placement check finds. When a Place is refused for this as it applies, the sim says why
    /// (`SimEvent::PlacementRefused`).
    Unplaceable(PlacementProblem),
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.0),
            Reject::GameOver => write!(f, "the game is over"),
            Reject::Defeated(p) => write!(f, "player {} is defeated", p.0),
            Reject::DebugDisabled => write!(f, "debug commands are off"),
            Reject::UnknownObject(id) => write!(f, "object {} doesn't exist", id.0),
            Reject::NotOwned(id) => write!(f, "object {} belongs to someone else", id.0),
            Reject::Spawn(problem) => write!(f, "can't spawn: {problem}"),
            Reject::OutOfIds => write!(f, "every object id is used up"),
            Reject::TileObject(id) => write!(f, "object {} is tile-based and can't move", id.0),
            Reject::OffMap => write!(f, "the position is off the map"),
            Reject::NoUnits => write!(f, "none of the ordered units exists"),
            Reject::TooManyUnits => write!(f, "an order may name at most {MAX_ORDER_UNITS} units"),
            Reject::NotAttackable(id) => write!(f, "object {} can't be attacked", id.0),
            Reject::NotVisible(id) => write!(f, "object {} is out of sight", id.0),
            Reject::LacksAbility => write!(f, "none of the units has the ability"),
            Reject::CannotAttack => write!(f, "none of the units can attack with its shield up"),
            Reject::Blocked => write!(f, "the unit doesn't fit there: blocked ground"),
            Reject::Occupied(id) => write!(f, "unit {} is in the way", id.0),
            Reject::Overlaps(id) => write!(f, "the footprint overlaps object {}", id.0),
            Reject::NoProducers => write!(f, "none of the objects produces units"),
            Reject::CannotProduce(kind) => write!(f, "none of the producers trains a {}", kind.def().name),
            Reject::QueueFull => write!(f, "the build queue is full"),
            Reject::InsufficientCrystals => write!(f, "not enough Space Crystals"),
            Reject::UnitControlCap => write!(f, "not enough Unit Control"),
            Reject::NothingQueued => write!(f, "nothing is queued"),
            Reject::NoConstructor => write!(f, "none of the objects is a constructor of the player's"),
            Reject::SeveralConstructors => write!(f, "a building is placed by one constructor"),
            Reject::CannotConstruct(kind) => write!(f, "the constructor doesn't construct a {}", kind.def().name),
            Reject::ConstructorBusy(id) => write!(f, "constructor {} is busy", id.0),
            Reject::NotConstructing => write!(f, "nothing is under construction"),
            Reject::NothingReady => write!(f, "no building is ready to place"),
            Reject::Unplaceable(problem) => write!(f, "the building can't go there: {problem}"),
        }
    }
}

/// A command that passed the gate, with what the checks resolved for it.
pub(super) enum Checked {
    /// `units`: unique, in id order, all existing units of the player.
    Order {
        units: Vec<ObjectId>,
        kind: OrderKind,
        queue: bool,
    },
    /// `units`: unique, in id order, all existing units of the player that have the ability.
    Ability {
        units: Vec<ObjectId>,
        ability: Ability,
    },
    Spawn {
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
        placement: Option<Placement>,
    },
    Remove(ObjectId),
    Teleport {
        id: ObjectId,
        pos: Vec2,
    },
    AutoTargeting(bool),
    /// `player`: in the game.
    SetResources {
        player: PlayerId,
        crystals: u32,
        supplies: u32,
    },
    /// `player`'s production command, with what its checks resolved (`sim/producers.rs`).
    Production {
        player: PlayerId,
        plan: ProductionPlan,
    },
    /// `player`'s constructors carry out what the construction checks resolved (`sim/construction.rs`).
    Construction {
        player: PlayerId,
        resolved: Resolved,
    },
}

impl Sim {
    /// The command gate: `Ok` if `cmd` would be applied now, else why it would be ignored. `Sim::step` passes every
    /// command through it before applying it, each against the state the earlier commands of the tick left. In
    /// order, it checks that:
    /// 1. the issuing player is in the game, the game isn't over ([`Reject::GameOver`]) and the player isn't defeated
    ///    ([`Reject::Defeated`]: every command, debug commands included, design change 0101);
    /// 2. a debug command comes while [`super::SimConfig::debug`] is on;
    /// 3. every object the command refers to exists, and every object a player's own order, ability, production or
    ///    construction command commands belongs to that player (debug commands may refer to anyone's objects). These
    ///    drop the ids of objects that no longer exist, and repeated ids; an order is rejected if no unit remains
    ///    (`docs/movement.md > 3`);
    /// 4. the command's own rules hold: an order's size and target (which the player must see,
    ///    [`Reject::NotVisible`]), and for an Attack, units that can attack now ([`Reject::CannotAttack`]); the units
    ///    of an ability that have it ([`Reject::LacksAbility`]); the producers of a production command and what they
    ///    can queue (`sim/producers.rs`); the constructors of a construction command that can carry it out, the bank
    ///    that pays and the placement check of a Place (`sim/construction.rs`); a spawn's owner and placement (a
    ///    footprint apart from the others but for a plate on its patch, [`Reject::Overlaps`]), a teleport to a unit
    ///    and onto the map, a unit's box clear of blocked ground and of the units of its layer, and the player whose
    ///    banks a debug command sets.
    pub fn validate(&self, cmd: &PlayerCommand) -> Result<(), Reject> {
        self.check(cmd).map(|_| ())
    }

    /// [`Sim::validate`], keeping what the checks resolved for applying the command.
    pub(super) fn check(&self, cmd: &PlayerCommand) -> Result<Checked, Reject> {
        let Some(player) = self.player(cmd.player) else {
            return Err(Reject::UnknownPlayer(cmd.player));
        };
        // `core/game_rules.md > End of Game`, `> On Defeat` (design change 0101).
        if self.outcome.is_some() {
            return Err(Reject::GameOver);
        }
        if player.is_defeated() {
            return Err(Reject::Defeated(cmd.player));
        }
        match &cmd.command {
            Command::Order { units, kind, queue } => self.check_order(cmd.player, units, *kind, *queue),
            Command::Ability { units, ability } => self.check_ability(cmd.player, units, *ability),
            Command::Production { objects, command } => self.check_production(cmd.player, objects, *command),
            Command::Construction { objects, command } => self.check_construction(cmd.player, objects, *command),
            Command::Debug(debug) => {
                if !self.config.debug {
                    return Err(Reject::DebugDisabled);
                }
                self.check_debug(debug)
            }
        }
    }

    /// The player's units among `units` that `keep` keeps, unique and in id order: at most [`MAX_ORDER_UNITS`] ids,
    /// none of them someone else's. Ids that no longer exist are dropped (a unit may die in the tick its command
    /// arrives: not the player's mistake), and so are the player's objects that `keep` leaves out.
    pub(super) fn own_units(
        &self,
        player: PlayerId,
        units: &[ObjectId],
        keep: impl Fn(&Object) -> bool,
    ) -> Result<Vec<ObjectId>, Reject> {
        if units.len() > MAX_ORDER_UNITS {
            return Err(Reject::TooManyUnits);
        }
        let mut ids = units.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let mut kept = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(o) = self.objects.get(&id) else {
                continue;
            };
            if o.owner != Some(player) {
                return Err(Reject::NotOwned(id));
            }
            if keep(o) {
                kept.push(id);
            }
        }
        Ok(kept)
    }

    /// An object a command refers to: it must exist and, when `orderer` is given, belong to that player (the
    /// objects a player's own order commands; targets, such as an enemy to attack, may belong to anyone).
    pub(super) fn referenced(&self, id: ObjectId, orderer: Option<PlayerId>) -> Result<&Object, Reject> {
        let object = self.objects.get(&id).ok_or(Reject::UnknownObject(id))?;
        match orderer {
            Some(player) if object.owner != Some(player) => Err(Reject::NotOwned(id)),
            _ => Ok(object),
        }
    }
}
