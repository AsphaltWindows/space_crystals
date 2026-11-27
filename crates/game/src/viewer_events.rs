//! What each player may perceive of the sim's events (slice M4-V): the one filter between the sim's events and every
//! view of them.
//!
//! The sim tells every event to everyone (`sim_driver::SimEventMessage`). `filter_events` alone reads that stream, in
//! `ClientSystems::SyncSim` ([`FilterEvents`]), and gives each event its [`Audience`]: whom it reaches and how, worked
//! out once from the event and the sim as it arrives. The consumers read [`ViewerEvents`] instead and ask how a viewer
//! perceives each event ([`Perception`]): whole; as a path the viewer sees only over their Visible tiles, which the
//! drawing code cuts (`combat_effects::visible_spans`); or not at all. Those that act at once take a player's
//! perceptions ([`ViewerEvents::perceived_by`]: the feedback line and the notices the local player's, the selection
//! sync each player's); the combat effects keep each event's [`Sighting`] with its effect
//! ([`ViewerEvents::sightings`]) and ask it at every frame, for the viewer the client shows then. Nothing else in the
//! client reads the stream but the QA state dump, which is tooling (`qa`).
//!
//! The rules are the design's, none new here:
//! - **A player's own business** ([`Audience::Owner`]): what happens to one player's objects that only that player
//!   knows of (feedback, refusals, notices, refunds, and how its operations go: production, construction, a Tunnel's
//!   upgrade, which is "the owner's business", `SimEvent::UpgradeStarted`) reaches the owner alone.
//! - **What happens to an object** ([`Audience::Object`]): whoever is shown the object ([`Sight::sees`]): its owner,
//!   and another player while it is on their Visible tiles (`core/vision.md > Visible`), never one it is hidden from
//!   (`factions/syndicate/structures/README.md > Tunnel Expansions`: "Expansions are invisible to enemies without
//!   detection"; `sc_sim::vision::Hiding`).
//! - **What happens at a place** ([`Audience::Seen`], a [`Sighting`]): every shot, tracer and projectile is drawn only
//!   over the viewer's Visible tiles, whoever fires it and at whatever, its shooter's owner's too
//!   (`core/vision.md > Attacks from outside vision`, design change 0293: "No exceptions"); a blast shows where its
//!   centre is Visible, and what it hits underground shows nothing (plan decision T52: "nothing is drawn for a hidden
//!   victim"); a death shows to its object's owner, and to the others where it is Visible, never to one it was hidden
//!   from as it died (`SimEvent::Died::hiding`). (M5-V4) A crush is a death (`SimEvent::Crushed`), and reaches them
//!   alike.
//! - **A removal** ([`Audience::Removal`]): to whoever was shown the object where it went, but never to a player who
//!   remembers it out of sight, whose record of it stays as last seen (`core/vision.md > Explored`).
//! - **Everyone** ([`Audience::Everyone`]): a defeat, and the end of the game (`core/game_rules.md > End of Game`:
//!   "Every player sees Victory or Defeat"); (M5-N4b) and a Shroud's release, which happens to a cloud every player
//!   sees over their Visible tiles, and which the drawing keeps to them (`factions/cults/units/disciple.md > Spells >
//!   Shroud`: "the cloud is shown to **every player**, over the tiles that are Visible to them").
//!
//! With the fog off (`fog::FogOfWar`), everything that happens on the map reaches every viewer whole, as if each tile
//! were Visible; a player's own business stays theirs.
//!
//! **Hidden objects** are `sc_sim::vision::Hiding`'s alone: a live one's through [`Sight::sees`] (the sim's vision asks
//! it), a dead one's through the `Hiding` its death carries, and a removed one's through the filter's snapshot of the
//! objects ([`Sighting::of`]). So a new arm of the rule in `Hiding::of`, such as the pop-up turret's, hidden while
//! fully lowered (`factions/syndicate/structures/laser_turret.md > Pop-up rules`: "**Hidden only while fully
//! lowered**"), holds for all three at once.
//!
//! Q0090's points, answered as built (design change 0357):
//! - A shot's glow and a projectile's body are in the colour of their shooter's owner, whom the viewer may not see
//!   (`combat_effects`; `core/vision.md > Attacks from outside vision`: "a shot is always drawn in the **shooter's
//!   player colour**, even when the viewer doesn't see the shooter").
//! - A defeat before the end of the game reaches every player, as the end does (`core/game_rules.md > On Defeat`:
//!   "when a player is defeated while the game goes on, **all players** are told"), which the defeat notice tells them
//!   (`hud::notices`, slice M5-S1).
//! - A death marker or a blast mark shows wherever the viewer's tile is Visible during its fade, as it is drawn
//!   (`core/vision.md > Attacks from outside vision`: "a death marker or blast mark is shown on any of the viewer's
//!   Visible tiles while it is still fading, even if the viewer didn't see the event itself").
//!
//! Open, the design being silent: a shot or a projectile at an object hidden from the viewer shows over the viewer's
//! Visible tiles all the same, up to where the object is (the `Placeholder` at [`Sighting::path`]).

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use sc_sim::vision::Hiding;
use sc_sim::{Object, ObjectId, PlayerId, Sim, SimEvent, Tick, VisibilityState};

