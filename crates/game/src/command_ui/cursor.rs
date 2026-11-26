//! The pointer's look (`systems/controls/selection_and_hud.md > PointerDisplayType`), placeholders until the art
//! (Q0001).
//!
//! [`PointerLook`] is what the pointer shows this frame: the PointerDisplayType over the map, whether a target is
//! awaited, and how the object under the cursor relates to the local player. Its image is drawn in code and cached
//! per look ([`CursorImages`]): an arrow in DefaultState and a crosshair while a target is awaited, coloured by the
//! PointerDisplayType (`> Types`: Inactive muted grey; Move green, Attack red, Patrol orange as their indicators;
//! AttackGround crimson; GatherResources the cyan of the crystals, ReturnResources gold, slice M3-J2; Enter violet,
//! slice M4-J1; Assist blue and Repair tan, slice M5-J1 (design change 0322: placeholder looks; Repair's shows from
//! slice N8 on); Mixed, "a generic hybrid cursor" whose visual is TBD, striped in the Attack red and the Move green),
//! with a dot in the
//! relation's colour (own green, enemy red, neutral yellow, as the selection markers) while
//! it points at an object. With nothing to preview (Inactive, no object, no target awaited) and over the UI, a window
//! shows the system's arrow. While a building is placed there is no pointer over the map (`> Resolution Rules >
//! AwaitingPlacement`: "No pointer display type — the building ghost preview serves as the cursor";
//! `crate::placement`): a window hides its cursor there.
//!
//! A window takes the image as its cursor (`CursorIcon`, Bevy's custom cursors). QA runs ignore the real mouse and
//! render offscreen, where no system cursor is drawn: there the image is a UI node at the injected cursor
//! ([`CursorSprite`]), so screenshots show it.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{CursorIcon, CursorOptions, CustomCursor, CustomCursorImage, PrimaryWindow, SystemCursorIcon};

use super::indicators::{AGGRESSIVE_COLOR, ATTACK_COLOR, GATHER_COLOR, MOVE_COLOR, RETURN_COLOR};
use super::interface::{Modes, commandable};
use super::resolution::{MapTarget, PointerDisplay, UnitCapabilities, capabilities, pointer_display};
use crate::fog::LocalSight;
use crate::input::{ActionState, RawInput};
use crate::selection::markers::relation_color;
use crate::selection::{ControlStates, Relation, Selectables};
use crate::sim_driver::RenderAlpha;
use crate::{ClientMode, ClientSystems};

/// The side of a cursor image, pixels.
pub const CURSOR_SIZE: u32 = 32;
/// The cursor sprite draws above every other UI node.
const SPRITE_Z: i32 = 10_000;
/// Placeholder colours of the PointerDisplayTypes without an indicator colour. GatherResources and ReturnResources
/// have the Gather marker's cyan and the Return Resources marker's gold (`super::indicators::gathering`).
const INACTIVE_COLOR: Color = Color::srgb(0.80, 0.82, 0.86);
const ATTACK_GROUND_COLOR: Color = Color::srgb(0.90, 0.20, 0.50);
/// Enter's (slice M4-J1), a placeholder: the violet of nothing else on the map.
pub const ENTER_COLOR: Color = Color::srgb(0.62, 0.42, 0.98);
/// Assist's (slice M5-J1, design change 0322: "Two new pointer types (placeholder looks)"), a placeholder: a
/// chartreuse of nothing else on the map (player 1's blue would have hidden it).
pub const ASSIST_COLOR: Color = Color::srgb(0.78, 1.00, 0.20);
/// Repair's (slice M5-J1, the hook of slice N8's Repair, design change 0322), a placeholder: a tan of nothing else on
/// the map.
pub const REPAIR_COLOR: Color = Color::srgb(0.80, 0.62, 0.40);
/// The Mixed look (`> Mixed Right-Click Preview`, design change 0063: visual TBD), a placeholder: diagonal stripes
/// of these two colours, each this many pixels wide.
const MIXED_COLORS: [Color; 2] = [ATTACK_COLOR, MOVE_COLOR];
const MIXED_STRIPE_PX: f32 = 4.0;
const OUTLINE: [u8; 4] = [16, 16, 20, 255];

