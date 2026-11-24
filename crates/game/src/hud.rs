//! The game-mode HUD frame (`core/camera.md > HUD Layout`, `systems/controls/selection_and_hud.md > HUD Bar`).
//!
//! A top resource bar and, at the bottom, the HUD Bar with its five sections in fixed slots, left to right. All
//! sections share one height, and a slot stays reserved while its panel has nothing to show. Sizes are in percent
//! of the surface height (`vh`), so the HUD scales with the window and the world view between the bars keeps its
//! shape at every resolution. The top bar shows the player, their resources as their faction's DisplayHud lists them
//! ([`resources`]) and the tick, and while a replay plays back (`crate::replay::playback`) its last tick, speed and
//! state, and a desync. The sections are placed at the rectangles
//! of [`layout`], which also find the portrait or the command button under a click. Two panels show the selection:
//! the InfoPanel ([`info_panel`]) and the SelectionPanel ([`selection_panel`]); the CommandPanel
//! ([`command_panel`]) shows the commands for it (`crate::command_ui`). The others show their names, and so do these
//! three while they have nothing to show.
//!
//! The HUD is specific to the game mode: it tells the camera which strip it leaves free through
//! [`ViewportInsets`], and another mode (the map editor) brings its own UI instead and hides the HUD.

pub mod command_panel;
pub mod info_panel;
pub mod layout;
pub mod resources;
pub mod selection_panel;

use bevy::prelude::*;

use crate::camera::ViewportInsets;
use crate::platform::SurfaceSize;
use crate::replay::Playback;
use crate::selection::SelectionSystems;
use crate::sim_driver::{LocalPlayer, Pace, SimControl, SimRes};
use crate::{ClientMode, ClientSystems};
use layout::{SECTION_BORDER_PX, SECTION_PADDING_VH, section_rect};

/// Height of the top resource bar, in percent of the surface height (placeholder size).
pub const TOP_BAR_HEIGHT_VH: f32 = 4.0;
/// Height of the bottom HUD Bar (placeholder size).
pub const HUD_BAR_HEIGHT_VH: f32 = 24.0;
/// Padding inside the HUD Bar and the gap between its sections.
const HUD_GAP_VH: f32 = 0.6;
/// Height of the sections: the HUD Bar minus its padding.
const SECTION_HEIGHT_VH: f32 = HUD_BAR_HEIGHT_VH - 2.0 * HUD_GAP_VH;

const BAR_BACKGROUND: Color = Color::srgb(0.07, 0.08, 0.10);
const SECTION_BACKGROUND: Color = Color::srgb(0.12, 0.13, 0.16);
const SECTION_BORDER: Color = Color::srgb(0.28, 0.31, 0.36);
const LABEL_COLOR: Color = Color::srgb(0.55, 0.60, 0.66);
const TEXT_COLOR: Color = Color::srgb(0.88, 0.90, 0.93);
/// The top bar's replay note after a desync (placeholder).
const DESYNC_COLOR: Color = Color::srgb(1.0, 0.35, 0.3);
/// Size of a section's name.
const LABEL_VH: f32 = 1.6;

/// The HUD Bar's sections (`systems/controls/selection_and_hud.md > HUD Bar > Sections`).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum HudSection {
    Minimap,
    Portrait,
    InfoPanel,
    SelectionPanel,
    CommandPanel,
}

impl HudSection {
    /// Left to right.
    pub const ALL: [HudSection; 5] = [
        HudSection::Minimap,
        HudSection::Portrait,
        HudSection::InfoPanel,
        HudSection::SelectionPanel,
        HudSection::CommandPanel,
    ];

    pub fn label(self) -> &'static str {
        match self {
            HudSection::Minimap => "Minimap",
            HudSection::Portrait => "Portrait",
            HudSection::InfoPanel => "InfoPanel",
            HudSection::SelectionPanel => "SelectionPanel",
            HudSection::CommandPanel => "CommandPanel",
        }
    }

    /// The slot's width: a fixed size (vh) or a share of the width left over. Both depend only on the surface
    /// size, never on what the panels show, so the slots never shift (`> HUD Bar > Layout Rules`).
    fn width(self) -> SlotWidth {
        match self {
            HudSection::Minimap => SlotWidth::Fixed(SECTION_HEIGHT_VH),
            HudSection::Portrait => SlotWidth::Fixed(SECTION_HEIGHT_VH * 0.8),
            HudSection::InfoPanel => SlotWidth::Share(1.0),
            HudSection::SelectionPanel => SlotWidth::Share(1.0),
            // `> CommandPanel > Grid Layout`: 4 × 3 square slots.
            HudSection::CommandPanel => SlotWidth::Fixed(SECTION_HEIGHT_VH * 4.0 / 3.0),
        }
    }
}

