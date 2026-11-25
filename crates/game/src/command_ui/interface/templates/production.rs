//! The Production template's grid (plan decision T23): a producing structure's panel, built from its `ProductionDef`,
//! with the bottom row of `systems/controls/selection_and_hud.md > CommandPanel > Standard Slot Assignments`.

use sc_sim::ObjectKind;

use super::{PanelCommand, slot_index};
use crate::input::CommandSlot;
use crate::kinds;

/// How many units a producing structure's panel trains: one in each of Q, W, E, R, A, S, D and F; the bottom row is
/// `> Standard Slot Assignments`' (Z Back, X Cancel Production, C Set Rally Point).
const TRAIN_SLOTS: usize = 8;

/// A producing structure's DefaultState commands (plan decision T23): Train each of the units of its `ProductionDef`,
/// in the design's order, from Q on (Q, W, E, R, then A, S, D, F); X Cancel Production and C Set Rally Point
/// (`> Standard Slot Assignments`); Z is Back while a target is awaited.
///
/// - The Barracks: Q Build Peacekeeper, X, C (`factions/gdo/structures/barracks.md >
///   ObjectInterfaceState[Barracks]`). "W: Build Rocketman (hotkey tentative)" stays empty: the Rocketman isn't in
///   the game yet (its cost and time are TBD, and so is its Arsenal; M2 plan § 1.2), so it isn't among the recipes
///   and its button is hidden.
/// - The Robo Portal: Q Drone, W Scout, X, C; the Bio Portal: Q Sabre Legionnaire, W Lancer, E psionic unit, X, C
///   (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`,
///   `factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`, design change 0124). The Scout,
///   the Lancer and the psionic unit aren't in the game (their stats, cost and time are TBD), so they aren't among the
///   recipes: 0124 has them show in their slot, unavailable ([`PanelCommand::Planned`], [`PanelTemplate::planned`];
///   slice M3-J2).
/// - The Recruitment Hall: X and C alone (`factions/cults/economy.md > RecruitmentHall >
///   ObjectInterfaceState[RecruitmentHall]`, slice M5-C). It recruits by itself (`ProductionDef::recruitment`), so its
///   recipe, the Recruit, has no Train; its X is available only while a Recruit is in production ("Only available if
///   ProductionProgress is not None": the gate's `Reject::NothingQueued` otherwise).
pub(super) fn production_grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
    let trains = production.filter(|production| production.recruitment.is_none());
    let recipes = trains.map_or(&[][..], |production| production.recipes);
    debug_assert!(
        recipes.len() <= TRAIN_SLOTS,
        "{kind:?} trains more units than its panel has slots"
    );
    for (slot, recipe) in grid.iter_mut().zip(recipes.iter().take(TRAIN_SLOTS)) {
        *slot = Some(PanelCommand::Train(recipe.kind));
    }
    for &(slot, planned) in kinds::of(kind).template.planned {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    grid[slot_index(CommandSlot::CANCEL_PRODUCTION)] = Some(PanelCommand::CancelProduction);
    grid[slot_index(CommandSlot::RALLY_POINT)] = Some(PanelCommand::SetRallyPoint);
    grid
}
