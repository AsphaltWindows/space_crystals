//! The resource bar in the top bar (`core/camera.md > HUD Layout`: "Top: Resource bar"): the local player's
//! resources, as their faction's DisplayHud lists them (`core/entities.md > Player > DisplayHudInfo[Faction]`,
//! `core/entities.md > Faction > DisplayHud`), read from the sim at the end of the latest tick.
//!
//! - **GDO** (`factions/gdo/overview.md > DisplayHud`), left to right:
//!   - Space Crystals and Supplies, the banks (`Player::crystals`, `Player::supplies`); Supplies stay 0 for now, as
//!     nothing gathers or costs them (design change 0077);
//!   - Power as used / available: what the player's structures consume over what they generate (`Sim::power`),
//!     shown in red while consumption exceeds generation (design change 0090);
//!   - Unit Control as used / the cap (`Sim::unit_control`, `UNIT_CONTROL_CAP`). "Used" counts the units in
//!     production too, not the ones only queued (design change 0143: "The HUD's 'used' count includes in-production
//!     units, not merely queued ones"): the sim's one accessor counts them, so the bar follows it without a rule of
//!     its own.
//! - **Colonists** (`factions/colonists/overview.md > DisplayHud`; slice M3-C, plan decision T23), left to right:
//!   - Space Crystals, Alloys, Essence and Flux, the banks (`Player::alloys` and the others, plan decision T21);
//!     the refined ones stay 0 until refining exists (question Q0043 point 1);
//!   - Beacon Capacity as used / available (`Sim::army_cap`, plan decision T20): the units and the units in
//!     production (design change 0143) over 5 per Colonists building, at most 200; in red while more is used than
//!     available (`factions/colonists/overview.md > Beacon Capacity > Over capacity`, design change 0112). No Power:
//!     the Colonists have none (T21).
//! - **Syndicate** (`factions/syndicate/overview.md > DisplayHud`; slice M4-B), left to right:
//!   - Space Crystals and Supplies, the banks (`Player::crystals`, `Player::supplies`: 100 and 1 at the start,
//!     `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`);
//!   - Tunnel Space as used / available (`Sim::army_cap`): the units and the units in production (design change
//!     0143) over min(200, ⌊the unused tunnel-region cells of the player's Tunnels / 4⌋); in red while more is used
//!     than available (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: "usage is simply displayed over
//!     capacity (e.g. 12/10)"), as Beacon Capacity is.
//! - **The Cults** (`factions/cults/overview.md > DisplayHud`; slice M5-C), left to right:
//!   - Space Crystals, the bank;
//!   - Unit Control as used / available (`Sim::army_cap`, plan decision T56): the usage of their Recruitment Halls
//!     and their orphaned units over the sum of their Halls' capacities, each rounded down
//!     (`factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`; design change 0341: "Capacity
//!     is rounded down everywhere"). A Hall's Recruit counts from the start of its production (design change 0342:
//!     "At 19/20 with a Recruit in production, the HUD shows 20/20"). Over capacity, after a Hall is lost, it reads
//!     so, as Beacon Capacity does, but not in red: the design gives the Cults' figure no warning ("for display
//!     purposes only"; design change 0344: "Global total is for display only").
//! - A local player who isn't in the game shows no resources.
//!
//! Placeholder look, the design giving none: one ASCII text per figure (the UI font has no other glyphs), with the
//! value after the name, side by side.

use bevy::prelude::*;
use sc_sim::{ArmyResource, Faction, PlayerId, Sim, UNIT_CONTROL_CAP};

use super::{TEXT_COLOR, set_text, show_node, text};
use crate::sim_driver::{LocalPlayer, SimRes};

/// Size of the figures' text, as the top bar's other texts (placeholder).
const FIGURE_VH: f32 = 2.0;
/// The space between two figures (placeholder).
const FIGURE_GAP_VH: f32 = 3.0;
/// A figure that warns: power in the red (design change 0090), Beacon Capacity or Tunnel Space over capacity (0112,
/// `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`); the shade is a placeholder.
pub const ALERT_COLOR: Color = Color::srgb(1.0, 0.35, 0.3);

/// One figure of a DisplayHud: a resource and how it reads. Declared in the order the bar shows them, every faction's
/// figures being in that order.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceFigure {
    /// `factions/gdo/overview.md > Space Crystals: current amount` (every faction's).
    SpaceCrystals,
    /// `factions/gdo/overview.md > Supplies: current amount` (GDO) and
    /// `factions/syndicate/overview.md > Supplies: current amount` (Syndicate).
    Supplies,
    /// `factions/gdo/overview.md > Power: used / available` (GDO; design change 0090).
    Power,
    /// `factions/gdo/overview.md > Unit Control: current used / 200` (GDO).
    UnitControl,
    /// `factions/colonists/overview.md > Alloys: current amount` (Colonists).
    Alloys,
    /// `factions/colonists/overview.md > Essence: current amount` (Colonists).
    Essence,
    /// `factions/colonists/overview.md > Flux: current amount` (Colonists).
    Flux,
    /// `factions/colonists/overview.md > Beacon Capacity: current used / current available (max 200)` (Colonists).
    BeaconCapacity,
    /// `factions/syndicate/overview.md > Tunnel Space: current used / current available (max 200)` (Syndicate).
    TunnelSpace,
    /// `factions/cults/overview.md > Unit Control: current used / current available` (the Cults).
    CultsUnitControl,
}

/// `factions/gdo/overview.md > DisplayHud`.
const GDO: [ResourceFigure; 4] = [
    ResourceFigure::SpaceCrystals,
    ResourceFigure::Supplies,
    ResourceFigure::Power,
    ResourceFigure::UnitControl,
];

