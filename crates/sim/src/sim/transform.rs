//! Transformations in a game (`crate::transform`, slice M3-E): the rules of the Warp In and its cancel in the command
//! gate, what applying them does, and the step that advances every transformation and completes it.

use super::Sim;
use super::construction::Resolved;
use super::gate::Reject;
use super::kinds::Capabilities;
use crate::defs::ObjectKind;
use crate::economy::{RATIO_ONE, Refund};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::production::Production;
use crate::ruleset::rules;
use crate::scenario::BuildingOrientation;
use crate::tech;
use crate::transform::{self, Transform, follow_max, keep_fraction};

impl Sim {
    /// The Beacon of a Warp In among `objects` (plan decision T22: "names one Beacon, as Place names one
    /// constructor"): the player's objects with a Warp In list ([`transform::warp_in_recipes`]), as the player's
    /// constructors are found (ids that no longer exist dropped, anyone else's object [`Reject::NotOwned`]): exactly
    /// one, else [`Reject::NoConstructor`] or [`Reject::SeveralConstructors`].
    pub(super) fn warper(&self, player: PlayerId, objects: &[ObjectId]) -> Result<ObjectId, Reject> {
        let beacons = self.own_units(player, objects, |o| !transform::warp_in_recipes(o.kind).is_empty())?;
        match beacons[..] {
            [] => Err(Reject::NoConstructor),
            [beacon] => Ok(beacon),
            _ => Err(Reject::SeveralConstructors),
        }
    }

    /// `ConstructionCommand::WarpIn` (`factions/colonists/economy.md > Warp In > Warp In Command`), in this order:
    /// one Beacon of the player's ([`Sim::warper`]), complete, not transforming ([`Reject::ConstructorBusy`]: a
    /// Beacon still being deployed, slice M3-F); `kind` on its Constructs List ([`Reject::CannotConstruct`]); what its
    /// entry and the building require owned ([`Reject::Requires`], slice M4-K, `crate::tech`: "Warp In begun",
    /// `core/tech_and_research.md > Tech Prerequisites > Check Timing`); a bank that covers the cost as the command
    /// applies, after the earlier commands of the tick ([`Reject::InsufficientCrystals`]); and the footprint, derived
    /// from the Beacon ([`transform::warp_in_footprint`], design change 0121), allowed for a Warp In by the placement
    /// check ([`Sim::warp_in_check`]: else [`Reject::Unplaceable`] with its first problem, a tile out of the player's
    /// sight included, design change 0119).
    pub(super) fn check_warp_in(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        kind: ObjectKind,
        orientation: BuildingOrientation,
    ) -> Result<Resolved, Reject> {
        let beacon = self.warper(player, objects)?;
        let o = self.objects.get(&beacon).ok_or(Reject::UnknownObject(beacon))?;
        if o.is_transforming() {
            return Err(Reject::ConstructorBusy(beacon));
        }
        let recipe = transform::warp_in_recipe(o.kind, kind).ok_or(Reject::CannotConstruct(kind))?;
        self.check_requires(player, tech::recipe_requires(recipe))?;
        if self.player(player).is_none_or(|p| p.crystals < recipe.cost) {
            return Err(Reject::InsufficientCrystals);
        }
        let check = self
            .warp_in_check(beacon, kind, orientation)
            .ok_or(Reject::NoConstructor)?;
        if let Some(problem) = check.first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        // A valid footprint is a structure's.
        let placement = o
            .placement
            .and_then(|beacon| transform::warp_in_footprint(&beacon, kind, orientation))
            .ok_or(Reject::CannotConstruct(kind))?;
        Ok(Resolved::WarpIn {
            beacon,
            kind,
            placement,
        })
    }

    /// `ConstructionCommand::CancelTransform` (`factions/colonists/economy.md > Cancel Warp In`): the player's objects
    /// among `objects` that are transforming, in id order (ids that no longer exist dropped, anyone else's object
    /// [`Reject::NotOwned`]); none is [`Reject::NotTransforming`].
    pub(super) fn check_cancel_transform(&self, player: PlayerId, objects: &[ObjectId]) -> Result<Resolved, Reject> {
        let transforming = self.own_units(player, objects, Object::is_transforming)?;
        if transforming.is_empty() {
            return Err(Reject::NotTransforming);
        }
        Ok(Resolved::CancelTransform { objects: transforming })
    }

