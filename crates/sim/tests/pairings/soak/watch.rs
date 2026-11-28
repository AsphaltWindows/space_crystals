//! The soak's checks after every tick (`super`, module docs): Unit Control, held units, sites, Gliders, crushing,
//! Suppress and buffs.

use std::collections::BTreeMap;

use sc_sim::buffs::Buff;
use sc_sim::defs::MovementAttrs;
use sc_sim::economy::RATIO_ONE;
use sc_sim::network::Place;
use sc_sim::recruitment::{self, Lineages};
use sc_sim::suppress;
use sc_sim::{ArmyResource, AttackPhase, Faction, Object, ObjectId, ObjectKind, SU_PER_GU, Sim, SimEvent, Vec2};
use sc_sim::{carry, holding};

use super::{Tally, everything};

/// The farthest a Glider may be off the map (the addendum's "overshoot bound"; `systems/units.md > Air Collision`:
/// "Gliders may unavoidably travel a little outside the map edge while turning or circling"; design change 0354: "no
/// extra cap on overshooting the map edge"): the diameter of its tightest turn at MaxSpeed (MaxSpeed² /
/// MaxCentripetalAcceleration, a radius), beyond a strafing run's flying on until its target's centre is out of its
/// range from its edge (`movement::glider`), and two ticks of flight at MaxSpeed.
fn overshoot_bound(o: &Object, max_speed: f32, max_centripetal_acceleration: f32) -> f32 {
    let def = o.def();
    let range = (def.unit.as_ref())
        .and_then(|u| u.attack.as_ref())
        .map_or(0.0, |a| a.range);
    let half = def.half_extents().map_or(0.0, Vec2::length);
    2.0 * max_speed * max_speed / max_centripetal_acceleration + range + half + 2.0 * max_speed
}

/// The soak's checks after every tick (module docs), and what they remember of the tick before.
#[derive(Default)]
pub(super) struct Watch {
    /// Each object's attack phase, map and held, as the tick before left it.
    phases: BTreeMap<ObjectId, Option<AttackPhase>>,
    /// Each object's kind and buffs, as the tick before left them.
    buffs: BTreeMap<ObjectId, (ObjectKind, Vec<Buff>)>,
}

impl Watch {
    pub(super) fn check(&mut self, sim: &Sim, tally: &mut Tally) {
        unit_control(sim);
        held(sim);
        sites(sim);
        gliders(sim, tally);
        crushes(sim, tally);
        self.suppressed(sim, tally);
        self.buffs_last(sim, tally);
        self.remember(sim);
    }

    pub(super) fn remember(&mut self, sim: &Sim) {
        self.phases = everything(sim)
            .map(|o| (o.id, o.attack.as_ref().and_then(|a| a.phase)))
            .collect();
        self.buffs = (everything(sim))
            .filter(|o| !o.buffs.is_empty())
            .map(|o| (o.id, (o.kind, o.buffs.clone())))
            .collect();
    }

    /// Suppress (module docs): a unit that holds its fire doesn't aim, and its Firing, and the attack it makes at the
    /// end of one (`SimEvent::Attacked`), is the one under way as the tick before left it.
    fn suppressed(&self, sim: &Sim, tally: &mut Tally) {
        let tick = sim.tick();
        let firing = |phase: Option<AttackPhase>| {
            matches!(phase, Some(AttackPhase::Firing { .. } | AttackPhase::FiringAt { .. }))
        };
        let before = |id: ObjectId| self.phases.get(&id).copied().flatten();
        for o in everything(sim).filter(|o| suppress::holds_fire(o)) {
            tally.suppressed += 1;
            let phase = o.attack.as_ref().and_then(|a| a.phase);
            assert!(
                !matches!(phase, Some(AttackPhase::Aiming { .. } | AttackPhase::AimingAt { .. })),
                "tick {tick}: #{} ({:?}) aims while it holds its fire: {phase:?}",
                o.id.0,
                o.kind
            );
            assert!(
                !firing(phase) || firing(before(o.id)),
                "tick {tick}: #{} ({:?}) began firing while it holds its fire: {:?} after {:?}",
                o.id.0,
                o.kind,
                phase,
                before(o.id)
            );
        }
        for e in sim.events() {
            let SimEvent::Attacked { attacker, .. } = *e else {
                continue;
            };
            let Some(o) = everything(sim).find(|o| o.id == attacker) else {
                continue;
            };
            assert!(
                !suppress::holds_fire(o) || firing(before(attacker)),
                "tick {tick}: #{} ({:?}) attacked while it holds its fire, after {:?}",
                attacker.0,
                o.kind,
                before(attacker)
            );
        }
    }