/// What the pointer shows this frame.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerLook {
    /// `> PointerDisplayType` over the map; `None` over the UI, off the world view and outside the game modes.
    pub display: Option<PointerDisplay>,
    /// A target is awaited (`> AwaitingTarget`): a crosshair instead of an arrow.
    pub targeting: bool,
    /// A building is placed (`> AwaitingPlacement`) and the cursor is over the map: no pointer, the ghost is the
    /// cursor (`crate::placement`). Not for a Warp In, whose ghost stands on its Beacon (design change 0121).
    pub placing: bool,
    /// How the object under the cursor relates to the local player; none while a building is placed.
    pub hovered: Option<Relation>,
}

impl PointerLook {
    /// Nothing to preview: a plain arrow, the system's in a window.
    pub fn is_plain(&self) -> bool {
        matches!(self.display, None | Some(PointerDisplay::Inactive))
            && !self.targeting
            && !self.placing
            && self.hovered.is_none()
    }

    /// The pixel of its image that points.
    pub fn hotspot(&self) -> (u16, u16) {
        if self.targeting { (16, 16) } else { (1, 1) }
    }
}

/// The cursor images made so far, one per look.
#[derive(Resource, Default)]
pub struct CursorImages {
    images: HashMap<PointerLook, Handle<Image>>,
}

impl CursorImages {
    /// The image of `look`, made the first time it is needed.
    fn get(&mut self, look: PointerLook, assets: &mut Assets<Image>) -> Handle<Image> {
        self.images
            .entry(look)
            .or_insert_with(|| assets.add(cursor_image(look)))
            .clone()
    }
}

/// The UI node that draws the cursor in QA runs.
#[derive(Component)]
pub struct CursorSprite;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerLook>()
            .init_resource::<CursorImages>()
            .add_systems(Startup, spawn_cursor_sprite)
            .add_systems(
                Update,
                (update_pointer_look, set_window_cursor, place_cursor_sprite)
                    .chain()
                    .in_set(ClientSystems::Present),
            );
    }
}

/// `> PointerDisplayType > Resolution Rules`: from the local player's ControlState and what is under the cursor,
/// among the objects it may select and the records of those it remembers, which it may target (design change 0139):
/// the fog of war hides the others (`crate::fog`). In DefaultState each selected unit resolves the right-click for
/// itself, with its mode (the Riot Shield), and the pointer shows the one thing they would all do, or Mixed (design
/// change 0063).
fn update_pointer_look(
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    sight: LocalSight,
    alpha: Res<RenderAlpha>,
    states: Res<ControlStates>,
    mut look: ResMut<PointerLook>,
) {
    let pointer = actions.pointer();
    let new = match (pointer.view, pointer.ground) {
        (Some(view), Some(ground)) if mode.is_game() && !pointer.over_ui => {
            let (sim, sight) = (sight.sim(), sight.get());
            let player = sight.player();
            let state = states.of(player);
            let objects = Selectables::new(sim, &sight, alpha.0);
            let units: Vec<UnitCapabilities> = if commandable(sim, player, &state.selection) {
                let modes = Modes::of(sim, &state.selection);
                capabilities(&state.selection, &modes)
                    .into_iter()
                    .map(|(_, unit)| unit)
                    .collect()
            } else {
                Vec::new()
            };
            let target = MapTarget::at(sim, player, &objects, view, ground);
            let placing = state
                .interface
                .placing()
                .is_some_and(|placing| placing.follows_cursor());
            PointerLook {
                display: Some(pointer_display(state.interface, &units, &target)),
                targeting: state.interface.pending().is_some(),
                placing,
                hovered: objects.target_at(view).filter(|_| !placing).map(|o| o.relation),
            }
        }
        _ => PointerLook::default(),
    };
    look.set_if_neq(new);
}

/// The primary window's cursor: the window, its cursor image and whether it shows.
type WindowCursor = (Entity, Option<&'static CursorIcon>, Option<&'static mut CursorOptions>);

/// A window shows the look as its cursor while the real mouse drives the game, and hides it while a building is
/// placed over the map.
fn set_window_cursor(
    mut commands: Commands,
    raw: Res<RawInput>,
    look: Res<PointerLook>,
    mut images: ResMut<CursorImages>,
    mut assets: ResMut<Assets<Image>>,
    mut windows: Query<WindowCursor, With<PrimaryWindow>>,
) {
    if !raw.enabled {
        return;
    }
    let Ok((window, current, options)) = windows.single_mut() else {
        return;
    };
    if let Some(mut options) = options
        && options.visible == look.placing
    {
        options.visible = !look.placing;
    }
    if look.placing {
        return;
    }
    let icon = if look.is_plain() {
        CursorIcon::System(SystemCursorIcon::Default)
    } else {
        CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: images.get(*look, &mut assets),
            hotspot: look.hotspot(),
            ..default()
        }))
    };
    if current != Some(&icon) {
        commands.entity(window).insert(icon);
    }
}