    /// Beacon `beacon` of `player`'s, checked by the gate, warps in `kind` at `placement`
    /// (`factions/colonists/economy.md > Warp In Command`: "The Beacon is consumed and begins transforming in place
    /// into the selected building"): the bank pays the cost (`factions/colonists/economy.md > Constructs List`), and
    /// the Beacon becomes the building from this frame on (plan decision T15, [`Sim::change_kind`]), on its footprint,
    /// with its functions held back ([`hold_back`]) and its HP and Shields as they were: at no frame done, their maxima
    /// are still the Beacon's (`factions/colonists/economy.md > HP/Shields Interpolation`).
    pub(super) fn start_warp_in(&mut self, player: PlayerId, beacon: ObjectId, kind: ObjectKind, placement: Placement) {
        let Some(o) = self.objects.get(&beacon) else {
            return;
        };
        let Some(recipe) = transform::warp_in_recipe(o.kind, kind) else {
            return;
        };
        let (from, from_placement, hp, shields) = (o.kind, o.placement, o.hp, o.shields);
        let Some(bank) = self.players.iter_mut().find(|p| p.id == player) else {
            return;
        };
        // The gate made sure that the bank pays for it.
        if !bank.spend_crystals(recipe.cost) {
            return;
        }
        self.change_kind(beacon, kind, Some(placement));
        let Some(o) = self.objects.get_mut(&beacon) else {
            return;
        };
        o.hp = hp;
        o.shields = shields;
        hold_back(o);
        o.transform = Some(Transform::new(from, from_placement, recipe));
    }

