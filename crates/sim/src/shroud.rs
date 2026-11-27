//! (M5-N4b) Shroud's clouds (plan decision T77, amended by design change 0334): `factions/cults/units/disciple.md >
//! Spells > Shroud`. The Disciple's channel is its order (`OrderKind::Shroud`, `orders::shroud`: it stands, its energy
//! drained); this module keeps the clouds of the units that channel, their buffers and their releases, and phase 5e
//! absorbs the hits and applies the releases (`combat::damage::absorb`, `combat::damage::release`).
//!
//! - **A cloud**: "The cloud is centered on the Disciple", a circle of its type's radius ("cloud radius **3 grid
//!   units** around the Disciple", `defs::ShroudDef::radius`) around each unit that channels ([`Member`]), and "Clouds
//!   are connected when their circles **touch or overlap** (user, 2026-11-26; Q0113); a connected cloud is simply one
//!   cloud": a [`Cloud`] is a connected component of the circles, found by union-find over the channelling units in id
//!   order ([`Clouds::update`]).
//! - **When**: the clouds follow the channels as phase 5a leaves them, where every channel starts and where a new order
//!   or a dry pool ends one (`Sim::step`, [`Clouds::update`]), so that a cloud protects from the tick its channel
//!   starts; and again at the end of the tick, for the channels that ended after 5a: a unit killed in 5e, or a
//!   defeated player's units, whose orders the defeat clears ([`Clouds::settle`]).
//! - **The linger** (M6-CM, design change 0448): "when the channel ends, by the Disciple's death or for any other
//!   reason, the cloud **stays for 1.5 s (24 frames, placeholder)** and keeps absorbing as usual. It stays **where it
//!   was** (designer's reading: the Disciple may walk away meanwhile). When it vanishes, its buffer is released as
//!   below": a channel that ends leaves its circle where it was, a lingering member ([`Member::until`]),
//!   `Spells::linger` ticks after the tick of its end, and the first update from then on drops it, in 5a or at the
//!   end of a tick. Meanwhile it is a member as any other: its cloud goes on, absorbs, releases on its timer, joins
//!   what touches it ("a new channel next to a lingering cloud joins it by the existing rules") and splits as the
//!   circles part. A Disciple that channels again where its circle lingers has two, the old one and its new
//!   channel's. A cloud that vanishes there releases in that tick's 5e, the first damage phase after its end, or, at
//!   the end of a tick, the next tick's: a `Spells::linger` of 0 is the cloud that ends with its channel, as it was
//!   before the linger.
//! - **Matching**: a cloud whose members are exactly a new cloud's (the same units, in the same channels, where they
//!   stood, lingering or not: [`Member::is`]) is that cloud, its buffer and its timer carried on. Every other one ends:
//!   "if a cloud **vanishes** (at the end of its linger) or the connected cloud splits, the buffer is **released
//!   immediately** over the cloud as it was", and "**When clouds join** (e.g. a new cloud forms between two others;
//!   user, 2026-11-26; Q0113), each joining cloud **releases its buffer immediately** over its own area as it was, and
//!   the joined cloud then **starts buffering together** with a fresh buffer". So a cloud that loses a member (its
//!   linger over), splits, joins another, or that a new channel joins, releases its buffer over its own circles as
//!   they were, in the next phase 5e ([`Clouds::take_releases`]), and the clouds that follow it start empty.
//! - **The buffer**: "anti-ground damage aimed at objects under the cloud goes into the shared buffer instead of
//!   hitting its target", each hit's raw damage ([`Clouds::absorb`]); "The buffer is released **0.5 s (8 frames),
//!   placeholder, after the first hit it absorbs** (each connected cloud keeps its own timer; user, 2026-11-26; Q0113),
//!   as **one area hit over the whole connected cloud**, and empties; the next absorbed hit starts the timer again"
//!   ([`Cloud::release_at`], `tuning::spells::Spells::release_delay`).
//! - **Under the cloud**: "an object is under a cloud when its **centre** is inside the cloud's area (this decides
//!   whose damage is absorbed and which attackers fire from under it). Shares of a release still go by overlap area"
//!   ([`Clouds::over`]: a circle's edge counts as inside, as touching circles join). The release's areas are counted on
//!   a lattice ([`lattice`], plan decision T77).
//! - **State**: `Sim::clouds`, history (the buffers, the timers), hashed and saved only while there is a cloud or a
//!   buffer waiting for its release, so that the states without one are as they were before Shroud.

pub mod lattice;
#[cfg(test)]
mod tests;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::UnitDef;
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::{ActiveOrder, OrderKind};
use crate::ruleset::rules;
use crate::units::{Tick, gu};

