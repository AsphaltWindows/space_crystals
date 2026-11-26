//! What a type offers (`systems/controls/interface.md > ObjectInterfaceState[ObjectEnum]`): every command its
//! panel may show ([`commands`]), and whether it offers a command ([`offers`]).

use sc_sim::{RecruitKind, ResearchDef, construction, transform};

use super::{BuildMenu, PanelCommand, can_target_ground, constructs, cults, grid, menu_grid, place_slot, syndicate};
use crate::selection::GroupType;

/// The commands of an unfinished building, the Cancels of its two panels ([`TRANSFORMING_GRID`], [`SITE_GRID`]), in
/// order.
const UNFINISHED: [PanelCommand; 2] = [PanelCommand::CancelConstruction, PanelCommand::CancelTransform];

/// Every command objects of type `ty` (a kind: a finished one) may show, in any state ([`state_grid`]): an
/// unfinished building's Cancel alone, the Cancel of a transformation ([`TRANSFORMING_GRID`], design change 0185) or
/// of a site's construction ([`SITE_GRID`], design change 0237), whichever it is; a finished type's DefaultState
/// grid, and a constructor's buildings to construct and to place (from its Place slot too, the Deployment Center's E:
/// design change 0388), and its two Cancels; a Beacon's buildings to warp
/// in; a Tunnel's EjectMenu tiles and the expansions of its ExpandMenu; a pool building's KindMenus' entries, every
/// kind's; an Altar's AugmentMenus' tiles and entries, every type's and state's (slice M5-N1b); a Recruit's buildings
/// of its Construct menus (slice M5-J1: [`menu_builds`]); a research structure's keys, each of its technologies' and
/// the same in progress (slice M5-N3, `PanelCommand::Researching`). In order, each once.
pub fn commands(ty: impl Into<GroupType>) -> Vec<PanelCommand> {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return UNFINISHED.to_vec();
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
            commands.extend(place_slot(kind).map(|_| PanelCommand::Place(recipe.kind)));
        }
        commands.extend([PanelCommand::CancelConstruction, PanelCommand::CancelReady]);
    }
    if ejects(kind) {
        commands.extend(
            syndicate::eject_menu::kinds()
                .iter()
                .map(|&unit| PanelCommand::Eject(unit)),
        );
    }
    if expands(kind) {
        commands.extend(syndicate::expand_menu::kinds().map(PanelCommand::Expand));
    }
    if pools(kind) {
        commands.extend(cults::armory::kind_menus(kind));
    }
    if augments(kind) {
        commands.extend(cults::altar::menus(kind));
    }
    commands.extend(menu_builds(kind));
    if let Some(research) = research_of(kind) {
        for &tech in research.techs {
            commands.extend([PanelCommand::Research(tech), PanelCommand::Researching(tech)]);
        }
    }
    commands.sort_unstable();
    commands.dedup();
    commands
}

