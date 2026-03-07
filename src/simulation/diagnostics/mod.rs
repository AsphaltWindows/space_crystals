pub mod types;
pub mod utils;
mod overlay;
mod instrumentation;

use bevy::prelude::*;
use crate::simulation::types::DiagCategory;
use types::{PerformanceMetrics, TimerStarts};
use instrumentation::{start_timer, stop_timer, prune_metrics, console_log_metrics_system};
use overlay::{spawn_overlay, toggle_overlay, update_overlay_text};

/// Plugin that adds per-system performance diagnostics.
/// Toggle the debug overlay with F3.
pub struct PerformanceDiagnosticsPlugin;

impl Plugin for PerformanceDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceMetrics>()
            .init_resource::<TimerStarts>()
            .add_systems(Startup, spawn_overlay);

        // --- FixedUpdate instrumentation ---
        app.add_systems(FixedUpdate, (
            start_timer("FogOfWar").before(DiagCategory::FogOfWar),
            stop_timer("FogOfWar").after(DiagCategory::FogOfWar),
            start_timer("Construction").before(DiagCategory::Construction),
            stop_timer("Construction").after(DiagCategory::Construction),
            start_timer("SupplyDelivery").before(DiagCategory::SupplyDelivery),
            stop_timer("SupplyDelivery").after(DiagCategory::SupplyDelivery),
        ));

        // --- Update instrumentation ---
        app.add_systems(Update, (
            start_timer("Combat").before(DiagCategory::Combat),
            stop_timer("Combat").after(DiagCategory::Combat),
            start_timer("Turrets").before(DiagCategory::Turrets),
            stop_timer("Turrets").after(DiagCategory::Turrets),
            start_timer("Projectiles").before(DiagCategory::Projectiles),
            stop_timer("Projectiles").after(DiagCategory::Projectiles),
            start_timer("Movement").before(DiagCategory::Movement),
            stop_timer("Movement").after(DiagCategory::Movement),
            start_timer("Commands").before(DiagCategory::Commands),
            stop_timer("Commands").after(DiagCategory::Commands),
            start_timer("Selection").before(DiagCategory::Selection),
            stop_timer("Selection").after(DiagCategory::Selection),
        ));

        // Split into separate add_systems to stay under Bevy's tuple limits
        app.add_systems(Update, (
            start_timer("Map").before(DiagCategory::Map),
            stop_timer("Map").after(DiagCategory::Map),
            start_timer("Faction").before(DiagCategory::Faction),
            stop_timer("Faction").after(DiagCategory::Faction),
            start_timer("UI/HUD").before(DiagCategory::UiHud),
            stop_timer("UI/HUD").after(DiagCategory::UiHud),
            start_timer("Camera").before(DiagCategory::Camera),
            stop_timer("Camera").after(DiagCategory::Camera),
        ));

        // Overlay and maintenance systems
        app.add_systems(Update, (
            toggle_overlay,
            update_overlay_text,
            prune_metrics,
            console_log_metrics_system,
        ));
    }
}