/// `factions/colonists/overview.md > DisplayHud`.
const COLONISTS: [ResourceFigure; 5] = [
    ResourceFigure::SpaceCrystals,
    ResourceFigure::Alloys,
    ResourceFigure::Essence,
    ResourceFigure::Flux,
    ResourceFigure::BeaconCapacity,
];

/// `factions/syndicate/overview.md > DisplayHud`.
const SYNDICATE: [ResourceFigure; 3] = [
    ResourceFigure::SpaceCrystals,
    ResourceFigure::Supplies,
    ResourceFigure::TunnelSpace,
];

/// `factions/cults/overview.md > DisplayHud`.
const CULTS: [ResourceFigure; 2] = [ResourceFigure::SpaceCrystals, ResourceFigure::CultsUnitControl];

impl ResourceFigure {
    /// Every figure, in the order the bar shows them.
    pub const ALL: [ResourceFigure; 10] = [
        ResourceFigure::SpaceCrystals,
        ResourceFigure::Supplies,
        ResourceFigure::Power,
        ResourceFigure::UnitControl,
        ResourceFigure::Alloys,
        ResourceFigure::Essence,
        ResourceFigure::Flux,
        ResourceFigure::BeaconCapacity,
        ResourceFigure::TunnelSpace,
        ResourceFigure::CultsUnitControl,
    ];

    /// The figures `faction`'s DisplayHud shows, left to right.
    pub fn of(faction: Faction) -> &'static [ResourceFigure] {
        match faction {
            Faction::Gdo => &GDO,
            Faction::Colonists => &COLONISTS,
            Faction::Syndicate => &SYNDICATE,
            Faction::Cults => &CULTS,
        }
    }
}

/// What a figure shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub text: String,
    /// Shown as a warning, in [`ALERT_COLOR`]: power in the red (design change 0090), Beacon Capacity or Tunnel Space
    /// over capacity (0112, `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`).
    pub alert: bool,
}

impl Reading {
    fn plain(text: String) -> Self {
        Self { text, alert: false }
    }
}

/// `player`'s resource bar in `sim`: each figure of their faction's DisplayHud with its reading, left to right. Empty
/// for a player who isn't in the game.
pub fn resource_bar(sim: &Sim, player: PlayerId) -> Vec<(ResourceFigure, Reading)> {
    let Some(p) = sim.player(player) else {
        return Vec::new();
    };
    let reading = |figure| match figure {
        ResourceFigure::SpaceCrystals => Reading::plain(format!("Space Crystals {}", p.crystals)),
        ResourceFigure::Supplies => Reading::plain(format!("Supplies {}", p.supplies)),
        ResourceFigure::Power => {
            let power = sim.power(player);
            Reading {
                text: format!("Power {}/{}", power.consumed, power.generated),
                alert: power.is_negative(),
            }
        }
        ResourceFigure::UnitControl => {
            Reading::plain(format!("Unit Control {}/{UNIT_CONTROL_CAP}", sim.unit_control(player)))
        }
        ResourceFigure::Alloys => Reading::plain(format!("Alloys {}", p.alloys)),
        ResourceFigure::Essence => Reading::plain(format!("Essence {}", p.essence)),
        ResourceFigure::Flux => Reading::plain(format!("Flux {}", p.flux)),
        ResourceFigure::BeaconCapacity | ResourceFigure::TunnelSpace => {
            let (used, available) = sim.army_cap(player).map_or((0, 0), |cap| (cap.used, cap.available));
            let resource = match figure {
                ResourceFigure::TunnelSpace => ArmyResource::TunnelSpace,
                _ => ArmyResource::BeaconCapacity,
            };
            Reading {
                text: format!("{} {used}/{available}", resource.name()),
                alert: used > available,
            }
        }
        ResourceFigure::CultsUnitControl => {
            let (used, available) = sim.army_cap(player).map_or((0, 0), |cap| (cap.used, cap.available));
            Reading::plain(format!("{} {used}/{available}", ArmyResource::CultsUnitControl.name()))
        }
    };
    ResourceFigure::of(p.faction)
        .iter()
        .map(|&figure| (figure, reading(figure)))
        .collect()
}

/// The bar's node, a row of one text per figure, each hidden until the local player's faction shows it.
pub(super) fn spawn_bar(bar: &mut ChildSpawnerCommands) {
    bar.spawn((
        Name::new("Resource bar"),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: vh(FIGURE_GAP_VH),
            ..default()
        },
    ))
    .with_children(|row| {
        for figure in ResourceFigure::ALL {
            row.spawn((
                figure,
                text("", FIGURE_VH, TEXT_COLOR),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        }
    });
}

/// Shows the local player's resources: after every tick, and when the local player changes (the debug player switch).
pub(super) fn update_resource_bar(
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut figures: Query<(&ResourceFigure, &mut Text, &mut TextColor, &mut Node)>,
) {
    if !sim.is_changed() && !player.is_changed() {
        return;
    }
    let bar = resource_bar(&sim, player.0);
    for (figure, mut text, mut color, mut node) in &mut figures {
        let reading = bar.iter().find(|(f, _)| f == figure).map(|(_, reading)| reading);
        show_node(&mut node, reading.is_some());
        if let Some(reading) = reading {
            set_text(&mut text, &reading.text);
            color.set_if_neq(TextColor(if reading.alert { ALERT_COLOR } else { TEXT_COLOR }));
        }
    }
}

#[cfg(test)]
mod tests;
