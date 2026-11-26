//! The Armory's panel (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`, slice M5-F2), the pool
//! buildings' (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`:
//! "**Pool buildings** (Armory, Cathedral, Chop Shop, Airstrip)"), built from the building's `ProductionDef` as the
//! Production template is (plan decision T23), so that the next pool building's is its data:
//! - DefaultState ([`GRID`]): "**Q** Plain, **W** Constitution, **E** Metabolism, **R** Both (each enters
//!   KindMenu\[kind\]); **A** Eject All; **C** Set Rally Point; **X** Cancel Production";
//! - KindMenu\[kind\] ([`kind_menu`]): "the building's production options, using Recruits of that kind (e.g. Armory:
//!   **Q** Train Militant, **W** Train Gunner [...]), plus **A: Release**; **Z** back. Clicks stay in the menu so they
//!   can be repeated." The Armory's are its two recipes', Q Train Militant and W Train Gunner (slice M5-G2), the
//!   Gunner's greyed while its owner has no Gunworks, as the gate refuses it (`sc_sim::Reject::Requires`). A unit not in
//!   the game would hold its slot ([`PanelTemplate::planned`]), as the Gunner's did until slice M5-G2.
//!
//! Its right-click sets the rally point (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory] >
//! Right-click resolution`: "Right-click Ground: issues SetRallyPoint command to that location; Right-click Object:
//! issues SetRallyPoint command to that object"), as a producing structure's. A pool building is Ungroupable (design
//! change 0348 point 5, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "a command
//! goes to one building"), so each command goes to the one Tabbed to (`super::super::super::is_common`).

use sc_sim::{ObjectKind, RecruitKind};

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, slot_index};
use crate::input::CommandSlot;
use crate::kinds;

/// The Armory's panel template: the pool buildings', its KindMenus its recipes'.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::PoolProducer);

/// A pool building's DefaultState, slot by slot (`CommandSlot::all` order): the four kinds in Q, W, E and R, Eject
/// All in A, X Cancel Production and C Set Rally Point (`systems/controls/interface.md > Standard Slot
/// Assignments`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, SetRallyPoint};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    [
        Some(KindMenu(Plain)), Some(KindMenu(Constitution)), Some(KindMenu(Metabolism)), Some(KindMenu(Both)),
        Some(EjectAll),        None,                         None,                       None,
        None,                  Some(CancelProduction),       Some(SetRallyPoint),        None,
    ]
};

/// The slots of a KindMenu's production options, in order: the top two rows but A, which is Release's.
const TRAIN_SLOTS: [usize; 7] = [0, 1, 2, 3, 5, 6, 7];

/// "A: Release"'s slot.
const RELEASE_SLOT: usize = 4;

/// The KindMenu\[`from`\] of a `kind` pool building, slot by slot (`CommandSlot::all` order): a training of each unit
/// of its `ProductionDef`, of Recruits of kind `from`, in the design's order from Q on; its units not in the game yet in
/// their slots ([`PanelTemplate::planned`]); A: Release. Z is Back (`super::super::super::panel`), to DefaultState.
pub fn kind_menu(kind: ObjectKind, from: RecruitKind) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
    let recipes = production.map_or(&[][..], |production| production.recipes);
    debug_assert!(
        recipes.len() <= TRAIN_SLOTS.len(),
        "{kind:?} trains more units than its KindMenu has slots"
    );
    for (&slot, recipe) in TRAIN_SLOTS.iter().zip(recipes) {
        grid[slot] = Some(PanelCommand::TrainFrom(recipe.kind, from));
    }
    for &(slot, planned) in kinds::of(kind).template.planned {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    grid[RELEASE_SLOT] = Some(PanelCommand::Release(from));
    debug_assert_eq!(slot_index(CommandSlot::BACK), 8, "Z stays Back's");
    grid
}

/// The commands a `kind` pool building's KindMenus may show, every kind's: its trainings and its Releases
/// (`super::super::commands`).
pub fn kind_menus(kind: ObjectKind) -> impl Iterator<Item = PanelCommand> {
    RecruitKind::ALL
        .into_iter()
        .flat_map(move |from| kind_menu(kind, from).into_iter().flatten())
}
