//! What a type offers (`systems/controls/selection_and_hud.md > ObjectInterfaceState[ObjectEnum]`): every command its
//! panel may show ([`commands`]), and whether it offers a command ([`offers`]).

use sc_sim::{construction, transform};

use super::{PanelCommand, can_target_ground, constructs, grid, menu_grid};
use crate::selection::GroupType;

/// Every command objects of type `ty` (a kind: a finished one) may show, in any state ([`state_grid`]): an
/// unfinished building's Cancel alone ([`TRANSFORMING_GRID`], design change 0185); a finished type's DefaultState
/// grid, and a constructor's buildings to construct and to place, and its two Cancels; a Beacon's buildings to warp
/// in. In order, each once.
pub fn commands(ty: impl Into<GroupType>) -> Vec<PanelCommand> {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return vec![PanelCommand::CancelTransform];
    }
    let mut commands: Vec<PanelCommand> = grid(kind).into_iter().flatten().collect();
    commands.extend(
        transform::warp_in_recipes(kind)
            .iter()
            .map(|r| PanelCommand::WarpIn(r.kind)),
    );
    if constructs(kind) {
        for recipe in construction::recipes(kind) {
            commands.extend([
                PanelCommand::Construct(recipe.kind),
                PanelCommand::PlaceReady(recipe.kind),
            ]);
        }
        commands.extend([PanelCommand::CancelConstruction, PanelCommand::CancelReady]);
    }
    commands.sort_unstable();
    commands.dedup();
    commands
}

/// Whether objects of type `ty` (a kind: a finished one) offer `command`. An unfinished building, in a group of its
/// own "as if it were another type" (design change 0185), offers its Cancel and nothing else (`factions/colonists/
/// economy.md > Beacon Deployment > While transforming: only Cancel`, [`TRANSFORMING_GRID`]). A finished one offers
/// what its grid has ([`grid`]), and a conditional command only when the type can (`> BasicCombatUnitInterfaceState`:
/// Reverse with the UnitBase's CanReverse, AttackGround with the AttackType's CanTargetGround). A constructor offers
/// its own buildings (its `Constructs` list) and its Cancels, a Beacon the buildings of its Constructs List; a
/// building or a unit that isn't in the game is offered by no one (design change 0124: "shown but unavailable"). A
/// build menu is offered while it shows something, so the Deployment Center's W, Advanced Construction, shows
/// unavailable: none of its buildings exists yet, and they are hidden (design change 0104: "greyed placeholders"); the
/// Beacon's Warp In (Advanced) opens, on its unavailable buildings.
pub fn offers(ty: impl Into<GroupType>, command: PanelCommand) -> bool {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return command == PanelCommand::CancelTransform;
    }
    match command {
        PanelCommand::Construct(building) | PanelCommand::PlaceReady(building) => {
            return construction::recipe(kind, building).is_some();
        }
        PanelCommand::CancelConstruction | PanelCommand::CancelReady => return constructs(kind),
        PanelCommand::BuildMenu(menu) => {
            return grid(kind).contains(&Some(command)) && menu_grid(kind, menu).iter().any(Option::is_some);
        }
        PanelCommand::WarpIn(building) => return transform::warp_in_recipe(kind, building).is_some(),
        PanelCommand::Planned(_) | PanelCommand::CancelTransform => return false,
        _ => {}
    }
    if !grid(kind).contains(&Some(command)) {
        return false;
    }
    let unit = kind.def().unit.as_ref();
    match command {
        PanelCommand::Reverse => unit.is_some_and(|unit| unit.base_def().can_reverse),
        PanelCommand::AttackGround => unit
            .and_then(|unit| unit.attack)
            .is_some_and(|attack| can_target_ground(attack.attack_type)),
        _ => true,
    }
}