use crate::ClientSystems;
use crate::fog::Sight;
use crate::sim_driver::{SimEventMessage, SimRes};

/// The set of the filter's system, in `ClientSystems::SyncSim`: the consumers there run after it; those of the later
/// sets do anyway.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilterEvents;

pub struct ViewerEventsPlugin;

impl Plugin for ViewerEventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewerEvents>()
            .configure_sets(Update, FilterEvents.in_set(ClientSystems::SyncSim))
            .add_systems(Update, filter_events.in_set(FilterEvents));
    }
}

/// The sim's events of this frame, each with its [`Audience`] (module docs), in the order the sim sent them.
#[derive(Resource, Debug, Default)]
pub struct ViewerEvents {
    events: Vec<ViewerEvent>,
    /// The objects as the filter last saw them, on the map and off it, at the end of the latest frame whose sim
    /// changed, ids in order: for an event that names an object gone since without a death to tell of it (a site
    /// cancelled, a unit stranded in a Tunnel Network).
    known: Vec<(ObjectId, Sighting)>,
    /// The `SimRes::generation` of the objects it knows: a replaced sim's ids are other objects'.
    generation: Option<u32>,
}

/// One of the sim's events, and whom it reaches.
#[derive(Clone, Debug, PartialEq)]
struct ViewerEvent {
    tick: Tick,
    event: SimEvent,
    audience: Audience,
}

impl ViewerEvents {
    /// The events of this frame that `sight`'s player perceives, each with its tick and how they perceive it
    /// ([`Audience::perceived_by`], in `sim` as it is now), in the order the sim sent them: for the consumers that act
    /// on them at once.
    pub fn perceived_by<'a>(
        &'a self,
        sim: &'a Sim,
        sight: Sight<'a>,
    ) -> impl Iterator<Item = (Tick, &'a SimEvent, Perception)> {
        self.events
            .iter()
            .filter_map(move |e| match e.audience.perceived_by(sim, &sight) {
                Perception::Nothing => None,
                perception => Some((e.tick, &e.event, perception)),
            })
    }

    /// The events of this frame that happen at a place ([`Audience::Seen`]: shots, projectiles, blasts and deaths),
    /// each with its tick and its [`Sighting`]: for the consumers that keep them and ask each viewer as they show them
    /// ([`Sighting::perceived_by`]), as the combat effects do at every frame.
    pub fn sightings(&self) -> impl Iterator<Item = (Tick, &SimEvent, Sighting)> {
        self.events.iter().filter_map(|e| match e.audience {
            Audience::Seen(seen) => Some((e.tick, &e.event, seen)),
            _ => None,
        })
    }

    /// Whether the sim sent no event this frame.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Whom an event reaches (module docs), worked out from the event and the sim as it arrived.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Audience {
    /// The one player whose business it is: the owner of the object it is about. Nobody for `None`: a neutral object,
    /// or one the filter never saw.
    Owner(Option<PlayerId>),
    /// Whoever is shown object `id` ([`Sight::sees`]), which was on the map as the event arrived.
    Object(ObjectId),
    /// Whoever sees where it happens.
    Seen(Sighting),
    /// The removal of object `id`: perceived as the object was seen where it went, `seen` (none if the filter never saw
    /// it, as one that came and went within a frame's ticks, which no player was shown), by no player who remembers it
    /// out of sight ([`Sight::remembered_record`]).
    Removal { id: ObjectId, seen: Option<Sighting> },
    /// Every player.
    Everyone,
}