    /// Object `id` gives up its transformation (`factions/colonists/economy.md > Cancel Warp In`: "Cancelling refunds
    /// 75% of the building's resource cost (rounded down), and the structure reverts to being a normal,
    /// fully-functional Beacon (no HP/Shields loss)"): its owner gets back [`Refund::Ready`] of what it paid, with a
    /// `SimEvent::TransformCancelled`; it turns back into what it was where it stood ([`Sim::change_kind`]: its old
    /// footprint, design change 0109, or for a unit where it is), its capabilities as that kind's; and it keeps the
    /// fraction of its HP it had, at least 1 ([`keep_fraction`], `Construction::min_hp_after_cancel`, design change
    /// 0111), and the fraction of its Shields, with no minimum ("Shields by the same fraction (no minimum needed)").
    /// Nothing for an object that isn't an owned one transforming.
    pub(super) fn cancel_transform(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get(&id) else {
            return;
        };
        let (Some(owner), Some(t)) = (o.owner, o.transform.clone()) else {
            return;
        };
        let kind = o.kind;
        let restored = t.from.def().max_hp;
        let hp = match (o.hp, o.max_hp(), restored) {
            (Some(hp), Some(max), Some(restored)) => {
                Some(keep_fraction(hp, max, restored).max(rules().tuning.construction.min_hp_after_cancel))
            }
            _ => restored,
        };
        // 0111: "Shields by the same fraction (no minimum needed)" (`crate::shields`).
        let restored = t.from.def().max_shields;
        let shields = match (o.shields, o.max_shields(), restored) {
            (Some(shields), Some(max), Some(restored)) => Some(keep_fraction(shields, max, restored)),
            _ => restored,
        };
        let refund = Refund::Ready.of(t.paid);
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
            bank.credit_crystals(refund);
        }
        self.events.push(SimEvent::TransformCancelled { id, kind, refund });
        self.change_kind(id, t.from, t.from_placement);
        if let Some(o) = self.objects.get_mut(&id) {
            o.transform = None;
            o.hp = hp;
            o.shields = shields;
            // A deployment's Drone "returns still carrying" its load (design change 0114, `crate::deploy`).
            if let Some(worker) = &mut o.worker {
                worker.load = t.load;
            }
        }
    }

    /// The transformations' part of the economy phase (`crate::transform`; plan decision T5's position, after
    /// construction), for the objects from before this tick (`first_new`) in id order: each transformation, one
    /// that a command of this tick started included (its first frame), advances a frame (plan decision T21: the
    /// Colonists have no power). Its maxima follow ([`Object::max_hp`], [`Object::max_shields`]), and its HP and its
    /// Shields rise by what their maxima rise ([`follow_max`], design change 0110: "current += (newMax − oldMax) each
    /// frame"; `crate::shields`). One whose frames are all done completes ([`Sim::complete_transform`]).
    pub(super) fn step_transforms(&mut self, first_new: ObjectId) {
        let mut completed = Vec::new();
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            if !o.is_transforming() {
                continue;
            }
            let before = (o.max_hp(), o.max_shields());
            let complete = o.transform.as_mut().is_some_and(|t| t.advance(RATIO_ONE));
            let after = (o.max_hp(), o.max_shields());
            if let (Some(hp), Some(before), Some(after)) = (&mut o.hp, before.0, after.0) {
                *hp = follow_max(*hp, before, after);
            }
            if let (Some(shields), Some(before), Some(after)) = (&mut o.shields, before.1, after.1) {
                *shields = follow_max(*shields, before, after);
            }
            if complete {
                completed.push(o.id);
            }
        }
        for id in completed {
            self.complete_transform(id);
        }
    }

    /// Object `id`'s transformation is complete (`factions/colonists/economy.md > Warp In`;
    /// `factions/colonists/economy.md > Beacon Deployment > Completion`: "the transformation is final and one-way"): it
    /// no longer transforms, and its functions start, as a new object of its kind has them ([`Capabilities::fresh`]): a
    /// portal accepts Train from the next tick. With a `SimEvent::TransformCompleted`.
    fn complete_transform(&mut self, id: ObjectId) {
        self.start_functions(id);
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        o.transform = None;
        self.events.push(SimEvent::TransformCompleted { id, kind: o.kind });
    }

    /// What [`hold_back`] held back of object `id` starts: its production, its construction and its mining, as a new
    /// object of its kind has them ([`Capabilities::fresh`]). A transformation's at its completion, and a Tunnel's
    /// expansion's (slice M4-F, `sim/expansions.rs`). (M5-S6) The rally point it was given meanwhile stays (design
    /// change 0396: "The rally point is kept when the building completes"; `sim/producers.rs`).
    pub(super) fn start_functions(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get(&id) else {
            return;
        };
        let fresh = Capabilities::fresh(&self.objects, o.kind, o.placement.as_ref(), self.tick);
        if let Some(o) = self.objects.get_mut(&id) {
            let rally = o.production.as_ref().and_then(|production| production.rally);
            o.production = fresh.production.map(|production| Production { rally, ..production });
            o.constructor = fresh.constructor;
            o.extractor = fresh.extractor;
        }
    }
}

/// What a transforming object holds back until it completes (design change 0116: "Production, drop-off, research
/// and abilities start at completion"): the capabilities a structure works with, its production (so it takes no
/// Train), its construction and its mining, which completion gives it as a new object of its kind has them
/// ([`Sim::start_functions`]). (M5-S6) Not its rally point, which it takes meanwhile (design change 0396, over 0116's
/// "no Set Rally Point"; `sim/producers.rs`): (M5-S8) one it has already, a vehicle building's site's from its
/// placement (`production::default_rally`), stays, in a production that holds it alone, as one set meanwhile is
/// (`sim/producers.rs`'s `set_rally`). What its kind does by itself, such as an Egress Node taking drop-offs
/// (`sim/workers.rs`), checks [`Object::is_transforming`]. A Tunnel's expansion holds them back while it is built
/// (slice M4-F).
pub(super) fn hold_back(o: &mut Object) {
    let rally = o.production.as_ref().and_then(|production| production.rally);
    o.production = rally.map(|rally| Production {
        rally: Some(rally),
        ..Production::default()
    });
    o.constructor = None;
    o.extractor = None;
}
