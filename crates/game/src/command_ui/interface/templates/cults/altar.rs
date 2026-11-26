//! The Altar's panel (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`, slice M5-N1b):
//! "Augmenting is a three-stage selection: **unit type → current AugmentationState → Augmentation to apply**. Each click
//! in the third stage queues one more order; there is no separate confirm step. Z returns to the previous stage at
//! every stage." Built from the Altar's `ProductionDef::augment`, as a pool building's KindMenus are from its recipes
//! (`super::armory`):
//! - DefaultState ([`GRID`]): the unit type tiles, "**Q: Recruit**", "**W: Militant**", "**E: Gunner**", "**R:
//!   Disciple**", each with its count; "**A: Eject All**", "**X: Cancel Production**" and "**C: Set Rally Point**";
//! - AugmentMenu\[UnitType\] ([`type_menu`]): "**Q: Unaugmented**", "**W: Enhanced Constitution only**", "**E: Improved
//!   Metabolism only**", "**R: Fully augmented** — enabled when there are any (only Release is possible)", each with its
//!   count; Z back to DefaultState;
//! - AugmentMenu\[UnitType, State\] ([`state_menu`]): "Shows only the Augmentations the selected group is missing", each
//!   in its slot ("From Unaugmented: **Q: Enhanced Constitution**, **W: Improved Metabolism**, **E: Both**"; "From
//!   Enhanced Constitution only: **W: Improved Metabolism**"; "From Improved Metabolism only: **Q: Enhanced
//!   Constitution**"), and "In every state: **A: Release**" (design change 0337); Z back to AugmentMenu\[UnitType\].
//!   "Each click queues one order (CommandIssuingTransition) and stays in this menu so further clicks queue more."
//!
//! Its right-click sets the rally point (`> Right-click resolution`: "Right-click Ground: issues SetRallyPoint command to
//! that location; Right-click Object: issues SetRallyPoint command to that object"), as a producing structure's. It is
//! Ungroupable (design change 0348 point 5), so each command goes to the one Tabbed to
//! (`super::super::super::is_common`), as at the Armory.

use sc_sim::{AugmentOrder, ObjectKind, RecruitKind};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, slot_index};
use crate::input::CommandSlot;

/// The Altar's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Altar);

/// The Altar's DefaultState, slot by slot (`CommandSlot::all` order): a tile per unit type it holds in Q, W, E and R,
/// Eject All in A, X Cancel Production and C Set Rally Point (`systems/controls/interface.md > Standard Slot
/// Assignments`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use ObjectKind::{Disciple, Gunner, Militant, Recruit};
    use PanelCommand::{AugmentMenu, CancelProduction, EjectAll, SetRallyPoint};
    [
        Some(AugmentMenu(Recruit)), Some(AugmentMenu(Militant)), Some(AugmentMenu(Gunner)), Some(AugmentMenu(Disciple)),
        Some(EjectAll),             None,                        None,                      None,
        None,                       Some(CancelProduction),      Some(SetRallyPoint),       None,
    ]
};

/// "A: Release"'s slot, in every state.
const RELEASE_SLOT: usize = 4;

/// The unit types of the DefaultState's tiles, Q to R.
fn types() -> impl Iterator<Item = ObjectKind> {
    GRID.into_iter().flatten().filter_map(|command| match command {
        PanelCommand::AugmentMenu(unit) => Some(unit),
        _ => None,
    })
}

/// AugmentMenu\[`unit`\], slot by slot (`CommandSlot::all` order): a tile per AugmentationState in Q, W, E and R, in
/// the design's order (`RecruitKind::ALL`). Z is Back (`super::super::super::panel`), to DefaultState.
pub fn type_menu(unit: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    for (slot, state) in grid.iter_mut().zip(RecruitKind::ALL) {
        *slot = Some(PanelCommand::AugmentState(unit, state));
    }
    grid
}

/// The slot of `order` in an AugmentMenu\[UnitType, State\]: the same in every state, Q Enhanced Constitution, W
/// Improved Metabolism and E Both ("From Enhanced Constitution only: **W: Improved Metabolism**").
fn order_slot(order: AugmentOrder) -> usize {
    match order {
        AugmentOrder::EnhancedConstitution => 0,
        AugmentOrder::ImprovedMetabolism => 1,
        AugmentOrder::Both => 2,
    }
}

/// AugmentMenu\[`unit`, `state`\] of a `kind` Altar, slot by slot (`CommandSlot::all` order): each order of its
/// `ProductionDef::augment` that a unit in `state` may take (`AugmentOrder::fits`: "An order can only target a unit that
/// is missing every Augmentation the order applies"), in its slot; none from Fully augmented; A: Release. Z is Back, to
/// AugmentMenu\[`unit`\] (`super::super::super::InterfaceState::back`).
pub fn state_menu(kind: ObjectKind, unit: ObjectKind, state: RecruitKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
    let recipes = production.map_or(&[][..], |production| production.augment);
    for recipe in recipes.iter().filter(|recipe| recipe.order.fits(state)) {
        grid[order_slot(recipe.order)] = Some(PanelCommand::Augment(unit, state, recipe.order));
    }
    grid[RELEASE_SLOT] = Some(PanelCommand::ReleaseOf(unit, state));
    debug_assert_eq!(slot_index(CommandSlot::BACK), 8, "Z stays Back's");
    grid
}

/// The commands a `kind` Altar's AugmentMenus may show, of every type and state: the state tiles, the orders and the
/// Releases (`super::super::commands`).
pub fn menus(kind: ObjectKind) -> impl Iterator<Item = PanelCommand> {
    types().flat_map(move |unit| {
        let entries = (RecruitKind::ALL.into_iter()).flat_map(move |state| state_menu(kind, unit, state));
        type_menu(unit).into_iter().chain(entries).flatten()
    })
}
