//! (M5-N3) Research in a game (plan decisions T70 and T74; `core/tech_and_research.md > Research`): a Temple researches
//! the Cults' technologies (`defs/cults/research.rs`) on its production queue (`QueueEntry::Research`), staffed by the
//! Recruits inside it (`factions/cults/structures/temple.md > Recruit Staffing`).
//!
//! - **Queuing** ([`Sim::plan_research`], the gate of `ProductionCommand::Research`), checked in T74's order: no
//!   duplicate (`core/tech_and_research.md > Research > Shared Rules` 4: "a technology cannot be queued or in progress
//!   at more than one structure at the same time. Once completed, it no longer appears in any research menu";
//!   [`Reject::Researched`], [`Reject::ResearchUnderWay`]); the previous level completed (`core/tech_and_research.md >
//!   Tech Prerequisites > Research Prerequisites`: "A next level can only be queued once the previous level has
//!   **completed**"; [`Reject::RequiresTech`]); the structures it requires, at queue only ("**checked only when it is
//!   queued**"; [`Reject::Requires`], `crate::tech`); then room in a queue and the cost, "deducted when the technology
//!   is queued" (Shared Rules 2). One structure takes it: of several, the one with the least work queued, ties by id
//!   (design change 0083's order, `Sim::serve`'s).
//! - **Progress** (`Sim::step_production`): a staffed building's research gains one `RATIO_ONE` a frame for each unit
//!   staffing it ([`Sim::staff`]), none without one (`core/tech_and_research.md > Research > Faction Exceptions > Cults
//!   — research is staffed by Recruits`: "**Linear scaling**: effective research time scales down linearly with the
//!   number of Recruits inside — 2 Recruits halve it, 3 reduce it to a third"; "**Pauses when empty**: if no Recruits
//!   are inside, the in-progress technology pauses at its current progress (no decay) until Recruits return. Queued
//!   research waits likewise"). Exact in integers: Enhanced Constitution Level 1's 960 frames take 320 with three
//!   Recruits.
//! - **Completion** ([`Sim::complete_research`]): the technology joins its owner's (`Player::techs`) as it leaves the
//!   queue, the next one starting in the next tick (Shared Rules 3: "processed in queue order"), and every unit of
//!   the owner's with the line's Augmentation, on the map or held, takes the new amounts at once
//!   (`factions/cults/augmentation.md > Research applies to everyone`). Its speed and its weapon follow by themselves,
//!   read through the accessors (`augmentation::Research`); its HP follows its MaxHP (`augmentation::follow_max`). In
//!   the producers' step, so before an Altar's unit that comes out in the same tick is augmented (`Sim::step_holding`,
//!   `Sim::augment`): "the research level in effect at that moment" includes research finishing that frame (T74).
//! - **X and destruction** are every entry's (`Sim::apply_production`, `Sim::refund_production`): the full cost back
//!   (Shared Rules 6 and 7), the staff evacuated as from any building that holds units (`sim/holding/evacuation.rs`).
//!   Losing the Temple or the Cathedral later revokes nothing (Shared Rules 8, and `core/tech_and_research.md > Tech
//!   Prerequisites > Research Prerequisites`: "losing a prerequisite never cancels or disables anything already started
//!   or finished").

use super::{ProductionPlan, producer};
use crate::augmentation::{Levels, Research, follow_max};
use crate::defs::{ResearchDef, ResearchLine, Tech};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::network::Place;
use crate::object::Object;
use crate::production::QueueEntry;
use crate::sim::{Reject, Sim};

/// What `o`'s type researches (`StructureDef::research`): `None` for anything but a research structure.
pub(in crate::sim) fn research_of(o: &Object) -> Option<&'static ResearchDef> {
    o.def().structure.as_ref()?.research.as_ref()
}

impl Sim {
    /// How many units staff `building` (module docs): those inside it that aren't on their way out (`Held::leaving`),
    /// Recruits all, the only kind a Temple takes in ("Only Recruits can staff research"; `HoldsDef::accepts`). 0 for
    /// anything but a building that holds units. It allocates nothing.
    pub fn staff(&self, building: ObjectId) -> u32 {
        let inside = (self.held.values())
            .filter(|held| held.place == Place::Building(building) && held.leaving.is_none())
            .count();
        u32::try_from(inside).unwrap_or(u32::MAX)
    }

