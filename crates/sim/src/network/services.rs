//! (M6-SN) The Tunnel Network's services (plan decision T98): what a Syndicate player's structures do for the units
//! inside their network. The Null Chamber's (`factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`,
//! `factions/syndicate/structures/null_chamber.md > Placeholder stats`; its numbers are its def's,
//! `StructureDef::chamber`), and (M6-SB) the Vault's (`factions/syndicate/structures/vault.md`; `StructureDef::vault`).
//!
//! - **The turns** ([`step`], in the network's step of the economy phase, before the eject queues): "each Null Chamber
//!   buffs **4 units at a time**, each buff taking **2 s** inside before the unit can leave buffed". While a player
//!   owns a Null Chamber in operation (`Object::is_operational`: not one under construction), the manned units in
//!   their network ([`manned`]) take their turns ([`Service::chamber`]) first come first served, in the order they
//!   came in (the Vault's "network-wide, first come first served"): by the tick each entered the network, a produced
//!   unit from its production (`Held::since`, the EjectMenu's order too), ties by id. Every chamber of the player's
//!   serves the network: the first so many units whose turn isn't done, the chambers' slots together
//!   ([`ChamberDef::slots`]), each take a tick of it, from the tick they came in; the others wait, keeping what of
//!   their turn is done. A unit whose turn is done keeps it while it stays inside, however long, and no longer takes a
//!   slot; and keeps it if its owner loses the last chamber meanwhile (design change 0463, answering Q0187: "a unit
//!   whose turn is done **leaves buffed** even if no Null Chamber is left when it leaves").
//! - **Side by side** (design change 0424, answering Q0158: "a unit's Vault turn, Null Chamber turn (and any repair)
//!   run **at the same time** in separate queues"): each service has its queue and its turn of its own in
//!   [`Service`], so that a unit's turn at one never waits for its turn at another. The EjectMenu still takes the unit
//!   inside longest, done or not ("the EjectMenu keeps taking the unit **inside the network longest**"). The Vault's
//!   queue is a pass of its own ([`vaults`]), before the chamber's.
//! - (M6-SB) **The Vault** ([`vaults`]): "a Vault re-arms and recharges units **anywhere in the Tunnel Network**";
//!   "each Vault re-arms or recharges **2 units at a time**; others wait their turn (network-wide, first come first
//!   served) ... Shadebringer tanks refill at 5 per second each" (`factions/syndicate/structures/vault.md > Placeholder
//!   stats`). While a player owns a Vault in operation, the units in their network with a tank that isn't full
//!   (`patches::Tanks`) take their turns in the chamber's order, by the tick each came in, ties by id: the first so
//!   many, the Vaults' slots together ([`VaultDef::slots`]), each have every tank of theirs filled a tick's refill
//!   (`TankDef::refill`, `patches::refill`), the others wait. A unit's tanks are its turn: it has no other state, its
//!   turn done once they are full, and over when it leaves, however full (design change 0424: "half-filled if ejected
//!   early"). Nothing drains a held unit's tank (its toggles went off as it came in), so the units waiting keep their
//!   order, the first come first served. **Placeholder (not in the design: how a turn shares a slot)**: a slot serves
//!   a whole unit, both its tanks at once. (Slice M6-SA) A Specter's re-arming joins the queue.
//! - **The buff** ([`leaves_buffed`], [`buff`]): "While the player owns at least one Null Chamber anywhere in the
//!   Tunnel Network, every **manned** Syndicate unit that exits the Tunnel Network through any Tunnel receives a
//!   temporary buff": a unit whose turn is done, out of Side A's exit path or (M6-SD) a Tier 3 Tunnel's air exit
//!   door (`Sim::leave_network`), holds `BuffKind::NullChamber` for its duration from the tick it comes out, whatever
//!   became of its owner's chambers since its turn was done (design change 0463, answering Q0187: "**A finished turn
//!   always buffs**"; the turn, not the chamber, is what the player owning one gave it), one it held already
//!   restarted ("exiting the Tunnel Network again instantly
//!   resets the buff to full duration"). One out before its turn is done leaves without ("a unit that leaves before
//!   its turn leaves unbuffed"), and keeps a buff it still had from before, which wears off. It "ends abruptly", as
//!   every buff does (`crate::buffs`): dropped in its last tick, the 319th after the one its unit came out in, after
//!   that tick's hits and before its vision, so the hits of the 320th meet no reduction.
//! - **Its effects**, while it lasts, wherever the unit is: "**+2 grid units SightRange**" in the unit's eye
//!   ([`sight_bonus`], `vision/sight.rs`), from the tick it comes out; "**20%** damage-taken reduction", "applied
//!   **before armor**: damage taken = Attack Damage × (1 − reduction) − armor" (`systems/combat.md >
//!   DamageModifierOrder` step 1; [`damage_factor`], in `combat::pre_armor_multiplier`). The other readers of a type's
//!   SightRange, the auto-targeting scan (`orders::world`) and the AttackMove reach (`orders::reach`), keep the type's
//!   (design change 0463, answering Q0187: "**vision only**: the unit and its owner see farther, but its target scans
//!   (idle auto-targeting, AttackMove, Patrol) keep the **unbuffed** SightRange, so a buff never changes how the unit
//!   behaves").
//!   **Detection**, the design's first effect, isn't built ("stealth/detection mechanic not yet designed"): detection
//!   comes after the factions' rosters (design change 0267).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Held, Place};
use crate::buffs::{self, BuffKind};
use crate::defs::{ChamberDef, ObjectKind, VaultDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::patches;
use crate::player::Faction;
use crate::progress::Progress;
use crate::units::{Tick, seconds};

/// A held unit's turns at its network's services (`Held::service`, module docs): one for each service, each in a
/// queue of its own (design change 0424), so that one unit is served by several at once.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    /// Its Null Chamber turn: the ticks of it done, of the turn's ([`ChamberDef::turn`]), from the tick it began.
    /// Done, the unit is charged ([`Service::charged`]). `None` before its turn begins. Optional in files, so that a
    /// unit with only another service's turn (the Vault's, slice M6-SB) saves without it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chamber: Option<Progress>,
}

