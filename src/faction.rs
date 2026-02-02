use bevy::prelude::*;
use crate::units::Owner;

/// The four playable factions in Space Crystals
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    GlobalDefenseOrdinance,
    Syndicate,
    Cults,
    Colonists,
}

impl Faction {
    /// Get the display name for this faction
    pub fn name(&self) -> &str {
        match self {
            Faction::GlobalDefenseOrdinance => "Global Defense Ordinance",
            Faction::Syndicate => "The Syndicate",
            Faction::Cults => "The Cults",
            Faction::Colonists => "Colonists",
        }
    }

    /// Get the visual color for this faction
    pub fn color(&self) -> Color {
        match self {
            Faction::GlobalDefenseOrdinance => Color::srgb(0.2, 0.4, 0.8),  // Blue
            Faction::Syndicate => Color::srgb(0.8, 0.2, 0.2),              // Red
            Faction::Cults => Color::srgb(0.5, 0.2, 0.6),                  // Purple
            Faction::Colonists => Color::srgb(0.2, 0.8, 0.3),              // Green
        }
    }

    /// Get abbreviated name (for UI)
    pub fn abbrev(&self) -> &str {
        match self {
            Faction::GlobalDefenseOrdinance => "GDO",
            Faction::Syndicate => "SYN",
            Faction::Cults => "CULT",
            Faction::Colonists => "COL",
        }
    }
}

/// Component storing faction affiliation for units/buildings
#[derive(Component, Clone, Copy, Debug)]
pub struct FactionMember {
    pub faction: Faction,
    pub player_id: u8,
}

impl FactionMember {
    /// Create a faction member for a player
    pub fn new(faction: Faction, player_id: u8) -> Self {
        Self { faction, player_id }
    }

    /// Get color for this faction member (faction color)
    pub fn color(&self) -> Color {
        self.faction.color()
    }

    /// Convert to Owner for compatibility
    pub fn to_owner(&self) -> Owner {
        Owner::Player(self.player_id)
    }
}

/// Resource tracking for Global Defense Ordinance
#[derive(Clone, Debug)]
pub struct GdoResources {
    pub space_crystals: u32,
    pub supplies: u32,
    pub power_generated: i32,
    pub power_consumed: i32,
}

impl Default for GdoResources {
    fn default() -> Self {
        Self {
            space_crystals: 500,
            supplies: 100,
            power_generated: 100,
            power_consumed: 0,
        }
    }
}

impl GdoResources {
    /// Get net power (generated - consumed)
    pub fn net_power(&self) -> i32 {
        self.power_generated - self.power_consumed
    }

    /// Check if power is sufficient
    pub fn has_power(&self) -> bool {
        self.net_power() >= 0
    }

    /// Get power efficiency ratio (0.0 to 1.0)
    pub fn power_efficiency(&self) -> f32 {
        if self.power_consumed == 0 {
            1.0
        } else {
            (self.power_generated as f32 / self.power_consumed as f32).min(1.0)
        }
    }
}

/// Resource tracking for The Syndicate
#[derive(Clone, Debug)]
pub struct SyndicateResources {
    pub space_crystals: u32,
    pub supplies: u32,
    pub tunnel_space_provided: u32,
    pub tunnel_space_used: u32,
}

impl Default for SyndicateResources {
    fn default() -> Self {
        Self {
            space_crystals: 500,
            supplies: 50,
            tunnel_space_provided: 20,
            tunnel_space_used: 0,
        }
    }
}

impl SyndicateResources {
    /// Get available tunnel space
    pub fn available_tunnel_space(&self) -> u32 {
        self.tunnel_space_provided.saturating_sub(self.tunnel_space_used)
    }

    /// Check if enough tunnel space is available
    pub fn has_tunnel_space(&self, required: u32) -> bool {
        self.available_tunnel_space() >= required
    }
}

/// Resource tracking for The Cults
#[derive(Clone, Debug)]
pub struct CultsResources {
    pub space_crystals: u32,
    pub recruits: u32,
    pub max_recruits: u32, // From recruitment centers
}

impl Default for CultsResources {
    fn default() -> Self {
        Self {
            space_crystals: 500,
            recruits: 5,
            max_recruits: 20,
        }
    }
}

impl CultsResources {
    /// Check if at recruit cap
    pub fn at_recruit_cap(&self) -> bool {
        self.recruits >= self.max_recruits
    }

    /// Get available recruit slots
    pub fn available_recruit_slots(&self) -> u32 {
        self.max_recruits.saturating_sub(self.recruits)
    }
}

/// Resource tracking for Colonists
#[derive(Clone, Debug)]
pub struct ColonistsResources {
    pub space_crystals: u32,
    pub alloys: u32,
    pub extracts: u32,
    pub ascension_credits: u32,
    pub beacon_capacity_provided: u32,
    pub beacon_capacity_used: u32,
}

impl Default for ColonistsResources {
    fn default() -> Self {
        Self {
            space_crystals: 500,
            alloys: 50,
            extracts: 50,
            ascension_credits: 0,
            beacon_capacity_provided: 20,
            beacon_capacity_used: 0,
        }
    }
}

