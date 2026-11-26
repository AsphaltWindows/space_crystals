//! Repair (slice M5-N8, plan decision T83; design changes 0321, 0322, 0325, 0328, 0329 and 0360): "**Repair rules
//! shared by every repairer** (user, 2026-11-26): several repairers on one target repair **faster, scaling linearly**
//! (2 repairers = twice the rate), as construction does" (`factions/cults/units/recruit.md > Repair`,
//! `factions/syndicate/units/agent.md > Repair`). One mechanism for every repairer, whose rules are here:
//!
//! - **Who repairs what** is the repairer's def (`UnitDef::repairs`, [`payment`]): the Recruit its owner's buildings
//!   and vehicles, for Space Crystals ("Recruits can **repair** own **buildings and vehicles**. Repair **costs Space
//!   Crystals**"), and so does the Field Engineer (slice M5-N9, `factions/gdo/units/field_engineer.md > Abilities`:
//!   "repairs buildings and vehicles. Repair costs Space Crystals (not free)"); the Agent its owner's Tunnels, for
//!   free, and Laser Turrets, for Space Crystals ("Agents can **repair** own **Tunnels** and **Laser Turrets**, but
//!   **not expansions** ... **Laser Turrets cost Space Crystals to repair; Tunnels are repaired for free**"). A vehicle ([`is_vehicle`]) is a unit of neither infantry base that
//!   isn't a glider: "**Gliders can't be repaired** (they can't stop)", told by its movement, not its name. No one
//!   repairs a Tunnel's expansions, the underground structures (`factions/syndicate/structures/README.md > Tunnel
//!   Expansions`: "**Not repaired; they regenerate**"; their regeneration is `sim/expansions.rs`'s).
//! - **The terms** ([`terms`]) follow from what built the target: "**Placeholder rate and cost** (designer,
//!   2026-11-26): repairing from 0 to full HP takes **half the target's build time** with one repairer, and costs
//!   **half its price** in the resource it was bought with, paid gradually as HP is restored" (`tuning::repair`). The
//!   build is a worker's site (`SiteDef`: a Cults building's, and the Tunnel's, whose "rate uses its construction time,
//!   480 frames, at every Tier", design change 0328), a Tunnel's expansion's (`ExpansionDef`: the Laser Turret's, which
//!   only Tunnels build since design change 0359), or an entry of a list that makes it (`Recipe`: a producer's for a
//!   unit, "A vehicle's price for this counts its **Space Crystals only**, not the Recruits it consumed"; a
//!   constructor's or a Beacon's for a building). A kind with none of these has no terms, and isn't repaired.
//!   **Placeholder (not in the design: a paid repair of a target bought with another resource)**: a repairer that pays
//!   pays in Space Crystals, so it doesn't repair a target whose price isn't in them (a Tunnel's is in Supplies, and
//!   only the Agent, which repairs it for free, repairs it).
//! - **The target** ([`can_repair`]): an object of the repairer's owner's, complete (no site, nothing transforming:
//!   `factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed) →
//!   Repair"), below its MaxHP ([`damage`]), that the repairer repairs. A neutral turret, its Tunnel lost, is nobody's
//!   (design change 0360). The gate (`Sim::check_order`), the right-click (`orders::resolution`) and the client's
//!   pointer all go by this.
//! - **The order** (`OrderKind::Repair`, `orders::repair`) takes the repairer to a building's edge or to a vehicle, and
//!   **the step** (`Sim::step_repairs`, after the sites' in the economy phase) restores HP for each repairer at work,
//!   derived each tick as a site's builders are (`crate::spot_work`): `n` repairers restore `n` × MaxHP / the repair's
//!   frames a tick, up to the MaxHP. It ends at full HP: "repairers **stop once the target is at full HP**".
//! - **Payment**, in whole crystals and with no state of its own ([`step_of`], [`top_of_step`]): a repair's price
//!   spread over the MaxHP makes steps of MaxHP / price HP, the HP from 0 up to `hp` taking ceil(`hp` × price / MaxHP)
//!   of them. A repair that takes HP from one step into a later one pays a crystal for each step it enters, so that
//!   from 0 to full it pays the whole price rounded up (a Seeker's 37.5 as 38), and one that begins in the middle of a
//!   step has the rest of that step free, less than a crystal's worth. "If the player **runs out of Space Crystals**
//!   mid-repair, the order **ends** (they stop; a free Tunnel repair never runs out)": a step the bank can't pay for
//!   isn't entered, the HP stopping at the top of the last one it paid for, and the repairers that pay stop.

#[cfg(test)]
pub(crate) mod tests;

use crate::augmentation::Research;
use crate::defs::{Category, MovementModel, ObjectKind, Recipe, RepairPayment, RepairTargets, SitePrice};
use crate::object::Object;
use crate::tuning::repair::{REPAIR_COST_SHARE, REPAIR_TIME_SHARE};

/// The terms of the repair of an object of some kind (module docs, [`terms`]): its build's, shared out by
/// `tuning::repair`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Terms {
    /// The ticks one repairer takes to restore the whole MaxHP: half the build time.
    pub frames: f32,
    /// What restoring the whole MaxHP costs a repairer that pays, Space Crystals: half the price. `None` if the price
    /// isn't in Space Crystals (the Tunnel's Supplies).
    pub price: Option<f32>,
}