    /// Buffs (module docs): every buff of the tick before whose time isn't up after this tick is still held, as long
    /// or longer, by its unit, if that is still there and of a kind its ability affects; those whose last tick this
    /// was are counted.
    fn buffs_last(&self, sim: &Sim, tally: &mut Tally) {
        let tick = sim.tick();
        for (&id, (_, buffs)) in &self.buffs {
            let now = everything(sim).find(|o| o.id == id);
            for buff in buffs {
                let still = now.and_then(|o| o.buffs.iter().find(|b| b.ability == buff.ability));
                if buff.until > tick + 1 {
                    let Some(o) = now.filter(|o| buff.ability.affects(o.kind)) else {
                        continue;
                    };
                    assert!(
                        still.is_some_and(|b| b.until >= buff.until),
                        "tick {tick}: #{} ({:?}) lost its {:?} until tick {} early: {:?}",
                        id.0,
                        o.kind,
                        buff.ability,
                        buff.until,
                        o.buffs
                    );
                } else if still.is_none_or(|b| b.until != buff.until) {
                    tally.ended += 1;
                }
            }
        }
    }
}

/// Unit Control (module docs): each Cults player's usages and orphans are their units' costs and their Halls'
/// Recruits in production, and the HUD's figure.
fn unit_control(sim: &Sim) {
    let tick = sim.tick();
    let lineages = Lineages::of(everything(sim));
    let cost = |kind| ArmyResource::CultsUnitControl.cost_of(kind);
    for p in sim
        .players()
        .iter()
        .filter(|p| p.faction == Faction::Cults && !p.is_defeated())
    {
        let halls: Vec<&Object> = (sim.objects())
            .filter(|o| o.owner == Some(p.id) && recruitment::def_of(o.kind).is_some())
            .collect();
        let usages: u32 = halls.iter().map(|hall| lineages.usage(hall.id)).sum();
        let orphans = lineages.orphaned(p.id);
        let alive: u32 = everything(sim)
            // (M6-CM) An illusion costs none (`sc_sim::mirage`: "They use **no Unit Control**").
            .filter(|o| o.owner == Some(p.id) && !sc_sim::mirage::is_illusion(o))
            .map(|o| cost(o.kind))
            .sum();
        let producing: u32 = (halls.iter())
            .flat_map(|hall| hall.production.iter().flat_map(|production| &production.queue))
            .filter_map(|entry| entry.unit())
            .map(cost)
            .sum();
        assert_eq!(
            usages + orphans,
            alive + producing,
            "tick {tick}: player {}'s usages ({usages}) and orphans ({orphans}) against their units ({alive}) and \
             Recruits in production ({producing})",
            p.id.0
        );
        let used = sim.army_cap(p.id).map(|cap| cap.used);
        assert_eq!(
            used,
            Some(usages + orphans),
            "tick {tick}: player {}'s Unit Control",
            p.id.0
        );
    }
}

