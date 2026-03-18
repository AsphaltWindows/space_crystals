// Shared imports and re-exports for QA-generated test files.
// Each generated test module uses `use crate::helpers::*;` to access these.

pub use space_crystals::testing::TestApp;
pub use space_crystals::testing::TestHarness;
#[allow(unused_imports)]
pub use space_crystals::testing::assertions::*;

pub use bevy::prelude::*;
pub use space_crystals::types::*;
pub use space_crystals::game::types::objects::ObjectInstance;
#[allow(unused_imports)]
pub use space_crystals::game::units::types::state::UnitCommand;
#[allow(unused_imports)]
pub use space_crystals::game::units::types::state::commands::CommandType;
#[allow(unused_imports)]
pub use space_crystals::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation, TilePresetEnum, FogOfWarMap, GridMap};

// Input event helpers for simulating mouse/keyboard input in tests.
// Direct ButtonInput::press() gets cleared by Bevy's input system before game systems run.
// Sending events via World::send_event() is processed by the input system AFTER clearing.
pub use bevy::input::ButtonState;
pub use bevy::input::mouse::MouseButtonInput;
pub use bevy::input::keyboard::KeyboardInput;
pub use bevy::input::keyboard::Key;

/// Simulate a mouse button press via the event system.
/// Must be called before `app.update()` to be processed in the same frame.
#[allow(dead_code)]
pub fn send_mouse_press(app: &mut App, button: MouseButton) {
    // Find the PrimaryWindow entity for the event
    let window = app.world_mut()
        .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
        .iter(app.world())
        .next()
        .unwrap_or(Entity::PLACEHOLDER);
    app.world_mut().send_event(MouseButtonInput {
        button,
        state: ButtonState::Pressed,
        window,
    });
}

/// Simulate a key press via the event system.
#[allow(dead_code)]
pub fn send_key_press(app: &mut App, key_code: KeyCode) {
    let window = app.world_mut()
        .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
        .iter(app.world())
        .next()
        .unwrap_or(Entity::PLACEHOLDER);
    app.world_mut().send_event(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
}