enum SlotWidth {
    Fixed(f32),
    Share(f32),
}

#[derive(Component)]
pub struct TopBar;

#[derive(Component)]
pub struct HudBar;

/// The HUD's top-level nodes: the top bar and the HUD Bar.
#[derive(Component)]
pub struct HudRoot;

/// The name a section shows while its panel has nothing to show.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SectionLabel(pub HudSection);

#[derive(Component)]
struct PlayerNameText;

#[derive(Component)]
struct TickText;

/// The top bar's note on the replay being played back: hidden in a game.
#[derive(Component)]
pub struct ReplayText;

/// The strip the HUD leaves for the world view: everything between its two bars.
pub const HUD_INSETS: ViewportInsets = ViewportInsets {
    top_vh: TOP_BAR_HEIGHT_VH,
    bottom_vh: HUD_BAR_HEIGHT_VH,
    left_vh: 0.0,
    right_vh: 0.0,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HUD_INSETS)
            .init_resource::<selection_panel::SelectionPanelView>()
            .init_resource::<command_panel::CommandPanelView>()
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_top_bar, update_replay_note, show_hud_in_game_modes))
            // After the frame's input, which may switch the local player.
            .add_systems(Update, resources::update_resource_bar.in_set(ClientSystems::Present))
            // `> SelectionPanel > Interactions`: after the map's clicks and the keys of the same frame.
            .add_systems(
                Update,
                selection_panel::click_portraits
                    .after(SelectionSystems::Input)
                    .in_set(ClientSystems::Actions),
            )
            .add_systems(
                Update,
                (
                    place_sections,
                    info_panel::update_info_panel,
                    selection_panel::update_selection_panel,
                    command_panel::update_command_panel,
                )
                    .chain()
                    .in_set(ClientSystems::Present),
            );
    }
}

/// The HUD belongs to the game: the map editor hides it, and its play-test shows it.
fn show_hud_in_game_modes(mode: Res<ClientMode>, mut bars: Query<&mut Visibility, With<HudRoot>>) {
    let visibility = if mode.is_game() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut bars {
        v.set_if_neq(visibility);
    }
}

fn text(content: impl Into<String>, size_vh: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(FontSize::Vh(size_vh)),
        TextColor(color),
    )
}

fn spawn_hud(mut commands: Commands, size: Res<SurfaceSize>) {
    commands
        .spawn((
            Name::new("Top bar"),
            TopBar,
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: vh(TOP_BAR_HEIGHT_VH),
                padding: UiRect::horizontal(vh(1.2)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BAR_BACKGROUND),
        ))
        .with_children(|bar| {
            // The player's name on the left and the tick on the right take equal shares of the width the resources
            // leave, so the resources stay in the middle, whatever the texts beside them say.
            let side = |justify_content| Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                flex_direction: FlexDirection::Row,
                justify_content,
                align_items: AlignItems::Center,
                column_gap: vh(2.0),
                ..default()
            };
            bar.spawn(side(JustifyContent::Start))
                .with_child((PlayerNameText, text("", 2.0, TEXT_COLOR)));
            resources::spawn_bar(bar);
            bar.spawn(side(JustifyContent::End)).with_children(|right| {
                right.spawn((
                    ReplayText,
                    text("", 2.0, TEXT_COLOR),
                    Node {
                        display: Display::None,
                        ..default()
                    },
                ));
                right.spawn((TickText, text("", 2.0, TEXT_COLOR)));
            });
        });
    commands
        .spawn((
            Name::new("HUD Bar"),
            HudBar,
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(0),
                left: px(0),
                width: percent(100),
                height: vh(HUD_BAR_HEIGHT_VH),
                ..default()
            },
            BackgroundColor(BAR_BACKGROUND),
        ))
        .with_children(|bar| {
            for section in HudSection::ALL {
                let mut slot = bar.spawn((
                    Name::new(section.label()),
                    section,
                    section_node(section, size.logical()),
                    BackgroundColor(SECTION_BACKGROUND),
                    BorderColor::all(SECTION_BORDER),
                ));
                slot.with_child((SectionLabel(section), text(section.label(), LABEL_VH, LABEL_COLOR)));
                if section == HudSection::InfoPanel {
                    info_panel::spawn_content(&mut slot);
                }
            }
        });
}

