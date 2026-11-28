//! (M5-X2) The numbers of a ruleset within what the game is made for ([`Ruleset::check`]), so that a data file's
//! tuning (`DebugCommand::SetRules`, plan decision T66) can't stop the game: a number may change, but not to one that
//! a count, a division or a timer of the sim isn't made for. The gate refuses the rules otherwise
//! (`Reject::RulesInvalid`). Two rules, for rules of the built-in rules' structure (`Ruleset::structural_diff`, which
//! the gate checks first):
//!
//! - **Every number keeps its sign**, one rule for every number, those of later slices included: one the built-in
//!   rules have positive stays positive (a speed, a cost, a time, a radius, a capacity, a MaxHP, a tunable's step or
//!   count), one they have at 0 may grow but not go negative (an immobile kind's speed of 0, plan decision T87; an
//!   armour of 0), and one they have negative stays negative. So a zero speed of a unit that moves, a zero time or a
//!   zero lattice step is refused, as `defs/tests.rs` asserts of the built-in defs ("max speed", the recipes' frames,
//!   the projectiles' speed, the areas' radii), and so is a negative cost.
//! - **The bounds `defs/tests.rs` asserts** beyond the sign: a recipe or a technology takes at most [`LONGEST_RECIPE`]
//!   ticks, its progress counted in fixed point (`crate::construction`); a Recruitment Area covers its building and
//!   stays within [`WIDEST_RECRUITMENT_AREA`] tiles a side; an air unit's SeparationRadius is longer than its
//!   Silhouette's longer side (`systems/units.md > Air Collision`: "Must be larger than the unit's Silhouette"); a
//!   default rally point lies out of its building (`ProductionDef::default_rally`, which isn't structure, so the
//!   signs don't take it); (M6-GD) a stock's purchase takes at most [`LONGEST_RECIPE`] ticks, as a recipe, and a unit
//!   starts with at most its stock's max (`UnitDef::stock`); (M6-SD) so does an aircraft's rise and descent at an air
//!   door (`UnitDef::air_door_ticks`), counted in fixed point as well (`Mover::rise`, `orders::state::Enter::descent`);
//!   (M6-SN) a Null Chamber turn (`ChamberDef::turn`, in seconds, counted in ticks in fixed point:
//!   `network::Service::chamber`); (M6-GA) and a vehicle deploy (`UnitDef::vehicle_deploy`, `crate::vehicle_deploy`);
//!   (M6-GH2) and an aircraft's landing and take-off at an Airfield (`AirfieldDef::landing_ticks`) and its rearm of a
//!   round (`AmmoDef::rearm_ticks_per_ammo`), counted in fixed point as well (`crate::landing`). An Airfield has a
//!   landing slot for each of its assignments, so its capacity is at most [`LANDING_SLOTS`], the slots of the
//!   placeholder layout (`landing::slots`), which lie inside its footprint (`AirfieldDef::slot_offset_su`).
//!   (M6-SB) A tank holds at most [`LARGEST_TANK`] units of charge, counted in fixed point (`crate::patches`), and a
//!   patch's slow takes less than the whole of a unit's speed (`SlowDef::by`), so that a unit in sludge still moves
//!   out of it. An aircraft's own landing and take-off on a spot take at most [`LONGEST_RECIPE`] ticks
//!   (`LandingDef`), counted in fixed point as at an Airfield (`crate::landing`), and a Supply
//!   Tower's spot lies inside its footprint (`SupplyTowerDef::spot_offset_su`).

use super::structure::{Skeleton, variant_name};
use super::{RuleField, Ruleset};
use crate::defs::{ObjectDef, ObjectKind};
use crate::economy::RATIO_ONE;
use crate::landing::LANDING_SLOTS;
use crate::units::{SU_PER_GU, seconds};

/// The longest recipe or research, ticks: its progress, `frames × RATIO_ONE` (`crate::construction`), fits a `u32`
/// (`defs/tests.rs`: "short enough for their progress in fixed point").
const LONGEST_RECIPE: u32 = u32::MAX / RATIO_ONE;

/// (M6-SB) The largest tank, units of charge (`TankDef::capacity`): its charge in fixed point, `capacity × RATIO_ONE`
/// (`crate::patches`), fits a `u32`.
const LARGEST_TANK: f32 = (u32::MAX / RATIO_ONE) as f32;