fn spawn_cursor_sprite(mut commands: Commands) {
    commands.spawn((
        Name::new("Cursor sprite"),
        CursorSprite,
        Node {
            position_type: PositionType::Absolute,
            width: px(CURSOR_SIZE as f32),
            height: px(CURSOR_SIZE as f32),
            ..default()
        },
        ImageNode::default(),
        GlobalZIndex(SPRITE_Z),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

/// QA runs: the look as a UI node at the injected cursor, in the game modes; none while a building is placed over
/// the map.
fn place_cursor_sprite(
    raw: Res<RawInput>,
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    look: Res<PointerLook>,
    mut images: ResMut<CursorImages>,
    mut assets: ResMut<Assets<Image>>,
    mut sprite: Query<(&mut Node, &mut ImageNode, &mut Visibility), With<CursorSprite>>,
) {
    let Ok((mut node, mut image, mut visibility)) = sprite.single_mut() else {
        return;
    };
    let Some(cursor) = actions
        .cursor()
        .filter(|_| !raw.enabled && mode.is_game() && !look.placing)
    else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    let handle = images.get(*look, &mut assets);
    if image.image != handle {
        image.image = handle;
    }
    let (x, y) = look.hotspot();
    let (left, top) = (px(cursor.x - f32::from(x)), px(cursor.y - f32::from(y)));
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    visibility.set_if_neq(Visibility::Inherited);
}

/// The colour of a PointerDisplayType at the pixel `p`: one colour per type, and Mixed's stripes.
fn display_color(display: Option<PointerDisplay>, p: Vec2) -> Color {
    match display {
        None | Some(PointerDisplay::Inactive) => INACTIVE_COLOR,
        Some(PointerDisplay::Move) => MOVE_COLOR,
        Some(PointerDisplay::Attack) => ATTACK_COLOR,
        Some(PointerDisplay::AttackGround) => ATTACK_GROUND_COLOR,
        Some(PointerDisplay::Patrol) => AGGRESSIVE_COLOR,
        Some(PointerDisplay::GatherResources) => GATHER_COLOR,
        Some(PointerDisplay::ReturnResources) => RETURN_COLOR,
        Some(PointerDisplay::Enter) => ENTER_COLOR,
        Some(PointerDisplay::Assist) => ASSIST_COLOR,
        Some(PointerDisplay::Repair) => REPAIR_COLOR,
        Some(PointerDisplay::Mixed) => {
            let stripe = ((p.x + p.y) / MIXED_STRIPE_PX).floor().rem_euclid(2.0);
            MIXED_COLORS[usize::from(stripe >= 1.0)]
        }
    }
}

/// The classic arrow, tip at the top left (pixels, y down).
const ARROW: [Vec2; 7] = [
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 24.0),
    Vec2::new(7.0, 19.0),
    Vec2::new(11.0, 28.0),
    Vec2::new(15.0, 26.0),
    Vec2::new(11.0, 17.0),
    Vec2::new(18.0, 17.0),
];

/// Draws `look`'s image: its shape in its colour inside a dark outline, and the hovered object's dot.
pub fn cursor_image(look: PointerLook) -> Image {
    let badge = look.hovered.map(|r| relation_color(r).to_srgba().to_u8_array());
    let mut pixels = Vec::with_capacity((CURSOR_SIZE * CURSOR_SIZE * 4) as usize);
    for y in 0..CURSOR_SIZE {
        for x in 0..CURSOR_SIZE {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let (core, rim) = if look.targeting { crosshair(p) } else { arrow(p) };
            let mut pixel = if core {
                display_color(look.display, p).to_srgba().to_u8_array()
            } else if rim {
                OUTLINE
            } else {
                [0; 4]
            };
            if let Some(badge) = badge {
                let d = p.distance(Vec2::new(26.0, 26.0));
                if d <= 4.0 {
                    pixel = badge;
                } else if d <= 5.5 {
                    pixel = OUTLINE;
                }
            }
            pixels.extend(pixel);
        }
    }
    let size = Extent3d {
        width: CURSOR_SIZE,
        height: CURSOR_SIZE,
        depth_or_array_layers: 1,
    };
    // Kept in the main world too: a window's cursor is made from its pixels there.
    Image::new(
        size,
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// The arrow's fill and outline at `p`.
fn arrow(p: Vec2) -> (bool, bool) {
    let inside = inside(&ARROW, p);
    let edge = edge_distance(&ARROW, p);
    (inside && edge > 1.2, inside || edge <= 1.0)
}

/// The crosshair's fill and outline at `p`: a ring and four arms around the hotspot, and a dot in the middle.
fn crosshair(p: Vec2) -> (bool, bool) {
    let d = p - Vec2::splat(16.0);
    let r = d.length();
    let ring = (r - 9.0).abs();
    let (along, across) = (d.x.abs().max(d.y.abs()), d.x.abs().min(d.y.abs()));
    let arm = (3.5..=14.5).contains(&along) && across <= 1.0;
    let arm_rim = (2.5..=15.5).contains(&along) && across <= 2.0;
    (ring <= 1.2 || arm || r <= 1.5, ring <= 2.2 || arm_rim || r <= 2.5)
}

/// Whether `p` lies inside the polygon (even-odd rule).
fn inside(polygon: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    for (i, &a) in polygon.iter().enumerate() {
        let b = polygon[(i + 1) % polygon.len()];
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            inside = !inside;
        }
    }
    inside
}

/// How far `p` is from the polygon's outline.
fn edge_distance(polygon: &[Vec2], p: Vec2) -> f32 {
    polygon
        .iter()
        .enumerate()
        .map(|(i, &a)| {
            let b = polygon[(i + 1) % polygon.len()];
            let t = ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
            p.distance(a + (b - a) * t)
        })
        .fold(f32::INFINITY, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
        let data = image.data.as_ref().unwrap();
        let i = ((y * CURSOR_SIZE + x) * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    }

    #[test]
    fn each_look_draws_its_shape_colour_and_dot() {
        let attack = PointerLook {
            display: Some(PointerDisplay::Attack),
            targeting: true,
            placing: false,
            hovered: None,
        };
        let image = cursor_image(attack);
        assert_eq!(pixel(&image, 25, 16), ATTACK_COLOR.to_srgba().to_u8_array(), "the ring");
        assert_eq!(pixel(&image, 2, 2)[3], 0, "no arrow");
        let over_enemy = PointerLook {
            display: Some(PointerDisplay::Attack),
            targeting: false,
            placing: false,
            hovered: Some(Relation::Enemy),
        };
        let image = cursor_image(over_enemy);
        assert_eq!(pixel(&image, 3, 8), ATTACK_COLOR.to_srgba().to_u8_array(), "the arrow");
        assert_eq!(
            pixel(&image, 26, 26),
            relation_color(Relation::Enemy).to_srgba().to_u8_array()
        );
        assert_eq!(pixel(&image, 30, 2)[3], 0, "transparent around");
        assert!(PointerLook::default().is_plain() && !over_enemy.is_plain());
        assert_eq!(attack.hotspot(), (16, 16));
    }

    /// The Mixed placeholder: an arrow in stripes of the Attack red and the Move green.
    #[test]
    fn the_mixed_look_is_striped() {
        let mixed = PointerLook {
            display: Some(PointerDisplay::Mixed),
            targeting: false,
            placing: false,
            hovered: Some(Relation::Enemy),
        };
        let image = cursor_image(mixed);
        assert_eq!(pixel(&image, 3, 6), ATTACK_COLOR.to_srgba().to_u8_array());
        assert_eq!(pixel(&image, 3, 8), MOVE_COLOR.to_srgba().to_u8_array());
        assert!(!mixed.is_plain());
    }

    /// Slice M5-J1 (design change 0322: "Two new pointer types (placeholder looks)"): Assist and Repair are arrows of
    /// their own colours.
    #[test]
    fn assist_and_repair_have_their_own_colours() {
        let looks = [
            (PointerDisplay::Assist, ASSIST_COLOR),
            (PointerDisplay::Repair, REPAIR_COLOR),
        ];
        for (display, color) in looks {
            let look = PointerLook {
                display: Some(display),
                targeting: false,
                placing: false,
                hovered: Some(Relation::Own),
            };
            let image = cursor_image(look);
            assert_eq!(pixel(&image, 3, 8), color.to_srgba().to_u8_array(), "{display:?}");
        }
        assert_ne!(ASSIST_COLOR, REPAIR_COLOR);
    }
}
