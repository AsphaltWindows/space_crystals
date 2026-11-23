//! The game-mode HUD frame (`core/camera.md > HUD Layout`, `systems/controls/selection_and_hud.md > HUD Bar`).
//!
//! A top resource bar and, at the bottom, the HUD Bar with its five sections in fixed slots, left to right. All
//! sections share one height, and a slot stays reserved while its panel has nothing to show. Sizes are in percent
//! of the surface height (`vh`), so the HUD scales with the window and the world view between the bars keeps its
//! shape at every resolution. Only the frame exists so far: the top bar shows the player and the tick, and each
//! section shows its name.
//!
//! The HUD is specific to the game mode: it tells the camera which strip it leaves free through
//! [`ViewportInsets`], and another mode (the map editor) brings its own UI instead and hides the HUD.

use bevy::prelude::*;

use crate::ClientMode;
use crate::camera::ViewportInsets;
use crate::sim_driver::{LocalPlayer, SimRes};

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

#[derive(Component)]
struct PlayerNameText;

#[derive(Component)]
struct TickText;

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
            .add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_top_bar, show_hud_in_game_modes));
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

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
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
        children![
            (PlayerNameText, text("", 2.0, TEXT_COLOR)),
            (TickText, text("", 2.0, TEXT_COLOR)),
        ],
    ));
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
                padding: UiRect::all(vh(HUD_GAP_VH)),
                column_gap: vh(HUD_GAP_VH),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(BAR_BACKGROUND),
        ))
        .with_children(|bar| {
            for section in HudSection::ALL {
                let (width, flex_grow, flex_basis) = match section.width() {
                    SlotWidth::Fixed(vh_width) => (vh(vh_width), 0.0, auto()),
                    SlotWidth::Share(share) => (auto(), share, px(0)),
                };
                bar.spawn((
                    Name::new(section.label()),
                    section,
                    Node {
                        width,
                        height: percent(100),
                        flex_grow,
                        flex_shrink: 0.0,
                        flex_basis,
                        border: UiRect::all(px(1)),
                        padding: UiRect::all(vh(0.5)),
                        ..default()
                    },
                    BackgroundColor(SECTION_BACKGROUND),
                    BorderColor::all(SECTION_BORDER),
                    children![text(section.label(), 1.6, LABEL_COLOR)],
                ));
            }
        });
}

fn update_top_bar(
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
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
        set_text(&mut text, &format!("Tick {}", sim.tick()));
    }
}

/// Changes the text only when it differs, so unchanged text isn't marked changed and laid out again.
fn set_text(text: &mut Mut<Text>, value: &str) {
    if text.0 != value {
        value.clone_into(&mut text.0);
    }
}