    /// Where `player`'s `tech` is queued or in progress: the structure of theirs whose queue holds it
    /// (`core/tech_and_research.md > Research > Shared Rules` 4: one at most). `None` while it is neither.
    pub fn research_site(&self, player: PlayerId, tech: Tech) -> Option<ObjectId> {
        let entry = QueueEntry::Research { tech };
        (self.objects.values())
            .find(|o| o.owner == Some(player) && o.production.as_ref().is_some_and(|p| p.queue.contains(&entry)))
            .map(|o| o.id)
    }

    /// The next level of `line` for `player`: the first of its technologies they haven't completed, what the Temple's
    /// keys offer (`factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced Constitution (next
    /// level), **W** Improved Metabolism (next level)"). `None` once both are completed, or for no player of the game.
    pub fn next_tech(&self, player: PlayerId, line: ResearchLine) -> Option<Tech> {
        let p = self.player(player)?;
        line.techs().into_iter().find(|&tech| !p.has(tech))
    }

    /// The gate's rule for a `ProductionCommand::Research` of `tech` from `player` to `producers`, theirs (module
    /// docs): [`Reject::CannotResearch`] if none of them offers it, then T74's checks, each refusing the whole command,
    /// then the structure that takes it, the one with room in its queue (`ResearchDef::queue`, else
    /// [`Reject::QueueFull`]) and the least work queued, if the bank covers the cost
    /// ([`Reject::InsufficientCrystals`]).
    pub(super) fn plan_research(
        &self,
        player: PlayerId,
        producers: &[ObjectId],
        tech: Tech,
    ) -> Result<ProductionPlan, Reject> {
        let offering: Vec<&Object> = (producers.iter())
            .map(|id| &self.objects[id])
            .filter(|o| research_of(o).is_some_and(|research| research.offers(tech)))
            .collect();
        if offering.is_empty() {
            return Err(Reject::CannotResearch(tech));
        }
        let p = self.player(player).ok_or(Reject::UnknownPlayer(player))?;
        if p.has(tech) {
            return Err(Reject::Researched(tech));
        }
        if self.research_site(player, tech).is_some() {
            return Err(Reject::ResearchUnderWay(tech));
        }
        let def = tech.def();
        if let Some(previous) = def.requires.filter(|&previous| !p.has(previous)) {
            return Err(Reject::RequiresTech(previous));
        }
        self.check_requires(player, def.requires_structures.iter().copied())?;
        let with_room = (offering.into_iter()).filter_map(|o| {
            let ((production, production_def), research) = (producer(o)?, research_of(o)?);
            (production.queue.len() < research.queue).then(|| (production.remaining_work(production_def), o.id))
        });
        let Some((_, id)) = with_room.min() else {
            return Err(Reject::QueueFull);
        };
        if p.crystals < def.cost {
            return Err(Reject::InsufficientCrystals);
        }
        Ok(ProductionPlan::Train {
            entry: QueueEntry::Research { tech },
            accepted: vec![(id, Vec::new())],
            refused: Vec::new(),
        })
    }

    /// `building` completed `tech`, which has left its queue, in the producers' step (module docs): its owner has it
    /// from now on, and each of the owner's units with the Augmentation of `tech`'s line, on the map or held, takes the
    /// new amounts, its HP following its MaxHP ([`follow_max`]: "if MaxHP goes up, HP rises by the same amount"; one
    /// that goes down only caps it): an Enhanced Constitution Recruit at 40/60 goes to 60/80 with Level 1. A
    /// `SimEvent::ResearchCompleted` says so.
    pub(in crate::sim) fn complete_research(&mut self, building: ObjectId, tech: Tech) {
        let Some(owner) = self.objects.get(&building).and_then(|o| o.owner) else {
            return;
        };
        let Some(p) = self.players.iter_mut().find(|p| p.id == owner) else {
            return;
        };
        let before = Levels::of(p);
        p.techs.insert(tech);
        let after = Levels::of(p);
        let line = tech.def().line;
        let held = self.held.values_mut().map(|held| &mut held.object);
        for o in self.objects.values_mut().chain(held) {
            if o.owner != Some(owner) || !line.improves(o.augmentation) {
                continue;
            }
            let max = |levels| o.max_hp_with(Research::at(levels));
            if let (Some(hp), Some(before), Some(after)) = (o.hp, max(before), max(after)) {
                o.hp = Some(follow_max(hp, before, after));
            }
        }
        self.events.push(SimEvent::ResearchCompleted {
            producer: building,
            tech,
        });
    }
}
