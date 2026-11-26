//! The command gate (`ARCHITECTURE.md > Commands, events and replays`): the one check every command passes before
//! it is applied. A rejected command changes nothing and never panics the sim.
//!
//! This file holds what every command shares: the reasons for a rejection ([`Reject`]), what a passed command
//! resolved ([`Checked`]), the dispatch by command ([`Sim::check`]) and the lookups of the objects a command names.
//! Each family of commands has its own checks (plan decision T28): orders and abilities in `gate/orders.rs`, debug
//! commands in `gate/debug.rs`, production in `sim/producers.rs`, construction in `sim/construction.rs`, the Tunnels'
//! in `sim/network.rs`, the buildings that hold units' (slice M5-F1) in `sim/holding.rs` and the casts (slice M5-N5) in
//! `sim/casts.rs`.

mod debug;
mod orders;
mod reject;
mod targets;

pub use reject::Reject;

use glam::Vec2;

use super::Sim;
use super::construction::Resolved;
use super::producers::ProductionPlan;
use crate::augmentation::Augmentation;
use crate::command::{Ability, Command, HoldCommand, PlayerCommand};
use crate::defs::{AreaAbility, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::orders::{MAX_ORDER_UNITS, OrderKind};
use crate::player::Banks;

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
        banks: Banks,
    },
    /// `id`: a Tunnel, whose table has `tier`.
    SetTier {
        id: ObjectId,
        tier: u8,
    },
    /// `tunnel`: the player's, in operation; `unit`: the unit of the network an Eject reserves (`sim/network.rs`).
    Eject {
        tunnel: ObjectId,
        unit: ObjectId,
    },
    /// `tunnel`: the player's, in operation and free, building a `kind` on `placement`, valid by the truth
    /// (`sim/expansions.rs`).
    Expand {
        tunnel: ObjectId,
        kind: ObjectKind,
        placement: Placement,
    },
    /// `tunnel`: the player's, in operation, with an operation to cancel (`sim/expansions.rs`).
    CancelOperation {
        tunnel: ObjectId,
    },
    /// `tunnel`: the player's, in operation and free, upgrading to tier `to` for `price` Supplies, which the bank pays,
    /// its area at `to` clear by the truth (`sim/upgrades.rs`).
    Upgrade {
        tunnel: ObjectId,
        to: u8,
        price: u32,
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
    /// `buildings`: the player's buildings that hold units and can carry `command` out, in id order
    /// (`sim/holding.rs`).
    Holding {
        buildings: Vec<ObjectId>,
        command: HoldCommand,
    },
    /// `id`: a unit on the map (slice M5-F2).
    SetAugmentation {
        id: ObjectId,
        augmentation: Augmentation,
    },
    /// (M5-N5) `caster`: the player's, in operation, casting `ability`, with the energy for it; `at`: a point the
    /// player sees (`sim/casts.rs`).
    Cast {
        caster: ObjectId,
        ability: AreaAbility,
        at: Vec2,
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
    ///    [`Reject::NotVisible`]), and for an Attack, units that can attack now ([`Reject::CannotAttack`]), for an
    ///    Attack Ground, a point on the map and units that can target the ground ([`Reject::CannotTargetGround`]); the units
    ///    of an ability that have it ([`Reject::LacksAbility`]); the producers of a production command and what they
    ///    can queue (`sim/producers.rs`); the constructors of a construction command that can carry it out, the bank
    ///    that pays and the placement check of a Place (`sim/construction.rs`), and the same of a Warp In's Beacon,
    ///    or the transforming objects a cancel goes to (`sim/transform.rs`); a spawn's owner and placement (a
    ///    footprint apart from the others but for a plate on its patch, [`Reject::Overlaps`]), a teleport to a unit
    ///    and onto the map, a unit's box clear of blocked ground and of the units of its layer, and the player whose
    ///    banks a debug command sets; (M5-N5) a cast's caster, its energy and its point (`sim/casts.rs`).
    pub fn validate(&self, cmd: &PlayerCommand) -> Result<(), Reject> {
        self.check(cmd).map(|_| ())
    }

    /// [`Sim::validate`], keeping what the checks resolved for applying the command.
    pub(super) fn check(&self, cmd: &PlayerCommand) -> Result<Checked, Reject> {
        let Some(player) = self.player(cmd.player) else {
            return Err(Reject::UnknownPlayer(cmd.player));
        };
        // `core/game_rules.md > End of Game`, `core/game_rules.md > On Defeat` (design change 0101).
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
            Command::Tunnel { tunnel, command } => self.check_tunnel(cmd.player, *tunnel, *command),
            Command::Holding { objects, command } => self.check_holding(cmd.player, objects, *command),
            Command::Cast { casters, ability, at } => self.check_cast(cmd.player, casters, *ability, *at),
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
