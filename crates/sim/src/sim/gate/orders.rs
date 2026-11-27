//! The gate's checks of a player's orders and abilities (`Command::Order`, `Command::Ability`;
//! `docs/movement.md > 3`). What an order may target is what the player sees or remembers (design change 0139,
//! `gate/targets.rs`).

use glam::Vec2;

use super::Checked;
use crate::command::Ability;
use crate::construction::site::assists;
use crate::defs::{Domain, SpellDef};
use crate::energy;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::sim::{Reject, Sim};
use crate::stock::{self, Unbuyable};
use crate::{aircraft, combat, conversion, tether, vehicle_deploy};

impl Sim {
    /// `docs/movement.md > 3`, with the approved gate change: ids that no longer exist are dropped. (M6-GA) A unit that
    /// deploys passes the capability filters (`crate::vehicle_deploy`: "Orders arriving during deploying are stored,
    /// not executed or rejected"), its order's target checked as any; the last filter, the vehicle deploy model's,
    /// holds the order behind its deploy, and holds every other unit to what its type can carry out
    /// ([`Sim::takers`]).
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
        // (M6-GA) What a unit that deploys can carry out is checked as its deploy completes, against its new type
        // (`crate::vehicle_deploy`): the filters of what a type can do pass it.
        let deploying = |o: &Object| vehicle_deploy::is_deploying(o);
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
                ordered.retain(|id| {
                    (self.objects.get(id)).is_some_and(|o| deploying(o) || combat::takes_attack_orders(o.kind))
                });
                if ordered.is_empty() {
                    return Err(Reject::CannotAttack);
                }
                // (M6-GF) `systems/combat.md > ValidTarget` 3: the units whose weapon can hit the target's domain, as
                // the player knows the target (a structure stands on the ground), take it, the others ignore it, as a
                // unit whose attack can't target the ground ignores an Attack Ground: none of them, and the Attack is
                // refused (the plan's "An Attack on a Peacekeeper is refused, with feedback", for the Twister's
                // anti-air machine gun; `Reject::CannotHit`).
                let domain = self
                    .known_kind(player, id)?
                    .def()
                    .unit
                    .as_ref()
                    .map_or(Domain::Ground, |unit| unit.base_def().domain);
                let research = self.research();
                let hits = |o: &Object| {
                    deploying(o)
                        || combat::weapon(o, research)
                            .is_some_and(|attrs| combat::domain_allows(attrs.target_domain, domain))
                };
                ordered.retain(|id| self.objects.get(id).is_some_and(hits));
                if ordered.is_empty() {
                    return Err(Reject::CannotHit(id));
                }
                // (M6-GH1) Nor an aircraft out of ammo (`crate::aircraft::armed`): of them, those with ammo left.
                ordered = self.armed_among(ordered)?;
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
                    deploying(o)
                        || combat::weapon(o, research).is_some_and(|attrs| combat::can_target_ground(attrs.attack_type))
                };
                ordered.retain(|id| self.objects.get(id).is_some_and(can));
                if ordered.is_empty() {
                    return Err(Reject::CannotTargetGround);
                }
                // (M6-GH1) Nor an aircraft out of ammo, as an Attack.
                ordered = self.armed_among(ordered)?;
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
            // (M5-V2) `systems/controls/commands.md > Reverse`: "Only available to bases with CanReverse = true": the
            // units that can back up take it (`UnitDef::can_reverse`), the others ignore it. The point is on the map.
            OrderKind::Reverse(_) => {
                let can = |o: &Object| deploying(o) || o.def().unit.as_ref().is_some_and(|unit| unit.can_reverse());
                ordered.retain(|id| self.objects.get(id).is_some_and(can));
                if ordered.is_empty() {
                    return Err(Reject::CannotReverse);
                }
            }
            // (M5-K) `factions/gdo/units/rocketman.md > Target Lock` (`crate::combat::target_lock`): the units whose
            // type has a Target Lock take it, the others ignore it.
            OrderKind::TargetLock(target) => ordered = self.check_target_lock(player, ordered, target)?,
            // (M6-GD) `factions/gdo/units/drone_operator.md > Launching and piloting` (`crate::tether`): the units
            // that launch from their stock take it, the others ignore it.
            OrderKind::Launch => ordered = self.check_launch(ordered)?,
            // (M6-GH1) `factions/gdo/structures/airfield.md > Assignment` (`crate::aircraft`): the aircraft that the
            // Airfield has room for take it, the others ignore it.
            OrderKind::Rebase(airfield) => ordered = self.check_rebase(player, ordered, airfield)?,
            // (M6-GH1) `factions/gdo/structures/airfield.md > Returning and rearming`: the aircraft with a base take it.
            OrderKind::ReturnToBase => ordered = self.check_return(ordered)?,
            // (M6-GA) `factions/gdo/units/artillery_rig.md > Modes` (`crate::vehicle_deploy`): the units whose type
            // deploys into the type it names take it, the others ignore it ([`Sim::takers`]).
            OrderKind::VehicleDeploy { .. } => {}
        }
        // (M6-GD) "The Drone Operator **cannot move** while it has a drone active" (`crate::tether`): a unit that holds
        // still for its drone ignores an order that would move it; if every unit does, the order is refused, with
        // feedback.
        if !tether::allows(kind) {
            ordered.retain(|id| !self.objects.get(id).is_some_and(tether::holds));
            if ordered.is_empty() {
                return Err(Reject::DroneOut);
            }
        }
        // (M6-GA) The vehicle deploy model's filter, last (`crate::vehicle_deploy`).
        let ordered = self.takers(player, ordered, kind)?;
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
            OrderKind::Reverse(p) => OrderKind::Reverse(onto(p)?),
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

    /// (M5-K) The gate for a Target Lock on `target` (`factions/gdo/units/rocketman.md > Target Lock`: "The player
    /// orders Target Lock on one enemy unit"; "Target Lock can target structures (user, 2026-11-26) as well as units"):
    /// the player knows of it (`Sim::check_target_known`, as an Attack's target), and it is destructible
    /// ([`Reject::NotAttackable`]) and another player's ([`Reject::NotAnEnemy`]), as the player knows it (a remembered
    /// one by its record, `Sim::aimed`). The units of `units` whose type has a Target Lock (`UnitDef::target_lock`)
    /// take it, else [`Reject::LacksAbility`]. Nothing asks whether the target is within LockRange: a lock on one
    /// beyond it breaks at once (`crate::combat::target_lock`).
    fn check_target_lock(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        target: ObjectId,
    ) -> Result<Vec<ObjectId>, Reject> {
        self.check_target_known(player, target)?;
        let aimed = self.aimed(player, target).ok_or(Reject::NotVisible(target))?;
        if !aimed.destructible {
            return Err(Reject::NotAttackable(target));
        }
        if !aimed.enemy {
            return Err(Reject::NotAnEnemy(target));
        }
        let locks = |o: &Object| (o.def().unit.as_ref()).is_some_and(|unit| unit.target_lock.is_some());
        let locking: Vec<ObjectId> = (units.into_iter())
            .filter(|id| self.objects.get(id).is_some_and(locks))
            .collect();
        if locking.is_empty() {
            return Err(Reject::LacksAbility);
        }
        Ok(locking)
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

    /// (M6-GD) The gate for a Launch (`crate::tether`): the units of `units` whose stock holds drones take it (their
    /// stock's kind is tethered, `UnitDef::tether`), else [`Reject::LacksAbility`]; of them those without a drone out
    /// ("Only **one drone can be active at a time** per operator"), else [`Reject::DroneOut`]; and of them those with a
    /// drone in stock, else [`Reject::StockEmpty`]. A queued Launch is judged as it is given, as a channel's energy
    /// is: one that starts without a drone to launch ends at once (`orders::launch`).
    fn check_launch(&self, units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        let launchers: Vec<&Object> = (units.iter())
            .filter_map(|id| self.objects.get(id))
            .filter(|o| stock::def_of(o.kind).is_some_and(|def| tether::def_of(def.item).is_some()))
            .collect();
        if launchers.is_empty() {
            return Err(Reject::LacksAbility);
        }
        let free: Vec<&Object> = launchers.into_iter().filter(|o| !tether::holds(o)).collect();
        if free.is_empty() {
            return Err(Reject::DroneOut);
        }
        let stocked: Vec<ObjectId> = (free.into_iter())
            .filter(|o| o.stock.is_some_and(|stock| stock.count >= 1))
            .map(|o| o.id)
            .collect();
        if stocked.is_empty() {
            return Err(Reject::StockEmpty);
        }
        Ok(stocked)
    }

    /// (M6-GA) Of `units`, which the gate's other checks of an order of `kind` took, those that take it by the vehicle
    /// deploy model (`crate::vehicle_deploy`, plan decision T44): a unit that deploys takes any order but a vehicle
    /// deploy ([`Reject::Deploying`]), held behind its deploy and checked against its new type as it completes
    /// (`Sim::step_vehicle_deploys`); every other unit an order its type can carry out by the rule that check goes by,
    /// T44's one `can_take` (`vehicle_deploy::can_take`, an Attack's target as `player` knows it), which adds to the
    /// checks before it that a unit whose type can't move ignores an order that would take it somewhere
    /// ([`Reject::CannotMove`]), and that a vehicle deploy goes to the units whose type deploys into the type it names
    /// ([`Reject::LacksAbility`]). With none of them left, the reason of the first, in id order.
    fn takers(&self, player: PlayerId, units: Vec<ObjectId>, kind: OrderKind) -> Result<Vec<ObjectId>, Reject> {
        let research = self.research();
        let domain = match kind {
            OrderKind::Attack(id) => Some(
                (self.known_kind(player, id)?.def().unit.as_ref())
                    .map_or(Domain::Ground, |unit| unit.base_def().domain),
            ),
            _ => None,
        };
        let mut first = None;
        let taking: Vec<ObjectId> = (units.into_iter())
            .filter(|id| {
                let Some(o) = self.objects.get(id) else {
                    return false;
                };
                let why = if vehicle_deploy::is_deploying(o) {
                    matches!(kind, OrderKind::VehicleDeploy { .. }).then_some(Reject::Deploying)
                } else {
                    vehicle_deploy::can_take(o, kind, domain, research).err()
                };
                if let Some(why) = why {
                    first.get_or_insert(why);
                }
                why.is_none()
            })
            .collect();
        match first {
            Some(why) if taking.is_empty() => Err(why),
            _ => Ok(taking),
        }
    }

    /// (M6-GH1) The units of `units` with ammo left, or whose weapon never runs out (`crate::aircraft::armed`), else
    /// [`Reject::OutOfAmmo`]: an Attack's and an Attack Ground's, which an aircraft out of ammo ignores.
    fn armed_among(&self, units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        let armed: Vec<ObjectId> = (units.into_iter())
            .filter(|id| self.objects.get(id).is_some_and(aircraft::armed))
            .collect();
        if armed.is_empty() {
            return Err(Reject::OutOfAmmo);
        }
        Ok(armed)
    }

    /// (M6-GH1) The gate for a Rebase to `airfield` (`factions/gdo/structures/airfield.md > Assignment`: "**Reassigning**:
    /// the player can move an aircraft's assignment to another Airfield, if that Airfield has room: its assignments
    /// plus the aircraft in its production queue must be **fewer than 4**"; plan decision T88): it is an Airfield of
    /// the player's in operation (`aircraft::is_airfield`), else [`Reject::NotAnAirfield`]; the aircraft of `units`
    /// take it (`Object::aircraft`), else [`Reject::LacksAbility`]: those assigned to it already, which need no room,
    /// and of the others, in the command's order, as many as it has room for (`aircraft::room`); the others ignore
    /// it. With no room for any of them and none assigned to it: [`Reject::AirfieldFull`]. `Sim::give_order` assigns
    /// them as it gives the order.
    fn check_rebase(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        airfield: ObjectId,
    ) -> Result<Vec<ObjectId>, Reject> {
        let target = self.referenced(airfield, None)?;
        if target.owner != Some(player) || !aircraft::is_airfield(target) {
            return Err(Reject::NotAnAirfield(airfield));
        }
        let aircraft: Vec<&Object> = (units.iter())
            .filter_map(|id| self.objects.get(id))
            .filter(|o| o.aircraft.is_some())
            .collect();
        if aircraft.is_empty() {
            return Err(Reject::LacksAbility);
        }
        let mut room = aircraft::room(&self.objects, airfield);
        let mut taking = Vec::new();
        for o in aircraft {
            if aircraft::base_of(o) == Some(airfield) {
                taking.push(o.id);
            } else if room > 0 {
                room -= 1;
                taking.push(o.id);
            }
        }
        if taking.is_empty() {
            return Err(Reject::AirfieldFull);
        }
        Ok(taking)
    }

    /// (M6-GH1) The gate for a Return to Base (`factions/gdo/structures/airfield.md > Returning and rearming`: "An
    /// unassigned aircraft has no base [...] Return to Base is unavailable until it is reassigned"): the aircraft of
    /// `units` take it, else [`Reject::LacksAbility`], and of them those with a base (`aircraft::base_of`), else
    /// [`Reject::NoBase`].
    fn check_return(&self, units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        let aircraft: Vec<&Object> = (units.iter())
            .filter_map(|id| self.objects.get(id))
            .filter(|o| o.aircraft.is_some())
            .collect();
        if aircraft.is_empty() {
            return Err(Reject::LacksAbility);
        }
        let based: Vec<ObjectId> = (aircraft.into_iter())
            .filter(|o| aircraft::base_of(o).is_some())
            .map(|o| o.id)
            .collect();
        if based.is_empty() {
            return Err(Reject::NoBase);
        }
        Ok(based)
    }

    /// (M6-GD) The gate for a Buy (`crate::stock`, plan decision T89), of `units`, the player's own units with a stock:
    /// those that may start a purchase now (`stock::refusal`), else the reason of the first in id order
    /// ([`Reject::DroneOut`], [`Reject::PurchaseUnderWay`], [`Reject::StockFull`]); and the bank pays for the first
    /// of them, else [`Reject::InsufficientCrystals`] (T7's checked spend). The others are paid at the press in id
    /// order, until the bank can't pay for one (`Sim::use_ability`).
    fn check_buy(&self, player: PlayerId, units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        let mut first = None;
        let mut buying = Vec::new();
        for o in units.iter().filter_map(|id| self.objects.get(id)) {
            let Some(stock) = &o.stock else {
                continue;
            };
            match stock::refusal(o, stock) {
                Some(reason) => {
                    first.get_or_insert(reason);
                }
                None => buying.push(o.id),
            }
        }
        let Some(&id) = buying.first() else {
            return Err(match first {
                Some(Unbuyable::DroneOut) => Reject::DroneOut,
                Some(Unbuyable::Buying) => Reject::PurchaseUnderWay,
                Some(Unbuyable::Full) | None => Reject::StockFull,
            });
        };
        let price = (self.objects.get(&id))
            .and_then(|o| stock::def_of(o.kind))
            .map_or(0, |def| def.price);
        if self.player(player).is_none_or(|p| p.crystals < price) {
            return Err(Reject::InsufficientCrystals);
        }
        Ok(buying)
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
            // (M6-GD) A unit with a stock (`crate::stock`).
            Ability::Buy => o.stock.is_some(),
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
        let units = match ability {
            Ability::Buy => self.check_buy(player, units)?,
            Ability::RiotShield { .. } | Ability::SetDropOff(_) | Ability::Convert => units,
        };
        Ok(Checked::Ability { units, ability })
    }
}
