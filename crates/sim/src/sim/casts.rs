//! (M5-N5) The casts of the area abilities (plan decision T76, `Command::Cast`; `factions/cults/structures/shrine.md >
//! Abilities`): the gate's checks ([`Sim::check_cast`]), the part of them the panel greys Q and W by
//! ([`Sim::can_cast`]), and the cast, in the commands phase ([`Sim::cast`]), whose buffs are `crate::buffs`'. (M5-N11)
//! The Prayer-mobile's Inspire and Suppress are casts too, round it and ahead of it (`AbilityArea`), which its R and F
//! send at once ("instant, no target"). (M6-GR, plan decision T92) So is the Radar Station's Scan, free but on a
//! cooldown (`crate::cooldown`), cast at any point of the map, whose effect is sight (`crate::vision::scans`).
//!
//! The gate takes a cast when:
//! - one of the casters, the player's, casts the ability (`StructureDef::abilities`; (M5-N11) a unit's spells,
//!   `SpellDef::Cast`), else [`Reject::LacksAbility`];
//! - one of those is in operation (`Object::is_operational`), not a site, else [`Reject::UnderConstruction`];
//! - one of those has the energy it costs (`factions/cults/structures/shrine.md > Abilities`: "spend Shrine
//!   energy"; a free ability, Scan's, costs none), else [`Reject::NotEnoughEnergy`], and (M6-GR) waits out no cooldown
//!   (`Object::cooldown`), else [`Reject::CoolingDown`]: the first such one in id order casts ("each press casts from
//!   one Shrine", Q0118; the client sends a press to the one Shrine of the ActiveGroup, which is Ungroupable; (M5-N11)
//!   the Prayer-mobile is too, "Groupable **false** (one cast spends one vehicle's energy, as for the Disciple)", and
//!   (M6-GR) the Radar Station, "**Groupable false** (user, 2026-11-26; Q0156: each Scan press scans from one station,
//!   as the Shrine; Q, Tab, Q scans two places)", design change 0422). **Placeholder (not in the design: a command
//!   naming several casters)**;
//! - its point is finite, else [`Reject::OffMap`], and, for an ability cast at a point (the Shrine's), on the map,
//!   else [`Reject::OffMap`], and the player sees it now, as the last tick left their vision
//!   (`factions/cults/structures/shrine.md > Abilities`: "cast on an area anywhere the player currently has vision";
//!   plan decision T76: "as a Place is judged by knowledge"): its tile is Visible, else [`Reject::OutOfSight`].
//!   (M6-GR) Scan's point is anywhere on the map, seen or not ("targeted: left-click any point on the map";
//!   `AbilityArea::AnyPoint`), else [`Reject::OffMap`].
//!   (M5-N11) An ability cast round or ahead of its caster takes its area from the caster, where the last tick left
//!   it, its position and its facing, and the command's point names nothing: the client sends the caster's position.
//!
//! "No cooldowns": a Shrine with the energy casts again in the next tick, or the same one; (M5-N11) so does a
//! Prayer-mobile (`factions/cults/units/prayer_mobile.md > Placeholder stats` gives none). (M6-GR) A Radar Station
//! casts again once its cooldown is over (`crate::cooldown`).

use glam::Vec2;

use super::Sim;
use super::gate::{Checked, Reject};
use crate::buffs;
use crate::cooldown;
use crate::defs::{AbilityArea, AreaAbility, BuffEffect};
use crate::energy;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::units::TilePos;
use crate::vision::{Scan, VisibilityState};

