//! Where the minimap draws the map on the surface, and how its points map to the map's: one function each way,
//! [`MinimapFrame::ground_at`] (a point of the panel to the ground point it shows) and [`MinimapFrame::screen_at`]
//! (a ground point to where the panel shows it), and the camera's rectangle from them
//! ([`MinimapFrame::camera_rect`]). Pure geometry, from the surface size and the map's size alone, like the HUD's
//! other rectangles (`super::super::layout`): a click finds its ground point without UI picking, which headless tests
//! have none of.

use bevy::math::{Rect, UVec2, Vec2};

use crate::hud::HudSection;
use crate::hud::layout::{SECTION_BORDER_PX, SECTION_PADDING_VH, section_rect};
use crate::platform::SurfaceSize;

/// The minimap's map area: the whole map with its aspect kept, as large as fits in the Minimap section's content box
/// and centred there, so a map that isn't square leaves bars across its short side (letterboxed). Its corners sit on
/// whole physical pixels and its image has one texel per physical pixel, so nothing is resampled.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MinimapFrame {
    /// The map's area on the surface, in logical pixels from its top-left corner.
    pub area: Rect,
    /// The image's size in texels: the area's in physical pixels.
    pub texels: UVec2,
    /// The map's size in space units ([`crate::camera::map_extent_su`]).
    pub map: Vec2,
}

impl MinimapFrame {
    /// The frame of a map of `map` space units on a surface of `size`, in the Minimap section
    /// (`systems/controls/hud.md > HUD Bar`: the leftmost) inside its border and padding.
    pub fn on(size: SurfaceSize, map: Vec2) -> Self {
        let surface = size.logical();
        let inset = SECTION_BORDER_PX + SECTION_PADDING_VH * surface.y / 100.0;
        let content = section_rect(HudSection::Minimap, surface).inflate(-inset);
        Self::in_box(content, size.scale_factor, map)
    }

    /// The frame of a map of `map` space units fitted into `content` (logical pixels), on a surface of `scale`
    /// physical pixels per logical pixel.
    pub fn in_box(content: Rect, scale: f32, map: Vec2) -> Self {
        let map = map.max(Vec2::ONE);
        let physical = content.size().max(Vec2::ZERO) * scale;
        let fit = (physical / map).min_element();
        let texels = (map * fit).floor().max(Vec2::ONE);
        let min = (content.center() * scale - texels / 2.0).round();
        Self {
            area: Rect::from_corners(min / scale, (min + texels) / scale),
            texels: texels.as_uvec2(),
            map,
        }
    }

    /// Physical pixels per logical pixel.
    pub fn scale(&self) -> f32 {
        self.texels.x as f32 / self.area.width()
    }

    /// Whether the cursor at `screen` (logical pixels) is over the map area: where a press is the minimap's.
    pub fn contains(&self, screen: Vec2) -> bool {
        screen.cmpge(self.area.min).all() && screen.cmplt(self.area.max).all()
    }

    /// The ground point the minimap shows at `screen` (logical pixels), kept on the map: a point off the map area, as
    /// a drag goes on outside it, gives the nearest point on the map's edge. The map's far edges are just short of its
    /// size, as `Map::clamp_point` keeps them.
    pub fn ground_at(&self, screen: Vec2) -> Vec2 {
        let ground = (screen - self.area.min) / self.area.size() * self.map;
        Vec2::new(
            ground.x.clamp(0.0, self.map.x.next_down()),
            ground.y.clamp(0.0, self.map.y.next_down()),
        )
    }

    /// Where the minimap shows the ground point `ground`, in logical pixels: [`MinimapFrame::ground_at`]'s inverse on
    /// the map, and off the map area for a point off the map.
    pub fn screen_at(&self, ground: Vec2) -> Vec2 {
        self.area.min + ground / self.map * self.area.size()
    }

    /// Where the image has the ground point `ground`, in texels from its top-left corner: texel (x, y) covers
    /// [x, x + 1) × [y, y + 1).
    pub fn texel_at(&self, ground: Vec2) -> Vec2 {
        ground / self.map * self.texels.as_vec2()
    }

    /// The ground point at the centre of texel `texel`.
    pub fn texel_center(&self, texel: UVec2) -> Vec2 {
        (texel.as_vec2() + 0.5) / self.texels.as_vec2() * self.map
    }

    /// `systems/controls/hud.md > Minimap > What it shows`: "The camera's view as a rectangle outline", in logical
    /// pixels: a rectangle of the world view's extent, `visible` space units ([`crate::camera::visible_extent_su`]),
    /// around `center`, the ground point drawn at the view's centre (`crate::coords::ground_under`). A reading: the
    /// view looks at the ground lifted by its height, so the ground it shows isn't exactly a rectangle; this is the
    /// rectangle the view would show of flat ground there. Cut to the map area, so that each of its sides shows, on
    /// the map's edge where the view goes past it, and its corners snapped to whole physical pixels; `None` for a
    /// view wholly off the map.
    pub fn camera_rect(&self, center: Vec2, visible: Vec2) -> Option<Rect> {
        let ground = Rect::from_center_size(center, visible);
        let shown = Rect::from_corners(self.screen_at(ground.min), self.screen_at(ground.max)).intersect(self.area);
        if shown.is_empty() {
            return None;
        }
        let scale = self.scale();
        let snap = |p: Vec2| (p * scale).round() / scale;
        Some(Rect::from_corners(snap(shown.min), snap(shown.max)))
    }
}