impl Service {
    /// Whether it has no turn, under way or done: no service has taken the unit in yet.
    pub fn is_none(&self) -> bool {
        let Service { chamber } = self;
        chamber.is_none()
    }

    /// Whether its Null Chamber turn is done: the unit leaves the network buffed while its owner owns a chamber.
    pub fn charged(&self) -> bool {
        self.chamber.is_some_and(|turn| turn.done())
    }
}

impl StateHash for Service {
    fn hash(&self, h: &mut StateHasher) {
        let Service { chamber } = self;
        // Each turn only while there is one, under a tag of its own: a service added beside it leaves these bytes be.
        if let Some(turn) = chamber {
            h.tag(0);
            turn.hash(h);
        }
    }
}

/// Whether a unit of `kind` is manned, so that it takes its turns and holds the buff
/// (`factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`: "**Manned units**: all Syndicate units
/// except satellites (infantry, vehicles, and crewed aircraft)"): a Syndicate unit type's but a satellite's
/// (`UnitDef::satellite`: the Prism).
pub fn manned(kind: ObjectKind) -> bool {
    let def = kind.def();
    def.unit.as_ref().is_some_and(|unit| !unit.satellite) && def.faction == Some(Faction::Syndicate)
}

/// The Null Chamber's service (`StructureDef::chamber`), in the current rules: its turns' and its buff's numbers. The
/// Null Chamber is the one type with one (`defs::syndicate::structures::null_chamber`), and structure keeps it so
/// (`Ruleset::structural_diff`).
fn chamber() -> &'static ChamberDef {
    chamber_of(ObjectKind::NullChamber).expect("the Null Chamber's service")
}

/// The service of a structure of `kind`, if it gives one.
fn chamber_of(kind: ObjectKind) -> Option<&'static ChamberDef> {
    kind.def().structure.as_ref()?.chamber.as_ref()
}