/// The terms of the repair of an object of `kind` (module docs), from what built it: its site (`SiteDef`), its
/// expansion's build (`ExpansionDef`), or the first entry that makes it ([`recipe_of`]). `None` for a kind with none,
/// or whose build takes no time.
pub fn terms(kind: ObjectKind) -> Option<Terms> {
    let structure = kind.def().structure.as_ref();
    let (frames, price) = if let Some(site) = structure.and_then(|s| s.site) {
        let price = match site.price {
            SitePrice::Crystals(crystals) => Some(crystals),
            SitePrice::SuppliesPerOwned => None,
        };
        (site.frames, price)
    } else if let Some(expansion) = structure.and_then(|s| s.expansion) {
        (expansion.frames, Some(expansion.cost))
    } else {
        let recipe = recipe_of(kind)?;
        (recipe.frames, Some(recipe.cost))
    };
    let frames = frames as f32 * REPAIR_TIME_SHARE;
    (frames > 0.0).then(|| Terms {
        frames,
        price: price.map(|crystals| crystals as f32 * REPAIR_COST_SHARE),
    })
}

/// The first entry that makes a `kind`, in the registry's order of the types whose lists have one: a producer's
/// recipe, a constructor's `Constructs` entry, or a Beacon's Warp In entry. **Placeholder (not in the design: a kind
/// two types make at different prices)**: the first type's, there being none so far.
fn recipe_of(kind: ObjectKind) -> Option<&'static Recipe> {
    (ObjectKind::ALL.iter())
        .filter_map(|maker| maker.def().structure.as_ref())
        .find_map(|s| {
            let produced = s.production.as_ref().map_or(&[][..], |production| production.recipes);
            (produced.iter().chain(s.constructs).chain(s.warp_in)).find(|recipe| recipe.kind == kind)
        })
}

/// Whether units of `kind` are vehicles to a repairer (module docs; plan decision T83): units of neither infantry base
/// (`UnitBase::is_infantry`) that aren't gliders, by their movement (`MovementModel::Glider`: "they can't stop").
pub fn is_vehicle(kind: ObjectKind) -> bool {
    (kind.def().unit.as_ref())
        .is_some_and(|unit| !unit.base.is_infantry() && unit.movement.model() != MovementModel::Glider)
}

/// Whether objects of `kind` are buildings to a repairer: structures (`Category::Structure`), but for a Tunnel's
/// expansions, the underground ones (`StructureDef::underground`: "Not repaired; they regenerate").
fn is_building(kind: ObjectKind) -> bool {
    let def = kind.def();
    def.category == Category::Structure && !def.structure.as_ref().is_some_and(|s| s.underground)
}

/// How a unit of `repairer` repairs an object of `target` (module docs): free or paid, if its def's `repairs` covers
/// the kind, and the kind has terms ([`terms`]) that it can pay by: a price in Space Crystals for a repairer that
/// pays. `None` if it doesn't repair it.
pub fn payment(repairer: ObjectKind, target: ObjectKind) -> Option<RepairPayment> {
    let payment = match repairer.def().unit.as_ref()?.repairs?.targets {
        RepairTargets::OwnBuildingsAndVehicles => {
            (is_building(target) || is_vehicle(target)).then_some(RepairPayment::Paid)?
        }
        RepairTargets::Kinds(kinds) => kinds.iter().find(|(kind, _)| *kind == target)?.1,
    };
    let terms = terms(target)?;
    let expansion = target.def().structure.as_ref().is_some_and(|s| s.underground);
    let paid_for = payment == RepairPayment::Free || terms.price.is_some();
    (!expansion && paid_for).then_some(payment)
}

/// Whether units of `repairer` repair objects of `target` ([`payment`]).
pub fn repairs(repairer: ObjectKind, target: ObjectKind) -> bool {
    payment(repairer, target).is_some()
}

/// The HP of `o` and its MaxHP now, (M5-N1a) its Augmentations' included, at its owner's levels in `research`
/// (`Object::max_hp_with`), if it is complete (no site, nothing transforming) and below that MaxHP: what a repair may
/// restore (module docs). The one place where repair reads a MaxHP.
pub fn damage(o: &Object, research: Research) -> Option<(f32, f32)> {
    if o.is_site() || o.is_transforming() {
        return None;
    }
    let (hp, max) = (o.hp?, o.max_hp_with(research)?);
    (hp < max).then_some((hp, max))
}

/// Whether `unit` can repair `target` now (module docs): another object of its owner's, damaged and complete
/// ([`damage`], at the owners' levels in `research`), that its type repairs ([`payment`]).
pub fn can_repair(unit: &Object, target: &Object, research: Research) -> bool {
    unit.owner.is_some()
        && target.owner == unit.owner
        && unit.id != target.id
        && damage(target, research).is_some()
        && repairs(unit.kind, target.kind)
}

/// The step of a repair's price that HP `hp` out of `max` is in (module docs): the crystals of `price` that the HP from
/// 0 up to `hp` costs, rounded up: ceil(`hp` × `price` / `max`), in 64 bits, so that a whole HP at a step's top is in
/// that step. 0 for a `max` of 0.
pub fn step_of(hp: f32, max: f32, price: f32) -> u64 {
    if max <= 0.0 {
        return 0;
    }
    // A float's cast saturates: a negative HP is in step 0.
    (f64::from(hp) * f64::from(price) / f64::from(max)).ceil() as u64
}

/// The highest HP up to `cap` that step `step` of a repair's price takes in, out of `max` ([`step_of`]): what `step`
/// crystals pay for from 0. Rounded down where the top of the step isn't a whole HP, so that it is never in the next
/// step, which isn't paid for.
pub fn top_of_step(step: u64, max: f32, price: f32, cap: f32) -> f32 {
    if price <= 0.0 {
        return cap;
    }
    let mut top = ((step as f64 * f64::from(max) / f64::from(price)) as f32).min(cap);
    while top > 0.0 && step_of(top, max, price) > step {
        top = top.next_down();
    }
    top
}
