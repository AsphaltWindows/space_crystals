//! Where the HUD Bar's sections, the SelectionPanel's portraits and the CommandPanel's buttons are on the surface,
//! in logical pixels from its top-left corner. The HUD places its nodes with these rectangles, and a click finds its
//! portrait or button with them, so both agree without UI picking (headless tests have none).

use bevy::math::{Rect, Vec2};

use super::{HUD_BAR_HEIGHT_VH, HUD_GAP_VH, HudSection, SECTION_HEIGHT_VH, SlotWidth};
use crate::input::CommandSlot;

/// A section's border, in logical pixels.
pub const SECTION_BORDER_PX: f32 = 1.0;
/// Padding inside a section, in percent of the surface height.
pub const SECTION_PADDING_VH: f32 = 0.5;
/// Side of a SelectionPanel portrait, and the gap between two (placeholder sizes).
pub const PORTRAIT_VH: f32 = 5.2;
pub const PORTRAIT_GAP_VH: f32 = 0.5;
/// The gap between two CommandPanel buttons (placeholder size).
pub const COMMAND_GAP_VH: f32 = 0.5;

/// `systems/controls/selection_and_hud.md > HUD Bar > Layout Rules`: the rectangle of `section` on a surface of
/// `surface` logical pixels. The sections sit side by side in the bottom bar, all as tall as the bar minus its
/// padding; fixed slots keep their size and the others share the width left over, so a slot depends on the surface
/// alone, never on what the panels show.
pub fn section_rect(section: HudSection, surface: Vec2) -> Rect {
    let vh = surface.y / 100.0;
    let gap = HUD_GAP_VH * vh;
    let top = surface.y - HUD_BAR_HEIGHT_VH * vh + gap;
    let height = SECTION_HEIGHT_VH * vh;
    let (mut fixed, mut shares) = (0.0, 0.0);
    for s in HudSection::ALL {
        match s.width() {
            SlotWidth::Fixed(w) => fixed += w * vh,
            SlotWidth::Share(share) => shares += share,
        }
    }
    let gaps = gap * (HudSection::ALL.len() + 1) as f32;
    let free = (surface.x - gaps - fixed).max(0.0);
    let mut x = gap;
    for s in HudSection::ALL {
        let width = match s.width() {
            SlotWidth::Fixed(w) => w * vh,
            SlotWidth::Share(share) if shares > 0.0 => free * share / shares,
            SlotWidth::Share(_) => 0.0,
        };
        if s == section {
            return Rect::new(x, top, x + width, top + height);
        }
        x += width + gap;
    }
    unreachable!("every section is in HudSection::ALL")
}

/// The grid of portraits inside the SelectionPanel (`systems/controls/selection_and_hud.md > SelectionPanel`): square
/// cells, row by row from the top left of the section's content box.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PortraitGrid {
    /// The first cell's top-left corner on the surface.
    pub origin: Vec2,
    /// A cell's side.
    pub cell: f32,
    /// From one cell to the next: its side plus the gap.
    pub step: f32,
    pub columns: usize,
    pub rows: usize,
}

impl PortraitGrid {
    /// The grid on a surface of `surface` logical pixels.
    pub fn on(surface: Vec2) -> Self {
        let vh = surface.y / 100.0;
        let inset = SECTION_BORDER_PX + SECTION_PADDING_VH * vh;
        let content = section_rect(HudSection::SelectionPanel, surface).inflate(-inset);
        let cell = PORTRAIT_VH * vh;
        let step = cell + PORTRAIT_GAP_VH * vh;
        let fit = |length: f32| ((length + step - cell) / step).floor().max(0.0) as usize;
        Self {
            origin: content.min,
            cell,
            step,
            columns: fit(content.width()),
            rows: fit(content.height()),
        }
    }

    /// How many portraits fit.
    pub fn capacity(&self) -> usize {
        self.columns * self.rows
    }

    /// Cell `index`'s rectangle on the surface.
    pub fn cell_rect(&self, index: usize) -> Rect {
        let columns = self.columns.max(1);
        let at = Vec2::new((index % columns) as f32, (index / columns) as f32) * self.step;
        Rect::from_corners(self.origin + at, self.origin + at + Vec2::splat(self.cell))
    }

    /// The cell under the surface point `point`, if any.
    pub fn index_at(&self, point: Vec2) -> Option<usize> {
        (0..self.capacity()).find(|&i| self.cell_rect(i).contains(point))
    }
}

/// The CommandPanel's 4 × 3 grid of buttons (`systems/controls/selection_and_hud.md > CommandPanel > Grid Layout`):
/// square cells as large as the section's content box allows, centred in it, row by row from the top left.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CommandGrid {
    /// The top-left button's top-left corner on the surface.
    pub origin: Vec2,
    /// A button's side.
    pub cell: f32,
    /// From one button to the next: its side plus the gap.
    pub step: f32,
}