/// How long the Null Chamber exit buff lasts, ticks: "duration **20 s**" (`ChamberDef::duration`), 320.
pub(crate) fn duration() -> Tick {
    seconds(chamber().duration)
}

/// The slots of the chambers in operation among `objects`, by owner ("each Null Chamber buffs **4 units at a
/// time**", [`ChamberDef::slots`]): together, for the network. A player who owns none has none; a chamber under
/// construction, and one left neutral by its lost Tunnel, serve no one.
fn slots<'a>(objects: impl IntoIterator<Item = &'a Object>) -> Vec<(PlayerId, u32)> {
    let mut slots: Vec<(PlayerId, u32)> = Vec::new();
    for o in objects {
        // The owner first: most objects of a map, its crystals, have none, and need no def read.
        let Some(owner) = o.owner else {
            continue;
        };
        let Some(chamber) = chamber_of(o.kind) else {
            continue;
        };
        if !o.is_operational() {
            continue;
        }
        match slots.iter_mut().find(|(player, _)| *player == owner) {
            Some((_, count)) => *count = count.saturating_add(chamber.slots),
            None => slots.push((owner, chamber.slots)),
        }
    }
    slots
}

/// Whether `held` waits for its Null Chamber turn, or takes it (module docs): a manned unit in its owner's network
/// whose turn isn't done; (M6-ST1) not one aboard a carrier, which is in no network (`crate::carry`, design change
/// 0432: "no Null Chamber turn").
fn waits(held: &Held) -> bool {
    let inside = match held.place {
        Place::Network => true,
        Place::Site(_) | Place::Building(_) | Place::Carrier(_) => false,
    };
    inside && !held.service.charged() && manned(held.object.kind)
}

/// The services' tick (module docs), in the network's step of the economy phase before the eject queues: (M6-SB) the
/// Vaults' queue ([`vaults`]); then, of each player's units in their network that wait for their Null Chamber turn,
/// the first come, as many as the slots of the player's chambers in operation among `objects`, take a tick of it.
pub(crate) fn step(objects: &BTreeMap<ObjectId, Object>, held: &mut BTreeMap<ObjectId, Held>) {
    // (M6-SB) A queue of its own (design change 0424), which the chamber's leaves be.
    vaults(objects, held);
    // The cheap checks first, so that a network without a chamber costs no list: a unit waiting, then a chamber.
    if !held.values().any(waits) {
        return;
    }
    let slots = slots(objects.values());
    if slots.is_empty() {
        return;
    }
    let mut waiting: Vec<(PlayerId, Tick, ObjectId)> = (held.values())
        .filter(|held| waits(held))
        .filter_map(|held| Some((held.object.owner?, held.since, held.object.id)))
        .collect();
    // First come first served: by owner, then by the tick each came in, then by id.
    waiting.sort_unstable();
    let total = seconds(chamber().turn);
    let mut player = None;
    let mut left = 0;
    for (owner, _, id) in waiting {
        if player != Some(owner) {
            player = Some(owner);
            left = (slots.iter()).find(|(p, _)| *p == owner).map_or(0, |&(_, count)| count);
        }
        if left == 0 {
            continue;
        }
        left -= 1;
        if let Some(held) = held.get_mut(&id) {
            held.service.chamber.get_or_insert(Progress::new(total)).step(RATIO_ONE);
        }
    }
}

/// (M6-SB) The Vault's service of a structure of `kind` (`StructureDef::vault`), if it gives one.
fn vault_of(kind: ObjectKind) -> Option<&'static VaultDef> {
    kind.def().structure.as_ref()?.vault.as_ref()
}