/// The widest Recruitment Area, tiles a side (`RecruitmentDef::area`; the design's is 10): a bound on the tiles a
/// Hall's claims look over (`crate::recruitment`). Placeholder (not in the design: a technical bound, the side of the
/// largest map).
const WIDEST_RECRUITMENT_AREA: u32 = 255;

/// A number of the rules that [`Ruleset::check`] refuses: where it is, and why (`Reject::RulesInvalid`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Invalid {
    pub field: RuleField,
    /// What the number must be: "must be positive", "must not be negative", ...
    pub why: &'static str,
}

impl Ruleset {
    /// (M5-X2) The first number of these rules that the game isn't made for, if one is (module docs): one whose sign
    /// isn't the built-in rules', or one beyond a bound of the defs. `None` for the built-in rules. Rules of another
    /// structure than the built-in rules' are checked as far as the two have the same (`Ruleset::structural_diff`
    /// refuses them).
    pub fn check(&self) -> Option<Invalid> {
        let builtin = Skeleton::of(Ruleset::builtin_ref());
        signs(&builtin, &Skeleton::of(self)).or_else(|| bounds(self))
    }
}

/// The first number of `rules` whose sign isn't that of the same number in `builtin` (module docs).
fn signs(builtin: &Skeleton, rules: &Skeleton) -> Option<Invalid> {
    builtin.numbers_beside(rules).find_map(|(was, is, field)| {
        let why = if was > 0.0 && is <= 0.0 {
            "must be positive"
        } else if was == 0.0 && is < 0.0 {
            "must not be negative"
        } else if was < 0.0 && is >= 0.0 {
            "must be negative"
        } else {
            return None;
        };
        Some(Invalid { field, why })
    })
}

/// The first number of `rules` beyond a bound of `defs/tests.rs` (module docs).
fn bounds(rules: &Ruleset) -> Option<Invalid> {
    let defs = rules.defs.iter().find_map(|(kind, def)| def_bounds(kind, def));
    defs.or_else(|| {
        let mut techs = rules.factions.cults.techs.iter();
        techs.find_map(|(tech, def)| {
            (def.frames > LONGEST_RECIPE).then(|| Invalid {
                field: RuleField {
                    entry: variant_name(&tech),
                    field: "frames",
                },
                why: "must be at most 65535 ticks",
            })
        })
    })
}

