//! The Cults' field (`super` module docs, 3): the seats its Storages give, and where each Recruit without work goes.
//!
//! - **Seats** ([`Field::of`]): a patch of the field with crystals left, as last seen, seats [`SEATS_ACROSS`] Recruits
//!   for each complete Storage of the player's straight across a 1-tile gap from it (`factions/cults/structures/storage.md`:
//!   "The best layout leaves a 1-tile gap between patch and Storage"; `factions/cults/economy.md > Economy check`: "full
//!   efficiency at about **2 Recruits per tile of face length**", a patch's face toward the Storage being its one
//!   tile), and [`SEATS_AT_CORNER`] for one at its corner a tile away. "**Distance** is measured **edge to edge**,
//!   patch to Storage, in whole tiles" (`factions/cults/economy.md > Economy check`). A seat is the pair; its lane is
//!   the middle of the gap between them, where its Recruits mine and deliver.
//! - **Dealing** ([`Cults::deal`]): each Recruit without work (idle, or whose seat is gone) goes to the free seat whose
//!   lane is nearest it, ties by the seat, while the crew is smaller than the player wants (`super` module docs, "The
//!   crew" and "The rush"; [`Cults::pull`] takes Recruits off the field). One whose seat lies across its patch from it first walks to the lane (Move,
//!   then Gather with Shift), as a player sends Recruits round a line to its far side (design change 0385: "Recruits on
//!   the far side are sent there by the scenario's orders, as a player would"; `factions/cults/units/recruit.md >
//!   Gathering`, Crowding: "it is the player's job to redistribute them"); the game then takes its loads to the
//!   Storage nearest it, its seat's. With no seat free it enters the Armory while the Armory has room (`armory.rs`),
//!   else it waits where it is.
//!
//! A Recruit keeps its seat while it builds (`builds.rs`) and while the game sends it on to another patch (design
//! change 0141), until it is idle again: so the seats bound how many Recruits mine, as `sc_sim::testing::cults`' lines
//! do.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::nav::TileRect;
use sc_sim::{Command, Object, ObjectId, ObjectKind, OrderKind, Placement, SU_PER_GU, Target, Vec2};

use super::{Cults, own};
use crate::Turn;
use crate::View;
use crate::tuning::cults::{MIN_CREW, PULL_TICKS, SEATS_ACROSS, SEATS_AT_CORNER};

/// A seat of the field (module docs): a patch and a complete Storage of the player's a tile from it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Seat {
    pub patch: ObjectId,
    pub storage: ObjectId,
}

/// A seat as the field stands: how many Recruits it seats, its lane, and its patch's centre.
#[derive(Copy, Clone, Debug)]
pub(super) struct Pair {
    pub seat: Seat,
    pub seats: u32,
    pub lane: Vec2,
    pub at: Vec2,
}

/// The field in one tick (module docs).
#[derive(Clone, Debug, Default)]
pub(super) struct Field {
    /// Its seats, in their order.
    pub pairs: Vec<Pair>,
    /// Its patches with crystals left as last seen, with their footprints, in id order.
    pub patches: Vec<(ObjectId, TileRect)>,
}

impl Field {
    /// The field of the player whose View it is: the patches of `field` (in id order) it knows to have crystals left,
    /// and the seats its complete Storages give them.
    pub(super) fn of(view: &View<'_>, field: &[ObjectId]) -> Field {
        let patches: Vec<(ObjectId, TileRect, Vec2)> = (view.patches())
            .filter(|k| field.binary_search(&k.id).is_ok() && k.crystals_left().is_some_and(|left| left > 0))
            .filter_map(|k| Some((k.id, rect(&k.placement?), k.pos)))
            .collect();
        let storages: Vec<(ObjectId, TileRect)> = own(view, ObjectKind::Storage)
            .filter(|o| o.is_operational())
            .filter_map(|o| Some((o.id, rect(o.placement.as_ref()?))))
            .collect();
        let mut pairs = Vec::new();
        for &(patch, p, at) in &patches {
            for &(storage, s) in &storages {
                let seats = seats(p, s);
                if seats > 0 {
                    let seat = Seat { patch, storage };
                    let lane = lane(p, s);
                    pairs.push(Pair { seat, seats, lane, at });
                }
            }
        }
        pairs.sort_by_key(|p| p.seat);
        let patches = patches.into_iter().map(|(id, rect, _)| (id, rect)).collect();
        Field { pairs, patches }
    }

