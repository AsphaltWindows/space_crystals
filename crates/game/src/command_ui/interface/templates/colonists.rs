//! The Colonists' panels (slices M3-J1 and M3-J2, design change 0124): the Beacon's, with its two Warp In menus
//! ([`beacon`]), the Drone's ([`drone`]), the Egress Node's ([`egress_node`]), and the one of an object that transforms
//! ([`TRANSFORMING_GRID`]). The portals have the Production template (`super::production_grid`, slice M3-D), with
//! their units not in the game yet in their slots ([`robo_portal`], [`bio_portal`]: [`Planned`]).

pub mod beacon;
pub mod bio_portal;
pub mod drone;
pub mod egress_node;
pub mod robo_portal;

use sc_sim::{ObjectKind, transform};

use super::super::PanelCommand;

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
    /// The Bio Portal's E, "Psionic unit (TBD)" (`factions/colonists/structures/bio_portal.md > Bio Portal
    /// ObjectInterfaceState`).
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
    /// `crate::kinds::short_label`).
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

/// `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel` (design change 0116): "A
/// Beacon under deployment and a building under Warp In show only the Cancel command in their CommandPanel (no rally
/// point, no other commands)", in X ("Cancel's hotkey: X"): the panel of their group, the unfinished buildings of a
/// type (design change 0185: "with its own panel (e.g. Cancel only)").
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
