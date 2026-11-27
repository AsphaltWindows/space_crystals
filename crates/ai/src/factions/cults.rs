//! The Cults' computer player (slice A2a; `factions/cults/overview.md`; `core/game_rules.md > Computer players
//! (playtest tool)`: its behavior "is the Director's placeholder"). From the Cults' Starting Conditions
//! (`factions/cults/economy.md > Starting Conditions`: a Recruitment Hall, a Storage and six Recruits), each tick, in
//! this order of priority for the bank:
//!
//! 1. **The Armory trains** (`armory.rs`; `factions/cults/structures/armory.md > Training`) a Militant
//!    (`factions/cults/units/militant.md`) of each Recruit inside that no order holds yet, or, once a Gunworks stands,
//!    a Gunner (`factions/cults/units/gunner.md`) every [`GUNNER_EVERY`] units, while the bank covers them: its basic
//!    army without pause, which the Armory rallies to the staging point.
//! 2. **One Build at a time** (`builds.rs`), the next once its site has started: the first the base wants of a
//!    Storage, once every seat of the field is taken; the Armory; a Recruitment Hall, up to [`HALLS`]; and a Gunworks,
//!    once it owns [`GUNWORKS_AFTER_HALLS`] complete Halls. Each goes to its site (`sites.rs`) and is built by the Recruits
//!    nearest it, who go back to their seats once it is done (`factions/cults/economy.md > Cults Building Mechanics >
//!    Build Command`). One the bank can't pay for yet holds the bank.
//! 3. **The Recruits** (`field.rs`): each one without work (a new one out of its Hall, a builder done, one whose seat
//!    is gone) is sent to a free seat of the field, until the field is saturated, else into the Armory: the Recruits it
//!    doesn't need become its army (`factions/cults/units/recruit.md > Armory Entry`).
//!
//! More Halls raise its Unit Control (`factions/cults/overview.md > CultsResources > Unit Control`: "Provided by
//! Recruitment Halls in proportion to the number of Recruitable tiles each hall is actively recruiting from"): each
//! goes on open Recruitable land, where it claims most of its area (`HALL_EFFECTIVENESS`), within the ground near its
//! start that an enemy's waves search (`HALL_SEARCH_GU`; `crate::army::search_points`, around the starts of the
//! factions in `crate::tuning::army::SEARCH_GROUND_OF`). The Halls make their Recruits by themselves
//! (`factions/cults/structures/recruitment_hall.md > Production Behavior`), so the player has nothing to train but the
//! Armory's units. The army layer (`crate::army`) leads the Militants and the Gunners. Every rule and number is a
//! placeholder (Q0147 points 2 and 3, `crate::tuning::cults`). Costs and times come from the defs.

mod armory;
mod builds;
mod field;
mod sites;

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::{Object, ObjectId, ObjectKind, Tick, Vec2};

use super::{FactionAi, Hook};
use crate::tuning::cults::{FIELD_RADIUS_GU, GUNNER_EVERY, GUNWORKS_AFTER_HALLS, HALLS};
use crate::{Profile, Turn, View};

use builds::Pending;
use field::{Field, Seat};

use ObjectKind::{Armory, Gunner, Militant};

pub(super) const HOOK: Option<Hook> = Some(new);

fn new(view: &View<'_>, _profile: Profile) -> Box<dyn FactionAi> {
    Box::new(Cults::new(view))
}

/// The Cults' computer player (module docs).
#[derive(Debug, Default)]
struct Cults {
    /// The patches of its starting field, by id: those the map placed within [`FIELD_RADIUS_GU`] of its start.
    field: Vec<ObjectId>,
    /// Each Recruit it sent to the field, with its seat, until the Recruit is idle again or the seat is gone.
    seated: BTreeMap<ObjectId, Seat>,
    /// The buildings ordered and not complete yet, in the order they were ordered.
    builds: Vec<Pending>,
    /// For each kind whose site a search didn't find, the tick from which it looks again.
    retry: BTreeMap<ObjectKind, Tick>,
    /// The units the Armory has been ordered to train.
    trained: u32,
}

impl Cults {
    /// The Cults' player whose View it is, as the game starts: its field, the patches near its start.
    fn new(view: &View<'_>) -> Self {
        let home = view.my_start().map(|s| s.point());
        let radius = sc_sim::gu(FIELD_RADIUS_GU);
        let field = (view.patches())
            .filter(|k| home.is_some_and(|home| k.pos.distance_squared(home) <= radius * radius))
            .map(|k| k.id)
            .collect();
        Self {
            field,
            ..Self::default()
        }
    }
}

impl FactionAi for Cults {
    fn think(&mut self, turn: &mut Turn<'_, '_>) {
        let view = turn.view();
        let field = Field::of(view, &self.field);
        let gone = self.keep_seats(view, &field);
        self.follow(view);
        let mut given = BTreeSet::new();
        self.train(turn);
        self.build(turn, &field, &mut given);
        self.deal(turn, &field, &given, &gone);
    }

    fn fights(&self, unit: &Object) -> bool {
        matches!(unit.kind, Militant | Gunner)
    }

    fn rallies(&self, building: &Object) -> bool {
        building.kind == Armory
    }
}

/// The player's own objects of `kind` on the map, in id order.
fn own<'a>(view: &View<'a>, kind: ObjectKind) -> impl Iterator<Item = &'a Object> + 'a {
    view.own().filter(move |o| o.kind == kind)
}

/// The centre of a footprint of `w` × `h` tiles whose top-left tile is `origin`, space units.
fn centre(origin: sc_sim::TilePos, (w, h): (u32, u32)) -> Vec2 {
    origin.corner() + Vec2::new(w as f32, h as f32) * (sc_sim::SU_PER_GU / 2.0)
}

/// The Halls the player owns, complete or under construction.
fn halls(view: &View<'_>) -> usize {
    own(view, ObjectKind::RecruitmentHall).count()
}

/// Whether the Armory's next unit is a Gunner: one every [`GUNNER_EVERY`] once the player owns a complete Gunworks
/// (the Gunner's TechPrerequisite, `factions/cults/structures/armory.md > Training`).
fn gunner_next(view: &View<'_>, trained: u32) -> bool {
    let gunworks = own(view, ObjectKind::Gunworks).any(|o| o.is_operational());
    gunworks && trained.saturating_add(1).is_multiple_of(GUNNER_EVERY)
}

/// Whether the base wants its Gunworks: it owns [`GUNWORKS_AFTER_HALLS`] complete Halls and a complete Armory, and no
/// Gunworks.
fn wants_gunworks(view: &View<'_>) -> bool {
    let complete = |kind| own(view, kind).filter(|o| o.is_operational()).count();
    complete(ObjectKind::RecruitmentHall) >= GUNWORKS_AFTER_HALLS.min(HALLS)
        && complete(Armory) > 0
        && own(view, ObjectKind::Gunworks).next().is_none()
}