    /// The seat `seat`, if it stands.
    pub(super) fn pair(&self, seat: Seat) -> Option<&Pair> {
        let i = self.pairs.binary_search_by_key(&seat, |p| p.seat).ok()?;
        self.pairs.get(i)
    }

    /// How many Recruits the field seats in all.
    pub(super) fn seats(&self) -> u32 {
        self.pairs.iter().map(|p| p.seats).sum()
    }
}

/// The tiles a footprint covers.
pub(super) fn rect(p: &Placement) -> TileRect {
    TileRect::new(p.origin, p.w, p.h)
}

/// The gap between the spans `[a0, a0 + aw)` and `[b0, b0 + bw)` of an axis, tiles, and how far they overlap
/// (negative when they don't).
fn axis(a0: i32, aw: u32, b0: i32, bw: u32) -> (i32, i32) {
    let overlap = (a0 + aw as i32).min(b0 + bw as i32) - a0.max(b0);
    ((-overlap).max(0), overlap)
}

/// How many Recruits a patch on `p` seats for a Storage on `s` (module docs): [`SEATS_ACROSS`] straight across a
/// 1-tile gap, [`SEATS_AT_CORNER`] at its corner a tile away, none otherwise.
pub(super) fn seats(p: TileRect, s: TileRect) -> u32 {
    let (gap_x, overlap_x) = axis(p.origin.x, p.w, s.origin.x, s.w);
    let (gap_y, overlap_y) = axis(p.origin.y, p.h, s.origin.y, s.h);
    match (gap_x, gap_y) {
        (0, 1) if overlap_x >= 1 => SEATS_ACROSS,
        (1, 0) if overlap_y >= 1 => SEATS_ACROSS,
        (0, 1) | (1, 0) => SEATS_AT_CORNER,
        _ => 0,
    }
}

/// The lane between a patch on `p` and a Storage on `s`: the middle of the gap between them, space units.
fn lane(p: TileRect, s: TileRect) -> Vec2 {
    let mid = |a0: i32, aw: u32, b0: i32, bw: u32| (a0.max(b0) + (a0 + aw as i32).min(b0 + bw as i32)) as f32 / 2.0;
    let x = mid(p.origin.x, p.w, s.origin.x, s.w);
    let y = mid(p.origin.y, p.h, s.origin.y, s.h);
    Vec2::new(x, y) * SU_PER_GU
}

/// Whether `o` has no order at all.
pub(super) fn idle(o: &Object) -> bool {
    o.orders.as_ref().is_none_or(|orders| orders.is_idle())
}

/// Whether `o`'s current order is a Gather.
pub(super) fn gathers(o: &Object) -> bool {
    let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
    current.is_some_and(|active| matches!(active.order.kind, OrderKind::Gather(_)))
}

/// The orders that send `units`, at `from`, to the seat of `pair`, after their orders if `queue` (module docs): a
/// Gather of its patch, after a Move to its lane if the lane lies across the patch from `from`.
pub(super) fn to_seat(units: Vec<ObjectId>, pair: &Pair, from: Vec2, queue: bool) -> Vec<Command> {
    let order = |units: Vec<ObjectId>, kind, queue| Command::Order { units, kind, queue };
    let across = (pair.lane - pair.at).dot(from - pair.at) < 0.0;
    let gather = OrderKind::Gather(pair.seat.patch);
    if across {
        vec![
            order(units.clone(), OrderKind::Move(Target::Point(pair.lane)), queue),
            order(units, gather, true),
        ]
    } else {
        vec![order(units, gather, queue)]
    }
}