/// Held units (module docs): only in buildings that are on the map, hold units and are their owner's, at sites, or
/// (M6-ST1) aboard their owner's carriers in the game, on the map or held (`sc_sim::carry`).
fn held(sim: &Sim) {
    let tick = sim.tick();
    for held in sim.held() {
        let unit = &held.object;
        match held.place {
            Place::Building(id) => {
                let building = sim.object(id);
                assert!(
                    building.is_some_and(|b| holding::holds(b).is_some() && b.owner == unit.owner),
                    "tick {tick}: #{} ({:?}) is held in {id:?}, which is {:?}",
                    unit.id.0,
                    unit.kind,
                    building.map(|b| (b.kind, b.owner))
                );
            }
            Place::Site(id) => {
                let site = sim.object(id);
                assert!(
                    site.is_some_and(|s| s.construction.is_some()),
                    "tick {tick}: #{} ({:?}) is held at {id:?}, which is no site: {:?}",
                    unit.id.0,
                    unit.kind,
                    site.map(|s| s.kind)
                );
            }
            Place::Carrier(id) => {
                let carrier = (sim.object(id)).or_else(|| sim.held_unit(id).map(|held| &held.object));
                assert!(
                    carrier.is_some_and(|c| carry::def_of(c).is_some() && c.owner == unit.owner),
                    "tick {tick}: #{} ({:?}) is aboard {id:?}, which is {:?}",
                    unit.id.0,
                    unit.kind,
                    carrier.map(|c| (c.kind, c.owner))
                );
            }
            Place::Network => {}
        }
    }
}

/// Sites (module docs): no site's progress is above its total.
fn sites(sim: &Sim) {
    for o in sim.objects() {
        let Some(c) = o.construction else { continue };
        assert!(
            c.progress.elapsed <= c.progress.total.saturating_mul(RATIO_ONE),
            "tick {}: site #{} ({:?}) is at {} of {} ticks",
            sim.tick(),
            o.id.0,
            o.kind,
            c.progress.elapsed as f32 / RATIO_ONE as f32,
            c.progress.total
        );
    }
}

/// Gliders (module docs): none farther off the map than [`overshoot_bound`].
fn gliders(sim: &Sim, tally: &mut Tally) {
    let size = Vec2::new(sim.map().width() as f32, sim.map().height() as f32) * SU_PER_GU;
    for o in sim.objects() {
        let Some(MovementAttrs::Glider {
            max_speed,
            max_centripetal_acceleration,
            ..
        }) = o.movement(sim.research())
        else {
            continue;
        };
        let off = (Vec2::ZERO - o.pos).max(o.pos - size).max(Vec2::ZERO).length();
        if off > 0.0 {
            tally.off_map += 1;
            tally.farthest = tally.farthest.max(off);
        }
        let bound = overshoot_bound(o, max_speed, max_centripetal_acceleration);
        assert!(
            off <= bound,
            "tick {}: #{} ({:?}) is {off} su off the map at {}, beyond {bound}",
            sim.tick(),
            o.id.0,
            o.kind,
            o.pos
        );
    }
}

/// Crushing (module docs): each unit crushed in the tick is gone, removed, with no Hit and no Died, crushable, and
/// another player's than its crusher, whose kind crushes.
fn crushes(sim: &Sim, tally: &mut Tally) {
    let tick = sim.tick();
    let events = sim.events();
    for e in events {
        let SimEvent::Crushed {
            id,
            kind,
            owner,
            crusher,
            ..
        } = *e
        else {
            continue;
        };
        tally.crushed += 1;
        assert!(
            everything(sim).all(|o| o.id != id),
            "tick {tick}: crushed {id:?} ({kind:?}) is still in the game"
        );
        assert!(
            events.contains(&SimEvent::Removed(id)),
            "tick {tick}: crushed {id:?} ({kind:?}) wasn't removed"
        );
        let hit_or_died = events.iter().any(|e| match *e {
            SimEvent::Hit { target, .. } => target == id,
            SimEvent::Died { id: died, .. } => died == id,
            _ => false,
        });
        assert!(
            !hit_or_died,
            "tick {tick}: crushed {id:?} ({kind:?}) was hit or died too"
        );
        let crushable = (kind.def().unit.as_ref()).is_some_and(|u| u.base.def().crushable);
        assert!(
            crushable,
            "tick {tick}: crushed {id:?} is a {kind:?}, which isn't crushable"
        );
        if let Some(by) = sim.object(crusher) {
            let crushes = (by.def().unit.as_ref()).is_some_and(|u| u.base.def().crushes);
            assert!(
                crushes,
                "tick {tick}: {crusher:?} ({:?}) crushed, and doesn't crush",
                by.kind
            );
            assert!(
                owner.is_some() && by.owner.is_some() && owner != by.owner,
                "tick {tick}: {crusher:?} of {:?} crushed {id:?} of {owner:?}",
                by.owner
            );
        }
    }
}