/// One channel of a cloud: a unit that channels Shroud, and its circle (module docs); (M6-CM) or one that did, its
/// circle lingering where it was.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    /// The unit, whose current order is the Shroud while it channels.
    pub id: ObjectId,
    /// The tick its channel, the order, started (`ActiveOrder::since`): a new Shroud order is a new channel
    /// (`orders::shroud`: "any other order" ends the one under way).
    pub since: Tick,
    /// Where it stood as it channelled, its circle's centre, space units: a unit that channels doesn't move.
    pub at: Vec2,
    /// Its circle's radius, space units: its type's Shroud's (`defs::ShroudDef::radius`).
    pub radius: f32,
    /// (M6-CM) Once its channel has ended, the tick its circle vanishes, `Spells::linger` after the tick of the end
    /// (module docs: "the cloud **stays for 1.5 s (24 frames, placeholder)** and keeps absorbing as usual"); `None`
    /// while the unit channels. Hashed and saved only while set: never with a linger of 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<Tick>,
}

impl Member {
    /// Whether it is the same channel as `other`, lingering or not (module docs: matching): the same unit, the same
    /// start, the same circle.
    pub fn is(&self, other: &Member) -> bool {
        (self.id, self.since, self.at, self.radius) == (other.id, other.since, other.at, other.radius)
    }

    /// Whether its channel has ended, its circle lingering ([`Member::until`]).
    pub fn lingers(&self) -> bool {
        self.until.is_some()
    }

    /// Whether the point `p` is under its circle: inside it, or on its edge.
    pub fn covers(&self, p: Vec2) -> bool {
        self.at.distance_squared(p) <= self.radius * self.radius
    }

    /// Whether its circle and `other`'s "touch or overlap", which connects them.
    pub fn touches(&self, other: &Member) -> bool {
        let reach = self.radius + other.radius;
        self.at.distance_squared(other.at) <= reach * reach
    }
}

/// The channel `o` has now, if it channels Shroud: its current order is a Shroud and its type casts it (module docs).
/// (M6-CM) Not an illusion's, whose order only mirrors its parent's channel (`crate::mirage`: "its illusions **play
/// the same channelling animation**, but make **no cloud**").
pub fn member(o: &Object) -> Option<Member> {
    let ActiveOrder { order, since, .. } = o.orders.as_ref()?.current.as_ref()?;
    if order.kind != OrderKind::Shroud || crate::mirage::is_illusion(o) {
        return None;
    }
    let shroud = o.def().unit.as_ref().and_then(UnitDef::shroud)?;
    Some(Member {
        id: o.id,
        since: *since,
        at: o.pos,
        radius: gu(shroud.radius),
        until: None,
    })
}

/// (M6-CM) Whether `o` stands channelling: its current order is a Shroud. A Disciple that channels, and an illusion
/// that mirrors its parent's channel, which makes no cloud ([`member`]): the stance the client draws alike for both
/// (`factions/cults/units/disciple.md > Spells > Mirage`: "its illusions **play the same channelling animation**, but
/// make **no cloud** (so a watching enemy can tell which one is real while it channels)").
pub fn stands_channelling(o: &Object) -> bool {
    (o.orders.as_ref())
        .and_then(|orders| orders.current.as_ref())
        .is_some_and(|a| a.order.kind == OrderKind::Shroud)
}

/// Whether the members `a` and `b` are the same channels, in the same order, lingering or not ([`Member::is`]).
fn same_channels(a: &[Member], b: &[Member]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.is(b))
}

/// A connected cloud (module docs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cloud {
    /// Its channels, in id order (a unit's lingering circle before its new channel's: in the order of their starts):
    /// at least one, their circles connected.
    pub members: Vec<Member>,
    /// The damage it has absorbed since its last release: each absorbed hit's raw damage, before any modifier.
    pub buffer: f32,
    /// The tick of its next release, `Spells::release_delay` after the first hit it absorbed since its last; none while
    /// it has absorbed nothing since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_at: Option<Tick>,
}

impl Cloud {
    /// A cloud of `members` with a fresh buffer.
    fn fresh(members: Vec<Member>) -> Self {
        Self {
            members,
            buffer: 0.0,
            release_at: None,
        }
    }

    /// Whether the point `p` is under it: under one of its circles.
    pub fn covers(&self, p: Vec2) -> bool {
        self.members.iter().any(|m| m.covers(p))
    }

    /// Its circles: each member's centre and radius, space units.
    pub fn circles(&self) -> Vec<(Vec2, f32)> {
        self.members.iter().map(|m| (m.at, m.radius)).collect()
    }
}

/// `Sim::clouds` (module docs): the clouds, and the buffers of those that ended, waiting for their release.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clouds {
    /// In the order of their first members' ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    clouds: Vec<Cloud>,
    /// The clouds that ended with something in their buffers, as they were, for the next phase 5e to release.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ended: Vec<Cloud>,
}

