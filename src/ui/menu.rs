use bevy::prelude::*;
use crate::types::{AppState, FactionEnum, SelectedFaction};

/// Marker component for the menu root node (for cleanup on state exit)
#[derive(Component)]
pub struct MenuRoot;

/// Marker component for the menu's dedicated UI camera
#[derive(Component)]
pub struct MenuCamera;

/// Component on clickable faction buttons, storing which faction they represent
#[derive(Component)]
pub struct FactionButton(pub FactionEnum);

/// Marker component for disabled (greyed-out) faction buttons
#[derive(Component)]
pub struct DisabledButton;

/// Factions that are currently available for selection
const AVAILABLE_FACTIONS: [FactionEnum; 2] = [
    FactionEnum::GlobalDefenseOrdinance,
    FactionEnum::TheSyndicate,
];

/// Check if a faction is available for selection
fn is_faction_available(faction: &FactionEnum) -> bool {
    AVAILABLE_FACTIONS.contains(faction)
}

/// All factions in display order
const ALL_FACTIONS: [FactionEnum; 4] = [
    FactionEnum::GlobalDefenseOrdinance,
    FactionEnum::TheSyndicate,
    FactionEnum::TheCults,
    FactionEnum::Colonists,
];

/// Setup the faction selection menu UI
pub fn setup_menu(mut commands: Commands) {
    // Spawn a dedicated 2D camera for the menu UI
    let menu_cam = commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            ..default()
        },
        MenuCamera,
    )).id();

    // Root container — full screen, centered content
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.02, 0.02, 0.06)),
            ..default()
        },
        TargetCamera(menu_cam),
        MenuRoot,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "Space Crystals",
            TextStyle {
                font_size: 48.0,
                color: Color::srgb(0.9, 0.9, 0.95),
                ..default()
            },
        ));

        // Subtitle
        parent.spawn(TextBundle::from_section(
            "Select Faction",
            TextStyle {
                font_size: 24.0,
                color: Color::srgb(0.6, 0.6, 0.7),
                ..default()
            },
        ));

        // Button container — 2x2 grid
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|grid| {
            // Two rows of two buttons
            for row in ALL_FACTIONS.chunks(2) {
                grid.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|row_node| {
                    for &faction in row {
                        let available = is_faction_available(&faction);
                        spawn_faction_button(row_node, faction, available);
                    }
                });
            }
        });
    });
}

/// Spawn a single faction button
fn spawn_faction_button(parent: &mut ChildBuilder, faction: FactionEnum, available: bool) {
    let bg_color = if available {
        faction_button_color(faction)
    } else {
        Color::srgb(0.2, 0.2, 0.2) // Greyed out
    };

    let text_color = if available {
        Color::srgb(0.95, 0.95, 0.95)
    } else {
        Color::srgb(0.5, 0.5, 0.5)
    };

    let mut btn = parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(220.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: BackgroundColor(bg_color),
            border_color: BorderColor(if available {
                Color::srgb(0.7, 0.7, 0.7)
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            }),
            ..default()
        },
    ));

    if available {
        btn.insert(FactionButton(faction));
    } else {
        btn.insert(DisabledButton);
    }

    btn.with_children(|btn_content| {
        let label = if available {
            faction.name().to_string()
        } else {
            format!("{} (Coming Soon)", faction.name())
        };
        btn_content.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 18.0,
                color: text_color,
                ..default()
            },
        ));
    });
}

/// Get the background color for an available faction button
fn faction_button_color(faction: FactionEnum) -> Color {
    // Darker version of faction color for button background
    match faction.color() {
        Color::Srgba(c) => Color::srgb(c.red * 0.4, c.green * 0.4, c.blue * 0.4),
        _ => Color::srgb(0.15, 0.15, 0.3),
    }
}

/// Handle faction button clicks — insert SelectedFaction and transition to InGame
pub fn faction_button_click(
    mut commands: Commands,
    interactions: Query<(&Interaction, &FactionButton), (Changed<Interaction>, Without<DisabledButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, faction_btn) in &interactions {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(SelectedFaction(faction_btn.0));
            next_state.set(AppState::InGame);
        }
    }
}