impl Audience {
    /// How `sight`'s player perceives an event of this audience, in `sim` as it is now.
    pub fn perceived_by(&self, sim: &Sim, sight: &Sight) -> Perception {
        match *self {
            Audience::Owner(owner) => Perception::whole_if(owner == Some(sight.player())),
            Audience::Object(id) => Perception::whole_if(sim.object(id).is_some_and(|o| sight.sees(o))),
            Audience::Seen(seen) => seen.perceived_by(sight),
            Audience::Removal { id, seen } => match seen {
                Some(seen) if sight.remembered_record(id).is_none() => seen.perceived_by(sight),
                _ => Perception::Nothing,
            },
            Audience::Everyone => Perception::Whole,
        }
    }
}

/// Something that happens at a place (module docs): who perceives it whole wherever it is, whom it is hidden from,
/// and where the others see it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sighting {
    /// The one player who perceives it whole wherever it is: the owner of the object it happens to. Nobody for a shot,
    /// a projectile or a blast, which their shooter's owner too sees only where the others do (design change 0293).
    pub whole_to: Option<PlayerId>,
    /// Whom it is hidden from: its object's (`sc_sim::vision::Hiding`).
    pub hiding: Hiding,
    pub place: Place,
}

/// Where a [`Sighting`] happens.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Place {
    /// Along the straight path from `from` to `to` (ground points, su): a shot's or a projectile's, seen over the
    /// viewer's Visible tiles ([`Perception::Clipped`]).
    Path { from: Vec2, to: Vec2 },
    /// At a ground point: seen whole while its tile is Visible to the viewer.
    At(Vec2),
    /// Off the map (a unit inside a Tunnel Network or a site, `sc_sim::network`): nowhere another player could see it.
    OffMap,
}

impl Sighting {
    /// A shot's or a projectile's path from `from` to `to` (ground points), drawn to every viewer only over their
    /// Visible tiles, its shooter's owner included (`core/vision.md > Attacks from outside vision`, design change 0293:
    /// "every shot, tracer or projectile, whoever fires it and at whatever, is drawn only over your Visible tiles").
    /// Placeholder (not in the design: it says nothing of hidden targets): one at an object hidden from the viewer shows
    /// over their Visible tiles all the same, up to where the object is (the design's "at whatever").
    pub fn path(from: Vec2, to: Vec2) -> Self {
        Self {
            whole_to: None,
            hiding: Hiding::default(),
            place: Place::Path { from, to },
        }
    }

    /// What happens at the ground point `at` to no object: a blast (plan decision T52: seen where the viewer has
    /// vision), its shooter's owner's too.
    pub fn at(at: Vec2) -> Self {
        Self {
            whole_to: None,
            hiding: Hiding::default(),
            place: Place::At(at),
        }
    }

    /// What happens to an object of `owner`'s at the ground point `at`, hidden as `hiding` says: whole to its owner, to
    /// the others where it is Visible and not hidden from them. A death; and an object's last place, for its removal.
    pub fn object_at(owner: Option<PlayerId>, hiding: Hiding, at: Vec2) -> Self {
        Self {
            whole_to: owner,
            hiding,
            place: Place::At(at),
        }
    }

