//! The Colonists' panels (slices M3-J1 and M3-J2, design change 0124): the Beacon's, with its two Warp In menus
//! (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`), the Drone's
//! (`factions/colonists/units/drone.md > Drone ObjectInterfaceState`), the Egress Node's
//! (`factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`), and the one of an object that
//! transforms (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`). The portals have
//! the Production template (`super::production_grid`, slice M3-D), with their units not in the game yet in their slots
//! ([`planned_units`]).

use sc_sim::{ObjectKind, transform};

use super::super::{BuildMenu, PanelCommand};

/// A building of the Beacon's Warp In menus, or a unit of a portal's panel, that isn't in the game yet: a building's
/// cost and WarpInTime are TBD (`factions/colonists/economy.md > Warp In > Constructs List`), and so is the rest of it
/// (Q0043); a unit's stats, cost and time are TBD (`factions/colonists/units/scout.md`,
/// `factions/colonists/units/lancer.md`; the psionic unit has no section yet). Each holds its slot, unavailable (design
/// change 0124: "Buildings and units whose costs are TBD show in their slot, unavailable").
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Planned {
    AlloyFurnace,
    EssenceRefinery,
    FluxReactor,
    ExophysicsEngine,
    Skylance,
    FusionCore,
    PlasmaTurret,
    RelayArray,
    NavigationSpire,
    PsionicAmplifier,
    StructuralAnalyzer,
    /// The Robo Portal's W (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`).
    Scout,
    /// The Bio Portal's W (`factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`).
    Lancer,
    /// The Bio Portal's E, "Psionic unit (TBD)" (`> Bio Portal ObjectInterfaceState`).
    PsionicUnit,
}

impl Planned {
    /// Every one, in declaration order.
    pub const ALL: [Planned; 14] = [
        Planned::AlloyFurnace,
        Planned::EssenceRefinery,
        Planned::FluxReactor,
        Planned::ExophysicsEngine,
        Planned::Skylance,
        Planned::FusionCore,
        Planned::PlasmaTurret,
        Planned::RelayArray,
        Planned::NavigationSpire,
        Planned::PsionicAmplifier,
        Planned::StructuralAnalyzer,
        Planned::Scout,
        Planned::Lancer,
        Planned::PsionicUnit,
    ];

    /// Its name, the design's.
    pub fn name(self) -> &'static str {
        match self {
            Planned::AlloyFurnace => "Alloy Furnace",
            Planned::EssenceRefinery => "Essence Refinery",
            Planned::FluxReactor => "Flux Reactor",
            Planned::ExophysicsEngine => "Exophysics Engine",
            Planned::Skylance => "Skylance",
            Planned::FusionCore => "Fusion Core",
            Planned::PlasmaTurret => "Plasma Turret",
            Planned::RelayArray => "Relay Array",
            Planned::NavigationSpire => "Navigation Spire",
            Planned::PsionicAmplifier => "Psionic Amplifier",
            Planned::StructuralAnalyzer => "Structural Analyzer",
            Planned::Scout => "Scout",
            Planned::Lancer => "Lancer",
            Planned::PsionicUnit => "Psionic unit",
        }
    }

    /// Its button's label: a placeholder for its icon (Q0001), the initials of its name, as a building warped in and a
    /// unit trained are labelled by their type's tag (`PanelCommand::label`); the Scout's its first and last letters,
    /// "ST", as the Guard's and the Beacon's are, since "SC" is the crystal patch's (slice M3-J3: every label its own,
    /// `crate::hud::selection_panel::short_label`).
    pub fn label(self) -> &'static str {
        match self {
            Planned::AlloyFurnace => "AF",
            Planned::EssenceRefinery => "ER",
            Planned::FluxReactor => "FR",
            Planned::ExophysicsEngine => "EE",
            Planned::Skylance => "SK",
            Planned::FusionCore => "FC",
            Planned::PlasmaTurret => "PT",
            Planned::RelayArray => "RA",
            Planned::NavigationSpire => "NS",
            Planned::PsionicAmplifier => "PA",
            Planned::StructuralAnalyzer => "SA",
            Planned::Scout => "ST",
            Planned::Lancer => "LA",
            Planned::PsionicUnit => "PU",
        }
    }

    /// Whether it is a unit, of a portal's panel; else a building of the Beacon's Warp In menus.
    pub fn is_unit(self) -> bool {
        matches!(self, Planned::Scout | Planned::Lancer | Planned::PsionicUnit)
    }

    /// The full name of its button (`PanelCommand::name`): the command it will be, "Train Scout" or "Warp In Alloy
    /// Furnace", as a unit in the game is trained and a building warped in.
    pub fn command_name(self) -> String {
        let verb = if self.is_unit() { "Train" } else { "Warp In" };
        format!("{verb} {}", self.name())
    }
}

/// The units of a `kind` portal's panel that aren't in the game yet ([`Planned`]), each with its slot
/// (`CommandSlot::all` order), which `super::production_grid` adds to the portal's recipes: the Robo Portal's "[W]
/// Scout" (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`), the Bio Portal's "[W]
/// Lancer" and "[E] Psionic unit (TBD)" (`factions/colonists/structures/bio_portal.md > Bio Portal
/// ObjectInterfaceState`); design change 0124: "Robo Portal: Q Drone, W Scout, X, C. Bio Portal: Q Sabre, W Lancer, E
/// psionic unit, X, C".
pub(super) fn planned_units(kind: ObjectKind) -> &'static [(usize, Planned)] {
    match kind {
        ObjectKind::RoboPortal => &[(1, Planned::Scout)],
        ObjectKind::BioPortal => &[(1, Planned::Lancer), (2, Planned::PsionicUnit)],
        _ => &[],
    }
}

