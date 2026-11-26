//! (M5-N5) The casts of the area abilities (plan decision T76, `Command::Cast`; `factions/cults/structures/shrine.md >
//! Abilities`): the gate's checks ([`Sim::check_cast`]), the part of them the panel greys Q and W by
//! ([`Sim::can_cast`]), and the cast, in the commands phase ([`Sim::cast`]), whose buffs are `crate::buffs`'.
//!
//! The gate takes a cast when:
//! - one of the casters, the player's, casts the ability (`StructureDef::abilities`), else [`Reject::LacksAbility`];
//! - one of those is in operation (`Object::is_operational`), not a site, else [`Reject::UnderConstruction`];
//! - one of those has the energy it costs (`> Abilities`: "spend Shrine energy"), else [`Reject::NotEnoughEnergy`]:
//!   the first such one in id order casts ("each press casts from one Shrine", Q0118; the client sends a press to the
//!   one Shrine of the ActiveGroup, which is Ungroupable). **Placeholder (not in the design: a command naming several
//!   casters)**;
//! - its point is on the map, else [`Reject::OffMap`], and the player sees it now, as the last tick left their vision
//!   (`> Abilities`: "cast on an area anywhere the player currently has vision"; plan decision T76: "as a Place is
//!   judged by knowledge"): its tile is Visible, else [`Reject::OutOfSight`].
//!
//! "No cooldowns": a Shrine with the energy casts again in the next tick, or the same one.

use glam::Vec2;

use super::Sim;
use super::gate::{Checked, Reject};
use crate::buffs;
use crate::defs::AreaAbility;
use crate::energy;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::units::TilePos;
use crate::vision::VisibilityState;

impl Sim {
    /// Which of `player`'s `casters` would cast `ability` now, wherever it is cast (module docs: the gate's checks but
    /// the point's): the first in id order that casts it, in operation, with the energy it costs. What the panel greys
    /// the Shrine's Q and W by (plan decision T10), and why it would refuse.
    pub fn can_cast(&self, player: PlayerId, casters: &[ObjectId], ability: AreaAbility) -> Result<ObjectId, Reject> {
        let casters = self.own_units(player, casters, |o| buffs::casts(o, ability))?;
        let Some(&first) = casters.first() else {
            return Err(Reject::LacksAbility);
        };
        let operational: Vec<ObjectId> = (casters.into_iter())
            .filter(|id| self.objects[id].is_operational())
            .collect();
        if operational.is_empty() {
            return Err(Reject::UnderConstruction(first));
        }
        let cost = ability.def().cost;
        (operational.into_iter())
            .find(|id| energy::has(&self.objects[id], cost))
            .ok_or(Reject::NotEnoughEnergy)
    }

    /// The gate's checks of `player`'s cast of `ability` at `at` by one of `casters` (module docs).
    pub(super) fn check_cast(
        &self,
        player: PlayerId,
        casters: &[ObjectId],
        ability: AreaAbility,
        at: Vec2,
    ) -> Result<Checked, Reject> {
        let caster = self.can_cast(player, casters, ability)?;
        // `contains_point` also rejects NaN and infinite points.
        if !self.map.contains_point(at) {
            return Err(Reject::OffMap);
        }
        if self.vision.state(player, TilePos::containing(at)) != VisibilityState::Visible {
            return Err(Reject::OutOfSight);
        }
        Ok(Checked::Cast { caster, ability, at })
    }

    /// `caster` casts `ability` at `at`, as the gate passed it: it pays the energy, and its owner's units in the circle
    /// get the buff (`crate::buffs::give`), in the commands phase, where the last tick left them.
    pub(super) fn cast(&mut self, caster: ObjectId, ability: AreaAbility, at: Vec2) {
        let Some(o) = self.objects.get_mut(&caster) else {
            return;
        };
        let (Some(owner), Some(energy)) = (o.owner, o.energy.as_mut()) else {
            return;
        };
        // The gate passed it with the energy (`energy::has`): it pays it all.
        if !energy::spend(energy, energy::units(ability.def().cost)) {
            return;
        }
        buffs::give(&mut self.objects, owner, ability, at, self.tick);
        self.events.push(SimEvent::Cast { caster, ability, at });
    }
}
