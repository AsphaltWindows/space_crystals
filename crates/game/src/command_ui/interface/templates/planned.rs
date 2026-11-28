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
//! Slice M6-SV made the ExpandMenu's Vehicle Assembly live (`super::super::PanelCommand::Expand`), and its panel holds
//! the three units that later slices have it train, W, E and R (`super::syndicate::vehicle_assembly`), each until its
//! slice: M6-SB the Shadebringer, M6-SL the Laser Battery, M6-ST1 the Subterranean Transport. Slice M6-SD made the
//! Forge and the Aerospace Assembly live, and the Aerospace Assembly's panel holds the two units that later slices have
//! it train, W and E (`super::syndicate::aerospace_assembly`): M6-SA the Specter, M6-SP the Prism. Slice M6-SN made the
//! Null Chamber live, and slice M6-SB the Vault and the Vehicle Assembly's Shadebringer; the Shadebringer's F holds
//! its Cover's slot until slice M6-SC (`super::syndicate::shadebringer`).
//! (M6-GK) The Deployment Center's build menus hold the slots of the eight buildings it doesn't construct yet
//! (`factions/gdo/structures/deployment_center.md > Two build menus`), each in the game but greyed until its slice makes
//! its entry live (`super::gdo::deployment_center`): slice M6-GR made the Radar Station and the Defense System live,
//! slice M6-GF the Factory, Advanced Mechanics and the Industrial Powerplant, slice M6-GH2 the Airfield, slice M6-GL
//! Propulsion Labs and the Airfield's W, the Dragonfly (`super::gdo::airfield`), and the Factory's panel held the two
//! units that later slices have it train, E and R (`super::gdo::factory`): slice M6-GA made the Artillery Rig live,
//! and E holds the Prospector until slice M6-GP. The Field Engineer's two are in the game
//! since M6-GK too, and stay greyed until slice M6-GB. The Twister's R and F hold its mines' two commands, Lay Mine and
//! Buy Mine, until slice M6-GM (slice M6-GF, `crate::kinds::gdo::twister`). (M6-ST1) Slice M6-ST1 made the Vehicle
//! Assembly's R live, the Subterranean Transport, whose own R holds Burrow on its surfaced type and Surface on its
//! burrowed one until slice M6-ST2 (`super::syndicate::transport`).