/// A section's node at its slot ([`section_rect`]), relative to the HUD Bar. Its content flows top down; the
/// SelectionPanel's portraits are placed absolutely, from [`layout::PortraitGrid`].
fn section_node(section: HudSection, surface: Vec2) -> Node {
    let rect = section_rect(section, surface);
    let bar_top = surface.y * (1.0 - HUD_BAR_HEIGHT_VH / 100.0);
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.min.x),
        top: px(rect.min.y - bar_top),
        width: px(rect.width()),
        height: px(rect.height()),
        border: UiRect::all(px(SECTION_BORDER_PX)),
        padding: UiRect::all(vh(SECTION_PADDING_VH)),
        flex_direction: FlexDirection::Column,
        row_gap: vh(SECTION_PADDING_VH),
        ..default()
    }
}

/// Keeps the sections at their slots when the surface changes size.
fn place_sections(size: Res<SurfaceSize>, mut sections: Query<(&HudSection, &mut Node)>) {
    if !size.is_changed() {
        return;
    }
    for (section, mut node) in &mut sections {
        let placed = section_node(*section, size.logical());
        if (node.left, node.top, node.width, node.height) != (placed.left, placed.top, placed.width, placed.height) {
            node.left = placed.left;
            node.top = placed.top;
            node.width = placed.width;
            node.height = placed.height;
        }
    }
}

/// The top bar's texts: the local player's name and the tick, in a replay's playback with the replay's last tick.
fn update_top_bar(
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    playback: Option<Res<Playback>>,
    mut name: Query<&mut Text, (With<PlayerNameText>, Without<TickText>)>,
    mut tick: Query<&mut Text, (With<TickText>, Without<PlayerNameText>)>,
) {
    if !sim.is_changed() && !player.is_changed() {
        return;
    }
    if let Ok(mut text) = name.single_mut() {
        set_text(
            &mut text,
            sim.player(player.0).map_or("(no player)", |p| p.name.as_str()),
        );
    }
    if let Ok(mut text) = tick.single_mut() {
        let value = match &playback {
            Some(playback) => format!("Tick {} / {}", sim.tick(), playback.end_tick()),
            None => format!("Tick {}", sim.tick()),
        };
        set_text(&mut text, &value);
    }
}

/// The top bar's note on a replay's playback ([`Playback::status`]), red after a desync; hidden in a game.
fn update_replay_note(
    sim: Res<SimRes>,
    playback: Option<Res<Playback>>,
    control: Res<SimControl>,
    pace: Res<Pace>,
    mut note: Query<(&mut Text, &mut TextColor, &mut Node), With<ReplayText>>,
) {
    let Ok((mut text, mut color, mut node)) = note.single_mut() else {
        return;
    };
    show_node(&mut node, playback.is_some());
    let Some(playback) = playback else {
        return;
    };
    if !sim.is_changed() && !playback.is_changed() && !control.is_changed() && !pace.is_changed() {
        return;
    }
    set_text(&mut text, &playback.status(sim.tick(), control.paused, *pace));
    let shown = if playback.desync.is_some() {
        DESYNC_COLOR
    } else {
        TEXT_COLOR
    };
    color.set_if_neq(TextColor(shown));
}

/// Changes the text only when it differs, so unchanged text isn't marked changed and laid out again.
fn set_text(text: &mut Mut<Text>, value: &str) {
    if text.0 != value {
        value.clone_into(&mut text.0);
    }
}

/// Shows or hides a node, taking no room while hidden; changes it only when that differs.
fn show_node(node: &mut Mut<Node>, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}