impl ColonistsResources {
    /// Get available beacon capacity
    pub fn available_beacon_capacity(&self) -> u32 {
        self.beacon_capacity_provided.saturating_sub(self.beacon_capacity_used)
    }

    /// Check if enough beacon capacity is available
    pub fn has_beacon_capacity(&self, required: u32) -> bool {
        self.available_beacon_capacity() >= required
    }
}

/// Unified faction resources enum
#[derive(Clone, Debug)]
pub enum FactionResources {
    Gdo(GdoResources),
    Syndicate(SyndicateResources),
    Cults(CultsResources),
    Colonists(ColonistsResources),
}

impl FactionResources {
    /// Create default resources for a faction
    pub fn for_faction(faction: Faction) -> Self {
        match faction {
            Faction::GlobalDefenseOrdinance => FactionResources::Gdo(GdoResources::default()),
            Faction::Syndicate => FactionResources::Syndicate(SyndicateResources::default()),
            Faction::Cults => FactionResources::Cults(CultsResources::default()),
            Faction::Colonists => FactionResources::Colonists(ColonistsResources::default()),
        }
    }

    /// Get faction type
    pub fn faction(&self) -> Faction {
        match self {
            FactionResources::Gdo(_) => Faction::GlobalDefenseOrdinance,
            FactionResources::Syndicate(_) => Faction::Syndicate,
            FactionResources::Cults(_) => Faction::Cults,
            FactionResources::Colonists(_) => Faction::Colonists,
        }
    }

    /// Get space crystals (common to all factions)
    pub fn space_crystals(&self) -> u32 {
        match self {
            FactionResources::Gdo(r) => r.space_crystals,
            FactionResources::Syndicate(r) => r.space_crystals,
            FactionResources::Cults(r) => r.space_crystals,
            FactionResources::Colonists(r) => r.space_crystals,
        }
    }

    /// Add space crystals
    pub fn add_space_crystals(&mut self, amount: u32) {
        match self {
            FactionResources::Gdo(r) => r.space_crystals += amount,
            FactionResources::Syndicate(r) => r.space_crystals += amount,
            FactionResources::Cults(r) => r.space_crystals += amount,
            FactionResources::Colonists(r) => r.space_crystals += amount,
        }
    }

    /// Remove space crystals (returns true if successful)
    pub fn spend_space_crystals(&mut self, amount: u32) -> bool {
        let current = self.space_crystals();
        if current >= amount {
            match self {
                FactionResources::Gdo(r) => r.space_crystals -= amount,
                FactionResources::Syndicate(r) => r.space_crystals -= amount,
                FactionResources::Cults(r) => r.space_crystals -= amount,
                FactionResources::Colonists(r) => r.space_crystals -= amount,
            }
            true
        } else {
            false
        }
    }
}

/// Resource component for players
#[derive(Component)]
pub struct PlayerResources {
    pub player_id: u8,
    pub resources: FactionResources,
}

impl PlayerResources {
    /// Create player resources for a faction
    pub fn new(player_id: u8, faction: Faction) -> Self {
        Self {
            player_id,
            resources: FactionResources::for_faction(faction),
        }
    }
}

/// Plugin for faction systems
pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player_resources)
            .add_systems(Update, display_resources_system);
    }
}

/// Setup initial player resources
fn setup_player_resources(mut commands: Commands) {
    // Create player 0 (GDO) resources
    commands.spawn(PlayerResources::new(0, Faction::GlobalDefenseOrdinance));

    // Create player 1 (Syndicate) resources
    commands.spawn(PlayerResources::new(1, Faction::Syndicate));

    info!("Initialized faction resources for players 0 and 1");
}

/// System to display resource information when 'R' key is pressed
fn display_resources_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_resources: Query<&PlayerResources>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for player_res in player_resources.iter() {
            let faction = player_res.resources.faction();
            info!("=== Player {} ({}) Resources ===", player_res.player_id, faction.name());
            info!("Space Crystals: {}", player_res.resources.space_crystals());

            match &player_res.resources {
                FactionResources::Gdo(r) => {
                    info!("Supplies: {}", r.supplies);
                    info!("Power: {} / {} (net: {})",
                        r.power_generated, r.power_consumed, r.net_power());
                    info!("Power Efficiency: {:.1}%", r.power_efficiency() * 100.0);
                }
                FactionResources::Syndicate(r) => {
                    info!("Supplies: {}", r.supplies);
                    info!("Tunnel Space: {} / {} (available: {})",
                        r.tunnel_space_used, r.tunnel_space_provided,
                        r.available_tunnel_space());
                }
                FactionResources::Cults(r) => {
                    info!("Recruits: {} / {}", r.recruits, r.max_recruits);
                    info!("Available Recruit Slots: {}", r.available_recruit_slots());
                }
                FactionResources::Colonists(r) => {
                    info!("Alloys: {}", r.alloys);
                    info!("Extracts: {}", r.extracts);
                    info!("Ascension Credits: {}", r.ascension_credits);
                    info!("Beacon Capacity: {} / {} (available: {})",
                        r.beacon_capacity_used, r.beacon_capacity_provided,
                        r.available_beacon_capacity());
                }
            }
            info!("");
        }
    }
}
