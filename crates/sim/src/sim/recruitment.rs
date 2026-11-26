//! The Recruitment Halls in a game (`crate::recruitment`, plan decision T57, slice M5-C;
//! `factions/cults/structures/recruitment_hall.md > Production Behavior`): "The Recruitment Hall automatically and
//! continuously produces Recruits at no resource cost until LocalUnitControlUsage reaches LocalUnitControlCapacity.
//! Produced Recruits come out of the exit side (B) and move to its rally point. Production can't be cancelled [...].
//! Production logic is entirely local — global Unit Control state does not affect production decisions."
//!
//! A Hall's production is `crate::production`'s, in an automatic mode: what the producers' step does for it
//! (`sim/producers/step.rs`) differs only where its `ProductionDef::recruitment` says so.
//! - **The start** ([`Sim::start_recruits`], first in the producers' step): a completed Hall with nothing in production
//!   queues its Recruit while its usage is below its capacity rounded down (design change 0341 point 2: a Hall with 33
//!   claimed tiles stops at 6), whatever its owner's global Unit Control (design change 0344: "Global total is for
//!   display only"). The Recruit costs no Space Crystals, and counts toward the Hall's usage from the start of its
//!   production (design change 0342: "the Recruit in production counts toward LocalUnitControlUsage from the start of
//!   its production"), so the Hall at 19 with its 20th in production shows 20/20. At capacity no Recruit is in
//!   production, and production starts again from 0% when a unit of its lineage dies
//!   (`factions/cults/structures/recruitment_hall.md > Production Behavior`: "when capacity frees up (a unit dies), the
//!   next Recruit starts from 0"; 0342 point 4). A Hall that came into the game in the tick waits for the next, as
//!   every producer does. A Hall that releases its Recruit queues the next one at once, the released one counted
//!   ([`Sim::recruit_next`]): it shows 0% until the next tick's step gets it on, as the next unit in any queue does,
//!   rather than nothing; "automatically and continuously produces Recruits". So the k-th Recruit of a Hall at 100%
//!   comes out in tick 192 k.
//! - **The progress** gains the tiles the Hall claims each frame, whatever its owner's power; the Recruit is done at
//!   BaseRecruitProductionTime × the area's tiles (`production::total_of`): 192 frames at 100 tiles, 384 at 50.
//! - **The release** is every producer's (`Sim::release_unit`): out of the one exit side, B (`ExitRule::Side`; design
//!   change 0342: "It becomes AAAB with the exit on B (the side is a placeholder). Recruits come out of it one at a
//!   time through the Exit Doorway"); the Recruit's lineage is the Hall (`Object::lineage`, plan decision T56), and its
//!   order the rally's, the default order on what the rally is on (slice M3-R: a patch makes it gather).
//! - **No X** (design change 0342: "Recruits can't be cancelled"): a `CancelProduction` finds nothing to cancel at a
//!   Hall (`Reject::NothingQueued`, `Sim::check_production`).
//! - **A `Train`** is refused: a Hall trains nothing on command (`Reject::CannotProduce`).
//!
//! A Hall's ClaimedTiles, Effectiveness, LocalUnitControlUsage and LocalUnitControlCapacity are derived for the
//! interface on demand ([`Sim::hall_state`]).

use super::Sim;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::production::QueueEntry;
use crate::recruitment::{self, Capacity, Claims, HallState, Lineages};
use crate::units::TilePos;

impl Sim {
    /// Every object of the game: the map's and the units held off it (`Sim::held`), whose lineages count as theirs.
    fn every_object(&self) -> impl Iterator<Item = &Object> + '_ {
        self.objects.values().chain(self.held.values().map(|held| &held.object))
    }

    /// The tiles each completed Hall claims at the end of the latest tick (`crate::recruitment::Claims`).
    pub fn claims(&self) -> Claims {
        Claims::of(self.objects.values(), &self.map)
    }

    /// (M5-J2) The tiles `hall` claims at the end of the latest tick, in tile order (`crate::recruitment::claimed_tiles`):
    /// what a selected own Hall shows (`factions/cults/structures/recruitment_hall.md > Recruitment Area display`);
    /// none for an object that isn't a completed Hall.
    pub fn claimed_tiles(&self, hall: ObjectId) -> Vec<TilePos> {
        recruitment::claimed_tiles(self.objects.values(), &self.map, hall)
    }

    /// What Hall `id`'s state shows at the end of the latest tick (`factions/cults/structures/recruitment_hall.md >
    /// RecruitmentHallInstanceState`): the tiles it claims of its area's, its LocalUnitControlUsage and its
    /// LocalUnitControlCapacity. `None` for an object that isn't a Recruitment Hall. A Hall under construction claims
    /// nothing (design change 0341: "A Hall under construction does nothing until it is complete").
    pub fn hall_state(&self, id: ObjectId) -> Option<HallState> {
        let def = recruitment::def_of(self.objects.get(&id)?.kind)?;
        let claimed = self.claims().tiles(id);
        Some(HallState {
            claimed,
            tiles: def.tiles(),
            usage: Lineages::of(self.every_object()).usage(id),
            capacity: Capacity::of(def, claimed),
        })
    }

    /// The Halls' start (module docs), first in the producers' step: each completed Hall from before `first_new`, with
    /// an owner and nothing in production, queues its Recruit if its usage is below its capacity. Returns the claims,
    /// by which the step advances each Hall's Recruit. Nothing is derived in a game without a Hall.
    pub(super) fn start_recruits(&mut self, first_new: ObjectId) -> Claims {
        if !self.objects.values().any(|o| recruitment::def_of(o.kind).is_some()) {
            return Claims::default();
        }
        let claims = self.claims();
        let idle: Vec<ObjectId> = (self.objects.range(..first_new))
            .filter(|(_, o)| o.owner.is_some() && o.is_operational() && recruitment::def_of(o.kind).is_some())
            .filter(|(_, o)| o.production.as_ref().is_some_and(|p| p.queue.is_empty()))
            .map(|(&id, _)| id)
            .collect();
        if idle.is_empty() {
            return claims;
        }
        let lineages = Lineages::of(self.every_object());
        for id in idle {
            self.queue_recruit(id, &claims, &lineages);
        }
        claims
    }

    /// Hall `id` released its Recruit in the producers' step (module docs): the next one is queued at once if the Hall
    /// still has room, the released Recruit counted.
    pub(super) fn recruit_next(&mut self, id: ObjectId, claims: &Claims) {
        let empty = (self.objects.get(&id)).is_some_and(|o| o.production.as_ref().is_some_and(|p| p.queue.is_empty()));
        if empty {
            let lineages = Lineages::of(self.every_object());
            self.queue_recruit(id, claims, &lineages);
        }
    }

    /// Queues Hall `id`'s Recruit if its usage by `lineages` is below its capacity by `claims`, rounded down
    /// (`Capacity::has_room`).
    fn queue_recruit(&mut self, id: ObjectId, claims: &Claims, lineages: &Lineages) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let production_def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
        let (Some(def), Some(production)) = (production_def, &mut o.production) else {
            return;
        };
        let (Some(recruitment), Some(recipe)) = (def.recruitment, def.recipes.first()) else {
            return;
        };
        if Capacity::of(&recruitment, claims.tiles(id)).has_room(lineages.usage(id)) {
            production.push(QueueEntry::Unit(recipe.kind), Vec::new());
        }
    }
}