/// A panel entry that isn't in the game yet (module docs). The Colonists' are a building of the Beacon's Warp In menus
/// or a unit of a portal's panel: a building's cost and WarpInTime are TBD (`factions/colonists/economy.md > Warp In >
/// Constructs List`), and so is the rest of it (Q0043); a unit's stats, cost and time are TBD
/// (`factions/colonists/units/scout.md`, `factions/colonists/units/lancer.md`; the psionic unit has no section yet).
/// The Syndicate's are the ExpandMenu's entries that aren't in the game yet (slice M4-F:
/// `sc_sim::defs::ExpandEntry::Planned`, by name): its expansions; and (M6-SV) the Vehicle Assembly's and (M6-SD) the
/// Aerospace Assembly's units that they don't train yet. The Cults' were the Construct menus' buildings the Recruits
/// didn't build yet (slice M5-J1: `sc_sim::defs::ConstructEntry::Planned`, by name), until slice M5-N6.
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
    /// (M6-SV) The Vehicle Assembly's E (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`:
    /// "**W** Shadebringer, **E** Laser Battery, **R** Subterranean Transport (greyed until their prerequisites are
    /// owned)"), which it trains from slice M6-SL on, as it does its W from slice M6-SB on and its R from slice M6-ST1
    /// on. Its type is in the game (slice M6-SK), so its label is its type's ([`Planned::label`]).
    LaserBattery,
    /// (M6-SD) The Aerospace Assembly's W and E (`factions/syndicate/structures/aerospace_assembly.md > Placeholder
    /// stats`: "Panel: **Q** Phantom, **W** Specter, **E** Prism"), which it trains from slices M6-SA and M6-SP on.
    /// Their types are in the game (slice M6-SK), so each label is its type's ([`Planned::label`]).
    Specter,
    Prism,
    /// The ExpandMenu's expansions not in the game yet (`factions/syndicate/structures/README.md > Tech Tree
    /// Overview`; design change 0291: "**all nine expansions** are always listed, in the Tech Tree table's order, each
    /// holding its slot"), in the table's order: the sim's `sc_sim::defs::EXPAND_MENU` names them
    /// ([`Planned::expansion`]). The Optics Lab and the Chemical Plant, which were the first two, are in the game
    /// (slice M4-K), and so are the Vehicle Assembly (slice M6-SV), the Forge and the Aerospace Assembly (slice M6-SD),
    /// the Null Chamber between them (slice M6-SN), and the Vault (slice M6-SB): the Orbital Command is the last.
    OrbitalCommand,
    /// (M5-N9) The Field Engineer's Construct menu's buildings (`factions/gdo/units/field_engineer.md > Placeholder
    /// stats`: "**A** Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z** back)"),
    /// in its order (`super::gdo::field_engineer::CONSTRUCT_MENU`).
    Bunker,
    FirstAidTent,
    /// (M6-GK) The buildings of the Deployment Center's build menus that it doesn't construct yet
    /// (`factions/gdo/structures/deployment_center.md > Constructs`), in its Constructs order: each in the game (plan
    /// decision T84), its entry held until its slice (`sc_sim::defs::PLANNED_RECIPES`). The Radar Station and the
    /// Defense System are constructed (slice M6-GR), and so are the Factory, Advanced Mechanics and the Industrial
    /// Powerplant (slice M6-GF), the Airfield (slice M6-GH2) and Propulsion Labs (slice M6-GL).
    SupplyTower,
    /// (M6-GF) The Factory's E (`factions/gdo/structures/factory.md > Placeholder stats`: "**E** Prospector"), which it
    /// trains from slice M6-GP on. Its type is in the game (slice M6-GK), so its label is its type's ([`Planned::label`]).
    /// Its R, the Artillery Rig, held its slot too, until slice M6-GA.
    Prospector,
    /// (M6-GF) The Twister's R and F, its mines' commands (`factions/gdo/units/twister.md > Placeholder stats`: "Panel:
    /// BasicCombatUnitInterfaceState plus **R: Lay Mine** (targeted) and **F: Buy Mine**"), slice M6-GM's
    /// (`crate::kinds::gdo::twister`).
    LayMine,
    BuyMine,
    /// (M6-SB) The Shadebringer's F, its Cover (`factions/syndicate/units/shadebringer.md > Placeholder stats`:
    /// "Hotkeys: **R** Sludge, **F** Cover"), slice M6-SC's, which gives the Shadebringer its smoke tank
    /// (`super::syndicate::shadebringer`).
    Cover,
    /// (M6-ST1) The Subterranean Transport's R
    /// (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "Panel: Q Move, X Stop, **R: Burrow /
    /// Surface**, **F: Unload All**"): Burrow on its surfaced type, Surface on its burrowed one, slice M6-ST2's
    /// (`super::syndicate::transport`).
    Burrow,
    Surface,
}

