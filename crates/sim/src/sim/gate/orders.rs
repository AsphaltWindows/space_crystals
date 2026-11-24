//! The gate's checks of a player's orders and abilities (`Command::Order`, `Command::Ability`;
//! `docs/movement.md > 3`), and of what an order may target: what the player sees.

use super::Checked;
use crate::command::Ability;
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::sim::{Reject, Sim};
use crate::vision::VisibilityState;
use crate::{conversion, riot_shield};

impl Sim {
    /// `docs/movement.md > 3`, with the approved gate change: ids that no longer exist are dropped.
    pub(super) fn check_order(
        &self,
        player: PlayerId,
        units: &[ObjectId],
        kind: OrderKind,
        queue: bool,
    ) -> Result<Checked, Reject> {
        // Only units take orders; the player's other objects ignore this one.
        let mut ordered = self.own_units(player, units, |o| o.orders.is_some())?;
        if ordered.is_empty() {
            return Err(Reject::NoUnits);
        }
        match kind {
            OrderKind::Move(Target::Point(p)) | OrderKind::AttackMove(p) | OrderKind::Patrol(p) => {
                // `contains_point` also rejects NaN and infinite positions.
                if !self.map.contains_point(p) {
                    return Err(Reject::OffMap);
                }
            }
            OrderKind::Move(Target::Object(id)) => {
                self.check_target_seen(player, self.referenced(id, None)?)?;
            }
            OrderKind::Attack(id) => {
                let target = self.referenced(id, None)?;
                if !target.def().destructible {
                    return Err(Reject::NotAttackable(id));
                }
                self.check_target_seen(player, target)?;
                // `factions/gdo/units.md > Peacekeeper > Riot Shield`: "Attack orders are not possible" while the
                // shield is up ("the player must lower the shield before ordering an attack"): those units ignore it.
                ordered.retain(|id| self.objects.get(id).is_some_and(riot_shield::allows_attack));
                if ordered.is_empty() {
                    return Err(Reject::CannotAttack);
                }
            }
            OrderKind::Hold | OrderKind::Stop => {}
            // `factions/colonists/units.md > Drone > Unit Commands` (`sim/workers.rs`): the workers among the
            // units take it, the others ignore it.
            OrderKind::Gather(patch) => {
                self.check_gather(player, patch)?;
                ordered = self.workers_among(ordered)?;
            }
            OrderKind::DropOff(node) => {
                self.check_drop_off(player, node)?;
                ordered = self.workers_among(ordered)?;
            }
        }
        Ok(Checked::Order {
            units: ordered,
            kind,
            queue,
        })
    }

    /// [`Command::Ability`](crate::Command::Ability): the player's own units that have the ability. Ids that no
    /// longer exist and the player's objects without the ability are dropped: [`Reject::NoUnits`] if nothing of the
    /// player's remains at all, [`Reject::LacksAbility`] if none of it has the ability. A Convert goes to the units
    /// with another form that may convert now (`conversion::may_convert`), [`Reject::CannotConvert`] if none may.
    ///
    /// TODO(Bunker): units inside a Bunker can't use their abilities (`factions/gdo/structures.md > Bunker >
    /// Garrison`: "a Peacekeeper cannot raise its Riot Shield"); drop them here once units can enter one.
    pub(super) fn check_ability(
        &self,
        player: PlayerId,
        units: &[ObjectId],
        ability: Ability,
    ) -> Result<Checked, Reject> {
        let has = |o: &Object| match ability {
            Ability::RiotShield { .. } => o.riot_shield.is_some(),
            Ability::SetDropOff(_) => o.worker.is_some(),
            Ability::Convert => conversion::converts_to(o.kind).is_some(),
        };
        let own = self.own_units(player, units, |_| true)?;
        if own.is_empty() {
            return Err(Reject::NoUnits);
        }
        if let Ability::SetDropOff(node) = ability {
            self.check_drop_off(player, node)?;
        }
        let mut units: Vec<ObjectId> = own
            .into_iter()
            .filter(|id| self.objects.get(id).is_some_and(has))
            .collect();
        if units.is_empty() {
            return Err(Reject::LacksAbility);
        }
        if ability == Ability::Convert {
            // `factions/colonists/units.md > Weapon Conversion > Eligibility`: the units in Firing or Cooldown, and
            // those converting already, are left out, and the others convert (question Q0042 point 9).
            units.retain(|id| self.objects.get(id).is_some_and(conversion::may_convert));
            if units.is_empty() {
                return Err(Reject::CannotConvert);
            }
        }
        Ok(Checked::Ability { units, ability })
    }

    /// Question Q0017, point 2 (its default, until answered): an order may target an object only while the issuing
    /// player sees it: a tile it covers is Visible. The one exception is the design's
    /// (`core/vision.md > Explored Selectability`): a GDO player may target a Supply Delivery Station on Explored
    /// ground, the one its Explored tiles show: by its last-known record of the station
    /// (`PlayerVision::remembered_record`), as the client selects it, never by the live tiles. A station that came
    /// onto Explored tiles while the player looked away (a debug spawn) has no record, so the player doesn't know of
    /// it until it sees it. Anything else out of sight, a remembered structure included, is `Reject::NotVisible`;
    /// the client makes a right-click on one a Move to that spot.
    pub(in crate::sim) fn check_target_seen(&self, player: PlayerId, target: &Object) -> Result<(), Reject> {
        let seen = self.vision.object_state(player, target) == VisibilityState::Visible
            || self.player(player).is_some_and(|p| p.faction == Faction::Gdo)
                && self
                    .vision
                    .player(player)
                    .and_then(|vision| vision.remembered_record(target.id))
                    .is_some_and(|record| record.kind == ObjectKind::SupplyDeliveryStation);
        if seen {
            Ok(())
        } else {
            Err(Reject::NotVisible(target.id))
        }
    }
}
