//! An ASCII picture of the map with its units, for readable failures (`docs/movement.md > 8`).

use std::fmt::Write as _;

use glam::Vec2;

use crate::sim::Sim;
use crate::units::{SU_PER_GU, TilePos};

/// Characters per tile across and down: a character cell is 16 × 32 su, roughly square on a terminal.
const CELLS: (i32, i32) = (4, 2);
/// The picture shows at most this many tiles; a larger map is cut to a window around the focus.
const WINDOW: (i32, i32) = (30, 20);

/// An ASCII picture of the map and its units (`docs/movement.md > 8`): four characters by two per tile. `.` is
/// open ground, `:` Rugged Terrain, `#` a tile nothing walks on, `%` a blocking footprint. Each unit is a letter
/// (`A`–`Z`, then `a`–`z`, by id) on the cells its box covers, `*` where two units claim one cell; the legend
/// below lists every unit shown. A map larger than 30 × 20 tiles is cut to a window around `focus` (the map's
/// middle without one).
pub fn ascii(sim: &Sim, focus: Option<Vec2>) -> String {
    let map = sim.map();
    let (w, h) = (map.width() as i32, map.height() as i32);
    let focus = TilePos::containing(focus.unwrap_or(Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0)));
    let origin = |centre: i32, size: i32, window: i32| (centre - window / 2).clamp(0, (size - window).max(0));
    let (x0, y0) = (origin(focus.x, w, WINDOW.0), origin(focus.y, h, WINDOW.1));
    let (cols, rows) = (w.min(WINDOW.0) * CELLS.0, h.min(WINDOW.1) * CELLS.1);
    let cell = Vec2::new(SU_PER_GU / CELLS.0 as f32, SU_PER_GU / CELLS.1 as f32);
    let corner = TilePos::new(x0, y0).corner();

    let mut picture = vec![vec!['.'; cols as usize]; rows as usize];
    for (row, line) in picture.iter_mut().enumerate() {
        for (col, c) in line.iter_mut().enumerate() {
            let tile = TilePos::new(x0 + col as i32 / CELLS.0, y0 + row as i32 / CELLS.1);
            let preset = map.tile_preset(tile).expect("inside the window");
            *c = if !preset.traversible {
                '#'
            } else if preset.rugged {
                ':'
            } else {
                '.'
            };
        }
    }
    let mut paint = |lo: Vec2, hi: Vec2, centre: Vec2, with: char, over_units: bool| {
        let inside = |v: Vec2| ((v - corner) / cell).floor();
        let (c0, c1, at) = (
            inside(lo + cell / 2.0 - 0.01),
            inside(hi - cell / 2.0 + 0.01),
            inside(centre),
        );
        let (c0, c1) = (c0.min(at), c1.max(at));
        for row in c0.y as i32..=c1.y as i32 {
            for col in c0.x as i32..=c1.x as i32 {
                if row < 0 || col < 0 || row >= rows || col >= cols {
                    continue;
                }
                let c = &mut picture[row as usize][col as usize];
                *c = if over_units && (c.is_ascii_alphabetic() || *c == '*') {
                    '*'
                } else {
                    with
                };
            }
        }
    };
    for o in sim.objects() {
        if let Some(placement) = &o.placement {
            let lo = placement.origin.corner();
            let hi = lo + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU;
            paint(lo, hi, placement.center(), '%', false);
        }
    }
    let mut legend = String::new();
    let letters: Vec<char> = ('A'..='Z').chain('a'..='z').collect();
    let mut units = 0;
    for o in sim.objects() {
        let (Some(half), Some(mover)) = (o.def().half_extents(), &o.mover) else {
            continue;
        };
        let letter = letters[units % letters.len()];
        units += 1;
        let tile = TilePos::containing(o.pos);
        if tile.x < x0 || tile.y < y0 || tile.x >= x0 + WINDOW.0 || tile.y >= y0 + WINDOW.1 {
            continue;
        }
        paint(o.pos - half, o.pos + half, o.pos, letter, true);
        let _ = writeln!(
            legend,
            "{letter} #{} {:?} p{} at ({:.2}, {:.2}) {:?} goal {:?} next {}/{} blocked {} repaths left {}",
            o.id.0,
            o.kind,
            o.owner.map_or(0, |p| p.0),
            o.pos.x,
            o.pos.y,
            mover.status,
            mover.goal,
            mover.next,
            mover.path.len(),
            mover.blocked_ticks,
            mover.repaths_left,
        );
    }
    let mut out = format!(
        "tick {}, tiles ({x0}, {y0}) to ({}, {}):\n",
        sim.tick(),
        x0 + cols / CELLS.0 - 1,
        y0 + rows / CELLS.1 - 1
    );
    for line in picture {
        out.extend(line);
        out.push('\n');
    }
    out + &legend
}
