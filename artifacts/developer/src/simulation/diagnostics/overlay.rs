use bevy::prelude::*;
use crate::types::AppState;
use super::types::*;
use super::utils::{format_duration, format_percent};

/// System that toggles the diagnostics overlay with F3.
pub fn toggle_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut metrics: ResMut<PerformanceMetrics>,
    mut overlay_query: Query<&mut Visibility, With<DiagnosticsOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        metrics.overlay_visible = !metrics.overlay_visible;
        let target_vis = if metrics.overlay_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        for mut vis in overlay_query.iter_mut() {
            *vis = target_vis;
        }
    }
}

/// System that updates the diagnostics overlay text each frame.
pub fn update_overlay_text(
    metrics: Res<PerformanceMetrics>,
    mut text_query: Query<&mut Text, With<DiagnosticsText>>,
) {
    if !metrics.overlay_visible {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let mut lines = Vec::new();
    lines.push("Performance Diagnostics".to_string());
    lines.push(format!("{:-<52}", ""));

    // Group by clock type
    lines.push("[Sim - FixedUpdate]".to_string());
    for cat in CATEGORIES.iter().filter(|c| c.clock == ClockType::FixedUpdate) {
        if let Some(stats) = metrics.stats(cat.name) {
            lines.push(format!(
                "  {:<16} {:>8} {:>8} {:>8}",
                cat.name,
                format_duration(stats.min),
                format_duration(stats.avg),
                format_duration(stats.max),
            ));
        } else {
            lines.push(format!("  {:<16}      ---      ---      ---", cat.name));
        }
    }

    lines.push(String::new());
    lines.push("[Render - Update]".to_string());

    // Compute total avg for percentage
    let total_avg_us: u128 = CATEGORIES.iter()
        .filter(|c| c.clock == ClockType::Update)
        .filter_map(|c| metrics.stats(c.name))
        .map(|s| s.avg.as_micros())
        .sum();

    for cat in CATEGORIES.iter().filter(|c| c.clock == ClockType::Update) {
        if let Some(stats) = metrics.stats(cat.name) {
            let pct = if total_avg_us > 0 {
                (stats.avg.as_micros() as f64 / total_avg_us as f64) * 100.0
            } else {
                0.0
            };
            lines.push(format!(
                "  {:<16} {:>8} {:>8} {:>8} {:>6}",
                cat.name,
                format_duration(stats.min),
                format_duration(stats.avg),
                format_duration(stats.max),
                format_percent(pct),
            ));
        } else {
            lines.push(format!("  {:<16}      ---      ---      ---    ---", cat.name));
        }
    }

    lines.push(format!("{:-<52}", ""));
    lines.push(format!("{:<16}                  min      avg      max    pct", ""));

    **text = lines.join("\n");
}

/// Spawns the diagnostics overlay UI on entering InGame state.
/// Uses IsDefaultUiCamera on the HUD camera to render on the correct camera.
pub fn spawn_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            Visibility::Hidden,
            ZIndex(100),
            DiagnosticsOverlay,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Performance Diagnostics\nLoading..."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                DiagnosticsText,
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::utils::*;
    use std::time::Duration;

    #[test]
    fn format_stats_line() {
        let stats = CategoryStats {
            min: Duration::from_micros(50),
            avg: Duration::from_micros(100),
            max: Duration::from_micros(200),
            sample_count: 10,
        };
        let min_str = format_duration(stats.min);
        let avg_str = format_duration(stats.avg);
        let max_str = format_duration(stats.max);
        assert_eq!(min_str, "50us");
        assert_eq!(avg_str, "100us");
        assert_eq!(max_str, "200us");
    }

    #[test]
    fn overlay_percentage_calculation() {
        // If one category has avg 500us out of 1000us total, it should be 50%
        let pct = (500.0_f64 / 1000.0) * 100.0;
        assert_eq!(format_percent(pct), "50.0%");
    }
}
