//! The command gate (`ARCHITECTURE.md > Commands, events and replays`): the one check every command passes before
//! it is applied. A rejected command changes nothing and never panics the sim.

use std::fmt;

use glam::Vec2;

use super::Sim;
use crate::command::{Command, DebugCommand, PlayerCommand};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::scenario::{SpawnProblem, place_spawn};

/// Why the sim ignores a command ([`Sim::validate`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The issuing player is not in the game.
    UnknownPlayer(PlayerId),
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
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.0),
            Reject::DebugDisabled => write!(f, "debug commands are off"),
            Reject::UnknownObject(id) => write!(f, "object {} doesn't exist", id.0),
            Reject::NotOwned(id) => write!(f, "object {} belongs to someone else", id.0),
            Reject::Spawn(problem) => write!(f, "can't spawn: {problem}"),
            Reject::OutOfIds => write!(f, "every object id is used up"),
            Reject::TileObject(id) => write!(f, "object {} is tile-based and can't move", id.0),
            Reject::OffMap => write!(f, "the position is off the map"),
        }
    }
}

/// A command that passed the gate, with what the checks resolved for it.
pub(super) enum Checked {
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
}

impl Sim {
    /// The command gate: `Ok` if `cmd` would be applied now, else why it would be ignored. `Sim::step` passes every
    /// command through it before applying it, each against the state the earlier commands of the tick left. In
    /// order, it checks that:
    /// 1. the issuing player is in the game;
    /// 2. a debug command comes while [`super::SimConfig::debug`] is on;
    /// 3. every object the command refers to exists, and every object a player's own order commands belongs to
    ///    that player (debug commands may refer to anyone's objects);
    /// 4. the command's own rules hold: a spawn's owner and placement, a teleport to a unit and onto the map.
    pub fn validate(&self, cmd: &PlayerCommand) -> Result<(), Reject> {
        self.check(cmd).map(|_| ())
    }

    /// [`Sim::validate`], keeping what the checks resolved for applying the command.
    pub(super) fn check(&self, cmd: &PlayerCommand) -> Result<Checked, Reject> {
        if self.player(cmd.player).is_none() {
            return Err(Reject::UnknownPlayer(cmd.player));
        }
        match &cmd.command {
            Command::Debug(debug) => {
                if !self.config.debug {
                    return Err(Reject::DebugDisabled);
                }
                self.check_debug(debug)
            }
        }
    }

    fn check_debug(&self, command: &DebugCommand) -> Result<Checked, Reject> {
        // Debug commands act on anyone's objects.
        let anyone = None;
        match *command {
            DebugCommand::Spawn { kind, owner, at } => {
                if self.next_id == u32::MAX {
                    return Err(Reject::OutOfIds);
                }
                let (pos, placement) =
                    place_spawn(&self.map, kind, owner, at, |p| self.player(p).is_some()).map_err(Reject::Spawn)?;
                Ok(Checked::Spawn {
                    kind,
                    owner,
                    pos,
                    placement,
                })
            }
            DebugCommand::Remove { id } => {
                self.referenced(id, anyone)?;
                Ok(Checked::Remove(id))
            }
            DebugCommand::SetPosition { id, pos } => {
                if self.referenced(id, anyone)?.placement.is_some() {
                    return Err(Reject::TileObject(id));
                }
                // `contains_point` also rejects NaN and infinite positions.
                if !self.map.contains_point(pos) {
                    return Err(Reject::OffMap);
                }
                Ok(Checked::Teleport { id, pos })
            }
        }
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
