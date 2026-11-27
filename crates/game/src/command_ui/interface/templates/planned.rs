//! The panel entries that aren't in the game yet ([`Planned`]): each holds its slot, unavailable, and says why when
//! it is pressed (design change 0162; `super::super::Conditions::planned`). The Colonists' buildings and units whose
//! numbers are TBD (design change 0124: "Buildings and units whose costs are TBD show in their slot, unavailable";
//! slices M3-J1 and M3-J2), and the Syndicate's buildings that later slices bring (design change 0257: "unavailable
//! entries are greyed"; slice M4-F): the ExpandMenu's expansions not in the game yet. The slice
//! that brings one replaces its entry with the real command, as slice M4-K did the Hideout's Guard and the
//! ExpandMenu's Optics Lab and Chemical Plant, slice M4-G the Tunnel's Upgrade
//! (`super::super::PanelCommand::UpgradeTunnel`), slice M4-U the Hideout's Grenadier, slice M4-S1 the Agent's Laser
//! Turret (`super::super::PanelCommand::Build`, gone with design change 0359) and slice M4-S2 the ExpandMenu's
//! (`super::super::PanelCommand::Expand`).
//! The Cults' buildings of the Recruits' Construct menus that the Recruits don't build yet hold their slots (slice
//! M5-J1, design change 0327: "every building greyed until available, including ones not in the build yet"), each
//! until its slice; slice M5-G2 made the Gunworks, the Machine Shop and the Scrap Hangar live, and the Armory's Gunner,
//! which had held its slot of the Armory's kind menus (slice M5-F2) until its Gunworks; slice M5-N2 the Garrison;
//! slice M5-N1b the Altar; slice M5-N4a the Cathedral; slice M5-N3 the Temple; slice M5-N5 the Shrine; slice M5-N7 the
//! Airstrip; slice M5-N6 the Chop Shop, the last of them, so the Construct menus hold no placeholder any more
//! (`super::cults::construct_menu`). The Recruit's Repair and the Agent's held theirs until slice M5-N8
//! (`super::super::PanelCommand::Repair`).
//! The Field Engineer's Construct menu holds its two buildings' slots (slice M5-N9,
//! `factions/gdo/units/field_engineer.md > Placeholder stats`: "both greyed until designed"). The Disciple's R held its
//! spell's, Shroud, from slice M5-N4a until slice M5-N4b (`super::super::PanelCommand::Shroud`).

/// A panel entry that isn't in the game yet (module docs). The Colonists' are a building of the Beacon's Warp In menus
/// or a unit of a portal's panel: a building's cost and WarpInTime are TBD (`factions/colonists/economy.md > Warp In >
/// Constructs List`), and so is the rest of it (Q0043); a unit's stats, cost and time are TBD
/// (`factions/colonists/units/scout.md`, `factions/colonists/units/lancer.md`; the psionic unit has no section yet).
/// The Syndicate's are the ExpandMenu's entries that aren't in the game yet (slice M4-F:
/// `sc_sim::defs::ExpandEntry::Planned`, by name): its expansions. The Cults' were the Construct menus' buildings the
/// Recruits didn't build yet (slice M5-J1: `sc_sim::defs::ConstructEntry::Planned`, by name), until slice M5-N6.
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
    /// (M5-N9) The Field Engineer's Construct menu's buildings (`factions/gdo/units/field_engineer.md > Placeholder
    /// stats`: "**A** Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z** back)"),
    /// in its order (`super::gdo::field_engineer::CONSTRUCT_MENU`).
    Bunker,
    FirstAidTent,
}

impl Planned {
    /// Every one, in declaration order.
    pub const ALL: [Planned; 22] = [
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
        Planned::Bunker,
        Planned::FirstAidTent,
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
            Planned::Bunker => "Bunker",
            Planned::FirstAidTent => "First Aid Tent",
        }
    }

    /// Its button's label: a placeholder for its icon (Q0001), the initials of its name, as a building warped in and a
    /// unit trained are labelled by their type's tag (`PanelCommand::label`); the Scout's its first and last letters,
    /// "ST", as the Guard's and the Beacon's are, since "SC" is the crystal patch's (slice M3-J3: every label its own,
    /// `crate::kinds::short_label`). An expansion's is its initials, as a building's, a one-word name's its first and
    /// last letters, as the Scout's ("VT" Vault, "FE" Forge). A Cults building's was alike (the Chop Shop's "CS" until
    /// slice M5-N6, the Airstrip's "AP" until slice M5-N7; the Shrine's was "SR", "SE" being the Storage's, until slice
    /// M5-N5). The Field
    /// Engineer's: "FAT" the First Aid Tent, and "BKR" the Bunker, as the Seeker's "SKR", its first and last letters,
    /// "BR", being the Barracks'. An entry's type takes
    /// its label with it into the game (the Optics Lab's "OL", the Chemical Plant's "CP", the Grenadier's "GR", the
    /// Gunworks' "GS", the Garrison's "GA", `crate::kinds`), and a command its own: the Tunnel's Upgrade's "Upgr."
    /// (slice M4-G).
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
            Planned::Bunker => "BKR",
            Planned::FirstAidTent => "FAT",
        }
    }

    /// Whether it is a unit, of a production panel; else a building, of the Beacon's Warp In menus, the Tunnel's
    /// ExpandMenu or the Field Engineer's Construct menu.
    pub fn is_unit(self) -> bool {
        matches!(self, Planned::Scout | Planned::Lancer | Planned::PsionicUnit)
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
    /// (`PanelCommand::Expand`). The Field Engineer's as the design names its abilities
    /// (`factions/gdo/units/field_engineer.md > Abilities`): "**Build Bunker**", "**Set up First Aid Tent**".
    pub fn command_name(self) -> String {
        if self == Planned::FirstAidTent {
            format!("Set up {}", self.name())
        } else if self == Planned::Bunker {
            format!("Build {}", self.name())
        } else if self.is_unit() {
            format!("Train {}", self.name())
        } else if self.is_expansion() {
            format!("Build {}", self.name())
        } else {
            format!("Warp In {}", self.name())
        }
    }
}