impl Sim {
    /// Which of `player`'s `casters` would cast `ability` now, wherever it is cast (module docs: the gate's checks but
    /// the point's): the first in id order that casts it, in operation, with the energy it costs and (M6-GR) no
    /// cooldown to wait out. What the panel greys the Shrine's Q and W by (plan decision T10), (M5-N11) the
    /// Prayer-mobile's R and F and (M6-GR) the Radar Station's Q, and why it would refuse: [`Reject::CoolingDown`] if
    /// one has the energy but waits, else [`Reject::NotEnoughEnergy`].
    pub fn can_cast(&self, player: PlayerId, casters: &[ObjectId], ability: AreaAbility) -> Result<ObjectId, Reject> {
        let casters = self.own_units(player, casters, |o| buffs::casts(o, ability))?;
        let Some(&first) = casters.first() else {
            return Err(Reject::LacksAbility);
        };
        let operational: Vec<&Object> = (casters.iter())
            .map(|id| &self.objects[id])
            .filter(|o| o.is_operational())
            .collect();
        if operational.is_empty() {
            return Err(Reject::UnderConstruction(first));
        }
        let cost = ability.def().cost;
        // (M6-GR) A free ability, Scan's, needs no energy pool.
        let paid = |o: &Object| cost <= 0.0 || energy::has(o, cost);
        if let Some(o) = operational.iter().find(|o| paid(o) && cooldown::ready(o)) {
            return Ok(o.id);
        }
        Err(if operational.iter().any(|o| paid(o)) {
            Reject::CoolingDown
        } else {
            Reject::NotEnoughEnergy
        })
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
        if !at.is_finite() {
            return Err(Reject::OffMap);
        }
        match ability.def().area {
            AbilityArea::AtPoint => {
                if !self.map.contains_point(at) {
                    return Err(Reject::OffMap);
                }
                if !self.sees_point(player, at) {
                    return Err(Reject::OutOfSight);
                }
            }
            // (M6-GR) Scan's: "any point on the map".
            AbilityArea::AnyPoint => {
                if !self.map.contains_point(at) {
                    return Err(Reject::OffMap);
                }
            }
            AbilityArea::AroundCaster | AbilityArea::Ahead { .. } => {}
        }
        Ok(Checked::Cast { caster, ability, at })
    }

    /// Whether `player` sees the point `at` now, as the gate judges a cast's point (module docs: "anywhere the player
    /// currently has vision"): on the map, on a tile Visible to them as the last tick left their vision. What the
    /// client's cast cursor shows by, and its click is taken by.
    pub fn sees_point(&self, player: PlayerId, at: Vec2) -> bool {
        self.map.contains_point(at) && self.vision.state(player, TilePos::containing(at)) == VisibilityState::Visible
    }

    /// `caster` casts `ability` at `at`, as the gate passed it: it pays the energy, and the units in its area get the
    /// buff (`crate::buffs::give`), in the commands phase, where the last tick left them: the circle round `at`, or
    /// (M5-N11) round the caster or ahead of its facing (module docs), where the last tick left it. (M6-GR) An ability
    /// with a cooldown starts its caster's (`crate::cooldown`), from this tick; Scan's sight is its owner's scan of the
    /// circle round `at` for its duration, which phase 5f of this tick stamps (`crate::vision::scans`).
    pub(super) fn cast(&mut self, caster: ObjectId, ability: AreaAbility, at: Vec2) {
        let def = ability.def();
        let Some(o) = self.objects.get_mut(&caster) else {
            return;
        };
        let (pos, facing) = (o.pos, o.facing);
        let Some(owner) = o.owner else {
            return;
        };
        // The gate passed it with the energy (`energy::has`): it pays it all. (M6-GR) A free one pays nothing.
        if def.cost > 0.0 {
            let Some(energy) = o.energy.as_mut() else {
                return;
            };
            if !energy::spend(energy, energy::units(def.cost)) {
                return;
            }
        }
        if ability.cooldown() > 0 {
            o.cooldown = Some(cooldown::start(ability.cooldown()));
        }
        let at = match def.area {
            AbilityArea::AtPoint | AbilityArea::AnyPoint => at,
            AbilityArea::AroundCaster | AbilityArea::Ahead { .. } => pos,
        };
        match def.effect {
            BuffEffect::Sight => {
                let scan = Scan {
                    at,
                    radius: def.radius,
                    until: self.tick.saturating_add(ability.duration()),
                };
                self.vision.scan(owner, scan);
            }
            BuffEffect::Health { .. }
            | BuffEffect::Energy { .. }
            | BuffEffect::Inspire { .. }
            | BuffEffect::HoldFire => {
                buffs::give(&mut self.objects, caster, owner, ability, (at, facing), self.tick);
            }
        }
        self.events.push(SimEvent::Cast {
            caster,
            ability,
            at,
            facing,
        });
    }
}