/// `> Beacon ObjectInterfaceState > DefaultState`: "Q: Warp In (Basic)", "W: Warp In (Advanced)", the two menus
/// "like the Deployment Center's two build menus" (`PanelCommand::BuildMenu`, opened in `InterfaceState::BuildMenu`).
#[rustfmt::skip]
pub(super) const BEACON_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::BuildMenu as Menu;
    [
        Some(Menu(BuildMenu::Basic)), Some(Menu(BuildMenu::Advanced)), None, None,
        None,                         None,                            None, None,
        None,                         None,                            None, None,
    ]
};

/// `> Beacon ObjectInterfaceState > Warp In (Basic)`, "economy and production": Q Egress Node, W Robo Portal, E Bio
/// Portal, the buildings in the game, and A Alloy Furnace, S Essence Refinery, D Flux Reactor, which aren't yet; Z is
/// Back (`super::super::panel`).
#[rustfmt::skip]
const WARP_IN_BASIC: [Option<PanelCommand>; 12] = {
    use ObjectKind::{BioPortal, EgressNode, RoboPortal};
    use PanelCommand::{Planned as P, WarpIn as W};
    use Planned::{AlloyFurnace, EssenceRefinery, FluxReactor};
    [
        Some(W(EgressNode)),   Some(W(RoboPortal)),      Some(W(BioPortal)),   None,
        Some(P(AlloyFurnace)), Some(P(EssenceRefinery)), Some(P(FluxReactor)), None,
        None,                  None,                     None,                 None,
    ]
};

/// `> Beacon ObjectInterfaceState > Warp In (Advanced)`, "tech and defense (each defense next to the building it
/// requires)": Q Exophysics Engine, W Skylance, E Fusion Core, R Plasma Turret, A Relay Array, S Navigation Spire, D
/// Psionic Amplifier, F Structural Analyzer, none of which is in the game yet; Z is Back.
#[rustfmt::skip]
const WARP_IN_ADVANCED: [Option<PanelCommand>; 12] = {
    use PanelCommand::Planned as P;
    use Planned::{
        ExophysicsEngine, FusionCore, NavigationSpire, PlasmaTurret, PsionicAmplifier, RelayArray, Skylance,
        StructuralAnalyzer,
    };
    [
        Some(P(ExophysicsEngine)), Some(P(Skylance)),        Some(P(FusionCore)),       Some(P(PlasmaTurret)),
        Some(P(RelayArray)),       Some(P(NavigationSpire)), Some(P(PsionicAmplifier)), Some(P(StructuralAnalyzer)),
        None,                      None,                     None,                      None,
    ]
};

/// A Beacon's Warp In menu `menu`, slot by slot: every building of the design's menu shows, the ones not in the game
/// unavailable (`PanelCommand::Planned`).
pub(super) fn warp_in_menu(menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    match menu {
        BuildMenu::Basic => WARP_IN_BASIC,
        BuildMenu::Advanced => WARP_IN_ADVANCED,
    }
}

/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > DefaultState commands` ("No Hold Position, Patrol
/// or Attack"): Q Move and X Stop, "as in BasicCombatUnitInterfaceState", A Deploy, and S Set Drop-off, which awaits
/// its drop-off (AwaitingTarget\[SetDropOffLocation\]; design change 0124: "Drone: Q Move, A Deploy, S Set Drop-off, X
/// Stop").
#[rustfmt::skip]
pub(super) const DRONE_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Deploy, Move, SetDropOff, Stop};
    [
        Some(Move),   None,             None, None,
        Some(Deploy), Some(SetDropOff), None, None,
        None,         Some(Stop),       None, None,
    ]
};

/// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`: "F: Select Returning Drones"
/// (design change 0124: "Egress Node: F"), its one command.
#[rustfmt::skip]
pub(super) const EGRESS_NODE_GRID: [Option<PanelCommand>; 12] = [
    None, None, None, None,
    None, None, None, Some(PanelCommand::SelectReturningDrones),
    None, None, None, None,
];

/// `> While transforming: only Cancel` (design change 0116): "A Beacon under deployment and a building under Warp In
/// show only the Cancel command in their CommandPanel (no rally point, no other commands)", in X ("Cancel's hotkey:
/// X"): the panel of their group, the unfinished buildings of a type (design change 0185: "with its own panel (e.g.
/// Cancel only)").
#[rustfmt::skip]
pub const TRANSFORMING_GRID: [Option<PanelCommand>; 12] = [
    None, None,                                None, None,
    None, None,                                None, None,
    None, Some(PanelCommand::CancelTransform), None, None,
];

/// Whether objects of `kind` warp buildings in: the Beacon, with its Constructs List (`StructureDef::warp_in`).
pub(super) fn warps_in(kind: ObjectKind) -> bool {
    !transform::warp_in_recipes(kind).is_empty()
}
