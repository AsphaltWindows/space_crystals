//! The panel entries that aren't in the game yet ([`Planned`]): each holds its slot, unavailable, and says why when
//! it is pressed (design change 0162). The Colonists' buildings and units whose numbers are TBD (design change 0124:
//! "Buildings and units whose costs are TBD show in their slot, unavailable"; slices M3-J1 and M3-J2), and the
//! Syndicate's commands and units that later slices bring (design change 0257: "unavailable entries are greyed"; slice
//! M4-J2): the Tunnel's Upgrade, Expand and Cancel, the Hideout's Guard and Grenadier, and the Agent's Laser Turret
//! (slice M4-J1). The slice that brings one replaces its entry with the real command.

use sc_sim::ObjectKind;

use super::super::WhyUnavailable;
use crate::command_ui::feedback::{NOTHING_TO_CANCEL_TEXT, REQUIRES_CHEMICAL_PLANT_TEXT, REQUIRES_OPTICS_LAB_TEXT};
use crate::kinds::short_label;

/// A panel entry that isn't in the game yet (module docs). The Colonists' are a building of the Beacon's Warp In menus
/// or a unit of a portal's panel: a building's cost and WarpInTime are TBD (`factions/colonists/economy.md > Warp In >
/// Constructs List`), and so is the rest of it (Q0043); a unit's stats, cost and time are TBD
/// (`factions/colonists/units/scout.md`, `factions/colonists/units/lancer.md`; the psionic unit has no section yet).
/// The Syndicate's are the Tunnel's commands of slices M4-F and M4-G, the Hideout's units of slices M4-K and M4-U, and
/// the Agent's Laser Turret of slice M4-S1.
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
    /// The Tunnel's Q, "Upgrade Tunnel" (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, design change
    /// 0257), which slice M4-G brings.
    UpgradeTunnel,
    /// The Tunnel's W, "Expand Tunnel", which opens the ExpandMenu: slice M4-F's.
    ExpandTunnel,
    /// The Tunnel's X, "Cancel (upgrade, construction or the current operation)": a Tunnel has no operation before
    /// slices M4-F and M4-G, so there is nothing to cancel ([`Planned::why`]). An eject queue is none: "the queue
    /// can't be cancelled" (`factions/syndicate/tunnels.md > EjectMenu`).
    CancelOperation,
    /// The Hideout's W, "Build Guard" (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`):
    /// "Only available if the player owns at least one Optics Lab", which slice M4-K brings with the Hideout's Guard
    /// recipe; the recipe then takes the slot (`super::production_grid`).
    Guard,
    /// The Hideout's E, "Build Grenadier" (design change 0260): "Only available if the player owns at least one
    /// Chemical Plant". Slice M4-U brings the Grenadier and its recipe, which take the slot.
    Grenadier,
    /// The Agent's D, "Build Laser Turret" (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState > Panel
    /// letters`, design change 0257: "**D** Build Laser Turret (greyed until available)";
    /// `factions/syndicate/units/agent.md > DefaultState commands`: "Visible but greyed out until the player owns an
    /// Optics Lab"). Slice M4-S1 brings the Laser Turret, whose Build then takes the slot.
    LaserTurret,
}

impl Planned {
    /// Every one, in declaration order.
    pub const ALL: [Planned; 20] = [
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
        Planned::UpgradeTunnel,
        Planned::ExpandTunnel,
        Planned::CancelOperation,
        Planned::Guard,
        Planned::Grenadier,
        Planned::LaserTurret,
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
            Planned::UpgradeTunnel => "Upgrade Tunnel",
            Planned::ExpandTunnel => "Expand Tunnel",
            Planned::CancelOperation => "Cancel Operation",
            Planned::Guard => ObjectKind::Guard.def().name,
            Planned::Grenadier => "Grenadier",
            Planned::LaserTurret => "Laser Turret",
        }
    }

    /// The type in the game it stands for, if it is one: the Hideout's Guard, a unit in the game that the Hideout
    /// doesn't train yet.
    pub fn kind(self) -> Option<ObjectKind> {
        match self {
            Planned::Guard => Some(ObjectKind::Guard),
            _ => None,
        }
    }

    /// Its button's label: a placeholder for its icon (Q0001), the initials of its name, as a building warped in and a
    /// unit trained are labelled by their type's tag (`PanelCommand::label`); the Scout's its first and last letters,
    /// "ST", as the Guard's and the Beacon's are, since "SC" is the crystal patch's (slice M3-J3: every label its own,
    /// `crate::kinds::short_label`). A type in the game shows its own tag ([`Planned::kind`]: the Guard's "GD"), so
    /// that its button looks the same once its recipe replaces the entry; the Grenadier's is its first and second
    /// letters, as the Guard's. The Tunnel's commands are named as the other commands' labels are: "Upgr." shortens
    /// Upgrade as "Adv." does Advanced, Expand is short enough, and every Cancel is "Cancel". The Laser Turret's is its
    /// initials, "LT", as a building's.
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
            Planned::UpgradeTunnel => "Upgr.",
            Planned::ExpandTunnel => "Expand",
            Planned::CancelOperation => "Cancel",
            Planned::Guard => short_label(ObjectKind::Guard),
            Planned::Grenadier => "GR",
            Planned::LaserTurret => "LT",
        }
    }

    /// Whether it is a unit, of a production panel; else a building of the Beacon's Warp In menus, or a command.
    pub fn is_unit(self) -> bool {
        matches!(
            self,
            Planned::Scout | Planned::Lancer | Planned::PsionicUnit | Planned::Guard | Planned::Grenadier
        )
    }

    /// Whether it is a command of the Tunnel's panel, named as it is.
    fn is_command(self) -> bool {
        matches!(
            self,
            Planned::UpgradeTunnel | Planned::ExpandTunnel | Planned::CancelOperation
        )
    }

    /// The full name of its button (`PanelCommand::name`): the command it will be, "Train Scout" or "Warp In Alloy
    /// Furnace", as a unit in the game is trained and a building warped in, the Agent's "Build Laser Turret", as it
    /// builds a Tunnel (`PanelCommand::Build`), or the Tunnel's command by its name ("Upgrade Tunnel").
    pub fn command_name(self) -> String {
        if self.is_command() {
            self.name().into()
        } else if self.is_unit() {
            format!("Train {}", self.name())
        } else if self == Planned::LaserTurret {
            format!("Build {}", self.name())
        } else {
            format!("Warp In {}", self.name())
        }
    }

    /// Why its button is greyed out (design change 0162): it isn't in the game yet, but where the reason the command
    /// gate will give is known and true now, that one (slice M4-J2), so that the button says the same before and after
    /// its slice: the Hideout's units their prerequisites, which no player can own yet ("Requires an Optics Lab",
    /// "Requires a Chemical Plant", as slice M4-K will grey them), the Agent's Laser Turret its own (design change
    /// 0260: "greyed until available"; `factions/syndicate/units/agent.md > DefaultState commands`: "until the player
    /// owns an Optics Lab"), and the Tunnel's Cancel that there is nothing to cancel.
    pub fn why(self) -> WhyUnavailable {
        match self {
            Planned::Guard | Planned::LaserTurret => WhyUnavailable::Placeholder(REQUIRES_OPTICS_LAB_TEXT),
            Planned::Grenadier => WhyUnavailable::Placeholder(REQUIRES_CHEMICAL_PLANT_TEXT),
            Planned::CancelOperation => WhyUnavailable::Placeholder(NOTHING_TO_CANCEL_TEXT),
            _ => WhyUnavailable::NotYet,
        }
    }
}
