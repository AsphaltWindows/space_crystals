//! The panel entries that aren't in the game yet ([`Planned`]): each holds its slot, unavailable, and says why when
//! it is pressed (design change 0162; `super::super::Conditions::planned`). The Colonists' buildings and units whose
//! numbers are TBD (design change 0124: "Buildings and units whose costs are TBD show in their slot, unavailable";
//! slices M3-J1 and M3-J2), and the Syndicate's buildings that later slices bring (design change 0257: "unavailable
//! entries are greyed"; slice M4-F): the ExpandMenu's expansions not in the game yet. The slice
//! that brings one replaces its entry with the real command, as slice M4-K did the Hideout's Guard and the
//! ExpandMenu's Optics Lab and Chemical Plant, slice M4-G the Tunnel's Upgrade
//! (`super::super::PanelCommand::UpgradeTunnel`), slice M4-U the Hideout's Grenadier, slice M4-S1 the Agent's Laser
//! Turret (`super::super::PanelCommand::Build`) and slice M4-S2 the ExpandMenu's (`super::super::PanelCommand::Expand`).
//! The Cults' Armory trains its Gunner once the Gunworks it requires is in the game (slice M5-G2): until then the
//! Gunner holds its slot of the Armory's kind menus (slice M5-F2).

/// A panel entry that isn't in the game yet (module docs). The Colonists' are a building of the Beacon's Warp In menus
/// or a unit of a portal's panel: a building's cost and WarpInTime are TBD (`factions/colonists/economy.md > Warp In >
/// Constructs List`), and so is the rest of it (Q0043); a unit's stats, cost and time are TBD
/// (`factions/colonists/units/scout.md`, `factions/colonists/units/lancer.md`; the psionic unit has no section yet).
/// The Syndicate's are the ExpandMenu's entries that aren't in the game yet (slice M4-F:
/// `sc_sim::defs::ExpandEntry::Planned`, by name): its expansions.
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
    /// The ExpandMenu's expansions not in the game yet (`factions/syndicate/structures/README.md > Tech Tree
    /// Overview`; design change 0291: "**all nine expansions** are always listed, in the Tech Tree table's order, each
    /// holding its slot"), in the table's order: the sim's `sc_sim::defs::EXPAND_MENU` names them
    /// ([`Planned::expansion`]). The Optics Lab and the Chemical Plant, which were the first two, are in the game
    /// (slice M4-K).
    Vault,
    VehicleAssembly,
    Forge,
    NullChamber,
    AerospaceAssembly,
    OrbitalCommand,
    /// The Armory's KindMenu\[kind\] "**W** Train Gunner" (design change 0337, `factions/cults/structures/README.md >
    /// Choosing Recruits, Release and Eject All`): "TechPrerequisite: player owns at least one Gunworks"
    /// (`factions/cults/structures/armory.md > Training`), a building that isn't in the game yet, so the Armory's
    /// recipes leave the Gunner out (slice M5-F2), and its slot says what it requires (`super::super::Conditions::planned`).
    Gunner,
}

impl Planned {
    /// Every one, in declaration order.
    pub const ALL: [Planned; 21] = [
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
        Planned::Vault,
        Planned::VehicleAssembly,
        Planned::Forge,
        Planned::NullChamber,
        Planned::AerospaceAssembly,
        Planned::OrbitalCommand,
        Planned::Gunner,
    ];

    /// The ExpandMenu's entry not in the game yet that the sim names `name` (`sc_sim::defs::ExpandEntry::Planned`): an
    /// expansion of the Tech Tree; `None` for any other name.
    pub fn expansion(name: &str) -> Option<Planned> {
        Planned::ALL
            .into_iter()
            .find(|&planned| planned.is_expansion() && planned.name() == name)
    }

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
            Planned::Vault => "Vault",
            Planned::VehicleAssembly => "Vehicle Assembly",
            Planned::Forge => "Forge",
            Planned::NullChamber => "Null Chamber",
            Planned::AerospaceAssembly => "Aerospace Assembly",
            Planned::OrbitalCommand => "Orbital Command",
            Planned::Gunner => "Gunner",
        }
    }

    /// Its button's label: a placeholder for its icon (Q0001), the initials of its name, as a building warped in and a
    /// unit trained are labelled by their type's tag (`PanelCommand::label`); the Scout's its first and last letters,
    /// "ST", as the Guard's and the Beacon's are, since "SC" is the crystal patch's (slice M3-J3: every label its own,
    /// `crate::kinds::short_label`). An expansion's is its initials, as a building's, a one-word name's its first and last letters, as the Scout's ("VT" Vault, "FE" Forge). An entry's
    /// type takes its label with it into the game (the Optics Lab's "OL", the Chemical Plant's "CP" and the Grenadier's
    /// "GR", `crate::kinds`), and a command its own: the Tunnel's Upgrade's "Upgr." (slice M4-G).
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
            Planned::Vault => "VT",
            Planned::VehicleAssembly => "VA",
            Planned::Forge => "FE",
            Planned::NullChamber => "NC",
            Planned::AerospaceAssembly => "AA",
            Planned::OrbitalCommand => "OC",
            // The Gunner's own tag (`crate::kinds::cults::gunner`), which it takes into the Armory's panel.
            Planned::Gunner => "GU",
        }
    }

    /// Whether it is a unit, of a production panel; else a building, of the Beacon's Warp In menus or the Tunnel's
    /// ExpandMenu.
    pub fn is_unit(self) -> bool {
        matches!(
            self,
            Planned::Scout | Planned::Lancer | Planned::PsionicUnit | Planned::Gunner
        )
    }

    /// Whether it is an expansion of the Tunnel's ExpandMenu.
    pub fn is_expansion(self) -> bool {
        matches!(
            self,
            Planned::Vault
                | Planned::VehicleAssembly
                | Planned::Forge
                | Planned::NullChamber
                | Planned::AerospaceAssembly
                | Planned::OrbitalCommand
        )
    }

    /// The full name of its button (`PanelCommand::name`): the command it will be, "Train Scout", "Warp In Alloy
    /// Furnace" or "Build Vault", as a unit in the game is trained, a building warped in and an expansion built
    /// (`PanelCommand::Expand`).
    pub fn command_name(self) -> String {
        if self.is_unit() {
            format!("Train {}", self.name())
        } else if self.is_expansion() {
            format!("Build {}", self.name())
        } else {
            format!("Warp In {}", self.name())
        }
    }
}
