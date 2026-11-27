//! The Colonists' computer player (slice A2c; `factions/colonists/overview.md`; `core/game_rules.md > Computer players
//! (playtest tool)`: its behavior "is the Director's placeholder"). From the Colonists' Starting Conditions
//! (`factions/colonists/economy.md > Starting Conditions`: "one pre-built Egress Node, one pre-built Robo Portal ...
//! and four Drones"), each tick, in this order of priority for the bank:
//!
//! 1. **Warp In** ([`WARP_INS`]): the buildings it keeps, each on one of its complete Beacons whose Warp In the
//!    placement check takes now (`View::warp_in_check`, the gate's own), the Beacon nearest where the building serves,
//!    a tile clear of its other buildings but its Beacons ([`beacons`], "Doorways", slice A2d): an Egress Node and a
//!    Robo Portal if it has none, a Bio Portal on its first Beacon (as the golden game `colonists_opening` does), a
//!    second Robo Portal (slice A2d), and two more Bio Portals as its Drones grow (`factions/colonists/economy.md >
//!    Warp In`).
//! 2. **The Robo Portals** train Drones without pause, [`DRONE_QUEUE`] queued, while its gathering Drones fall short of
//!    what its field seats ([`DRONES_PER_PATCH`] a patch, [`field`]) or a Beacon is wanted (step 4).
//! 3. **Each Bio Portal** trains Sabre Legionnaires without pause (`factions/colonists/structures/bio_portal.md >
//!    Produces`), [`TRAIN_QUEUE`] queued, while Beacon Capacity has room for them
//!    (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`).
//!
//! A step that its bank can't pay for yet holds the bank: the steps after it wait. Then, at no cost:
//!
//! 4. **Beacons** ([`beacons`]): a Drone the Robo Portal releases deploys into a Beacon
//!    (`factions/colonists/economy.md > Beacon Deployment`, `View::deploy_check`) whenever Beacon Capacity, with the
//!    Beacons on their way, falls short of what the portals will take next, or a Warp In finds no Beacon.
//! 5. **Gathering** ([`field`]): each of its Drones with no order gathers the patch of its field that the fewest of
//!    its Drones gather.
//! 6. **Weapon Conversion** (`factions/colonists/units/weapon_conversion.md`, "No resource cost"): its newest Sabre
//!    Legionnaires convert into Rifle Legionnaires, so that one in [`RIFLE_EVERY`] is a Rifle.
//!
//! The army layer (`crate::army`) leads the Legionnaires, Sabres and Rifles, which the Bio Portals rally to the staging
//! point; the Drones never fight.
//!
//! **The rush** (`Profile::Rush`, slice A2d; Q0147 point 2: "a small early army and an early first attack"): Sabres at
//! once. Until its Bio Portals have been ordered [`RUSH_WAVE`] Sabre Legionnaires, it warps in only what it wants
//! from the start (the Bio Portal on its first Beacon), and trains them before the Robo Portal's Drones; its first
//! wave is those [`RUSH_WAVE`] Legionnaires, half of them Rifles by then (step 6, `FactionAi::waves`). Then it plays,
//! and reinforces, as standard does.
//!
//! Every rule and number is a placeholder (Q0147 points 2 and 3, `crate::tuning::colonists`). Costs, times and Beacon
//! Capacity come from the defs.

mod beacons;
mod field;

use sc_sim::nav::TileRect;
use sc_sim::transform::{warp_in_footprint, warp_in_recipe};
use sc_sim::{
    Ability, ArmyResource, BuildingOrientation, Command, ConstructionCommand, Object, ObjectId, ObjectKind,
    ProductionCommand, Vec2,
};

use super::{FactionAi, Hook};
use crate::army::Waves;
use crate::tuning::colonists::{DRONE_QUEUE, DRONES_PER_PATCH, RIFLE_EVERY, RUSH_WAVE, TRAIN_QUEUE, WARP_INS};
use crate::{Known, Profile, Turn, View};

use ObjectKind::{Beacon, BioPortal, Drone, EgressNode, RifleLegionnaire, RoboPortal, SabreLegionnaire};