/// The connected components of `members`' circles (module docs), `members` in id order: union-find, each component in
/// id order, the components in the order of their first members.
pub(crate) fn components(members: Vec<Member>) -> Vec<Vec<Member>> {
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut parent: Vec<usize> = (0..members.len()).collect();
    for i in 0..members.len() {
        for j in i + 1..members.len() {
            if members[i].touches(&members[j]) {
                let (a, b) = (root(&mut parent, i), root(&mut parent, j));
                // The smaller index is the root: each component's is its first member.
                parent[a.max(b)] = a.min(b);
            }
        }
    }
    let mut slots = vec![usize::MAX; members.len()];
    let mut groups: Vec<Vec<Member>> = Vec::new();
    for (i, m) in members.into_iter().enumerate() {
        let r = root(&mut parent, i);
        if slots[r] == usize::MAX {
            slots[r] = groups.len();
            groups.push(Vec::new());
        }
        groups[slots[r]].push(m);
    }
    groups
}

impl Clouds {
    /// No cloud, and no buffer waiting for its release: what the hash and the files leave out.
    pub fn is_empty(&self) -> bool {
        self.clouds.is_empty() && self.ended.is_empty()
    }

    /// The clouds, in the order of their first members' ids.
    pub fn list(&self) -> &[Cloud] {
        &self.clouds
    }

    /// The clouds that ended with something in their buffers since the last phase 5e, as they were, waiting for their
    /// release in the next.
    pub fn ended(&self) -> &[Cloud] {
        &self.ended
    }

    /// The clouds become those of the channels `channelling` (in id order) and of the circles that linger, in tick
    /// `now`, matched to the clouds they were (module docs): a channel that ended lingers from now on, `Spells::linger`
    /// ticks, a lingering circle whose time is up vanishes; each cloud that goes on carries its buffer and its timer,
    /// each one that ends with something in its buffer waits for its release. A [`SimEvent::ShroudEnded`] for each
    /// channel that ended, then a [`SimEvent::ShroudStarted`] for each that started.
    pub(crate) fn update(&mut self, channelling: Vec<Member>, now: Tick, events: &mut Vec<SimEvent>) {
        self.follow(channelling, now, rules().tuning.spells.linger, events);
    }

    /// [`Clouds::update`], with a linger of `linger` ticks.
    fn follow(&mut self, channelling: Vec<Member>, now: Tick, linger: Tick, events: &mut Vec<SimEvent>) {
        if self.clouds.is_empty() && channelling.is_empty() {
            return;
        }
        let mut before: Vec<Member> = self.clouds.iter().flat_map(|c| c.members.iter().copied()).collect();
        before.sort_by_key(|m| (m.id, m.since));
        // The channels as they were, none lingering: nothing changes.
        if before == channelling {
            return;
        }
        let mut after = Vec::with_capacity(before.len().max(channelling.len()));
        for m in &before {
            if m.lingers() {
                after.push(*m);
                continue;
            }
            match channelling.iter().find(|c| (c.id, c.since) == (m.id, m.since)) {
                Some(c) if c.is(m) => after.push(*m),
                // Its channel goes on, its circle moved or resized under it (a debug teleport, new rules: a unit that
                // channels stands): the old circle gives way to the new one, which starts below, and doesn't linger,
                // since the channel hasn't ended; ended and started anew, as before the linger.
                Some(_) => events.push(SimEvent::ShroudEnded { disciple: m.id }),
                None => {
                    events.push(SimEvent::ShroudEnded { disciple: m.id });
                    after.push(Member {
                        until: Some(now.saturating_add(linger)),
                        ..*m
                    });
                }
            }
        }
        for m in channelling
            .iter()
            .filter(|c| !before.iter().any(|m| !m.lingers() && m.is(c)))
        {
            events.push(SimEvent::ShroudStarted {
                disciple: m.id,
                at: m.at,
            });
            after.push(*m);
        }
        after.retain(|m| m.until.is_none_or(|until| until > now));
        after.sort_by_key(|m| (m.id, m.since));
        if after == before {
            return;
        }
        let mut old: Vec<Option<Cloud>> = std::mem::take(&mut self.clouds).into_iter().map(Some).collect();
        for members in components(after) {
            let same = (old.iter_mut())
                .find(|c| c.as_ref().is_some_and(|c| same_channels(&c.members, &members)))
                .and_then(Option::take);
            self.clouds.push(match same {
                Some(cloud) => Cloud { members, ..cloud },
                None => Cloud::fresh(members),
            });
        }
        self.ended.extend(old.into_iter().flatten().filter(|c| c.buffer > 0.0));
    }

