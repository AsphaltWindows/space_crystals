//! (M5-N4b) The Shroud clouds in the client (`shroud_view`; `factions/cults/units/disciple.md > Spells > Shroud`: "the
//! cloud is shown to **every player**, over the tiles that are Visible to them, so its edge can be seen even when the
//! Disciple can't"), and the flash of a release; a box of another size is a new texture that the layer shows (slice
//! M5-Z3). The texels themselves are the unit tests' (`shroud_view/tests.rs`).

use std::collections::BTreeMap;

use sc_sim::VisibilityState;
use space_crystals::shroud_view::{ShroudAssets, ShroudLayer, ShroudView, TEXELS_PER_TILE};

use super::*;

const DISCIPLE: ObjectId = ObjectId(1);
/// Player 2's Militants: the one that watches the cloud's edge, and the one that shoots the Disciple.
const WATCHER: ObjectId = ObjectId(2);
const SHOOTER: ObjectId = ObjectId(3);

fn shown_tiles(app: &App) -> Vec<TilePos> {
    app.world().resource::<ShroudView>().tiles.clone()
}

fn layer_visible(app: &mut App) -> bool {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<ShroudLayer>>();
    *query.single(app.world()).unwrap() != Visibility::Hidden
}

/// The size of the image the layer's material shows, which must be the texture the layer writes and never have
/// changed size (`sizes`: each image's first size). Bevy 0.20 keeps the GPU texture a 2D material was prepared with,
/// so an image resized under its handle stays on screen at its old size (`ARCHITECTURE.md > Bevy 0.20: verify, don't
/// recall`; slice M5-Z3).
fn shown_size(app: &mut App, sizes: &mut BTreeMap<AssetId<Image>, UVec2>) -> UVec2 {
    let mut layer = app
        .world_mut()
        .query_filtered::<&MeshMaterial2d<ColorMaterial>, With<ShroudLayer>>();
    let material = layer.single(app.world()).unwrap().0.clone();
    let world = app.world();
    let texture = world
        .resource::<Assets<ColorMaterial>>()
        .get(&material)
        .unwrap()
        .texture
        .clone();
    let texture = texture.expect("the layer's material has a texture");
    assert_eq!(
        texture,
        world.resource::<ShroudAssets>().texture,
        "it shows the texture written"
    );
    let size = world.resource::<Assets<Image>>().get(&texture).unwrap().size();
    assert_eq!(
        *sizes.entry(texture.id()).or_insert(size),
        size,
        "an image the layer shows changed size"
    );
    size
}

fn sight_of(app: &App, player: PlayerId) -> Vec<TilePos> {
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, player, true);
    shown_tiles(app)
        .into_iter()
        .filter(|&tile| sight.tile(tile) == VisibilityState::Visible)
        .collect()
}

/// Player 1's Disciple channels at the centre of tile (5, 6), its cloud 3 grid units round it; player 2's Militant
/// stands 7 grid units east, its sight of 5 reaching the cloud's east edge. Player 1 sees the whole cloud; player 2,
/// shown the game as the local player, sees its edge alone, over the tiles Visible to it. Another Militant of player
/// 2's, 3.6 grid units east of the Disciple, shoots it from outside the cloud, which releases 8 ticks after its first
/// hit (`sc_sim::shroud`) and flashes for `LookTuning::release_ticks`. (M6-CM) With the channel stopped, the circle
/// lingers where it was, shown as it was, for `Spells::linger` ticks (design change 0448: "the cloud **stays for 1.5 s
/// (24 frames, placeholder)**"), then vanishes; its flash faded, the layer hides.
#[test]
fn a_cloud_shows_over_each_player_s_visible_tiles_and_a_release_flashes() {
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    assert!(!layer_visible(&mut app), "no cloud");
    let mut sizes = BTreeMap::new();
    let tile = TEXELS_PER_TILE as u32;
    assert_eq!(shown_size(&mut app, &mut sizes), UVec2::splat(tile), "the stand-in");
    let centre = TilePos::new(5, 6).center();
    spawn(&mut app, ObjectKind::Disciple, P1, centre);
    spawn(
        &mut app,
        ObjectKind::Militant,
        P2,
        centre + Vec2::new(7.0 * SU_PER_GU, 0.0),
    );
    app.update();
    let order = |kind| Command::Order {
        units: vec![DISCIPLE],
        kind,
        queue: false,
    };
    submit(&mut app, P1, order(OrderKind::Shroud));
    app.update();
    app.update();
    assert!(layer_visible(&mut app));
    let own = shown_tiles(&app);
    assert_eq!(
        own.len(),
        7 * 7 - 4,
        "the tiles of its circle, its box but the corners: {own:?}"
    );
    assert_eq!(sight_of(&app, P1), own, "all Visible to player 1");
    assert_eq!(
        shown_size(&mut app, &mut sizes),
        UVec2::splat(7 * tile),
        "a new texture for its box"
    );

    app.world_mut().resource_mut::<LocalPlayer>().0 = P2;
    app.update();
    let edge = shown_tiles(&app);
    assert!(!edge.is_empty() && edge.len() < own.len() / 3, "its edge: {edge:?}");
    assert_eq!(sight_of(&app, P2), edge, "only over player 2's Visible tiles");
    assert!(edge.iter().all(|tile| tile.x >= 7), "its east edge: {edge:?}");

    spawn(&mut app, ObjectKind::Militant, P2, centre + Vec2::new(230.0, 0.0));
    app.update();
    attack(&mut app, P2, SHOOTER, DISCIPLE);
    let writes = app.world().resource::<ShroudView>().texture_writes;
    let mut released = None;
    for _ in 0..40 {
        app.update();
        let sim = app.world().resource::<SimRes>();
        if released.is_none()
            && sim
                .events()
                .iter()
                .any(|e| matches!(e, SimEvent::ShroudReleased { .. }))
        {
            released = Some(sim.tick());
        }
    }
    let released = released.expect("a release");
    assert!(released > 8, "8 ticks after the first hit");
    let view = app.world().resource::<ShroudView>();
    assert!(view.texture_writes > writes + 1, "the flash fades tick by tick");

    app.world_mut().resource_mut::<LocalPlayer>().0 = P1;
    submit(&mut app, P1, order(OrderKind::Stop));
    submit(
        &mut app,
        P2,
        Command::Order {
            units: vec![WATCHER, SHOOTER],
            kind: OrderKind::Stop,
            queue: false,
        },
    );
    app.update();
    let linger = app.world().resource::<SimRes>().rules().tuning.spells.linger;
    for _ in 1..linger {
        app.update();
        assert!(!app.world().resource::<SimRes>().clouds().is_empty(), "lingering");
        assert!(layer_visible(&mut app));
        assert_eq!(shown_tiles(&app), own, "where it was, as it was");
    }
    for _ in 0..space_crystals::tuning::look::BUILTIN.release_ticks + 2 {
        app.update();
    }
    assert!(app.world().resource::<SimRes>().clouds().is_empty());
    assert!(!layer_visible(&mut app), "nothing to show");
    assert!(shown_tiles(&app).is_empty());
}