pub(super) const HOOK: Option<Hook> = Some(new);

fn new(view: &View<'_>, profile: Profile) -> Box<dyn FactionAi> {
    Box::new(Colonists::new(view, profile))
}

/// The Colonists' computer player (module docs).
#[derive(Debug)]
struct Colonists {
    /// Whether it plays the rush (module docs).
    rush: bool,
    /// The Sabre Legionnaires it has ordered.
    ordered: usize,
    /// Where its Beacon sites gather ([`beacons`]): its Robo Portal as it took over, else its Egress Node, else its
    /// start.
    anchor: Vec2,
    /// Its Deploys on their way, and those that failed.
    deploys: beacons::Deploys,
}

impl Colonists {
    fn new(view: &View<'_>, profile: Profile) -> Self {
        let first = |kind| view.own().find(|o| o.kind == kind).map(|o| o.pos);
        let anchor = (first(RoboPortal).or_else(|| first(EgressNode)))
            .or_else(|| view.my_start().map(|s| s.point()))
            .unwrap_or_default();
        Self {
            rush: profile.rushes(),
            ordered: 0,
            anchor,
            deploys: beacons::Deploys::default(),
        }
    }

    /// Whether it still plays its rush opening (module docs): a rush that hasn't ordered its first wave yet.
    fn rushing(&self) -> bool {
        self.rush && self.ordered < RUSH_WAVE
    }
}

impl FactionAi for Colonists {
    fn think(&mut self, turn: &mut Turn<'_, '_>) {
        self.deploys.review(turn);
        let mut base = Base::of(turn);
        let opening = self.rushing();
        if warp_in(turn, &mut base, self.anchor, opening) {
            if opening {
                self.ordered += train_army(turn, &mut base);
                train_drones(turn, &mut base);
            } else if train_drones(turn, &mut base) {
                self.ordered += train_army(turn, &mut base);
            }
        }
        let deployer = self.deploys.send(turn, &base, self.anchor);
        field::gather(turn, &base, deployer);
        convert(turn);
    }

    fn fights(&self, unit: &Object) -> bool {
        matches!(unit.kind, SabreLegionnaire | RifleLegionnaire)
    }

    fn rallies(&self, building: &Object) -> bool {
        building.kind == BioPortal
    }

    /// The army layer's waves (`crate::tuning::army`), but a rush's first: its [`RUSH_WAVE`] Sabre Legionnaires
    /// (module docs).
    fn waves(&self) -> Waves {
        let waves = Waves::default();
        match self.rush {
            true => Waves {
                first: RUSH_WAVE,
                ..waves
            },
            false => waves,
        }
    }
}

/// What the player has this tick, and what the turn has started so far (module docs).
struct Base<'v> {
    /// Where its Drones deliver: its first Egress Node (by id), else its start.
    node: Vec2,
    /// Its field ([`field::of`]): the patches with crystals left near `node`, nearest it first.
    field: Vec<&'v Known>,
    /// Its Drones that gather or may: every Drone but those on their way to deploy, by id.
    workers: Vec<&'v Object>,
    /// Its Drones on their way to deploy (a Deploy order, current or queued).
    walking: u32,
    /// Its Beacons under deployment.
    deploying: u32,
    /// Its complete Beacons, which a Warp In may use, by id.
    beacons: Vec<&'v Object>,
    /// Its Drones queued at its Robo Portals, in production or not.
    drones_queued: usize,
    /// What its producers' queued units not yet in production will take of its Beacon Capacity as they start: those
    /// in production count already (`ArmyCap::used`).
    waiting: u32,
    /// Whether a Warp In it wants found no Beacon of its own at all, complete or on its way.
    beacon_wanted: bool,
}