    /// What happens to `object` where it stands ([`Sighting::object_at`]), hidden as it is now (`Hiding::of`).
    pub fn of(object: &Object) -> Self {
        Self::object_at(object.owner, Hiding::of(object), object.pos)
    }

    /// What happens to a unit of `owner`'s off the map ([`Place::OffMap`]): to its owner alone.
    pub fn off_map(owner: Option<PlayerId>) -> Self {
        Self {
            whole_to: owner,
            hiding: Hiding::default(),
            place: Place::OffMap,
        }
    }

    /// How `sight`'s player perceives it: whole if it is theirs; nothing off the map; whole while the fog doesn't
    /// apply; nothing if it is hidden from them; and else over their Visible tiles: a path clipped by the drawing code,
    /// a point whole while its tile is Visible.
    pub fn perceived_by(&self, sight: &Sight) -> Perception {
        let player = sight.player();
        match self.place {
            _ if self.whole_to == Some(player) => Perception::Whole,
            Place::OffMap => Perception::Nothing,
            _ if !sight.is_fogged() => Perception::Whole,
            _ if self.hiding.hides_from(player) => Perception::Nothing,
            Place::Path { from, to } => Perception::Clipped { from, to },
            Place::At(at) => Perception::whole_if(sight.at(at) == VisibilityState::Visible),
        }
    }
}

/// How a viewer perceives an event ([`Audience::perceived_by`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Perception {
    /// All of it, as it happened.
    Whole,
    /// Only along the parts of the path from `from` to `to` (ground points) over the viewer's Visible tiles, which the
    /// drawing code finds: a shot's ([`crate::combat_effects::visible_spans`]), a projectile's point on the ground
    /// ([`crate::combat_effects::shows_flight`]).
    Clipped { from: Vec2, to: Vec2 },
    /// Nothing of it.
    Nothing,
}

impl Perception {
    fn whole_if(perceived: bool) -> Self {
        if perceived {
            Perception::Whole
        } else {
            Perception::Nothing
        }
    }
}

/// Takes the frame's events from the sim and gives each its audience ([`Arrival::audience`]); then, if the sim changed,
/// takes the objects as they are now, for the events of the frames to come ([`ViewerEvents::known`]). A replaced sim
/// starts again from no object.
fn filter_events(sim: Res<SimRes>, mut messages: MessageReader<SimEventMessage>, mut viewer: ResMut<ViewerEvents>) {
    let ViewerEvents {
        events,
        known,
        generation,
    } = &mut *viewer;
    let replaced = *generation != Some(sim.generation());
    if replaced {
        known.clear();
        *generation = Some(sim.generation());
    }
    let messages: Vec<&SimEventMessage> = messages.read().collect();
    let arrival = Arrival::new(&sim, messages.iter().map(|m| &m.event), known);
    *events = (messages.iter())
        .map(|m| ViewerEvent {
            tick: m.tick,
            audience: arrival.audience(&m.event),
            event: m.event.clone(),
        })
        .collect();
    if replaced || sim.is_changed() {
        take_objects(&sim, known);
    }
}

/// Every object of `sim` as it is seen now ([`Sighting::of`], [`Sighting::off_map`] for the units off the map), into
/// `known`, ids in order.
fn take_objects(sim: &Sim, known: &mut Vec<(ObjectId, Sighting)>) {
    known.clear();
    known.extend(sim.objects().map(|o| (o.id, Sighting::of(o))));
    known.extend((sim.held()).map(|held| (held.object.id, Sighting::off_map(held.object.owner))));
    known.sort_unstable_by_key(|&(id, _)| id);
}

/// What the filter knows as a frame's events arrive: the sim after their ticks, the deaths among them, and the objects
/// as it last saw them.
struct Arrival<'a> {
    sim: &'a Sim,
    /// The frame's deaths: each dead object where it died, as hidden as it was (`SimEvent::Died::hiding`), (M5-V4) the
    /// crushed too (`SimEvent::Crushed`).
    deaths: Vec<(ObjectId, Sighting)>,
    known: &'a [(ObjectId, Sighting)],
}

