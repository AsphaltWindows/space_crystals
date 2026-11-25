//! The gate's checks of a player's orders and abilities (`Command::Order`, `Command::Ability`;
//! `docs/movement.md > 3`). What an order may target is what the player sees or remembers (design change 0139,
//! `gate/targets.rs`).

use super::Checked;
use crate::command::Ability;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::sim::{Reject, Sim};
use crate::{combat, conversion, riot_shield};

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
            // Design change 0139: an object the player sees or remembers (`Sim::check_target_known`).
            OrderKind::Move(Target::Object(id)) => {
                self.check_target_known(player, id)?;
            }
            OrderKind::Attack(id) => {
                // A remembered object by its record (`Sim::known_kind`): whether it still stands, the gate doesn't
                // tell.
                if !self.known_kind(player, id)?.def().destructible {
                    return Err(Reject::NotAttackable(id));
                }
                self.check_target_known(player, id)?;
                // `factions/gdo/units/peacekeeper.md > Riot Shield`: "Attack orders are not possible" while the
                // shield is up ("the player must lower the shield before ordering an attack"): those units ignore it.
                // Which orders a unit takes follows its type from the press (design change 0202): a Peacekeeper
                // (Shielded), its shield up or on its way up, takes none; a Peacekeeper whose shield is on its way
                // down takes it, and attacks once its shield is down.
                ordered.retain(|id| (self.objects.get(id)).is_some_and(|o| riot_shield::takes_attack_orders(o.kind)));
                if ordered.is_empty() {
                    return Err(Reject::CannotAttack);
                }
            }
            OrderKind::Hold | OrderKind::Stop => {}
            // `factions/colonists/units/drone.md > Unit Commands` (`sim/workers.rs`): the workers among the
            // units take it, the others ignore it; on a Supply Delivery Station, those that gather Supplies.
            OrderKind::Gather(target) => {
                self.check_gather(player, target)?;
                ordered = self.gatherers_among(player, ordered, target)?;
            }
            // Design change 0140: and only those that carry something.
            OrderKind::DropOff(node) => {
                self.check_drop_off(player, node)?;
                ordered = self.carrying_among(self.workers_among(ordered)?)?;
            }
            // `factions/colonists/economy.md > Beacon Deployment` (`sim/deploy.rs`): one Drone takes it.
            OrderKind::Deploy { origin } => ordered = self.check_deploy(ordered, origin, queue)?,
            // `systems/controls/commands.md > Enter` (`sim/network.rs`): the units the Tunnel's tier lets in take
            // it, the others ignore it.
            OrderKind::Enter(tunnel) => ordered = self.check_enter(player, ordered, tunnel)?,
            // (M4-O) `systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has
            // CanTargetGround" (plan decision T47): those units take it, the others ignore it. The point is the one
            // clicked, on the map; no one needs to see it.
            OrderKind::AttackGround(p) => {
                if !self.map.contains_point(p) {
                    return Err(Reject::OffMap);
                }
                let can =
                    |o: &Object| combat::weapon(o).is_some_and(|attrs| combat::can_target_ground(attrs.attack_type));
                ordered.retain(|id| self.objects.get(id).is_some_and(can));
                if ordered.is_empty() {
                    return Err(Reject::CannotTargetGround);
                }
            }
            // `core/entities.md > Worker-Built Structures` (`sim/builds.rs`): one worker takes it, and pays for it.
            OrderKind::Build {
                kind,
                origin,
                orientation,
            } => ordered = self.check_build(ordered, (kind, origin, orientation), queue)?,
        }
        Ok(Checked::Order {
            units: ordered,
            kind,
            queue,
        })
    }

    /// [`Command::Ability`](crate::Command::Ability): the player's own units that have the ability. Ids that no
    /// longer exist and the player's objects without the ability are dropped: [`Reject::NoUnits`] if nothing of the
    /// player's remains at all, [`Reject::LacksAbility`] if none of it has the ability. A toggle is never refused for
    /// what the unit is doing (design change 0198: a press flips the unit's target, `sim/modes.rs`): a Convert goes
    /// to every unit with another form (`conversion::converts_to`), a conversion waiting or under way included, and
    /// a Riot Shield's to every unit that carries one.
    ///
    /// TODO(Bunker): units inside a Bunker can't use their abilities (`factions/gdo/structures/bunker.md >
    /// Garrison`: "a Peacekeeper cannot raise its Riot Shield"); drop them here once units can enter one.
    pub(super) fn check_ability(
        &self,
        player: PlayerId,
        units: &[ObjectId],
        ability: Ability,
    ) -> Result<Checked, Reject> {
        let has = |o: &Object| match ability {
            Ability::RiotShield { .. } => o.riot_shield.is_some(),
            Ability::SetDropOff(_) => Sim::sets_drop_off(o),
            Ability::Convert => conversion::converts_to(o.kind).is_some(),
        };
        let own = self.own_units(player, units, |_| true)?;
        if own.is_empty() {
            return Err(Reject::NoUnits);
        }
        if let Ability::SetDropOff(node) = ability {
            self.check_drop_off(player, node)?;
        }
        let units: Vec<ObjectId> = own
            .into_iter()
            .filter(|id| self.objects.get(id).is_some_and(has))
            .collect();
        if units.is_empty() {
            return Err(Reject::LacksAbility);
        }
        Ok(Checked::Ability { units, ability })
    }
}
