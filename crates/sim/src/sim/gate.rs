//! The command gate (`ARCHITECTURE.md > Commands, events and replays`): the one check every command passes before
//! it is applied. A rejected command changes nothing and never panics the sim.

use std::fmt;

use glam::Vec2;

use super::Sim;
use super::construction::Resolved;
use super::producers::ProductionPlan;
use crate::command::{Ability, Command, DebugCommand, PlayerCommand};
use crate::defs::{Domain, ObjectKind};
use crate::ids::{ObjectId, PlayerId};
use crate::nav;
use crate::object::{Object, Placement};
use crate::orders::{MAX_ORDER_UNITS, OrderKind, Target};
use crate::paths::{blocking_footprint, box_overlaps_rect, boxes_overlap};
use crate::placement::PlacementProblem;
use crate::player::{Banks, Faction};
use crate::riot_shield;
use crate::scenario::{SpawnProblem, place_spawn};
use crate::vision::VisibilityState;

/// Why the sim ignores a command ([`Sim::validate`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The issuing player, or the player a debug command names, is not in the game.
    UnknownPlayer(PlayerId),
    /// The game is over (`core/game_rules.md > End of Game`, `Sim::outcome`): the sim accepts no command any more.
    GameOver,
    /// The issuing player is defeated, and nothing they had can be commanded again (`core/game_rules.md > On
    /// Defeat`, design change 0101); or a debug spawn would give a defeated player an object.
    Defeated(PlayerId),
    /// Debug commands are off in this game ([`super::SimConfig::debug`]).
    DebugDisabled,
    /// No object has this id: it never existed, or it is gone.
    UnknownObject(ObjectId),
    /// The object belongs to someone else, and only its owner may order it.
    NotOwned(ObjectId),
    /// The object can't be created as asked.
    Spawn(SpawnProblem),
    /// Every object id is used up.
    OutOfIds,
    /// A tile-based object keeps its placement: only units move.
    TileObject(ObjectId),
    /// The position is not finite or lies off the map.
    OffMap,
    /// None of the units an order names exists any more (or takes orders).
    NoUnits,
    /// An order names more than [`MAX_ORDER_UNITS`] units.
    TooManyUnits,
    /// The order's target can't be attacked: it is indestructible
    /// (`systems/controls/commands.md > Attack`: a destructible object).
    NotAttackable(ObjectId),
    /// The order's target isn't in the issuing player's sight (`core/vision.md > Explored Selectability`; question
    /// Q0017, point 2).
    NotVisible(ObjectId),
    /// None of the units an ability names has that ability.
    LacksAbility,
    /// None of the units an Attack order names can attack now: their Riot Shields are up
    /// (`factions/gdo/units.md > Peacekeeper > Riot Shield`: "Attack orders are not possible").
    CannotAttack,
    /// A unit's box there would overlap a blocked tile, a blocking footprint or the map's edge
    /// (`docs/movement.md > 3`).
    Blocked,
    /// A unit's box there would overlap this unit of its layer, or a blocking footprint would cover this unit
    /// (`docs/movement.md > 3`).
    Occupied(ObjectId),
    /// A footprint there would overlap this object's (`core/entities.md > Placement Validation`): only a plate
    /// shares its tile, with the uncovered patch it is placed onto (`ObjectKind::may_overlap`).
    Overlaps(ObjectId),
    /// None of the objects a production command names produces units (or exists any more).
    NoProducers,
    /// None of the producers can produce a unit of this kind (`factions/gdo/structures.md > Barracks > Produces`).
    CannotProduce(ObjectKind),
    /// The BuildQueue of every producer that could train the unit is full (design change 0081: 20 with the unit in
    /// production).
    QueueFull,
    /// The player's Space Crystals don't cover the unit, or the building to construct ("Only available if player
    /// has sufficient Space Crystals", `factions/gdo/structures.md > DeploymentCenter >
    /// ObjectInterfaceState[DeploymentCenter]`).
    InsufficientCrystals,
    /// The unit would take the player's army cap past what is available (plan decision T20, `economy::army`): the
    /// GDO's Unit Control (`factions/gdo/overview.md > GDOResources > Unit Control`, design change 0082) or the
    /// Colonists' Beacon Capacity (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`). M2's
    /// name, kept for every faction's cap.
    UnitControlCap,
    /// Cancel Production: no producer has anything queued.
    NothingQueued,
    /// None of the objects a construction command names is one of the player's constructors that exists
    /// (`crate::construction`).
    NoConstructor,
    /// A Place names more than one constructor: one building is placed by one constructor (design change 0071).
    SeveralConstructors,
    /// The building isn't on the constructor's `Constructs` list (`factions/gdo/structures.md > DeploymentCenter >
    /// Constructs`, `factions/gdo/economy.md > ExtractionFacility > Constructs`).
    CannotConstruct(ObjectKind),
    /// The constructor is constructing, or holds a building ready to place: "Other build options unavailable"
    /// (`> ObjectInterfaceState[DeploymentCenter]`).
    ConstructorBusy(ObjectId),
    /// A Cancel Construction to a constructor that isn't constructing.
    NotConstructing,
    /// A Cancel Ready Building or a Place to a constructor that holds no building ready to place.
    NothingReady,
    /// The ready building can't be placed there (`core/entities.md > Placement Validation`, `crate::placement`): the
    /// first problem the placement check finds. When a Place is refused for this as it applies, the sim says why
    /// (`SimEvent::PlacementRefused`).
    Unplaceable(PlacementProblem),
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.0),
            Reject::GameOver => write!(f, "the game is over"),
            Reject::Defeated(p) => write!(f, "player {} is defeated", p.0),
            Reject::DebugDisabled => write!(f, "debug commands are off"),
            Reject::UnknownObject(id) => write!(f, "object {} doesn't exist", id.0),
            Reject::NotOwned(id) => write!(f, "object {} belongs to someone else", id.0),
            Reject::Spawn(problem) => write!(f, "can't spawn: {problem}"),
            Reject::OutOfIds => write!(f, "every object id is used up"),
            Reject::TileObject(id) => write!(f, "object {} is tile-based and can't move", id.0),
            Reject::OffMap => write!(f, "the position is off the map"),
            Reject::NoUnits => write!(f, "none of the ordered units exists"),
            Reject::TooManyUnits => write!(f, "an order may name at most {MAX_ORDER_UNITS} units"),
            Reject::NotAttackable(id) => write!(f, "object {} can't be attacked", id.0),
            Reject::NotVisible(id) => write!(f, "object {} is out of sight", id.0),
            Reject::LacksAbility => write!(f, "none of the units has the ability"),
            Reject::CannotAttack => write!(f, "none of the units can attack with its shield up"),
            Reject::Blocked => write!(f, "the unit doesn't fit there: blocked ground"),
            Reject::Occupied(id) => write!(f, "unit {} is in the way", id.0),
            Reject::Overlaps(id) => write!(f, "the footprint overlaps object {}", id.0),
            Reject::NoProducers => write!(f, "none of the objects produces units"),
            Reject::CannotProduce(kind) => write!(f, "none of the producers trains a {}", kind.def().name),
            Reject::QueueFull => write!(f, "the build queue is full"),
            Reject::InsufficientCrystals => write!(f, "not enough Space Crystals"),
            Reject::UnitControlCap => write!(f, "the army cap (Unit Control, Beacon Capacity) is reached"),
            Reject::NothingQueued => write!(f, "nothing is queued"),
            Reject::NoConstructor => write!(f, "none of the objects is a constructor of the player's"),
            Reject::SeveralConstructors => write!(f, "a building is placed by one constructor"),
            Reject::CannotConstruct(kind) => write!(f, "the constructor doesn't construct a {}", kind.def().name),
            Reject::ConstructorBusy(id) => write!(f, "constructor {} is busy", id.0),
            Reject::NotConstructing => write!(f, "nothing is under construction"),
            Reject::NothingReady => write!(f, "no building is ready to place"),
            Reject::Unplaceable(problem) => write!(f, "the building can't go there: {problem}"),
        }
    }
}

