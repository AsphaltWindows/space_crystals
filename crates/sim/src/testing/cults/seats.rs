//! The seats of a [`Cults`] line (`testing::cults`'s module docs, 1 and 2): its groups, the room at its Storages, and
//! the Storages it builds as they're needed (slice M5-S3, design change 0385, answering Q0144 in part;
//! `factions/cults/economy.md > Economy check`: "Both **start with only the one starting Storage**, and add Storages
//! **as they're needed**: whenever the mining Recruits outnumber the room at the existing Storages. Each Storage is paid
//! for and built by Recruits taken off mining").
//!
//! **Room** is the design's "full efficiency at about **2 Recruits per tile of face length**": a group that faces a
//! Storage ([`Seats::facing`]) seats as many Recruits as the line deals it, two to a patch on the reference field, and
//! its seats are room while its Storage stands. **When a Storage is needed** (not in the design; slice M5-S3's literal
//! reading, reported): whenever the player's Recruits outnumber the room at the Storages standing, a site counting as
//! soon as the line orders it (else it would order the same Storage again in every tick until the site began), the line
//! orders the next Storage its groups face, in their order, once nothing else of its is being built ([`Cults::build`]'s
//! one building at a time) and the bank covers it. Its crew is [`Crew::Far`](super::Crew::Far): the Recruits whose
//! seats have no room yet, which mine at a loss, and those without a seat.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::command::{Command, PlayerCommand};
use crate::ids::ObjectId;
use crate::orders::{OrderKind, Target};
use crate::sim::Sim;

use super::{Build, Cults};

/// One of a line's groups of seats (`testing::cults`'s module docs, 1): each patch with how many Recruits it seats,
/// and, on the reference field of design change 0385, the Storage they face and the way a Recruit is sent to them.
#[derive(Clone, Debug, PartialEq)]
pub struct Seats {
    pub(super) seats: Vec<(ObjectId, u32)>,
    /// The Storage the seats face, which the line builds once it is needed (module docs).
    pub(super) storage: Option<Build>,
    /// Whether the seats are dealt only once that Storage is complete.
    pub(super) waits: bool,
    /// From each patch's centre to the point a Recruit dealt here is moved to before it gathers there.
    pub(super) lane: Option<Vec2>,
}

impl Seats {
    /// Each patch with how many Recruits it seats, dealt as soon as a Recruit is idle (module docs, 1).
    pub fn new(seats: impl IntoIterator<Item = (ObjectId, u32)>) -> Self {
        Self {
            seats: seats.into_iter().collect(),
            storage: None,
            waits: false,
            lane: None,
        }
    }

    /// The Storage the seats face: their room, once it stands, which the line builds when it is needed (module docs).
    /// Until it is complete a Recruit dealt here mines at a loss, taking its loads to the Storage nearest to itself.
    pub fn facing(mut self, storage: Build) -> Self {
        self.storage = Some(storage);
        self
    }

    /// The seats are dealt only once their Storage is complete ([`Self::facing`]).
    pub fn waiting(mut self) -> Self {
        self.waits = true;
        self
    }

    /// A Recruit dealt here is first moved to the point `offset` from its patch's centre, and then gathers the patch
    /// (Shift): the orders a player gives to send Recruits round the line's end to its far side (design change 0385:
    /// "Recruits reach the far side through the scenario's orders, as a player would send them (e.g. Move round the
    /// line, then Gather)"). A Recruit's leg to its patch heads for the face nearest to it (`orders::gather`), so from
    /// the lane in front of the patch it mines from that side.
    pub fn by(mut self, offset: Vec2) -> Self {
        self.lane = Some(offset);
        self
    }

    /// How many Recruits they seat.
    pub fn count(&self) -> u32 {
        self.seats.iter().map(|&(_, n)| n).sum()
    }
}

impl Cults {
    /// The seat an idle Recruit is sent to (`testing::cults`'s module docs, 1), its group and patch, the seat taken;
    /// `None` with no seat free. A group that waits for its Storage is passed over until that is complete.
    pub(super) fn seat(&self, sim: &Sim, taken: &mut BTreeMap<(usize, ObjectId), u32>) -> Option<(usize, ObjectId)> {
        for (g, group) in self.groups.iter().enumerate() {
            if group.waits && !group.storage.is_some_and(|build| self.complete(sim, build)) {
                continue;
            }
            let free = (group.seats.iter())
                .filter(|&&(patch, most)| taken[&(g, patch)] < most && sim.object(patch).is_some())
                .map(|&(patch, _)| patch);
            if let Some(patch) = free.min_by_key(|&patch| taken[&(g, patch)]) {
                *taken.entry((g, patch)).or_default() += 1;
                return Some((g, patch));
            }
        }
        None
    }

    /// The orders that send `units` to their seat, the group `g`'s `patch`: a Gather, after a Move to the lane in front
    /// of the patch for a group dealt by its lane ([`Seats::by`]); queued after the units' orders if `queue`.
    pub(super) fn to_seat(
        &self,
        sim: &Sim,
        units: Vec<ObjectId>,
        (g, patch): (usize, ObjectId),
        queue: bool,
    ) -> Vec<PlayerCommand> {
        let order = |units: Vec<ObjectId>, kind: OrderKind, queue: bool| PlayerCommand {
            player: self.player,
            command: Command::Order { units, kind, queue },
        };
        let lane = (self.groups[g].lane)
            .zip(sim.object(patch))
            .map(|(offset, o)| o.pos + offset);
        match lane {
            Some(at) => vec![
                order(units.clone(), OrderKind::Move(Target::Point(at)), queue),
                order(units, OrderKind::Gather(patch), true),
            ],
            None => vec![order(units, OrderKind::Gather(patch), queue)],
        }
    }

    /// Whether the Recruit seated at the group `g`'s seats mines with room (module docs): its group faces no Storage, or
    /// a complete one.
    pub(super) fn has_room(&self, sim: &Sim, g: usize) -> bool {
        self.groups[g].storage.is_none_or(|build| self.complete(sim, build))
    }

    /// The Storage needed next (module docs), once the player's `recruits` outnumber the room at the Storages standing:
    /// the first its groups face that doesn't stand. Only while nothing of the line's is being built.
    pub(super) fn needed(&self, sim: &Sim, recruits: usize) -> Option<Build> {
        let room: u32 = (self.groups.iter())
            .filter(|group| group.storage.is_some_and(|build| self.stands(sim, build)))
            .map(Seats::count)
            .sum();
        if recruits <= room as usize {
            return None;
        }
        (self.groups.iter())
            .filter_map(|group| group.storage)
            .find(|&build| !self.stands(sim, build))
    }

    /// Whether `build` is one of the Storages the line builds as needed (a group faces it).
    pub(super) fn is_needed(&self, build: Build) -> bool {
        self.groups.iter().any(|group| group.storage == Some(build))
    }

    /// Whether `build` stands for the player: a building of its kind with its footprint at its origin, complete or not.
    fn stands(&self, sim: &Sim, build: Build) -> bool {
        self.standing(sim, build).is_some()
    }

    /// Whether `build` stands for the player and is complete.
    pub(super) fn complete(&self, sim: &Sim, build: Build) -> bool {
        self.standing(sim, build).is_some_and(|operational| operational)
    }

    /// Whether the player's building of `build` is complete, if one stands at its origin.
    fn standing(&self, sim: &Sim, build: Build) -> Option<bool> {
        (sim.objects())
            .find(|o| {
                o.owner == Some(self.player)
                    && o.kind == build.kind
                    && o.placement.is_some_and(|p| p.origin == build.origin)
            })
            .map(|o| o.is_operational())
    }
}