impl<'a> Arrival<'a> {
    fn new<'e>(sim: &'a Sim, events: impl Iterator<Item = &'e SimEvent>, known: &'a [(ObjectId, Sighting)]) -> Self {
        let deaths = (events)
            .filter_map(|event| match *event {
                SimEvent::Died {
                    id, owner, pos, hiding, ..
                }
                | SimEvent::Crushed {
                    id, owner, pos, hiding, ..
                } => Some((id, Sighting::object_at(owner, hiding, pos))),
                _ => None,
            })
            .collect();
        Self { sim, deaths, known }
    }

    /// Object `id` as it is seen where it is now, on the map or off it, or, gone, where it was last: its death among
    /// the frame's events, else as the filter last saw it. None for an object that came and went within the frame's
    /// ticks.
    fn seen(&self, id: ObjectId) -> Option<Sighting> {
        if let Some(o) = self.sim.object(id) {
            return Some(Sighting::of(o));
        }
        if let Some(held) = self.sim.held_unit(id) {
            return Some(Sighting::off_map(held.object.owner));
        }
        let died = self.deaths.iter().find(|&&(dead, _)| dead == id);
        let known = || {
            let i = self.known.binary_search_by_key(&id, |&(known, _)| known).ok()?;
            Some(&self.known[i])
        };
        died.or_else(known).map(|&(_, seen)| seen)
    }

    /// The business of object `id`'s owner (an object's sighting is whole to its owner).
    fn owner(&self, id: ObjectId) -> Audience {
        Audience::Owner(self.seen(id).and_then(|seen| seen.whole_to))
    }

    /// What happens to object `id`: whoever is shown it, on the map; gone or off it, whoever was shown it where it was
    /// (its owner alone off the map); nobody if the filter never saw it.
    fn object(&self, id: ObjectId) -> Audience {
        if self.sim.object(id).is_some() {
            return Audience::Object(id);
        }
        self.seen(id).map_or(Audience::Owner(None), Audience::Seen)
    }

    /// Whom `event` reaches: one rule for each kind of event (module docs). The match lists them all, so that a new
    /// kind stops it compiling until it has a rule here, and a row in the table (`tests/table.rs`).
    fn audience(&self, event: &SimEvent) -> Audience {
        match *event {
            SimEvent::Spawned(id) | SimEvent::KindChanged { id, .. } | SimEvent::TransformCompleted { id, .. } => {
                self.object(id)
            }
            SimEvent::Removed(id) => Audience::Removal {
                id,
                seen: self.seen(id),
            },
            SimEvent::SuppliesDelivered { station, .. } => self.object(station),
            // Sent to each player whose workers gather there (design change 0256 point 3: "the owner gets a
            // notification").
            SimEvent::SupplyStationEmpty { owner, .. } => Audience::Owner(Some(owner)),
            SimEvent::MoveFailed { unit }
            | SimEvent::EnterFailed { unit, .. }
            | SimEvent::TargetTooClose { unit, .. }
            | SimEvent::LocationTooClose { unit, .. } => self.owner(unit),
            // A TailDisjointed shot that hit nothing is a shot as well (slice M5-O3).
            SimEvent::Attacked { from, to, .. }
            | SimEvent::Missed { from, to, .. }
            | SimEvent::ProjectileLaunched { from, to, .. } => Audience::Seen(Sighting::path(from, to)),
            // A blast's damage too: what it hit underground shows nothing (plan decision T52).
            SimEvent::Hit { target, .. } => self.object(target),
            // (M5-V4) A crush is a death.
            SimEvent::Died { owner, pos, hiding, .. } | SimEvent::Crushed { owner, pos, hiding, .. } => {
                Audience::Seen(Sighting::object_at(owner, hiding, pos))
            }
            SimEvent::PatchDepleted { plate, .. } => self.owner(plate),
            SimEvent::ProductionQueued { producer, .. }
            | SimEvent::ProductionRefused { producer, .. }
            | SimEvent::ProductionCancelled { producer, .. }
            | SimEvent::ProductionWaiting { producer, .. }
            | SimEvent::ReleaseQueued { producer, .. }
            | SimEvent::ReleaseCancelled { producer, .. }
            | SimEvent::AugmentQueued { producer, .. }
            | SimEvent::AugmentCancelled { producer, .. }
            | SimEvent::ResearchQueued { producer, .. }
            | SimEvent::ResearchCancelled { producer, .. }
            | SimEvent::ResearchCompleted { producer, .. } => self.owner(producer),
            SimEvent::UnitProduced { unit, .. } => self.object(unit),
            SimEvent::ConstructionCompleted { constructor, .. }
            | SimEvent::ConstructionCancelled { constructor, .. }
            | SimEvent::PlacementRefused { constructor, .. } => self.owner(constructor),
            SimEvent::StructurePlaced { structure, .. } => self.object(structure),
            // Every player sees the end of the game (`core/game_rules.md > End of Game`), and is told of a defeat
            // before it (`core/game_rules.md > On Defeat`: "**Everyone learns of it**", design change 0357), in the
            // defeat notice (`hud::notices`).
            SimEvent::PlayerDefeated { .. } | SimEvent::GameOver { .. } => Audience::Everyone,
            // (M5-X1b) The game's rules are everyone's (`DebugCommand::SetRules`).
            SimEvent::RulesChanged => Audience::Everyone,
            SimEvent::ResourcesDelivered { worker, .. }
            | SimEvent::BuildFailed { worker, .. }
            | SimEvent::BuildRefunded { worker, .. } => self.owner(worker),
            SimEvent::TransformCancelled { id, .. } => self.owner(id),
            SimEvent::DeployFailed { drone, .. } => self.owner(drone),
            // The unit walks into the Tunnel, or the building that holds it (slice M5-F1), and is off the map from then
            // on.
            SimEvent::EnteredNetwork { tunnel: into, .. } | SimEvent::EnteredBuilding { building: into, .. } => {
                self.object(into)
            }
            SimEvent::LeftNetwork { unit, .. }
            | SimEvent::LeftSite { worker: unit, .. }
            | SimEvent::LeftBuilding { unit, .. }
            | SimEvent::Evacuated { unit, .. } => self.object(unit),
            // An expansion's site is hidden as the expansion is (`SimEvent::ExpansionStarted`).
            SimEvent::SiteStarted { site, .. }
            | SimEvent::SiteCompleted { site, .. }
            | SimEvent::SiteBuilt { site }
            | SimEvent::ExpansionStarted { site, .. }
            | SimEvent::ExpansionCompleted { site, .. } => self.object(site),
            SimEvent::SiteCancelled { site, .. } => self.owner(site),
            SimEvent::ExpansionCancelled { tunnel, .. }
            | SimEvent::UpgradeStarted { tunnel, .. }
            | SimEvent::UpgradeCompleted { tunnel, .. }
            | SimEvent::UpgradeCancelled { tunnel, .. } => self.owner(tunnel),
            SimEvent::Blast { at, .. } => Audience::Seen(Sighting::at(at)),
            // (M5-N4b) A Disciple's channel starting or ending happens to the Disciple. A release happens to the cloud,
            // which every player sees over their Visible tiles (`factions/cults/units/disciple.md > Spells > Shroud`:
            // "the cloud is shown to **every player**, over the tiles that are Visible to them, so its edge can be seen
            // even when the Disciple can't"): to every player, whose drawing keeps it to those tiles, as the cloud's
            // (`crate::shroud_view`).
            SimEvent::ShroudStarted { disciple, .. } | SimEvent::ShroudEnded { disciple } => self.object(disciple),
            SimEvent::ShroudReleased { .. } => Audience::Everyone,
            // (M5-N5) A Shrine's cast shows where it was cast, as a blast: to its owner, who sees the point
            // (`Sim::can_cast`), and to anyone else who does.
            SimEvent::Cast { at, .. } => Audience::Seen(Sighting::at(at)),
        }
    }
}