/// A command that passed the gate, with what the checks resolved for it.
pub(super) enum Checked {
    /// `units`: unique, in id order, all existing units of the player.
    Order {
        units: Vec<ObjectId>,
        kind: OrderKind,
        queue: bool,
    },
    /// `units`: unique, in id order, all existing units of the player that have the ability.
    Ability {
        units: Vec<ObjectId>,
        ability: Ability,
    },
    Spawn {
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
        placement: Option<Placement>,
    },
    Remove(ObjectId),
    Teleport {
        id: ObjectId,
        pos: Vec2,
    },
    AutoTargeting(bool),
    /// `player`: in the game.
    SetResources {
        player: PlayerId,
        banks: Banks,
    },
    /// `player`'s production command, with what its checks resolved (`sim/producers.rs`).
    Production {
        player: PlayerId,
        plan: ProductionPlan,
    },
    /// `player`'s constructors carry out what the construction checks resolved (`sim/construction.rs`).
    Construction {
        player: PlayerId,
        resolved: Resolved,
    },
}

impl Sim {
    /// The command gate: `Ok` if `cmd` would be applied now, else why it would be ignored. `Sim::step` passes every
    /// command through it before applying it, each against the state the earlier commands of the tick left. In
    /// order, it checks that:
    /// 1. the issuing player is in the game, the game isn't over ([`Reject::GameOver`]) and the player isn't defeated
    ///    ([`Reject::Defeated`]: every command, debug commands included, design change 0101);
    /// 2. a debug command comes while [`super::SimConfig::debug`] is on;
    /// 3. every object the command refers to exists, and every object a player's own order, ability, production or
    ///    construction command commands belongs to that player (debug commands may refer to anyone's objects). These
    ///    drop the ids of objects that no longer exist, and repeated ids; an order is rejected if no unit remains
    ///    (`docs/movement.md > 3`);
    /// 4. the command's own rules hold: an order's size and target (which the player must see,
    ///    [`Reject::NotVisible`]), and for an Attack, units that can attack now ([`Reject::CannotAttack`]); the units
    ///    of an ability that have it ([`Reject::LacksAbility`]); the producers of a production command and what they
    ///    can queue (`sim/producers.rs`); the constructors of a construction command that can carry it out, the bank
    ///    that pays and the placement check of a Place (`sim/construction.rs`); a spawn's owner and placement (a
    ///    footprint apart from the others but for a plate on its patch, [`Reject::Overlaps`]), a teleport to a unit
    ///    and onto the map, a unit's box clear of blocked ground and of the units of its layer, and the player whose
    ///    banks a debug command sets.
    pub fn validate(&self, cmd: &PlayerCommand) -> Result<(), Reject> {
        self.check(cmd).map(|_| ())
    }