/// Cleanup menu UI when exiting Menu state
pub fn cleanup_menu(
    mut commands: Commands,
    menu_roots: Query<Entity, With<MenuRoot>>,
    menu_cameras: Query<Entity, With<MenuCamera>>,
) {
    for entity in &menu_roots {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &menu_cameras {
        commands.entity(entity).despawn_recursive();
    }
}

/// Visual feedback: highlight buttons on hover
pub fn menu_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, Option<&FactionButton>, Option<&DisabledButton>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg, faction_btn, disabled) in &mut buttons {
        if disabled.is_some() {
            // Disabled buttons don't change appearance
            continue;
        }
        let faction = match faction_btn {
            Some(fb) => fb.0,
            None => continue,
        };
        let base = faction_button_color(faction);
        match *interaction {
            Interaction::Hovered => {
                // Brighter on hover
                match base {
                    Color::Srgba(c) => {
                        *bg = BackgroundColor(Color::srgb(
                            (c.red * 1.6).min(1.0),
                            (c.green * 1.6).min(1.0),
                            (c.blue * 1.6).min(1.0),
                        ));
                    }
                    _ => {}
                }
            }
            Interaction::Pressed => {
                // Even brighter on press
                match base {
                    Color::Srgba(c) => {
                        *bg = BackgroundColor(Color::srgb(
                            (c.red * 2.0).min(1.0),
                            (c.green * 2.0).min(1.0),
                            (c.blue * 2.0).min(1.0),
                        ));
                    }
                    _ => {}
                }
            }
            Interaction::None => {
                *bg = BackgroundColor(base);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_faction_resource_stores_faction() {
        let sf = SelectedFaction(FactionEnum::GlobalDefenseOrdinance);
        assert_eq!(sf.0, FactionEnum::GlobalDefenseOrdinance);

        let sf2 = SelectedFaction(FactionEnum::TheSyndicate);
        assert_eq!(sf2.0, FactionEnum::TheSyndicate);
    }

    #[test]
    fn selected_faction_is_copy_and_eq() {
        let sf = SelectedFaction(FactionEnum::TheSyndicate);
        let copied = sf;
        assert_eq!(sf, copied);
    }

    #[test]
    fn gdo_is_available() {
        assert!(is_faction_available(&FactionEnum::GlobalDefenseOrdinance));
    }

    #[test]
    fn syndicate_is_available() {
        assert!(is_faction_available(&FactionEnum::TheSyndicate));
    }

    #[test]
    fn cults_is_not_available() {
        assert!(!is_faction_available(&FactionEnum::TheCults));
    }

    #[test]
    fn colonists_is_not_available() {
        assert!(!is_faction_available(&FactionEnum::Colonists));
    }

    #[test]
    fn all_factions_constant_has_four_entries() {
        assert_eq!(ALL_FACTIONS.len(), 4);
    }

    #[test]
    fn available_factions_constant_has_two_entries() {
        assert_eq!(AVAILABLE_FACTIONS.len(), 2);
    }

    #[test]
    fn faction_button_color_returns_dark_variant() {
        let color = faction_button_color(FactionEnum::GlobalDefenseOrdinance);
        // Should be a valid color (non-black)
        match color {
            Color::Srgba(c) => {
                assert!(c.red > 0.0 || c.green > 0.0 || c.blue > 0.0);
                // Should be darker than the faction color itself
                let faction_color = FactionEnum::GlobalDefenseOrdinance.color();
                match faction_color {
                    Color::Srgba(fc) => {
                        assert!(c.red < fc.red);
                        assert!(c.green < fc.green);
                        assert!(c.blue < fc.blue);
                    }
                    _ => panic!("Expected Srgba"),
                }
            }
            _ => panic!("Expected Srgba"),
        }
    }

    #[test]
    fn faction_button_color_for_all_available_factions() {
        for faction in &AVAILABLE_FACTIONS {
            let color = faction_button_color(*faction);
            match color {
                Color::Srgba(c) => {
                    // All components should be positive (non-black)
                    assert!(c.red > 0.0 || c.green > 0.0 || c.blue > 0.0);
                }
                _ => panic!("Expected Srgba for {:?}", faction),
            }
        }
    }
}
