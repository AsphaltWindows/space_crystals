use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::{
    MapPlugin, ResourcesPlugin, UnitsPlugin, CommandsPlugin,
    CombatPlugin, TurretPlugin, ProjectilePlugin, FactionPlugin,
};
use crate::simulation::SimulationCorePlugin;
use crate::ui::types::UiCameraEntity;

/// Headless test application for integration testing.
///
/// Creates a Bevy App with all game plugins except HudPlugin and GamePlugin
/// (which require rendering). Uses MinimalPlugins plus manually initialized
/// resources to satisfy system dependencies without a window or render pipeline.
pub struct TestApp {
    pub app: App,
}

impl TestApp {
    /// Create a new headless test app with all game systems registered (GDO default).
    pub fn new() -> Self {
        Self::new_with_faction(crate::types::FactionEnum::GlobalDefenseOrdinance)
    }

    /// Create a new headless test app with a specific faction selected.
    pub fn new_with_faction(faction: crate::types::FactionEnum) -> Self {
        let mut app = App::new();

        // MinimalPlugins provides TaskPool, Time, and ScheduleRunner — no window/rendering
        app.add_plugins(MinimalPlugins);

        // Add core engine plugins that game systems depend on (no rendering)
        app.add_plugins(bevy::input::InputPlugin);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_plugins(bevy::asset::AssetPlugin::default());

        // Manually init asset storages needed by spawn functions (Mesh3d, MeshMaterial3d)
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<Image>();

        // Gizmo infrastructure: GizmoPlugin needs Assets<Shader> from the render pipeline.
        // We init it manually since we don't have RenderPlugin.
        app.init_asset::<bevy::prelude::Shader>();
        app.add_plugins(bevy::gizmos::GizmoPlugin);

        // Spawn a dummy PrimaryWindow entity so systems that query
        // Query<&Window, With<PrimaryWindow>> don't panic
        app.world_mut().spawn((
            Window {
                title: "TestApp".into(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                ..default()
            },
            PrimaryWindow,
        ));

        // Spawn a dummy entity for UiCameraEntity resource
        // (normally created by HudPlugin's setup_hud)
        let dummy_ui_cam = app.world_mut().spawn_empty().id();
        app.insert_resource(UiCameraEntity(dummy_ui_cam));

        // Spawn a dummy MainCamera entity so systems that query
        // Query<(&Camera, &GlobalTransform), With<MainCamera>> don't panic.
        // Uses minimal components instead of Camera3dBundle (which needs render pipeline)
        app.world_mut().spawn((
            Camera {
                order: 0,
                ..default()
            },
            GlobalTransform::default(),
            Transform::from_xyz(0.0, 40.0, 25.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            crate::types::MainCamera,
        ));

        // Register all game plugins except HudPlugin and GamePlugin
        app.add_plugins((
            SimulationCorePlugin,
            MapPlugin,
            ResourcesPlugin,
            UnitsPlugin,
            CommandsPlugin,
            CombatPlugin,
            TurretPlugin,
            ProjectilePlugin,
            FactionPlugin,
        ));

        // Manually init resources that HudPlugin would normally provide
        app.init_resource::<crate::ui::types::ObjectInterfaceState>();
        app.init_resource::<crate::ui::types::CommandPanelTarget>();
        app.init_resource::<crate::ui::types::CursorOverUi>();
        app.init_resource::<crate::ui::types::CursorTarget>();

        // Set up AppState and SelectedFaction so OnEnter(InGame) systems fire.
        // StatesPlugin is needed for init_state (not included in MinimalPlugins)
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::types::AppState>();
        app.insert_resource(crate::types::SelectedFaction(faction));

        // Transition directly to InGame so game systems start on first step()
        let next = app.world_mut().resource_mut::<NextState<crate::types::AppState>>();
        next.into_inner().set(crate::types::AppState::InGame);

        TestApp { app }
    }

    /// Advance the app by one frame (calls app.update() once).
    pub fn step(&mut self) {
        self.app.update();
    }

    /// Advance the app by N frames.
    pub fn step_n(&mut self, n: usize) {
        for _ in 0..n {
            self.app.update();
        }
    }
}
