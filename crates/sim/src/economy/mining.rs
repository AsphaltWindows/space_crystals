//! Plate mining — `factions/gdo/economy.md > ExtractionPlate`: a plate "Mines Space Crystals from the patch at a
//! fixed rate. When the patch is depleted, continues mining at a residual rate indefinitely."
//!
//! - **The link** ([`Extractor`], `Object::extractor`): a structure that mines (`StructureDef::mining`: the plate)
//!   and the Space Crystals patch under it are linked as soon as both are in the game, whichever came first
//!   ([`extractor_for`] when the plate comes, [`link_patch`] when the patch does), so a map may list them in either
//!   order. **Not design** (the design always has a patch under a plate; reported to the Director): a plate on bare
//!   ground (the placement check and map validation require a patch, `crate::placement::Onto`: only a debug spawn
//!   may put a plate anywhere) has no extractor and mines nothing, not even the residual rate, and so does a plate
//!   whose patch a debug command removed (the removal path drops the link, `Sim::remove_objects`) and a plate
//!   without an owner, which has no bank to pay.
//! - **Cycles** (design change 0098, `> Paid in lumps`): a plate pays its whole load at once at the end of each
//!   48-frame cycle. The cycle advances by the owner's power ratio each tick (`> PowerPenalty`; the ratio in
//!   [`crate::economy::RATIO_ONE`]s, plan decision T6), the remainder carried over into the next cycle, so the lumps
//!   come less often at negative power and stay the same size. "A new plate's first payment comes 48 frames after it
//!   is placed": like every system, mining starts in the tick after the one that made the pair
//!   (`ARCHITECTURE.md > Tick pipeline > Spawn tick`), so a plate placed in tick `t` first pays in tick `t + 48`.
//! - **Loads** (design change 0099): 10 while the patch holds 10, then what is left (`> Last partial load`: 5 of a
//!   1505 patch), then the residual 1 forever: the patch stays under its plate with 0 left (`> Depleted patch under a
//!   plate`, GDO's exception to `core/map.md > SpaceCrystalsPatch`), which `SimEvent::PatchDepleted` announces.
//! - **On destruction** (`> On destruction`): the patch of a plate that leaves the game is uncovered, and a depleted
//!   one leaves with it ([`depleted_patch`], used by the removal path), which opens its tile (design change 0038).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::power::PowerSummary;
use crate::defs::{MiningDef, ObjectKind};
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement, ResourceState};
use crate::player::Player;

/// A plate's mining (module docs): the patch it mines and how far its cycle has got.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extractor {
    /// The Space Crystals patch under the plate.
    pub patch: ObjectId,
    /// The progress of the current cycle, in [`crate::economy::RATIO_ONE`]s of a tick at full speed: the cycle ends
    /// at `MiningDef::cycle_ticks × RATIO_ONE`, and whatever goes beyond starts the next one.
    pub cycle: u32,
}

impl Extractor {
    fn new(patch: ObjectId) -> Self {
        Self { patch, cycle: 0 }
    }

    /// One tick at `speed`: whether a cycle of `mining` ended in it.
    fn advance(&mut self, speed: u32, mining: &MiningDef) -> bool {
        let length = mining.cycle_ticks.saturating_mul(super::RATIO_ONE);
        self.cycle = self.cycle.saturating_add(speed);
        if self.cycle >= length {
            self.cycle -= length;
            true
        } else {
            false
        }
    }
}

impl StateHash for Extractor {
    fn hash(&self, h: &mut StateHasher) {
        let Extractor { patch, cycle } = self;
        patch.hash(h);
        cycle.hash(h);
    }
}

/// How a structure of `kind` mines, if it does.
fn mining_of(kind: ObjectKind) -> Option<MiningDef> {
    kind.def().structure.as_ref()?.mining
}

fn is_patch(o: &Object) -> bool {
    matches!(o.resource, Some(ResourceState::CrystalPatch { .. }))
}

/// The extractor a new object of `kind` at `placement` starts with: a mining structure on a patch mines that patch
/// (the first by id, should a map shown in the editor put two there); anything else mines nothing.
pub(crate) fn extractor_for(
    objects: &BTreeMap<ObjectId, Object>,
    kind: ObjectKind,
    placement: Option<&Placement>,
) -> Option<Extractor> {
    mining_of(kind)?;
    let placement = placement?;
    objects
        .values()
        .find(|o| is_patch(o) && o.placement.as_ref().is_some_and(|p| p.overlaps(placement)))
        .map(|patch| Extractor::new(patch.id))
}