impl CommandGrid {
    /// The grid on a surface of `surface` logical pixels.
    pub fn on(surface: Vec2) -> Self {
        let vh = surface.y / 100.0;
        let inset = SECTION_BORDER_PX + SECTION_PADDING_VH * vh;
        let content = section_rect(HudSection::CommandPanel, surface).inflate(-inset);
        let gap = COMMAND_GAP_VH * vh;
        let (columns, rows) = (f32::from(CommandSlot::COLUMNS), f32::from(CommandSlot::ROWS));
        let cell = ((content.width() - gap * (columns - 1.0)) / columns)
            .min((content.height() - gap * (rows - 1.0)) / rows)
            .max(0.0);
        let step = cell + gap;
        let size = Vec2::new(step * columns - gap, step * rows - gap);
        Self {
            origin: content.center() - size / 2.0,
            cell,
            step,
        }
    }

    /// The rectangle of `slot`'s button on the surface.
    pub fn slot_rect(&self, slot: CommandSlot) -> Rect {
        let at = self.origin + Vec2::new(f32::from(slot.column()), f32::from(slot.row())) * self.step;
        Rect::from_corners(at, at + Vec2::splat(self.cell))
    }

    /// The slot whose button is under the surface point `point`, if any.
    pub fn slot_at(&self, point: Vec2) -> Option<CommandSlot> {
        CommandSlot::all().find(|&slot| self.slot_rect(slot).contains(point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SURFACE: Vec2 = Vec2::new(1280.0, 720.0);

    #[test]
    fn sections_fill_the_bar_left_to_right_with_equal_gaps() {
        let rects: Vec<Rect> = HudSection::ALL.iter().map(|&s| section_rect(s, SURFACE)).collect();
        let gap = HUD_GAP_VH * 7.2;
        assert!((rects[0].min.x - gap).abs() < 1e-3);
        assert!((rects[4].max.x - (SURFACE.x - gap)).abs() < 1e-3, "{:?}", rects[4]);
        for pair in rects.windows(2) {
            assert!((pair[1].min.x - pair[0].max.x - gap).abs() < 1e-3);
            assert_eq!(pair[0].min.y, pair[1].min.y);
            assert_eq!(pair[0].height(), pair[1].height());
        }
        // The two shared slots split the width left over; the command panel is a 4 × 3 grid of squares.
        assert!((rects[2].width() - rects[3].width()).abs() < 1e-3);
        assert!((rects[4].width() / rects[4].height() - 4.0 / 3.0).abs() < 1e-4);
        assert!((rects[0].max.y - (SURFACE.y - gap)).abs() < 1e-3);
    }

    #[test]
    fn portraits_fill_the_selection_panel_row_by_row() {
        let grid = PortraitGrid::on(SURFACE);
        let panel = section_rect(HudSection::SelectionPanel, SURFACE);
        assert!(grid.capacity() >= 12, "{grid:?}");
        let last = grid.cell_rect(grid.capacity() - 1);
        assert!(
            panel.contains(grid.cell_rect(0).min) && panel.contains(last.max),
            "{panel:?} {last:?}"
        );
        assert_eq!(grid.cell_rect(1).min.y, grid.cell_rect(0).min.y);
        assert_eq!(grid.cell_rect(grid.columns).min.x, grid.cell_rect(0).min.x);
        assert_eq!(grid.index_at(grid.cell_rect(5).center()), Some(5));
        let between = grid.cell_rect(0).max + Vec2::splat(grid.step - grid.cell) / 2.0;
        assert_eq!(grid.index_at(between), None, "the gap between cells");
        assert_eq!(grid.index_at(Vec2::new(5.0, 5.0)), None);
    }

    #[test]
    fn command_buttons_fill_the_command_panel_four_by_three() {
        let grid = CommandGrid::on(SURFACE);
        let panel = section_rect(HudSection::CommandPanel, SURFACE);
        let slot = |row, column| CommandSlot::new(row, column).unwrap();
        let (first, last) = (grid.slot_rect(slot(0, 0)), grid.slot_rect(slot(2, 3)));
        assert!(
            panel.contains(first.min) && panel.contains(last.max),
            "{panel:?} {first:?} {last:?}"
        );
        assert!(grid.cell > panel.height() / 4.0, "{grid:?}");
        assert_eq!(grid.slot_rect(slot(0, 1)).min.y, first.min.y);
        assert_eq!(grid.slot_rect(slot(1, 0)).min.x, first.min.x);
        for s in CommandSlot::all() {
            assert_eq!(grid.slot_at(grid.slot_rect(s).center()), Some(s));
        }
        let between = first.max + Vec2::splat(grid.step - grid.cell) / 2.0;
        assert_eq!(grid.slot_at(between), None, "the gap between buttons");
        assert_eq!(grid.slot_at(Vec2::new(5.0, 5.0)), None);
    }
}