impl<'v> Base<'v> {
    fn of(turn: &Turn<'v, '_>) -> Self {
        let view = turn.view();
        let node = (view.own().find(|o| o.kind == EgressNode).map(|o| o.pos))
            .or_else(|| view.my_start().map(|s| s.point()))
            .unwrap_or_default();
        let mut base = Self {
            node,
            field: field::of(view, node),
            workers: Vec::new(),
            walking: 0,
            deploying: 0,
            beacons: Vec::new(),
            drones_queued: 0,
            waiting: 0,
            beacon_wanted: false,
        };
        let resource = ArmyResource::BeaconCapacity;
        for o in view.own() {
            match o.kind {
                Drone if beacons::walking(o) => base.walking += 1,
                Drone => base.workers.push(o),
                Beacon if o.is_deploying() => base.deploying += 1,
                Beacon if !o.is_transforming() => base.beacons.push(o),
                _ => {}
            }
            let Some(production) = &o.production else {
                continue;
            };
            let units = production.queue.iter().filter_map(|entry| entry.unit());
            // The first unit is in production once it has begun: its cost is used already.
            let started = usize::from(production.in_production().is_some());
            base.waiting += units
                .clone()
                .skip(started)
                .map(|kind| resource.cost_of(kind))
                .sum::<u32>();
            base.drones_queued += units.filter(|&kind| kind == Drone).count();
        }
        base
    }

    /// The gathering Drones its field seats: [`DRONES_PER_PATCH`] for each of its patches.
    fn seats(&self) -> usize {
        DRONES_PER_PATCH * self.field.len()
    }

    /// Whether its Robo Portals should train another Drone: its gathering Drones and those queued fall short of what
    /// its field seats.
    fn wants_drones(&self) -> bool {
        self.workers.len() + self.drones_queued < self.seats()
    }
}

/// 1: warps in what [`WARP_INS`] wants and the player lacks, each on the complete Beacon nearest where it serves
/// whose Warp In the placement check takes now; while a rush plays its `opening` (module docs), only those it wants
/// from the start. Returns whether the steps after it may spend: `false` when the bank doesn't cover a Warp In that is
/// ready.
fn warp_in(turn: &mut Turn<'_, '_>, base: &mut Base<'_>, anchor: Vec2, opening: bool) -> bool {
    let view = turn.view();
    let orientation = BuildingOrientation::default();
    let owned = |kind| view.own().filter(|o| o.kind == kind).count();
    let mut planned: Vec<ObjectKind> = Vec::new();
    // What this turn's Warp Ins cover, which the next keep clear (`beacons::apart`).
    let mut covered: Vec<TileRect> = Vec::new();
    for &(kind, drones) in &WARP_INS {
        planned.push(kind);
        let nth = planned.iter().filter(|&&k| k == kind).count();
        if base.workers.len() < drones || owned(kind) >= nth || (opening && drones > 0) {
            continue;
        }
        let Some(cost) = warp_in_recipe(Beacon, kind).map(|r| r.cost) else {
            continue;
        };
        // Where the building serves: a Bio Portal toward the staging point, where its units go; an Egress Node by
        // the field; a Robo Portal where the Beacon sites start.
        let toward = match kind {
            BioPortal => turn.staging().unwrap_or(base.node),
            EgressNode => field::centre(&base.field).unwrap_or(base.node),
            _ => anchor,
        };
        let mut beacons: Vec<&Object> = base.beacons.clone();
        beacons.sort_by(|a, b| {
            (a.pos
                .distance_squared(toward)
                .total_cmp(&b.pos.distance_squared(toward)))
            .then(a.id.cmp(&b.id))
        });
        let footprint = |b: &Object| {
            let p = warp_in_footprint(&b.placement?, kind, orientation)?;
            Some(TileRect::new(p.origin, p.w, p.h))
        };
        let ready = (beacons.into_iter()).find_map(|b| {
            let footprint = footprint(b).filter(|&rect| beacons::apart(view, rect, &covered))?;
            let valid = (view.warp_in_check(b.id, kind, orientation)).is_some_and(|check| check.is_valid());
            valid.then_some((b, footprint))
        });
        let Some((beacon, footprint)) = ready else {
            // None on the way either: a Beacon is wanted for it (step 4).
            base.beacon_wanted |= base.beacons.is_empty() && base.deploying == 0 && base.walking == 0;
            continue;
        };
        if !turn.can_pay(cost) {
            return false;
        }
        let command = Command::Construction {
            objects: vec![beacon.id],
            command: ConstructionCommand::WarpIn { kind, orientation },
        };
        if turn.buy(cost, command) {
            base.beacons.retain(|b| b.id != beacon.id);
            covered.push(footprint);
        }
    }
    true
}