/// (M6-SB) The slots of the Vaults in operation among `objects`, by owner ("each Vault re-arms or recharges **2 units
/// at a time**", [`VaultDef::slots`]): together, for the network, as the chambers' ([`slots`]).
fn vault_slots<'a>(objects: impl IntoIterator<Item = &'a Object>) -> Vec<(PlayerId, u32)> {
    let mut slots: Vec<(PlayerId, u32)> = Vec::new();
    for o in objects {
        // The owner first, as the chambers' do.
        let Some(owner) = o.owner else {
            continue;
        };
        let Some(vault) = vault_of(o.kind) else {
            continue;
        };
        if !o.is_operational() {
            continue;
        }
        match slots.iter_mut().find(|(player, _)| *player == owner) {
            Some((_, count)) => *count = count.saturating_add(vault.slots),
            None => slots.push((owner, vault.slots)),
        }
    }
    slots
}

/// (M6-SB) Whether `held` waits for its Vault turn, or takes it (module docs): a unit in its owner's network with a
/// tank that isn't full; (M6-ST1) not one aboard a carrier, which is in no network (`crate::carry`).
fn waits_for_vault(held: &Held) -> bool {
    let inside = match held.place {
        Place::Network => true,
        Place::Site(_) | Place::Building(_) | Place::Carrier(_) => false,
    };
    inside && held.object.tanks.is_some_and(|tanks| tanks.any_used())
}

/// (M6-SB) The Vaults' tick (module docs): of each player's units in their network that wait for their Vault turn, the
/// first come, as many as the slots of the player's Vaults in operation among `objects`, have their tanks refilled a
/// tick's worth (`patches::refill_tanks`).
fn vaults(objects: &BTreeMap<ObjectId, Object>, held: &mut BTreeMap<ObjectId, Held>) {
    // The cheap checks first, as the chambers': a unit waiting, then a Vault.
    if !held.values().any(waits_for_vault) {
        return;
    }
    let slots = vault_slots(objects.values());
    if slots.is_empty() {
        return;
    }
    let mut waiting: Vec<(PlayerId, Tick, ObjectId)> = (held.values())
        .filter(|held| waits_for_vault(held))
        .filter_map(|held| Some((held.object.owner?, held.since, held.object.id)))
        .collect();
    // First come first served: by owner, then by the tick each came in, then by id.
    waiting.sort_unstable();
    let mut player = None;
    let mut left = 0;
    for (owner, _, id) in waiting {
        if player != Some(owner) {
            player = Some(owner);
            left = (slots.iter()).find(|(p, _)| *p == owner).map_or(0, |&(_, count)| count);
        }
        if left == 0 {
            continue;
        }
        left -= 1;
        if let Some(held) = held.get_mut(&id) {
            patches::refill_tanks(&mut held.object);
        }
    }
}

/// Whether `held`, coming out of its network now, leaves buffed (module docs): its Null Chamber turn is done, with or
/// without a chamber of its owner's left (design change 0463: "**A finished turn always buffs**").
pub(crate) fn leaves_buffed(held: &Held) -> bool {
    held.service.charged()
}

/// `o`, out of the network in `tick`, holds the Null Chamber exit buff for its duration from `tick` (module docs):
/// one it held already is restarted.
pub(crate) fn buff(o: &mut Object, tick: Tick) {
    buffs::hold(o, BuffKind::NullChamber, tick.saturating_add(duration()));
}

/// Whether `o` holds the Null Chamber exit buff.
pub fn buffed(o: &Object) -> bool {
    !o.buffs.is_empty() && (o.buffs.iter()).any(|buff| buff.ability == BuffKind::NullChamber)
}

/// What `o`'s Null Chamber exit buff adds to its SightRange, grid units (module docs): "**+2 grid units SightRange**"
/// ([`ChamberDef::sight`]) while it holds one, else 0.
pub fn sight_bonus(o: &Object) -> f32 {
    if buffed(o) { chamber().sight } else { 0.0 }
}

/// The pre-armor multiplier `o`'s Null Chamber exit buff gives the hits against it (module docs): "damage taken =
/// Attack Damage × (1 − reduction) − armor", 0.8 while it holds one ([`ChamberDef::reduction`]), else 1.
pub fn damage_factor(o: &Object) -> f32 {
    if buffed(o) { 1.0 - chamber().reduction } else { 1.0 }
}