impl Cults {
    /// The seats taken (module docs): the Recruits it seated that are still the player's Recruits, not idle, with
    /// their seat standing. Returns the Recruits whose seat is gone while they work, to be dealt again.
    pub(super) fn keep_seats(&mut self, view: &View<'_>, field: &Field) -> Vec<ObjectId> {
        let recruits: BTreeMap<ObjectId, &Object> = own(view, ObjectKind::Recruit).map(|o| (o.id, o)).collect();
        let mut gone = Vec::new();
        self.seated.retain(|id, seat| match recruits.get(id) {
            None => false,
            Some(o) if idle(o) => false,
            Some(_) if field.pair(*seat).is_none() => {
                gone.push(*id);
                false
            }
            Some(_) => true,
        });
        gone
    }

    /// Sends each Recruit without work, idle or among those whose seat is `gone`, but those `given` an order in the
    /// turn, to a free seat, else into the Armory (module docs).
    pub(super) fn deal(
        &mut self,
        turn: &mut Turn<'_, '_>,
        field: &Field,
        given: &BTreeSet<ObjectId>,
        gone: &[ObjectId],
    ) {
        let view = turn.view();
        let mut taken: BTreeMap<Seat, u32> = BTreeMap::new();
        for seat in self.seated.values() {
            *taken.entry(*seat).or_default() += 1;
        }
        let crew = self.crew();
        let mut armory = self.armory_room(view);
        let dealt: Vec<&Object> = own(view, ObjectKind::Recruit)
            .filter(|o| !given.contains(&o.id) && (idle(o) || gone.contains(&o.id)))
            .collect();
        for o in dealt {
            let nearest = |a: &&Pair, b: &&Pair| {
                let (da, db) = (a.lane.distance_squared(o.pos), b.lane.distance_squared(o.pos));
                da.total_cmp(&db).then(a.seat.cmp(&b.seat))
            };
            let room = crew.is_none_or(|crew| self.seated.len() < crew);
            let free = (field.pairs.iter())
                .filter(|p| room && taken.get(&p.seat).copied().unwrap_or(0) < p.seats)
                .min_by(nearest);
            if let Some(pair) = free {
                let mut sent = true;
                for command in to_seat(vec![o.id], pair, o.pos, false) {
                    sent = sent && turn.give(command);
                }
                if sent {
                    self.seated.insert(o.id, pair.seat);
                    *taken.entry(pair.seat).or_default() += 1;
                }
                continue;
            }
            if let Some((id, room)) = &mut armory
                && *room > 0
            {
                let enter = Command::Order {
                    units: vec![o.id],
                    kind: OrderKind::Enter(*id),
                    queue: false,
                };
                if turn.give(enter) {
                    *room -= 1;
                }
            }
        }
    }

    /// Whether every seat of the field is taken: the next Recruit has none, and the base wants a Storage
    /// (`builds.rs`).
    pub(super) fn field_full(&self, field: &Field) -> bool {
        self.seated.len() as u32 >= field.seats()
    }

    /// While the bank outruns the army (`super` module docs, "The crew"), one seated Recruit every [`PULL_TICKS`],
    /// but those `given` an order in the turn, leaves the field for the Armory while it has room and the crew is
    /// larger than [`MIN_CREW`]: one that mines (not one building), the nearest the Armory, ties by id.
    pub(super) fn pull(&mut self, turn: &mut Turn<'_, '_>, given: &BTreeSet<ObjectId>) {
        let view = turn.view();
        let now = view.tick();
        if !self.spending || self.rushing() || now < self.pull_at || self.seated.len() <= MIN_CREW {
            return;
        }
        let Some((armory, _)) = self.armory_room(view).filter(|&(_, room)| room > 0) else {
            return;
        };
        let Some(at) = view.own_object(armory).map(|o| o.pos) else {
            return;
        };
        let pulled = (self.seated.keys())
            .filter(|id| !given.contains(id))
            .filter_map(|&id| view.own_object(id))
            .filter(|o| gathers(o))
            .min_by(|a, b| {
                (a.pos.distance_squared(at))
                    .total_cmp(&b.pos.distance_squared(at))
                    .then(a.id.cmp(&b.id))
            })
            .map(|o| o.id);
        let Some(id) = pulled else {
            return;
        };
        let enter = Command::Order {
            units: vec![id],
            kind: OrderKind::Enter(armory),
            queue: false,
        };
        if turn.give(enter) {
            self.seated.remove(&id);
            self.pull_at = now + PULL_TICKS;
        }
    }
}
