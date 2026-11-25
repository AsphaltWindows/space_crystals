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
//!     than available (`factions/syndicate/tunnels.md > Tunnel Space`: "usage is simply displayed over capacity (e.g.
//!     12/10)"), as Beacon Capacity is.
//! - **The Cults** show Space Crystals only, until their own resources (their Unit Control) exist in the sim (the
//!   M2 plan, slice G).
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
/// `factions/syndicate/tunnels.md > Tunnel Space`); the shade is a placeholder.
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

impl ResourceFigure {
    /// Every figure, in the order the bar shows them.
    pub const ALL: [ResourceFigure; 9] = [
        ResourceFigure::SpaceCrystals,
        ResourceFigure::Supplies,
        ResourceFigure::Power,
        ResourceFigure::UnitControl,
        ResourceFigure::Alloys,
        ResourceFigure::Essence,
        ResourceFigure::Flux,
        ResourceFigure::BeaconCapacity,
        ResourceFigure::TunnelSpace,
    ];

    /// The figures `faction`'s DisplayHud shows, left to right (module docs: the Cults' own resources don't exist
    /// yet).
    pub fn of(faction: Faction) -> &'static [ResourceFigure] {
        match faction {
            Faction::Gdo => &GDO,
            Faction::Colonists => &COLONISTS,
            Faction::Syndicate => &SYNDICATE,
            Faction::Cults => &[ResourceFigure::SpaceCrystals],
        }
    }
}

/// What a figure shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub text: String,
    /// Shown as a warning, in [`ALERT_COLOR`]: power in the red (design change 0090), Beacon Capacity or Tunnel Space
    /// over capacity (0112, `factions/syndicate/tunnels.md > Tunnel Space`).
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
mod tests {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{ObjectKind, SpawnAt, TilePos};

    use super::*;

    const P1: PlayerId = PlayerId(1);

    fn tile(x: i32, y: i32) -> SpawnAt {
        SpawnAt::Tile(TilePos::new(x, y))
    }

    /// The texts of `player`'s bar, a warning marked with `!`.
    fn texts(sim: &Sim, player: PlayerId) -> Vec<String> {
        resource_bar(sim, player)
            .into_iter()
            .map(|(_, r)| if r.alert { format!("{}!", r.text) } else { r.text })
            .collect()
    }