/// The patch `patch` came into the game: a mining structure already on its tiles that mines nothing mines it from
/// now on (a map may list a plate before its patch, and a debug command may spawn a patch under a plate).
pub(crate) fn link_patch(objects: &mut BTreeMap<ObjectId, Object>, patch: ObjectId) {
    let Some(placement) = objects.get(&patch).filter(|o| is_patch(o)).and_then(|o| o.placement) else {
        return;
    };
    for o in objects.values_mut() {
        if o.extractor.is_none()
            && mining_of(o.kind).is_some()
            && o.placement.as_ref().is_some_and(|p| p.overlaps(&placement))
        {
            o.extractor = Some(Extractor::new(patch));
        }
    }
}

/// The patch that leaves the game with `plate` (`> On destruction`: "If the underlying Space Crystal Patch is
/// depleted: the patch is removed from the map"): its patch if that has nothing left, else `None` ("the patch
/// becomes uncovered and accessible"), as for an object that mines nothing.
pub(crate) fn depleted_patch(objects: &BTreeMap<ObjectId, Object>, plate: ObjectId) -> Option<ObjectId> {
    let patch = objects.get(&plate)?.extractor?.patch;
    let depleted = matches!(
        objects.get(&patch)?.resource,
        Some(ResourceState::CrystalPatch { remaining: 0 })
    );
    depleted.then_some(patch)
}

/// The plates mine (module docs): each one from before this tick, on a patch from before it too, advances its cycle
/// at its owner's power ratio, and one whose cycle ends pays its owner the load it takes from its patch. In id order,
/// the cycles first, then the loads. A plate without an owner mines nothing.
pub(super) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    power: &BTreeMap<PlayerId, PowerSummary>,
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) {
    let mut ended = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(extractor), Some(owner)) = (&mut o.extractor, o.owner) else {
            continue;
        };
        // A patch spawned under the plate this tick: the pair mines from the next tick on.
        if extractor.patch >= first_new {
            continue;
        }
        let Some(mining) = mining_of(o.kind) else {
            continue;
        };
        // `> PowerPenalty`: "the same ratio as the global power penalty for buildings".
        let speed = power.get(&owner).copied().unwrap_or_default().ratio();
        if extractor.advance(speed, &mining) {
            ended.push((o.id, extractor.patch, owner, mining));
        }
    }
    for (plate, patch, owner, mining) in ended {
        let Some(ResourceState::CrystalPatch { remaining }) = objects.get_mut(&patch).and_then(|o| o.resource.as_mut())
        else {
            continue;
        };
        let (amount, depleted) = load(remaining, &mining);
        if let Some(player) = players.iter_mut().find(|p| p.id == owner) {
            player.credit_crystals(amount);
        }
        if depleted {
            events.push(SimEvent::PatchDepleted { patch, plate });
        }
    }
}

/// The load a cycle takes from a patch with `remaining` crystals (module docs), and whether it emptied the patch:
/// `MiningRate` while the patch holds that much, then what is left, then `ResidualMiningRate`, which leaves the
/// patch at 0.
fn load(remaining: &mut u32, mining: &MiningDef) -> (u32, bool) {
    if *remaining == 0 {
        return (mining.residual, false);
    }
    let amount = (*remaining).min(mining.load);
    *remaining -= amount;
    (amount, *remaining == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::RATIO_ONE;

    const PLATE: MiningDef = MiningDef {
        load: 10,
        residual: 1,
        cycle_ticks: 48,
    };

    /// Design change 0099's verification 1: a patch of 25 pays +10, +10, +5, then +1 every cycle; 0 never
    /// underflows.
    #[test]
    fn loads_take_ten_then_the_rest_then_the_residual() {
        let mut remaining = 25;
        let loads: Vec<(u32, bool)> = (0..6).map(|_| load(&mut remaining, &PLATE)).collect();
        assert_eq!(
            loads,
            [(10, false), (10, false), (5, true), (1, false), (1, false), (1, false)]
        );
        assert_eq!(remaining, 0);
        let mut ten = 10;
        assert_eq!(load(&mut ten, &PLATE), (10, true), "a full last load");
        assert_eq!(load(&mut ten, &PLATE), (1, false));
    }

    /// Design change 0098: a cycle of 48 ticks at full speed; at a ratio, the remainder carries over.
    #[test]
    fn a_cycle_advances_at_the_speed_and_carries_the_rest_over() {
        let mut extractor = Extractor::new(ObjectId(1));
        let ends = |extractor: &mut Extractor, speed: u32, ticks: u32| -> Vec<u32> {
            (1..=ticks).filter(|_| extractor.advance(speed, &PLATE)).collect()
        };
        assert_eq!(ends(&mut extractor, RATIO_ONE, 100), [48, 96]);
        assert_eq!(extractor.cycle, 4 * RATIO_ONE, "4 ticks into the third");
        // Half speed from here: the 44 ticks left of this cycle take 88.
        assert_eq!(ends(&mut extractor, RATIO_ONE / 2, 100), [88]);
        let none: [u32; 0] = [];
        assert_eq!(
            ends(&mut Extractor::new(ObjectId(1)), 0, 1000),
            none,
            "no power, no mining"
        );
    }
}
