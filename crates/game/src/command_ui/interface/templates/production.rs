//! The Production template's grid (plan decision T23): a producing structure's panel, built from its `ProductionDef`,
//! with the bottom row of `systems/controls/interface.md > CommandPanel > Standard Slot Assignments`.

use sc_sim::ObjectKind;

use super::{PanelCommand, slot_index};
use crate::input::CommandSlot;
use crate::kinds;

/// How many units a producing structure's panel trains: one in each of Q, W, E, R, A, S, D and F; the bottom row is
/// `> Standard Slot Assignments`' (Z Back, X Cancel Production, C Set Rally Point).
const TRAIN_SLOTS: usize = 8;

/// A producing structure's DefaultState commands (plan decision T23): Train each of the units of its `ProductionDef`,
/// in the design's order, from Q on (Q, W, E, R, then A, S, D, F), but for a unit the design puts in a slot of its own,
/// which takes that slot first ([`PanelTemplate::trains`], slice M5-N9); X Cancel Production and C Set Rally Point
/// (`> Standard Slot Assignments`); Z is Back while a target is awaited.
///
/// - The Barracks: Q Build Peacekeeper, E Build Field Engineer, X, C (`factions/gdo/structures/barracks.md >
///   ObjectInterfaceState[Barracks]`: "**E: Build Field Engineer** (placeholder letter)"). "W: Build Rocketman (hotkey
///   tentative)" stays empty: the Rocketman isn't in the game yet (plan slice K brings it, and its Arsenal), so it
///   isn't among the recipes and its button is hidden.
/// - The Robo Portal: Q Drone, W Scout, X, C; the Bio Portal: Q Sabre Legionnaire, W Lancer, E psionic unit, X, C
///   (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`,
///   `factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`, design change 0124). The Scout,
///   the Lancer and the psionic unit aren't in the game (their stats, cost and time are TBD), so they aren't among the
///   recipes: 0124 has them show in their slot, unavailable ([`PanelCommand::Planned`], [`PanelTemplate::planned`];
///   slice M3-J2).
/// - The Recruitment Hall: C alone (`factions/cults/structures/recruitment_hall.md >
///   ObjectInterfaceState[RecruitmentHall]`, slice M5-C). It recruits by itself (`ProductionDef::recruitment`), so its
///   recipe, the Recruit, has no Train, and it has no X (design change 0342: "Recruits can't be cancelled. The X button
///   is removed").
pub(super) fn production_grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
    let trains = production.filter(|production| production.recruitment.is_none());
    let recipes = trains.map_or(&[][..], |production| production.recipes);
    debug_assert!(
        recipes.len() <= TRAIN_SLOTS,
        "{kind:?} trains more units than its panel has slots"
    );
    let template = kinds::of(kind).template;
    let slot_of = |unit| (template.trains.iter()).find_map(|&(slot, placed)| (placed == unit).then_some(slot));
    for recipe in recipes {
        if let Some(slot) = slot_of(recipe.kind) {
            grid[slot] = Some(PanelCommand::Train(recipe.kind));
        }
    }
    for recipe in recipes.iter().filter(|recipe| slot_of(recipe.kind).is_none()) {
        if let Some(slot) = (0..TRAIN_SLOTS).find(|&slot| grid[slot].is_none()) {
            grid[slot] = Some(PanelCommand::Train(recipe.kind));
        }
    }
    for &(slot, planned) in template.planned {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    if trains.is_some() {
        grid[slot_index(CommandSlot::CANCEL_PRODUCTION)] = Some(PanelCommand::CancelProduction);
    }
    grid[slot_index(CommandSlot::RALLY_POINT)] = Some(PanelCommand::SetRallyPoint);
    grid
}