    /// `factions/gdo/overview.md > DisplayHud` on the starting base of `factions/gdo/economy.md > Starting Conditions`
    /// (design change 0090's first verification: "Power 21/30"; 0077: Supplies 0), then with a Barracks before any
    /// Generator (its second: "51/30", in red). The banks start at the Starting Conditions' 100 Space Crystals
    /// (slice M2-F1).
    #[test]
    fn the_gdo_bar_reads_the_banks_power_and_unit_control() {
        let base = || {
            ScenarioBuilder::new(&vec![".".repeat(16); 10].join("\n"))
                .spawn(ObjectKind::CrystalPatch, None, tile(1, 7))
                .spawn(ObjectKind::CrystalPatch, None, tile(3, 7))
                .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
                .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1))
                .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 7))
                .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 7))
        };
        let sim = Sim::new(&base().build());
        let figures: Vec<ResourceFigure> = resource_bar(&sim, P1).into_iter().map(|(f, _)| f).collect();
        use ResourceFigure::{Power, SpaceCrystals, Supplies, UnitControl};
        assert_eq!(figures, [SpaceCrystals, Supplies, Power, UnitControl]);
        assert_eq!(
            texts(&sim, P1),
            ["Space Crystals 100", "Supplies 0", "Power 21/30", "Unit Control 0/200"]
        );
        let all_in = base().spawn(ObjectKind::Barracks, Some(P1), tile(10, 1)).spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            SpawnAt::Point(Vec2::new(800.0, 500.0)),
        );
        let sim = Sim::new(&all_in.build());
        assert_eq!(
            texts(&sim, P1),
            ["Space Crystals 100", "Supplies 0", "Power 51/30!", "Unit Control 1/200"]
        );
    }

    /// Power exactly used up isn't in the red (design change 0090: red "when used exceeds available"), and a player
    /// without structures has nothing: 0/0.
    #[test]
    fn power_is_red_only_when_consumption_exceeds_generation() {
        let sim = |kinds: &[(ObjectKind, i32)]| {
            let mut builder = ScenarioBuilder::new(&vec![".".repeat(20); 8].join("\n"));
            for &(kind, x) in kinds {
                builder = builder.spawn(kind, Some(P1), tile(x, 1));
            }
            Sim::new(&builder.build())
        };
        let power = |sim: &Sim| texts(sim, P1)[2].clone();
        assert_eq!(power(&sim(&[])), "Power 0/0");
        assert_eq!(power(&sim(&[(ObjectKind::Generator, 1)])), "Power 0/20");
        // A Barracks (-30) on a Deployment Center (+30): all of it used, not more.
        let even = sim(&[(ObjectKind::DeploymentCenter, 1), (ObjectKind::Barracks, 6)]);
        assert_eq!(power(&even), "Power 30/30");
        let short = sim(&[(ObjectKind::Generator, 1), (ObjectKind::Barracks, 6)]);
        assert_eq!(power(&short), "Power 30/20!");
    }

    /// `factions/colonists/overview.md > DisplayHud` on the Colonists' starting base
    /// (`factions/colonists/economy.md > Starting Conditions`; the plan's acceptance for slice M3-C: "Space
    /// Crystals 100 ... Beacon Capacity 4/10", in ASCII): the four banks, the refined ones at 0, and Beacon Capacity
    /// as used / available, with no Power. The refined banks show what `SetResources` sets, the capacity follows the
    /// Drones in production and not those only queued (design change 0143, answering Q0041 point 1), and over
    /// capacity it shows in red.
    #[test]
    fn the_colonists_bar_reads_their_banks_and_beacon_capacity() {
        use ResourceFigure::{Alloys, BeaconCapacity, Essence, Flux, SpaceCrystals};
        let mut builder = ScenarioBuilder::new(&vec![".".repeat(16); 10].join("\n"))
            .faction(P1, Faction::Colonists)
            .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 1))
            .spawn(ObjectKind::RoboPortal, Some(P1), tile(6, 1));
        for x in 1..=4 {
            let at = TilePos::new(x, 8).center();
            builder = builder.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(at));
        }
        let mut t = builder.start();
        let figures: Vec<ResourceFigure> = resource_bar(t.sim(), P1).into_iter().map(|(f, _)| f).collect();
        assert_eq!(figures, [SpaceCrystals, Alloys, Essence, Flux, BeaconCapacity]);
        let start = texts(t.sim(), P1);
        assert_eq!(
            start,
            [
                "Space Crystals 100",
                "Alloys 0",
                "Essence 0",
                "Flux 0",
                "Beacon Capacity 4/10"
            ]
        );
        assert!(start.iter().all(|text| text.is_ascii()));
        t.debug_at(
            1,
            sc_sim::DebugCommand::SetResources {
                player: P1,
                crystals: 500,
                supplies: 0,
                alloys: 3,
                essence: 4,
                flux: 5,
            },
        );
        let train = sc_sim::ProductionCommand::Train(ObjectKind::Drone);
        let portal = sc_sim::ObjectId(2);
        t.at(
            2,
            P1,
            sc_sim::Command::Production {
                objects: vec![portal],
                command: train,
            },
        );
        t.run(2);
        assert_eq!(
            texts(t.sim(), P1),
            [
                "Space Crystals 450",
                "Alloys 3",
                "Essence 4",
                "Flux 5",
                "Beacon Capacity 5/10"
            ]
        );
        // Design change 0143: the Drone in production counts, a second one only queued doesn't; with two more Drones,
        // 7 of 10. Over capacity (`factions/colonists/overview.md > Beacon Capacity > Over capacity`, design change
        // 0112): the Egress Node lost leaves 7 used of 5, in red.
        t.at(
            3,
            P1,
            sc_sim::Command::Production {
                objects: vec![portal],
                command: train,
            },
        );
        for x in [5, 6] {
            t.spawn_unit_at(3, ObjectKind::Drone, P1, TilePos::new(x, 8).center());
        }
        t.run(1);
        assert_eq!(texts(t.sim(), P1)[4], "Beacon Capacity 7/10");
        t.debug_at(
            4,
            sc_sim::DebugCommand::Remove {
                id: sc_sim::ObjectId(1),
            },
        );
        t.run(1);
        assert_eq!(texts(t.sim(), P1)[4], "Beacon Capacity 7/5!");
    }

    /// `factions/syndicate/overview.md > DisplayHud` on the Syndicate's starting objects (slice M4-B,
    /// `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`): Space Crystals 100, Supplies 1, and Tunnel
    /// Space 1/11, the Tunnel's 48 tunnel-region cells less the Hideout's 4 over 4, of which the Agent uses 1. Over
    /// capacity (`factions/syndicate/tunnels.md > Tunnel Space`: "usage is simply displayed over capacity (e.g.
    /// 12/10)"): with a second Hideout and 11 more Agents, 12/10, in red.
    #[test]
    fn the_syndicate_bar_reads_their_banks_and_tunnel_space() {
        use ResourceFigure::{SpaceCrystals, Supplies, TunnelSpace};
        let agent = |x: i32| SpawnAt::Point(TilePos::new(x, 12).center());
        let base = || {
            ScenarioBuilder::new(&vec![".".repeat(16); 14].join("\n"))
                .faction(P1, Faction::Syndicate)
                .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 4))
                .spawn(ObjectKind::Hideout, Some(P1), tile(8, 8))
                .spawn(ObjectKind::Agent, Some(P1), agent(1))
        };
        let sim = Sim::new(&base().build());
        let figures: Vec<ResourceFigure> = resource_bar(&sim, P1).into_iter().map(|(f, _)| f).collect();
        assert_eq!(figures, [SpaceCrystals, Supplies, TunnelSpace]);
        assert_eq!(
            texts(&sim, P1),
            ["Space Crystals 100", "Supplies 1", "Tunnel Space 1/11"]
        );
        let mut full = base().spawn(ObjectKind::Hideout, Some(P1), tile(2, 2));
        for x in 2..13 {
            full = full.spawn(ObjectKind::Agent, Some(P1), agent(x));
        }
        let sim = Sim::new(&full.build());
        assert_eq!(texts(&sim, P1)[2], "Tunnel Space 12/10!");
    }

    /// The Cults show only what exists of their DisplayHud: Space Crystals. A player not in the game shows nothing.
    #[test]
    fn the_cults_show_space_crystals_only() {
        assert_eq!(ResourceFigure::of(Faction::Cults), [ResourceFigure::SpaceCrystals]);
        let mut scenario = ScenarioBuilder::new(&vec![".".repeat(8); 8].join("\n")).build();
        scenario.setup.players[1].faction = Faction::Cults;
        let sim = Sim::new(&scenario);
        assert_eq!(texts(&sim, PlayerId(2)), ["Space Crystals 0"]);
        assert!(texts(&sim, PlayerId(3)).is_empty());
    }
}
