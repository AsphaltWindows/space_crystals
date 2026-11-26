//! The gate's checks of a player's orders and abilities (`Command::Order`, `Command::Ability`;
//! `docs/movement.md > 3`). What an order may target is what the player sees or remembers (design change 0139,
//! `gate/targets.rs`).

use glam::Vec2;

use super::Checked;
use crate::command::Ability;
use crate::construction::site::assists;
use crate::defs::SpellDef;
use crate::energy;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::sim::{Reject, Sim};
use crate::{combat, conversion};

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
        let kind = self.onto_map(kind)?;
        match kind {
            // On the map ([`Sim::onto_map`]).
            OrderKind::Move(Target::Point(_)) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => {}
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
                // down takes it, and attacks once its shield is down. (M5-N4a) A unit without a weapon (the
                // Disciple's "AttackAttributes - None") ignores it too, as Attack Ground below
                // (`combat::takes_attack_orders`).
                ordered.retain(|id| (self.objects.get(id)).is_some_and(|o| combat::takes_attack_orders(o.kind)));
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
            // `systems/controls/commands.md > Enter` (`sim/network.rs`): the units the structure being entered lets
            // in take it (a Tunnel's tier, a building's kinds), the others ignore it.
            OrderKind::Enter(target) => ordered = self.check_enter(player, ordered, target)?,
            // (M4-O) `systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has
            // CanTargetGround" (plan decision T47): those units take it, the others ignore it. The point is the one
            // clicked, on the map; no one needs to see it.
            OrderKind::AttackGround(_) => {
                let research = self.research();
                let can = |o: &Object| {
                    combat::weapon(o, research).is_some_and(|attrs| combat::can_target_ground(attrs.attack_type))
                };
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
            // (M5-E) `factions/cults/economy.md > Assist Construction Command` (`crate::spot_work`): the workers that
            // assist the site take it, the others ignore it.
            OrderKind::Assist(site) => ordered = self.check_assist(player, ordered, site)?,
            // (M5-N8) `factions/cults/units/recruit.md > Repair` (`crate::repair`): the units that can repair the object
            // take it, the others ignore it.
            OrderKind::Repair(target) => ordered = self.check_repair(player, ordered, target)?,
            // (M5-N4b) `factions/cults/units/disciple.md > Spells > Shroud` (`orders::shroud`): the units whose type
            // casts it, with the energy it needs to start, take it; the others ignore it.
            OrderKind::Shroud => ordered = self.check_spell(ordered, |spell| matches!(spell, SpellDef::Shroud(_)))?,
        }
        Ok(Checked::Order {
            units: ordered,
            kind,
            queue,
        })
    }

    /// (M5-H1) `kind` with its point, if it has one, on the map (`systems/units.md > Air Collision`: "Orders can only
    /// target points inside the map"; plan decision T81: "The gate clamps orders' points into the map"): a point past
    /// the map's edge moves to the nearest point of the nearest edge tile (`Map::clamp_point`), so that a click past
    /// the edge sends the units to the edge (design change 0045: "Clicking outside the map issues no order beyond the
    /// edge"). Units that can't stand at the very edge go as near as they get
    /// (`systems/controls/command_behaviors.md > MovingToLocation > Unreachable target location`). A point that isn't
    /// finite is [`Reject::OffMap`]: a replay can't hold it (`ARCHITECTURE.md > Commands, events and replays`).
    fn onto_map(&self, kind: OrderKind) -> Result<OrderKind, Reject> {
        let onto = |p: Vec2| {
            if p.is_finite() {
                Ok(self.map.clamp_point(p))
            } else {
                Err(Reject::OffMap)
            }
        };
        Ok(match kind {
            OrderKind::Move(Target::Point(p)) => OrderKind::Move(Target::Point(onto(p)?)),
            OrderKind::AttackMove(p) => OrderKind::AttackMove(onto(p)?),
            OrderKind::Patrol(p) => OrderKind::Patrol(onto(p)?),
            OrderKind::AttackGround(p) => OrderKind::AttackGround(onto(p)?),
            other => other,
        })
    }

    /// (M5-E) The gate for an Assist of `site` (`factions/cults/economy.md > Assist Construction Command`: "Select one
    /// or more Recruits", "Left Click on an in-progress building"): the player knows of it (`Sim::check_target_known`;
    /// it always sees its own), and it is a site of the player's that its builders build from outside
    /// (`Object::is_built_from_outside`; design change 0339, `factions/cults/economy.md > Cults Building Mechanics >
    /// Group construction rules`: "**Assist** targets only a building **already under construction**, not a ghost
    /// whose order hasn't started"), else [`Reject::NotASite`]. The units of `units` that assist a site of its kind
    /// take it (`construction::site::assists`), else [`Reject::NoWorkers`].
    fn check_assist(&self, player: PlayerId, units: Vec<ObjectId>, site: ObjectId) -> Result<Vec<ObjectId>, Reject> {
        self.check_target_known(player, site)?;
        let target = (self.objects.get(&site))
            .filter(|o| o.owner == Some(player) && o.is_built_from_outside())
            .ok_or(Reject::NotASite(site))?;
        let assisting: Vec<ObjectId> = (units.into_iter())
            .filter(|id| (self.objects.get(id)).is_some_and(|o| assists(o.kind, target.kind)))
            .collect();
        if assisting.is_empty() {
            return Err(Reject::NoWorkers);
        }
        Ok(assisting)
    }

    /// (M5-N8) The gate for a Repair of `target` (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState`: "**D:
    /// Repair** — targeted command (StateOnlyTransition). Left-click a damaged own building or vehicle to repair it";
    /// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "a damaged own Tunnel or Laser Turret"): the
    /// player knows of it (`Sim::check_target_known`; it always sees its own), and the units of `units` that can repair
    /// it now take it (`Sim::can_repair`: an object of the player's, complete and below its MaxHP, of a kind their type
    /// repairs), else [`Reject::CantRepair`]: a site, an object at full HP, another player's or nobody's (a Laser
    /// Turret whose Tunnel is lost, design change 0360), a glider, a Tunnel's expansion.
    fn check_repair(&self, player: PlayerId, units: Vec<ObjectId>, target: ObjectId) -> Result<Vec<ObjectId>, Reject> {
        self.check_target_known(player, target)?;
        let repairing: Vec<ObjectId> = (units.into_iter()).filter(|&id| self.can_repair(id, target)).collect();
        if repairing.is_empty() {
            return Err(Reject::CantRepair(target));
        }
        Ok(repairing)
    }

    /// (M5-N4b) The gate for a spell's order: the units of `units` whose type casts the spell (the one of their type's
    /// `UnitDef::spells` that `is_spell` picks) take it, else [`Reject::LacksAbility`]; and of them only those with the
    /// energy it needs to start (`defs::SpellDef::needs`, `energy::has`), else [`Reject::NotEnoughEnergy`]: Shroud's
    /// "needs at least **10 energy** to start" (`factions/cults/units/disciple.md > Placeholder stats > Shroud
    /// numbers`). The check of every spell, the Prayer-mobile's to come too (`factions/cults/units/prayer_mobile.md`).
    fn check_spell(&self, units: Vec<ObjectId>, is_spell: impl Fn(&SpellDef) -> bool) -> Result<Vec<ObjectId>, Reject> {
        let spell_of = |o: &Object| (o.def().unit.as_ref()).and_then(|unit| unit.spells.iter().find(|s| is_spell(s)));
        let casters: Vec<(&Object, &SpellDef)> = (units.iter())
            .filter_map(|id| self.objects.get(id))
            .filter_map(|o| Some((o, spell_of(o)?)))
            .collect();
        if casters.is_empty() {
            return Err(Reject::LacksAbility);
        }
        let ready: Vec<ObjectId> = (casters.into_iter())
            .filter(|(o, spell)| energy::has(o, spell.needs()))
            .map(|(o, _)| o.id)
            .collect();
        if ready.is_empty() {
            return Err(Reject::NotEnoughEnergy);
        }
        Ok(ready)
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