    /// [`Sim::validate`], keeping what the checks resolved for applying the command.
    pub(super) fn check(&self, cmd: &PlayerCommand) -> Result<Checked, Reject> {
        let Some(player) = self.player(cmd.player) else {
            return Err(Reject::UnknownPlayer(cmd.player));
        };
        // `core/game_rules.md > End of Game`, `> On Defeat` (design change 0101).
        if self.outcome.is_some() {
            return Err(Reject::GameOver);
        }
        if player.is_defeated() {
            return Err(Reject::Defeated(cmd.player));
        }
        match &cmd.command {
            Command::Order { units, kind, queue } => self.check_order(cmd.player, units, *kind, *queue),
            Command::Ability { units, ability } => self.check_ability(cmd.player, units, *ability),
            Command::Production { objects, command } => self.check_production(cmd.player, objects, *command),
            Command::Construction { objects, command } => self.check_construction(cmd.player, objects, *command),
            Command::Debug(debug) => {
                if !self.config.debug {
                    return Err(Reject::DebugDisabled);
                }
                self.check_debug(debug)
            }
        }
    }

    /// `docs/movement.md > 3`, with the approved gate change: ids that no longer exist are dropped.
    fn check_order(
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
        }
        Ok(Checked::Order {
            units: ordered,
            kind,
            queue,
        })
    }

    /// The player's units among `units` that `keep` keeps, unique and in id order: at most [`MAX_ORDER_UNITS`] ids,
    /// none of them someone else's. Ids that no longer exist are dropped (a unit may die in the tick its command
    /// arrives: not the player's mistake), and so are the player's objects that `keep` leaves out.
    pub(super) fn own_units(
        &self,
        player: PlayerId,
        units: &[ObjectId],
        keep: impl Fn(&Object) -> bool,
    ) -> Result<Vec<ObjectId>, Reject> {
        if units.len() > MAX_ORDER_UNITS {
            return Err(Reject::TooManyUnits);
        }
        let mut ids = units.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let mut kept = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(o) = self.objects.get(&id) else {
                continue;
            };
            if o.owner != Some(player) {
                return Err(Reject::NotOwned(id));
            }
            if keep(o) {
                kept.push(id);
            }
        }
        Ok(kept)
    }

    /// [`Command::Ability`]: the player's own units that have the ability. Ids that no longer exist and the player's
    /// objects without the ability are dropped: [`Reject::NoUnits`] if nothing of the player's remains at all,
    /// [`Reject::LacksAbility`] if none of it has the ability.
    ///
    /// TODO(Bunker): units inside a Bunker can't use their abilities (`factions/gdo/structures.md > Bunker >
    /// Garrison`: "a Peacekeeper cannot raise its Riot Shield"); drop them here once units can enter one.
    fn check_ability(&self, player: PlayerId, units: &[ObjectId], ability: Ability) -> Result<Checked, Reject> {
        let has = |o: &Object| match ability {
            Ability::RiotShield { .. } => o.riot_shield.is_some(),
        };
        let own = self.own_units(player, units, |_| true)?;
        if own.is_empty() {
            return Err(Reject::NoUnits);
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

    /// Question Q0017, point 2 (its default, until answered): an order may target an object only while the issuing
    /// player sees it: a tile it covers is Visible. The one exception is the design's
    /// (`core/vision.md > Explored Selectability`): a GDO player may target a Supply Delivery Station on Explored
    /// ground, the one its Explored tiles show: by its last-known record of the station
    /// (`PlayerVision::remembered_record`), as the client selects it, never by the live tiles. A station that came
    /// onto Explored tiles while the player looked away (a debug spawn) has no record, so the player doesn't know of
    /// it until it sees it. Anything else out of sight, a remembered structure included, is `Reject::NotVisible`;
    /// the client makes a right-click on one a Move to that spot.
    pub(super) fn check_target_seen(&self, player: PlayerId, target: &Object) -> Result<(), Reject> {
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

    fn check_debug(&self, command: &DebugCommand) -> Result<Checked, Reject> {
        // Debug commands act on anyone's objects.
        let anyone = None;
        match *command {
            DebugCommand::Spawn {
                kind,
                owner,
                at,
                orientation,
            } => {
                if self.next_id == u32::MAX {
                    return Err(Reject::OutOfIds);
                }
                let is_player = |p| self.player(p).is_some();
                let (pos, placement) =
                    place_spawn(&self.map, kind, owner, at, orientation, is_player).map_err(Reject::Spawn)?;
                // A defeated player stays defeated and owns nothing (`core/game_rules.md > On Defeat`): no object
                // of theirs, and no eyes for their vision, comes back.
                if let Some(owner) = owner
                    && self.player(owner).is_some_and(|p| p.is_defeated())
                {
                    return Err(Reject::Defeated(owner));
                }
                match &placement {
                    Some(placement) => {
                        self.check_footprint_apart(kind, placement)?;
                        self.check_footprint_clear(kind, placement)?;
                    }
                    None => self.check_unit_fits(kind, pos, None)?,
                }
                Ok(Checked::Spawn {
                    kind,
                    owner,
                    pos,
                    placement,
                })
            }
            DebugCommand::Remove { id } => {
                self.referenced(id, anyone)?;
                Ok(Checked::Remove(id))
            }
            DebugCommand::SetPosition { id, pos } => {
                let object = self.referenced(id, anyone)?;
                if object.placement.is_some() {
                    return Err(Reject::TileObject(id));
                }
                // `contains_point` also rejects NaN and infinite positions.
                if !self.map.contains_point(pos) {
                    return Err(Reject::OffMap);
                }
                self.check_unit_fits(object.kind, pos, Some(id))?;
                Ok(Checked::Teleport { id, pos })
            }
            DebugCommand::SetAutoTargeting { on } => Ok(Checked::AutoTargeting(on)),
            DebugCommand::SetResources {
                player,
                crystals,
                supplies,
                alloys,
                essence,
                flux,
            } => {
                if self.player(player).is_none() {
                    return Err(Reject::UnknownPlayer(player));
                }
                let banks = Banks {
                    crystals,
                    supplies,
                    alloys,
                    essence,
                    flux,
                };
                Ok(Checked::SetResources { player, banks })
            }
        }
    }

    /// `docs/movement.md > 3`: a unit of `kind` placed at `pos` keeps the invariants: its box is clear of blocked
    /// tiles and blocking footprints and inside the map, and overlaps no other unit of its layer (`moving` is the
    /// unit itself, when it is teleported). Other kinds pass.
    fn check_unit_fits(&self, kind: ObjectKind, pos: Vec2, moving: Option<ObjectId>) -> Result<(), Reject> {
        let def = kind.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            return Ok(());
        };
        let half = silhouette / 2.0;
        let base = unit.base_def();
        if let Some(class) = base.nav_class() {
            let grid = self.nav.nav().grid(class, nav::size_class(silhouette));
            if grid.is_some_and(|grid| !nav::box_clear(grid, half, pos)) {
                return Err(Reject::Blocked);
            }
        }
        // The grids catch up with this tick's spawns and removals only in phase 5b: footprints are checked
        // directly too, so a unit can't be put on a station spawned earlier in the tick.
        if base.domain == Domain::Ground
            && self.objects.values().any(|o| {
                o.placement
                    .as_ref()
                    .and_then(|p| blocking_footprint(o.kind, p, self.nav.nav().rules()))
                    .is_some_and(|rect| box_overlaps_rect(pos, half, rect))
            })
        {
            return Err(Reject::Blocked);
        }
        if base.domain.hard_collision()
            && let Some(other) = self.objects.values().find(|o| {
                Some(o.id) != moving
                    && in_layer(o, base.domain).is_some_and(|other| boxes_overlap(pos, half, o.pos, other))
            })
        {
            return Err(Reject::Occupied(other.id));
        }
        Ok(())
    }

    /// A blocking footprint must not cover a ground unit (`docs/movement.md > 3`).
    /// `core/entities.md > Placement Validation`: a footprint of `kind` at `placement` overlaps no other footprint,
    /// a structure's or a map resource's (design change 0096: no buildings on resources), except that a plate goes
    /// onto an uncovered patch ([`ObjectKind::may_overlap`]). Else the first object in the way, by id.
    fn check_footprint_apart(&self, kind: ObjectKind, placement: &Placement) -> Result<(), Reject> {
        let in_the_way = self
            .objects
            .values()
            .find(|o| !kind.may_overlap(o.kind) && o.placement.as_ref().is_some_and(|other| other.overlaps(placement)));
        match in_the_way {
            Some(other) => Err(Reject::Overlaps(other.id)),
            None => Ok(()),
        }
    }

    fn check_footprint_clear(&self, kind: ObjectKind, placement: &Placement) -> Result<(), Reject> {
        let Some(rect) = blocking_footprint(kind, placement, self.nav.nav().rules()) else {
            return Ok(());
        };
        match self
            .objects
            .values()
            .find(|o| in_layer(o, Domain::Ground).is_some_and(|half| box_overlaps_rect(o.pos, half, rect)))
        {
            Some(unit) => Err(Reject::Occupied(unit.id)),
            None => Ok(()),
        }
    }

    /// An object a command refers to: it must exist and, when `orderer` is given, belong to that player (the
    /// objects a player's own order commands; targets, such as an enemy to attack, may belong to anyone).
    pub(super) fn referenced(&self, id: ObjectId, orderer: Option<PlayerId>) -> Result<&Object, Reject> {
        let object = self.objects.get(&id).ok_or(Reject::UnknownObject(id))?;
        match orderer {
            Some(player) if object.owner != Some(player) => Err(Reject::NotOwned(id)),
            _ => Ok(object),
        }
    }
}

/// The half extents of `o`'s box if it is a unit of `domain`'s layer (`docs/movement.md > 6.1`).
fn in_layer(o: &Object, domain: Domain) -> Option<Vec2> {
    let def = o.def();
    let unit = def.unit.as_ref()?;
    (unit.base_def().domain == domain).then(|| def.half_extents()).flatten()
}