/// Whether objects of type `ty` (a kind: a finished one) offer `command`. An unfinished building, in a group of its
/// own "as if it were another type" (design change 0185), offers its Cancel and nothing else (`factions/colonists/
/// economy.md > Beacon Deployment > While transforming: only Cancel`, [`TRANSFORMING_GRID`]; a site, design change
/// 0237, its Cancel Construction, [`SITE_GRID`]: the type doesn't say which, the panel shows the one it is). A
/// finished one offers
/// what its grid has ([`grid`]), and a conditional command only when the type can (`> BasicCombatUnitInterfaceState`:
/// Reverse with the UnitBase's CanReverse, AttackGround with the AttackType's CanTargetGround). A constructor offers
/// its own buildings (its `Constructs` list), from its Place slot too if it has one (the Deployment Center's E:
/// `super::place_slot`), and its Cancels, a Beacon the buildings of its Constructs List; a
/// building or a unit that isn't in the game is offered by no one (design change 0124: "shown but unavailable"). A
/// build menu is offered while it shows something, so the Deployment Center's W, Advanced Construction, shows
/// unavailable: none of its buildings exists yet, and they are hidden (design change 0104: "greyed placeholders"); the
/// Beacon's Warp In (Advanced) opens, on its unavailable buildings. A type whose DefaultState opens the EjectMenu, the
/// Tunnel, offers its tiles, and one that opens the ExpandMenu, the Tunnel, the expansions in the game
/// (`sc_sim::defs::EXPAND_MENU`), each a type a Tunnel builds (`StructureDef::expansion`). A pool building offers its
/// KindMenus' trainings and Releases (slice M5-F2), an Altar its AugmentMenus' tiles, orders and Releases (slice
/// M5-N1b). A worker offers the Build its grid has (the Agent's) and those of the build menus it opens (the Recruit's
/// Construct menus, slice M5-J1: [`menu_builds`]). A research structure offers the Research of each technology it
/// researches (`StructureDef::research`), its keys showing their line's next level, and the same in progress (slice
/// M5-N3).
pub fn offers(ty: impl Into<GroupType>, command: PanelCommand) -> bool {
    let GroupType { kind, unfinished } = ty.into();
    if unfinished {
        return UNFINISHED.contains(&command);
    }
    match command {
        PanelCommand::Construct(building) | PanelCommand::PlaceReady(building) => {
            return construction::recipe(kind, building).is_some();
        }
        PanelCommand::Place(building) => {
            return place_slot(kind).is_some() && construction::recipe(kind, building).is_some();
        }
        PanelCommand::CancelConstruction | PanelCommand::CancelReady => return constructs(kind),
        PanelCommand::BuildMenu(menu) => {
            return grid(kind).contains(&Some(command)) && menu_grid(kind, menu).iter().any(Option::is_some);
        }
        PanelCommand::WarpIn(building) => return transform::warp_in_recipe(kind, building).is_some(),
        PanelCommand::Eject(unit) => return ejects(kind) && syndicate::eject_menu::kinds().contains(&unit),
        PanelCommand::Expand(building) => {
            return expands(kind) && syndicate::expand_menu::kinds().any(|kind| kind == building);
        }
        PanelCommand::Planned(_) | PanelCommand::CancelTransform => return false,
        PanelCommand::TrainFrom(..) | PanelCommand::Release(_) => {
            return pools(kind) && cults::armory::kind_menus(kind).any(|entry| entry == command);
        }
        PanelCommand::AugmentState(..) | PanelCommand::Augment(..) | PanelCommand::ReleaseOf(..) => {
            return augments(kind) && cults::altar::menus(kind).any(|entry| entry == command);
        }
        PanelCommand::Build(_) if menu_builds(kind).any(|build| build == command) => return true,
        PanelCommand::Research(tech) | PanelCommand::Researching(tech) => {
            return research_of(kind).is_some_and(|research| research.offers(tech));
        }
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

/// The Builds of the build menus objects of `kind` open, in their slots' order: a Recruit's Construct menus' buildings
/// in the game (slice M5-J1, `super::cults::construct_menu`). None for another type: the Deployment Center's menus
/// construct, a Beacon's warp in.
fn menu_builds(kind: sc_sim::ObjectKind) -> impl Iterator<Item = PanelCommand> {
    let opens = grid(kind);
    BuildMenu::ALL
        .into_iter()
        .filter(move |&menu| opens.contains(&Some(PanelCommand::BuildMenu(menu))))
        .flat_map(move |menu| menu_grid(kind, menu).into_iter().flatten())
        .filter(|command| matches!(command, PanelCommand::Build(_)))
}

/// (M5-N3) What objects of `kind` research (`StructureDef::research`): the Temple's technologies. `None` for a type
/// that researches nothing.
fn research_of(kind: sc_sim::ObjectKind) -> Option<&'static ResearchDef> {
    kind.def().structure.as_ref()?.research.as_ref()
}

/// Whether objects of `kind` open the EjectMenu: the Tunnel (slice M4-J2).
fn ejects(kind: sc_sim::ObjectKind) -> bool {
    grid(kind).contains(&Some(PanelCommand::EjectMenu))
}

/// Whether objects of `kind` open the ExpandMenu: the Tunnel (slice M4-F).
fn expands(kind: sc_sim::ObjectKind) -> bool {
    grid(kind).contains(&Some(PanelCommand::ExpandMenu))
}

/// Whether objects of `kind` open the KindMenus: the pool buildings, the Armory (slice M5-F2).
fn pools(kind: sc_sim::ObjectKind) -> bool {
    grid(kind).contains(&Some(PanelCommand::KindMenu(RecruitKind::Plain)))
}

/// Whether objects of `kind` open the AugmentMenus: the Altar (slice M5-N1b).
fn augments(kind: sc_sim::ObjectKind) -> bool {
    grid(kind)
        .iter()
        .flatten()
        .any(|command| matches!(command, PanelCommand::AugmentMenu(_)))
}
