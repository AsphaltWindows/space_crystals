//! The GDO's computer player (`factions/gdo/overview.md`). Its play comes with its own commit; for now it gives no
//! commands.

use sc_sim::{Object, ObjectKind};

use super::{FactionAi, Hook};
use crate::{Profile, Turn, View};

pub(super) const HOOK: Option<Hook> = Some(new);

fn new(_view: &View<'_>, _profile: Profile) -> Box<dyn FactionAi> {
    Box::new(Gdo)
}

/// The GDO's computer player.
#[derive(Debug)]
struct Gdo;

impl FactionAi for Gdo {
    fn think(&mut self, _turn: &mut Turn<'_, '_>) {}

    fn fights(&self, unit: &Object) -> bool {
        unit.kind == ObjectKind::Peacekeeper
    }

    fn rallies(&self, building: &Object) -> bool {
        building.kind == ObjectKind::Barracks
    }
}
