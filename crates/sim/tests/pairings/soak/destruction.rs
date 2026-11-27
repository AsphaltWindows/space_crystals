//! The soak's destruction of a Cults player's Armory with units inside (`super`, module docs, 1), and the evacuation it
//! checks (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`, `Sim::evacuate`).

use sc_sim::economy::RATIO_ONE;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, ProductionCommand, SU_PER_GU, Sim, SimEvent,
    Tick, Vec2,
};

use super::Tally;

/// A Cults player's Armory destroyed with units inside (module docs, 1), and its evacuation checked.
pub(super) struct Destruction {
    pub(super) player: PlayerId,
    /// How many ticks the Armory holds units in before the one it is destroyed in: the seed's.
    skip: u32,
    seen: u32,
    pub(super) removal: Option<Removal>,
}

/// The Armory a [`Destruction`] removed, as the step that removed it found it.
pub(super) struct Removal {
    pub(super) tick: Tick,
    armory: ObjectId,
    /// Its footprint's corners, in space units.
    min: Vec2,
    max: Vec2,
    /// The units inside, and what each comes out as: its own kind, or the trained one of the finished front entry
    /// (design change 0349: "**A unit that had finished training** and was waiting for the doorway comes out **as the
    /// trained unit**, with no refund").
    pub(super) units: Vec<(ObjectId, ObjectKind)>,
    /// The refunds owed, the kind of each entry not finished, the step's own Trains among them, and its price
    /// (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Orders are cancelled with a
    /// full refund").
    refunds: Vec<(ObjectKind, u32)>,
}

impl Destruction {
    pub(super) fn new(player: PlayerId, skip: u32) -> Self {
        Self {
            player,
            skip,
            seen: 0,
            removal: None,
        }
    }

    /// The removal of the player's Armory (their first complete one, by id) for the step to the next tick, after the
    /// step's `commands`, once it has held units in [`Destruction::skip`] ticks: `None` before then, and after.
    pub(super) fn before(&mut self, sim: &Sim, commands: &[PlayerCommand]) -> Option<PlayerCommand> {
        if self.removal.is_some() {
            return None;
        }
        let armory = (sim.objects())
            .find(|o| o.owner == Some(self.player) && o.kind == ObjectKind::Armory && o.is_operational())?;
        let inside = sim.held_in(armory.id);
        if inside.is_empty() {
            return None;
        }
        self.seen += 1;
        if self.seen <= self.skip {
            return None;
        }
        let placement = armory.placement.expect("an Armory has a footprint");
        let min = placement.origin.corner();
        let max = min + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU;
        let recipes = (armory.def().structure.as_ref())
            .and_then(|s| s.production.as_ref())
            .map_or(&[][..], |p| p.recipes);
        let recipe = |kind: ObjectKind| {
            (recipes.iter().find(|r| r.kind == kind)).unwrap_or_else(|| panic!("the Armory trains no {kind:?}"))
        };
        let production = armory.production.as_ref().expect("an Armory produces");
        let front = production.queue.first().and_then(|entry| entry.unit());
        let finished = front.filter(|&kind| production.progress >= recipe(kind).frames.saturating_mul(RATIO_ONE));
        let mut refunds: Vec<(ObjectKind, u32)> = (production.queue.iter().enumerate())
            .filter(|&(i, _)| i > 0 || finished.is_none())
            .filter_map(|(_, entry)| entry.unit())
            .map(|kind| (kind, recipe(kind).cost))
            .collect();
        // The step's commands come before the removal: a Train the line gives the Armory in it is queued first.
        for c in commands.iter().filter(|c| c.player == self.player) {
            if let Command::Production {
                objects,
                command: ProductionCommand::TrainFrom { kind, .. },
            } = &c.command
                && objects.contains(&armory.id)
            {
                refunds.push((*kind, recipe(*kind).cost));
            }
        }
        let let_out = finished.and_then(|kind| Some((*production.reserved.first()?.first()?, kind)));
        let units = (inside.iter())
            .map(|held| {
                let id = held.object.id;
                let comes_out_as = let_out
                    .filter(|&(unit, _)| unit == id)
                    .map_or(held.object.kind, |(_, kind)| kind);
                (id, comes_out_as)
            })
            .collect();
        self.removal = Some(Removal {
            tick: sim.tick() + 1,
            armory: armory.id,
            min,
            max,
            units,
            refunds,
        });
        Some(PlayerCommand {
            player: self.player,
            command: Command::Debug(DebugCommand::Remove { id: armory.id }),
        })
    }

    /// After the step to `sim.tick()`: in the removal's tick, that the Armory is gone and each of its units came out
    /// (`Sim::evacuate`): a `SimEvent::Evacuated`, on the map within its footprint, idle, as the kind it comes out as;
    /// and that each entry not finished was refunded in full.
    pub(super) fn after(&self, sim: &Sim, tally: &mut Tally) {
        let Some(removal) = self.removal.as_ref().filter(|r| r.tick == sim.tick()) else {
            return;
        };
        let tick = removal.tick;
        assert!(
            sim.object(removal.armory).is_none(),
            "tick {tick}: the Armory {:?} is still there",
            removal.armory
        );
        let evacuated: Vec<ObjectId> = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::Evacuated { unit, building } if building == removal.armory => Some(unit),
                _ => None,
            })
            .collect();
        let mut inside: Vec<ObjectId> = removal.units.iter().map(|&(id, _)| id).collect();
        let mut out = evacuated.clone();
        inside.sort();
        out.sort();
        assert_eq!(out, inside, "tick {tick}: the units evacuated from the Armory");
        // Idle units push apart from the first movement phase on, in this tick too (`sim/holding/evacuation.rs`).
        let slack = SU_PER_GU / 4.0;
        for &(id, kind) in &removal.units {
            let unit = sim
                .object(id)
                .unwrap_or_else(|| panic!("tick {tick}: evacuated {id:?} isn't on the map"));
            assert_eq!(unit.kind, kind, "tick {tick}: evacuated {id:?}'s kind");
            let (lo, hi) = (removal.min - Vec2::splat(slack), removal.max + Vec2::splat(slack));
            assert!(
                unit.pos.cmpge(lo).all() && unit.pos.cmple(hi).all(),
                "tick {tick}: evacuated {id:?} at {} is outside the footprint {}..{}",
                unit.pos,
                removal.min,
                removal.max
            );
            let idle = unit.orders.as_ref().is_none_or(|orders| orders.is_idle());
            assert!(idle, "tick {tick}: evacuated {id:?} has orders: {:?}", unit.orders);
        }
        let mut refunded: Vec<(ObjectKind, u32)> = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::ProductionCancelled { producer, kind, refund } if producer == removal.armory => {
                    Some((kind, refund))
                }
                _ => None,
            })
            .collect();
        let mut owed = removal.refunds.clone();
        refunded.sort();
        owed.sort();
        assert_eq!(refunded, owed, "tick {tick}: the Armory's refunds");
        tally.destroyed += 1;
        tally.evacuated += u32::try_from(removal.units.len()).unwrap_or(u32::MAX);
        tally.refunded += u32::try_from(owed.len()).unwrap_or(u32::MAX);
    }
}