    /// The end of tick `now` (module docs): the clouds of the channels that ended since phase 5a, their units gone from
    /// `objects` or with their orders cleared, and of the circles whose linger is over, follow them ([`Clouds::update`]
    /// with the channels under way, as `objects` has them: a unit that channels stands, so where it was).
    pub(crate) fn settle<'a>(
        &mut self,
        object: impl Fn(ObjectId) -> Option<&'a Object>,
        now: Tick,
        events: &mut Vec<SimEvent>,
    ) {
        let channels = |m: &Member| !m.lingers() && object(m.id).and_then(member) == Some(*m);
        let settled = |m: &Member| m.until.map_or_else(|| channels(m), |until| until > now);
        if self.clouds.iter().all(|c| c.members.iter().all(settled)) {
            return;
        }
        // Each channel still under way, as its unit has it now (where it stands: as it was, but for a teleport).
        let under_way = |m: &Member| {
            object(m.id)
                .and_then(member)
                .filter(|c| (c.id, c.since) == (m.id, m.since))
        };
        let mut still: Vec<Member> = (self.clouds.iter())
            .flat_map(|c| c.members.iter())
            .filter(|m| !m.lingers())
            .filter_map(under_way)
            .collect();
        still.sort_by_key(|m| (m.id, m.since));
        self.update(still, now, events);
    }

    /// Whether phase 5e of `tick` releases a buffer: a cloud's that ended, or one whose timer is up
    /// ([`Clouds::take_releases`]).
    pub(crate) fn due(&self, tick: Tick) -> bool {
        !self.ended.is_empty() || self.clouds.iter().any(|c| c.release_at.is_some_and(|at| at <= tick))
    }

    /// The cloud that the point `p` is under (module docs), by its place in [`Clouds::list`]. Two clouds' circles
    /// never share a point, those that do being one cloud.
    pub(crate) fn over(&self, p: Vec2) -> Option<usize> {
        self.clouds.iter().position(|c| c.covers(p))
    }

    /// Whether the point `p` is under cloud `cloud` (by its place in [`Clouds::list`]).
    pub(crate) fn covers(&self, cloud: usize, p: Vec2) -> bool {
        self.clouds.get(cloud).is_some_and(|c| c.covers(p))
    }

    /// Cloud `cloud` (by its place in [`Clouds::list`]) absorbs a hit's `damage` in phase 5e of `tick`: into its buffer,
    /// and its timer starts if it isn't running.
    pub(crate) fn absorb(&mut self, cloud: usize, damage: f32, tick: Tick) {
        if let Some(c) = self.clouds.get_mut(cloud) {
            c.buffer += damage;
            c.release_at.get_or_insert(tick + rules().tuning.spells.release_delay);
        }
    }

    /// The buffers phase 5e of `tick` releases, each with the circles it is released over: those of the clouds that
    /// ended, as they were, then those of the clouds whose timer is up, which empty and wait for their next absorbed
    /// hit.
    pub(crate) fn take_releases(&mut self, tick: Tick) -> Vec<Cloud> {
        let mut released = std::mem::take(&mut self.ended);
        for c in &mut self.clouds {
            if c.release_at.is_some_and(|at| at <= tick) {
                released.push(Cloud {
                    members: c.members.clone(),
                    buffer: c.buffer,
                    release_at: None,
                });
                c.buffer = 0.0;
                c.release_at = None;
            }
        }
        released
    }

    /// Every unit that a cloud, or a buffer waiting for its release, names (`Sim` loading checks their ids).
    pub(crate) fn ids(&self) -> impl Iterator<Item = ObjectId> + '_ {
        (self.clouds.iter().chain(&self.ended)).flat_map(|c| c.members.iter().map(|m| m.id))
    }
}

impl StateHash for Member {
    fn hash(&self, h: &mut StateHasher) {
        let Member {
            id,
            since,
            at,
            radius,
            until,
        } = self;
        id.hash(h);
        since.hash(h);
        at.hash(h);
        radius.hash(h);
        // (M6-CM) Only a lingering circle has one, under a tag of its own, so that the states without one hash as
        // they did before the linger.
        if let Some(until) = until {
            h.tag(1);
            until.hash(h);
        }
    }
}

impl StateHash for Cloud {
    fn hash(&self, h: &mut StateHasher) {
        let Cloud {
            members,
            buffer,
            release_at,
        } = self;
        members.hash(h);
        buffer.hash(h);
        release_at.hash(h);
    }
}

impl StateHash for Clouds {
    fn hash(&self, h: &mut StateHasher) {
        let Clouds { clouds, ended } = self;
        clouds.hash(h);
        ended.hash(h);
    }
}