impl Planned {
    /// Every one, in declaration order.
    pub const ALL: [Planned; 27] = [
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
        Planned::LaserBattery,
        Planned::Specter,
        Planned::Prism,
        Planned::OrbitalCommand,
        Planned::Bunker,
        Planned::FirstAidTent,
        Planned::SupplyTower,
        Planned::Prospector,
        Planned::LayMine,
        Planned::BuyMine,
        Planned::Cover,
        Planned::Burrow,
        Planned::Surface,
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
            Planned::LaserBattery => "Laser Battery",
            Planned::Specter => "Specter",
            Planned::Prism => "Prism",
            Planned::OrbitalCommand => "Orbital Command",
            Planned::Bunker => "Bunker",
            Planned::FirstAidTent => "First Aid Tent",
            Planned::SupplyTower => "Supply Tower",
            Planned::Prospector => "Prospector",
            Planned::LayMine => "Lay Mine",
            Planned::BuyMine => "Buy Mine",
            Planned::Cover => "Cover",
            Planned::Burrow => "Burrow",
            Planned::Surface => "Surface",
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
    /// Gunworks' "GS", the Garrison's "GA", the Vehicle Assembly's "VA", (M6-SD) the Forge's "FE" and the Aerospace
    /// Assembly's "AA", (M6-SN) the Null Chamber's "NC", `crate::kinds`), and a command its own: the Tunnel's Upgrade's "Upgr." (slice M4-G). (M6-SV)
    /// The Vehicle Assembly's units not trained yet are labelled by their types, in the game already (slice M6-SK):
    /// "SBR", "LB" and "STR"; (M6-SD) the Aerospace Assembly's alike, "SPR" and "PRM". (M6-GK) The Deployment Center's
    /// one, whose type is in the game, has its type's label (the Airfield's "AD" until slice M6-GH2, Propulsion Labs'
    /// "PL" until slice M6-GL); (M6-GF) and so has the Factory's unit not trained
    /// yet, "PR" (the Artillery Rig's "ARG" until slice M6-GA), as (M6-GH1) the Airfield's Dragonfly had, "DY", until
    /// slice M6-GL. The Twister's mines' commands by their
    /// names' first words, "Lay" and "Buy", as the Drone Operator's Launch Drone is "Launch" (slice M6-GD,
    /// `PanelCommand::label`); not Buy Mine by the mine's tag, as Build Drone is by the drone's, "DRN": "ME" is the
    /// mine type's own, which no placeholder shares. (M6-SB) The Shadebringer's Cover by its name, "Cover", as its
    /// Sludge is "Sludge" (`PanelCommand::Toggle`). The Vault's "VT" and the Shadebringer's "SBR" went into the game
    /// with their types (slice M6-SB), and the Subterranean Transport's "STR" with its (slice M6-ST1). (M6-ST1) The
    /// Transport's R by its name, "Burrow" (6 letters, as "Shroud"), and a shortening of Surface, "Surf.", as
    /// Undeploy's is "Undep.".
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
            Planned::LaserBattery => "LB",
            Planned::Specter => "SPR",
            Planned::Prism => "PRM",
            Planned::OrbitalCommand => "OC",
            Planned::Bunker => "BKR",
            Planned::FirstAidTent => "FAT",
            Planned::SupplyTower => "SPT",
            Planned::Prospector => "PR",
            Planned::LayMine => "Lay",
            Planned::BuyMine => "Buy",
            Planned::Cover => "Cover",
            Planned::Burrow => "Burrow",
            Planned::Surface => "Surf.",
        }
    }

    /// Whether it is a unit, of a production panel; else a building, of the Beacon's Warp In menus, the Tunnel's
    /// ExpandMenu, the Field Engineer's Construct menu or (M6-GK) the Deployment Center's build menus.
    pub fn is_unit(self) -> bool {
        matches!(
            self,
            Planned::Scout
                | Planned::Lancer
                | Planned::PsionicUnit
                | Planned::LaserBattery
                | Planned::Specter
                | Planned::Prism
                | Planned::Prospector
        )
    }

    /// (M6-GK) Whether it is a building of the Deployment Center's build menus.
    pub fn is_constructed(self) -> bool {
        matches!(self, Planned::SupplyTower)
    }

    /// Whether it is an expansion of the Tunnel's ExpandMenu.
    pub fn is_expansion(self) -> bool {
        matches!(self, Planned::OrbitalCommand)
    }

    /// The full name of its button (`PanelCommand::name`): the command it will be, "Train Scout", "Warp In Alloy
    /// Furnace" or "Build Orbital Command" ("Train Laser Battery", M6-SV), as a unit in the game is trained, a building warped in
    /// and an expansion built (`PanelCommand::Expand`); (M6-GK) "Build Factory", as the Deployment Center's buildings
    /// are constructed (`PanelCommand::Construct`). The Field Engineer's as the design names its abilities
    /// (`factions/gdo/units/field_engineer.md > Abilities`): "**Build Bunker**", "**Set up First Aid Tent**". (M6-GF) The
    /// Twister's mines' commands by their names, as the design names them: "**R: Lay Mine**", "**F: Buy Mine**"; (M6-SB)
    /// the Shadebringer's Cover by its name, as its Sludge's toggle is ("**F** Cover"); (M6-ST1) and the Subterranean
    /// Transport's R alike: "**R: Burrow / Surface**".
    pub fn command_name(self) -> String {
        if matches!(
            self,
            Planned::LayMine | Planned::BuyMine | Planned::Cover | Planned::Burrow | Planned::Surface
        ) {
            self.name().to_string()
        } else if self == Planned::FirstAidTent {
            format!("Set up {}", self.name())
        } else if self == Planned::Bunker {
            format!("Build {}", self.name())
        } else if self.is_unit() {
            format!("Train {}", self.name())
        } else if self.is_expansion() || self.is_constructed() {
            format!("Build {}", self.name())
        } else {
            format!("Warp In {}", self.name())
        }
    }
}