/// The first number of `kind`'s def beyond a bound (module docs).
fn def_bounds(kind: ObjectKind, def: &ObjectDef) -> Option<Invalid> {
    let invalid = |field, why| {
        Some(Invalid {
            field: RuleField {
                entry: variant_name(&kind),
                field,
            },
            why,
        })
    };
    if recipe_frames(def).any(|frames| frames > LONGEST_RECIPE) {
        return invalid("frames", "must be at most 65535 ticks");
    }
    let production = def
        .structure
        .as_ref()
        .and_then(|structure| structure.production.as_ref());
    if let Some(recruitment) = production.and_then(|production| production.recruitment) {
        let (w, h) = def.tile_size().unwrap_or((1, 1));
        if recruitment.area < w.max(h) {
            return invalid("area", "must cover the building");
        }
        if recruitment.area > WIDEST_RECRUITMENT_AREA {
            return invalid("area", "must be at most 255 tiles");
        }
    }
    if production.and_then(|production| production.default_rally) == Some(0) {
        return invalid("default_rally", "must be positive");
    }
    let radius = def.unit.as_ref().and_then(|unit| unit.separation_radius);
    if let (Some(radius), Some(silhouette)) = (radius, def.silhouette())
        && radius <= silhouette.max_element()
    {
        return invalid("separation_radius", "must be longer than the Silhouette");
    }
    // (M6-GD) A stock's purchase counts its progress in fixed point as a recipe does (`crate::stock`), and a unit
    // starts with no more than its stock holds.
    if let Some(stock) = def.unit.as_ref().and_then(|unit| unit.stock) {
        if stock.frames > LONGEST_RECIPE {
            return invalid("frames", "must be at most 65535 ticks");
        }
        if stock.start > stock.max {
            return invalid("start", "must be at most the stock's max");
        }
    }
    // (M6-SD) The rise out of an air door and the descent into one count their progress in fixed point too.
    if def
        .unit
        .as_ref()
        .is_some_and(|unit| unit.air_door_ticks > LONGEST_RECIPE)
    {
        return invalid("air_door_ticks", "must be at most 65535 ticks");
    }
    // (M6-SN) And so does a unit's turn in a Null Chamber, of its seconds in ticks.
    let chamber = def.structure.as_ref().and_then(|structure| structure.chamber);
    if chamber.is_some_and(|chamber| seconds(chamber.turn) > LONGEST_RECIPE) {
        return invalid("turn", "must be at most 65535 ticks (4095 s)");
    }
    // (M6-GA) So does a vehicle deploy (`crate::vehicle_deploy`).
    let deploy = def.unit.as_ref().and_then(|unit| unit.vehicle_deploy);
    if deploy.is_some_and(|deploy| deploy.frames > LONGEST_RECIPE) {
        return invalid("frames", "must be at most 65535 ticks");
    }
    // (M6-GH2) So do an aircraft's rearm of a round and its landing and take-off at an Airfield (`crate::landing`); an
    // Airfield has a slot for each assignment, the placeholder layout's at most, inside its footprint.
    let ammo = def.unit.as_ref().and_then(|unit| unit.ammo);
    if ammo.is_some_and(|ammo| ammo.rearm_ticks_per_ammo > LONGEST_RECIPE) {
        return invalid("rearm_ticks_per_ammo", "must be at most 65535 ticks");
    }
    if let Some(airfield) = def.structure.as_ref().and_then(|structure| structure.airfield) {
        if airfield.landing_ticks > LONGEST_RECIPE {
            return invalid("landing_ticks", "must be at most 65535 ticks");
        }
        if airfield.capacity as usize > LANDING_SLOTS {
            return invalid("capacity", "must be at most 4, the Airfield's landing slots");
        }
        let (w, h) = def.tile_size().unwrap_or((1, 1));
        if airfield.slot_offset_su >= w.min(h) as f32 * SU_PER_GU / 2.0 {
            return invalid("slot_offset_su", "must keep the landing slots inside the footprint");
        }
    }
    // So do an aircraft's own landing and take-off on a spot (`crate::landing`), and a Supply Tower's spot
    // lies inside its footprint.
    let lands = def.unit.as_ref().and_then(|unit| unit.landing);
    if lands.is_some_and(|lands| lands.landing_ticks > LONGEST_RECIPE) {
        return invalid("landing_ticks", "must be at most 65535 ticks");
    }
    if lands.is_some_and(|lands| lands.takeoff_ticks > LONGEST_RECIPE) {
        return invalid("takeoff_ticks", "must be at most 65535 ticks");
    }
    if let Some(tower) = def.structure.as_ref().and_then(|structure| structure.supply_tower) {
        let (w, h) = def.tile_size().unwrap_or((1, 1));
        let [x, y] = tower.spot_offset_su;
        if x >= w as f32 * SU_PER_GU / 2.0 || y >= h as f32 * SU_PER_GU / 2.0 {
            return invalid("spot_offset_su", "must keep the landing spot inside the footprint");
        }
    }
    // (M6-SB) A tank's charge, in fixed point, and a slow that leaves a unit some speed (`crate::patches`).
    for tank in def.unit.as_ref().map_or(&[][..], |unit| unit.tanks) {
        if tank.capacity > LARGEST_TANK {
            return invalid("capacity", "must be at most 65535");
        }
        if tank.slow.is_some_and(|slow| slow.by >= 1.0) {
            return invalid("by", "must be less than 1");
        }
    }
    None
}

/// The frames of every recipe of `def`: what it constructs, warps in, produces, and the augmentation orders it takes.
fn recipe_frames(def: &ObjectDef) -> impl Iterator<Item = u32> + '_ {
    let structure = def.structure.as_ref();
    let production = structure.and_then(|structure| structure.production.as_ref());
    let recipes = structure.into_iter().flat_map(move |structure| {
        let produced = production.map_or(&[][..], |production| production.recipes);
        [structure.constructs, structure.warp_in, produced]
            .into_iter()
            .flatten()
    });
    let augment = production.into_iter().flat_map(|production| production.augment);
    (recipes.map(|recipe| recipe.frames)).chain(augment.map(|order| order.frames))
}

const _: () = assert!(LONGEST_RECIPE == 65535, "the bound the messages give");
const _: () = assert!(LARGEST_TANK == 65535.0, "the bound the message gives");

#[cfg(test)]
mod tests;
