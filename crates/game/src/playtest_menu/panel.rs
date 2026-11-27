//! The playtest menu's look, a placeholder (the design: "a simple temporary menu"; the brief: "a simple centred panel
//! with buttons, in the HUD's style"): a column of buttons in a panel centred across the surface, in the colours of the
//! HUD's bars and sections. F10's is in the middle of the world view, under its title, PAUSED; the game end's under the
//! outcome banner (`crate::hud::outcome`), which tells the result. The button under the cursor lights up.
//!
//! [`MenuLayout`] places the nodes, and finds the button under a click, so both agree without UI picking (headless
//! tests have none), as `crate::hud::layout` does for the HUD.

use bevy::picking::Pickable;
use bevy::prelude::*;

use super::{MenuButton, PlaytestMenu};
use crate::hud::outcome::BANNER_Z;
use crate::hud::{
    BAR_BACKGROUND, HUD_BAR_HEIGHT_VH, LABEL_COLOR, SECTION_BACKGROUND, SECTION_BORDER, TEXT_COLOR, TOP_BAR_HEIGHT_VH,
};
use crate::input::ActionState;
use crate::platform::SurfaceSize;

/// Placeholder sizes, in percent of the surface height: a button, the gap between two, the panel's padding, the
/// title's row and the texts.
const BUTTON_WIDTH_VH: f32 = 26.0;
const BUTTON_HEIGHT_VH: f32 = 5.0;
const GAP_VH: f32 = 1.2;
const PADDING_VH: f32 = 2.0;
const TITLE_ROW_VH: f32 = 4.4;
const TITLE_TEXT_VH: f32 = 3.6;
const LABEL_TEXT_VH: f32 = 2.4;
/// Where the game end's panel starts, from the top of the surface: under the outcome banner's band, which starts at
/// 36 vh and is about 16 vh tall.
const GAME_END_TOP_VH: f32 = 54.0;
/// The panel's and the buttons' border, in logical pixels.
const BORDER_PX: f32 = 1.0;
/// The button under the cursor (placeholder).
const HOVERED_BACKGROUND: Color = Color::srgb(0.20, 0.23, 0.29);
/// Over the outcome banner, and so over the results screen and the HUD.
const MENU_Z: i32 = BANNER_Z + 1;

/// Where the menu's panel, its title and its buttons are on the surface, in logical pixels from its top-left corner
/// (module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuLayout {
    pub panel: Rect,
    /// The title's row: F10's menu only.
    pub title: Option<Rect>,
    /// The buttons, top down.
    pub buttons: Vec<(MenuButton, Rect)>,
}

impl MenuLayout {
    /// The layout of `menu` on a surface of `surface` logical pixels; `None` while it is closed.
    pub fn of(menu: PlaytestMenu, surface: Vec2) -> Option<Self> {
        let buttons = menu.buttons();
        if buttons.is_empty() {
            return None;
        }
        let titled = menu == PlaytestMenu::Paused;
        let n = buttons.len() as f32;
        let title_vh = if titled { TITLE_ROW_VH + GAP_VH } else { 0.0 };
        let height_vh = 2.0 * PADDING_VH + title_vh + n * BUTTON_HEIGHT_VH + (n - 1.0) * GAP_VH;
        let top_vh = match menu {
            // The middle of the world view, between the top bar and the HUD Bar.
            PlaytestMenu::Paused => (TOP_BAR_HEIGHT_VH + 100.0 - HUD_BAR_HEIGHT_VH - height_vh) / 2.0,
            _ => GAME_END_TOP_VH,
        };
        let vh = surface.y / 100.0;
        let width = (BUTTON_WIDTH_VH + 2.0 * PADDING_VH) * vh;
        let left = (surface.x - width) / 2.0;
        let row = |top_vh: f32, height_vh: f32| {
            Rect::new(
                left + PADDING_VH * vh,
                top_vh * vh,
                left + width - PADDING_VH * vh,
                (top_vh + height_vh) * vh,
            )
        };
        let mut y = top_vh + PADDING_VH;
        let title = titled.then(|| row(y, TITLE_ROW_VH));
        y += title_vh;
        let buttons = (buttons.iter())
            .enumerate()
            .map(|(i, &button)| {
                (
                    button,
                    row(y + i as f32 * (BUTTON_HEIGHT_VH + GAP_VH), BUTTON_HEIGHT_VH),
                )
            })
            .collect();
        Some(Self {
            panel: Rect::new(left, top_vh * vh, left + width, (top_vh + height_vh) * vh),
            title,
            buttons,
        })
    }