/// 2: keeps [`DRONE_QUEUE`] Drones queued at each Robo Portal while the field wants more ([`Base::wants_drones`]) or
/// a Beacon is wanted ([`beacons::wanted`]): the Drones it releases are the ones that deploy. Returns whether the
/// steps after it may spend: `false` when the bank doesn't cover a Drone that is wanted.
fn train_drones(turn: &mut Turn<'_, '_>, base: &mut Base<'_>) -> bool {
    let view = turn.view();
    for portal in view.own().filter(|o| o.kind == RoboPortal && o.is_operational()) {
        if !base.wants_drones() && !beacons::wanted(turn, base) {
            break;
        }
        let Some(production) = &portal.production else {
            continue;
        };
        if production.queue.len() >= DRONE_QUEUE {
            continue;
        }
        let Some(price) = trains(portal, Drone) else {
            continue;
        };
        if !turn.can_pay(price) {
            return false;
        }
        if turn.buy(price, train(portal.id, Drone)) {
            base.drones_queued += 1;
            // Behind a unit, or with nothing yet begun: it starts later and takes its cost then.
            base.waiting += ArmyResource::BeaconCapacity.cost_of(Drone);
        }
    }
    true
}

/// 3: keeps [`TRAIN_QUEUE`] Sabre Legionnaires queued at each Bio Portal, while Beacon Capacity has room for them
/// once the units already queued have started. Returns how many it ordered.
fn train_army(turn: &mut Turn<'_, '_>, base: &mut Base<'_>) -> usize {
    let view = turn.view();
    let Some(cap) = view.army_cap() else {
        return 0;
    };
    let cost = cap.resource.cost_of(SabreLegionnaire);
    let mut ordered = 0;
    for portal in view.own().filter(|o| o.kind == BioPortal && o.is_operational()) {
        let Some(production) = &portal.production else {
            continue;
        };
        if production.queue.len() >= TRAIN_QUEUE {
            continue;
        }
        if cap.used.saturating_add(base.waiting).saturating_add(cost) > cap.available {
            return ordered;
        }
        let Some(price) = trains(portal, SabreLegionnaire) else {
            continue;
        };
        if turn.buy(price, train(portal.id, SabreLegionnaire)) {
            base.waiting += cost;
            ordered += 1;
        }
    }
    ordered
}

/// 6: converts its newest Sabre Legionnaires (the highest ids) into Rifle Legionnaires, so that one Legionnaire in
/// [`RIFLE_EVERY`] is a Rifle; a Rifle never converts back. A unit's type switches at the press
/// (`factions/colonists/units/weapon_conversion.md > A toggle sets a target form`), so one under way counts as a Rifle
/// at once and is never pressed twice.
fn convert(turn: &mut Turn<'_, '_>) {
    let view = turn.view();
    let mut sabres: Vec<ObjectId> = (view.own().filter(|o| o.kind == SabreLegionnaire))
        .map(|o| o.id)
        .collect();
    let rifles = view.own().filter(|o| o.kind == RifleLegionnaire).count();
    let wanted = ((sabres.len() + rifles) / RIFLE_EVERY).saturating_sub(rifles);
    if wanted == 0 {
        return;
    }
    // The player's objects come by id: the newest last.
    let units = sabres.split_off(sabres.len().saturating_sub(wanted));
    turn.give(Command::Ability {
        units,
        ability: Ability::Convert,
    });
}

/// A Train of a `kind` at `producer`.
fn train(producer: ObjectId, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(kind),
    }
}

/// What `producer` pays for a `kind`, if it trains them.
fn trains(producer: &Object, kind: ObjectKind) -> Option<u32> {
    let production = producer.kind.def().structure.as_ref()?.production.as_ref()?;
    production.recipe(kind).map(|r| r.cost)
}
