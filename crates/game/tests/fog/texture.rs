//! (Slice M5-Z3) The fog layer's texture across a change of its size. Bevy 0.20 keeps the GPU texture a 2D material
//! was prepared with, so an image resized under its handle stays on screen at its old size and contents
//! (`ARCHITECTURE.md > Bevy 0.20: verify, don't recall`): a new size is a new texture, which the material is pointed
//! at. Its size changes from the 1 × 1 stand-in in the first frame the fog applies after frames it didn't (an editor
//! play-test, F3 after `--no-fog`), and with the map between play-tests.

use std::collections::BTreeMap;

use space_crystals::editor::actions::{EditorAction, EditorActions};
use space_crystals::fog::fog_texels;

use super::*;

/// The size of the image the fog layer's material shows, which must be the texture the layer writes, hold player 1's
/// fog while the fog applies, and never have changed size (`sizes`: each image's first size).
fn shown_fog(app: &mut App, sizes: &mut BTreeMap<AssetId<Image>, UVec2>) -> UVec2 {
    let world = app.world_mut();
    let mut layer = world.query_filtered::<&MeshMaterial2d<ColorMaterial>, With<FogLayer>>();
    let material = layer.single(world).unwrap().0.clone();
    let materials = world.resource::<Assets<ColorMaterial>>();
    let texture = materials.get(&material).unwrap().texture.clone();
    let texture = texture.expect("the fog's material has a texture");
    assert_eq!(
        texture,
        world.resource::<FogAssets>().texture,
        "it shows the texture written"
    );
    let image = world.resource::<Assets<Image>>().get(&texture).unwrap();
    let first = *sizes.entry(texture.id()).or_insert(image.size());
    assert_eq!(
        image.size(),
        first,
        "the image the fog's material shows changed size: stale on the GPU"
    );
    if world.resource::<FogOfWar>().applies(*world.resource::<ClientMode>()) {
        let sight = Sight::new(world.resource::<SimRes>(), P1, true);
        let texels = fog_texels(&sight, image.width(), image.height());
        assert!(image.data.as_ref() == Some(&texels), "it holds player 1's fog");
    }
    image.size()
}

/// Queues `actions` and runs one frame, which applies them.
fn act(app: &mut App, actions: &[EditorAction]) {
    let mut queue = app.world_mut().resource_mut::<EditorActions>();
    queue.0.extend(actions.iter().cloned());
    app.update();
}

/// The editor shows no fog, its texture the 1 × 1 stand-in its material was made with; a play-test shows the map's
/// fog, and one after a resize the new size's, each time in a new texture that the material shows.
#[test]
fn a_play_test_of_another_map_size_shows_a_new_fog_texture() {
    let config = EditorConfig {
        path: None,
        maps_dir: std::env::temp_dir().join(format!("sc_fog_texture_test_{}", std::process::id())),
        script: Vec::new(),
    };
    let mut app = headless(AppConfig::editor(scenarios::empty(24, 16), config));
    app.update();
    let mut sizes = BTreeMap::new();
    assert_eq!(shown_fog(&mut app, &mut sizes), UVec2::ONE, "no fog in the editor");
    act(&mut app, &[EditorAction::PlayTest]);
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 5, 5);
    run_until(&mut app, 1);
    assert_eq!(shown_fog(&mut app, &mut sizes), UVec2::new(24, 16));
    act(&mut app, &[EditorAction::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    let resize = EditorAction::Resize { width: 30, height: 20 };
    act(&mut app, &[resize, EditorAction::PlayTest]);
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 25, 15);
    run_until(&mut app, 1);
    assert_eq!(shown_fog(&mut app, &mut sizes), UVec2::new(30, 20));
}

/// The fog turned on in a game started without it (`--no-fog`, then F3) shows the map's fog in a texture of its
/// size, not in the 1 × 1 stand-in its material was made with.
#[test]
fn the_fog_turned_on_mid_game_shows_a_texture_of_the_maps_size() {
    let mut config = AppConfig::new(scenarios::empty(24, 16));
    config.qa.no_fog = true;
    let mut app = headless(config);
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 5, 5);
    app.update();
    let mut sizes = BTreeMap::new();
    assert_eq!(shown_fog(&mut app, &mut sizes), UVec2::ONE, "no fog yet");
    tap(&mut app, KeyCode::F3);
    assert_eq!(shown_fog(&mut app, &mut sizes), UVec2::new(24, 16));
}