    /// The button at `at`, if one is.
    pub fn button_at(&self, at: Vec2) -> Option<MenuButton> {
        (self.buttons.iter())
            .find(|(_, rect)| rect.contains(at))
            .map(|(button, _)| *button)
    }

    /// Where `button` is, if the menu has it.
    pub fn button(&self, button: MenuButton) -> Option<Rect> {
        (self.buttons.iter()).find(|(b, _)| *b == button).map(|(_, rect)| *rect)
    }
}

/// The menu's root: the whole surface, so that its children are placed in surface coordinates.
#[derive(Component)]
pub struct MenuRoot;

/// A part of the menu, placed where [`MenuLayout`] puts it.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum MenuPart {
    /// The panel, behind the title and the buttons.
    Panel,
    /// The title, PAUSED.
    Title,
    Button(MenuButton),
}

fn label(content: &str, size_vh: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(FontSize::Vh(size_vh)),
        TextColor(color),
        Pickable::IGNORE,
    )
}

/// A node placed at the surface's `rect`, its content centred.
fn placed(rect: Rect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(rect.min.x),
        top: px(rect.min.y),
        width: px(rect.width()),
        height: px(rect.height()),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Spawns the menu, hidden.
pub(super) fn spawn_menu(mut commands: Commands) {
    let bordered = |rect| Node {
        border: UiRect::all(px(BORDER_PX)),
        ..placed(rect)
    };
    commands
        .spawn((
            Name::new("Playtest menu"),
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(MENU_Z),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                MenuPart::Panel,
                bordered(Rect::default()),
                BackgroundColor(BAR_BACKGROUND),
                BorderColor::all(SECTION_BORDER),
            ));
            root.spawn((
                MenuPart::Title,
                placed(Rect::default()),
                children![label("PAUSED", TITLE_TEXT_VH, LABEL_COLOR)],
            ));
            for button in MenuButton::ALL {
                root.spawn((
                    Name::new(button.label()),
                    MenuPart::Button(button),
                    bordered(Rect::default()),
                    BackgroundColor(SECTION_BACKGROUND),
                    BorderColor::all(SECTION_BORDER),
                    children![label(button.label(), LABEL_TEXT_VH, TEXT_COLOR)],
                ));
            }
        });
}

/// Moves `node` to `rect`, or hides it (`None`); changes it only when that differs.
fn place(node: &mut Mut<Node>, rect: Option<Rect>) {
    let Some(rect) = rect else {
        if node.display != Display::None {
            node.display = Display::None;
        }
        return;
    };
    let at = placed(rect);
    if (node.display, node.left, node.top, node.width, node.height)
        != (Display::Flex, at.left, at.top, at.width, at.height)
    {
        node.display = Display::Flex;
        node.left = at.left;
        node.top = at.top;
        node.width = at.width;
        node.height = at.height;
    }
}

/// Shows the menu that is open, where [`MenuLayout`] puts it, the button under the cursor lit; hides it while closed.
pub(super) fn show_menu(
    menu: Res<PlaytestMenu>,
    size: Res<SurfaceSize>,
    actions: Res<ActionState>,
    mut root: Query<&mut Visibility, With<MenuRoot>>,
    mut parts: Query<(&MenuPart, &mut Node, &mut BackgroundColor)>,
) {
    let layout = MenuLayout::of(*menu, size.logical());
    for mut visibility in &mut root {
        let shown = if layout.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(shown);
    }
    let Some(layout) = layout else {
        return;
    };
    let hovered = actions.cursor().and_then(|at| layout.button_at(at));
    for (&part, mut node, mut background) in &mut parts {
        let rect = match part {
            MenuPart::Panel => Some(layout.panel),
            MenuPart::Title => layout.title,
            MenuPart::Button(button) => layout.button(button),
        };
        place(&mut node, rect);
        if let MenuPart::Button(button) = part {
            let color = if hovered == Some(button) {
                HOVERED_BACKGROUND
            } else {
                SECTION_BACKGROUND
            };
            background.set_if_neq(BackgroundColor(color));
        }
    }
}
