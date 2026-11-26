//! A release's areas (plan decision T77): "areas are integer counts on a 4-su lattice (a 3-GU circle is about 7,200
//! points): deterministic, and within about 1% of exact". The lattice's points are the centres of the cells of a square
//! grid [`LATTICE_STEP`] space units on a side, from the map's origin: the point of column `i` and row `j` is at
//! `((i + ½) × step, (j + ½) × step)`. A cloud's area is the number of points under its circles ([`Union`]: inside one
//! or on its edge, as for what is under a cloud), an object's the number inside its shape's box, its left and top edges
//! in and its right and bottom ones out, so that a box whose sides are whole steps holds exactly its area's worth (a 3x3
//! footprint, 192 su on a side, 2,304 points, wherever it stands), and the part of an object under a cloud the number
//! of points in both.
//!
//! Only basic arithmetic, `floor`, `ceil` and `crate::math::sqrt` (libm): the same counts on every machine.

use glam::Vec2;

use super::Member;
use crate::math;
use crate::tuning::spells::LATTICE_STEP;

/// The first index of the lattice's points at or beyond the coordinate `v`, space units.
fn first_at_or_after(v: f32) -> i32 {
    (v / LATTICE_STEP - 0.5).ceil() as i32
}

/// The last index of the lattice's points at or before the coordinate `v`, space units.
fn last_at_or_before(v: f32) -> i32 {
    (v / LATTICE_STEP - 0.5).floor() as i32
}

/// The coordinate of the lattice's points of index `i`, space units.
fn coordinate(i: i32) -> f32 {
    (i as f32 + 0.5) * LATTICE_STEP
}

/// The lattice's points under the circles of a cloud (module docs), row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Union {
    /// The index of the first row with a point under it.
    first_row: i32,
    /// Each row's columns under it, from the first row on: sorted, disjoint, half-open ranges of column indices.
    rows: Vec<Vec<(i32, i32)>>,
    /// The first and the last column under it, past the last: the columns of its box.
    columns: (i32, i32),
    /// How many points are under it.
    count: u32,
}

impl Union {
    /// The points under the circles of `members` (module docs).
    pub fn of(members: &[Member]) -> Self {
        let rows_of = |m: &Member| {
            (
                first_at_or_after(m.at.y - m.radius),
                last_at_or_before(m.at.y + m.radius),
            )
        };
        let first_row = members.iter().map(|m| rows_of(m).0).min().unwrap_or(0);
        let last_row = members.iter().map(|m| rows_of(m).1).max().unwrap_or(-1);
        let mut rows: Vec<Vec<(i32, i32)>> = vec![Vec::new(); (last_row - first_row + 1).max(0) as usize];
        for m in members {
            let (first, last) = rows_of(m);
            for j in first..=last {
                let dy = coordinate(j) - m.at.y;
                let squared = m.radius * m.radius - dy * dy;
                if squared < 0.0 {
                    continue;
                }
                let half = math::sqrt(squared);
                let (start, end) = (first_at_or_after(m.at.x - half), last_at_or_before(m.at.x + half) + 1);
                if start < end {
                    rows[(j - first_row) as usize].push((start, end));
                }
            }
        }
        let mut count = 0;
        let mut columns = (i32::MAX, i32::MIN);
        for row in &mut rows {
            row.sort_unstable();
            let mut merged: Vec<(i32, i32)> = Vec::with_capacity(row.len());
            for &(start, end) in row.iter() {
                match merged.last_mut() {
                    Some(last) if start <= last.1 => last.1 = last.1.max(end),
                    _ => merged.push((start, end)),
                }
            }
            for &(start, end) in &merged {
                count += (end - start) as u32;
                columns = (columns.0.min(start), columns.1.max(end));
            }
            *row = merged;
        }
        Self {
            first_row,
            rows,
            columns,
            count,
        }
    }

    /// How many of the lattice's points are under it: its area, in points.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Of the lattice's points inside the box with corners `lo` and `hi` (space units; module docs), how many are under
    /// it, and how many there are in all: the part of an object's area under the cloud, and its whole area, in points.
    pub fn overlap(&self, lo: Vec2, hi: Vec2) -> (u32, u32) {
        let (c0, c1) = (first_at_or_after(lo.x), first_at_or_after(hi.x));
        let (r0, r1) = (first_at_or_after(lo.y), first_at_or_after(hi.y));
        let total = ((c1 - c0).max(0) as u32) * ((r1 - r0).max(0) as u32);
        let last_row = self.first_row + self.rows.len() as i32;
        if total == 0 || c1 <= self.columns.0 || c0 >= self.columns.1 || r1 <= self.first_row || r0 >= last_row {
            return (0, total);
        }
        let mut inside = 0;
        for j in r0.max(self.first_row)..r1.min(last_row) {
            for &(start, end) in &self.rows[(j - self.first_row) as usize] {
                inside += (end.min(c1) - start.max(c0)).max(0) as u32;
            }
        }
        (inside, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::ObjectId;
    use crate::math::PI;

    fn circle(x: f32, y: f32, radius: f32) -> Member {
        Member {
            id: ObjectId(1),
            since: 0,
            at: Vec2::new(x, y),
            radius,
        }
    }

    /// A 3-GU circle is "about 7,200 points" (plan decision T77), within 1% of its exact area, wherever it stands; a
    /// box of whole steps holds its area's worth wherever it stands, a 24x24 Silhouette 36 points.
    #[test]
    fn a_cloud_s_area_and_a_box_s_are_counted_on_the_lattice() {
        let exact = PI * 192.0 * 192.0 / (LATTICE_STEP * LATTICE_STEP);
        for (x, y) in [(500.0, 500.0), (513.3, 487.9), (1001.0, 66.6)] {
            let count = Union::of(&[circle(x, y, 192.0)]).count() as f32;
            assert!((count - exact).abs() / exact < 0.01, "({x}, {y}): {count} for {exact}");
            assert!((7_150.0..7_300.0).contains(&count), "{count}");
        }
        let union = Union::of(&[circle(500.0, 500.0, 192.0)]);
        for lo in [Vec2::new(404.0, 404.0), Vec2::new(410.3, 399.9)] {
            assert_eq!(union.overlap(lo, lo + Vec2::splat(24.0)).1, 36, "{lo}");
            assert_eq!(union.overlap(lo, lo + Vec2::splat(24.0)).0, 36, "wholly under it: {lo}");
        }
        let footprint = union.overlap(Vec2::new(404.0, 404.0), Vec2::new(596.0, 596.0));
        assert_eq!(footprint, (2_304, 2_304), "a 3x3 footprint wholly under it");
        let share = footprint.0 as f32 / union.count() as f32;
        assert!(
            (share - 0.32).abs() < 0.003,
            "about 32% of the absorbed damage: {share}"
        );
    }

    /// Circles that overlap count their common points once; a box far from the circles has none under them, and one
    /// half under a circle about half.
    #[test]
    fn overlapping_circles_count_once() {
        let one = Union::of(&[circle(500.0, 500.0, 192.0)]).count();
        let same = Union::of(&[circle(500.0, 500.0, 192.0), circle(500.0, 500.0, 192.0)]).count();
        assert_eq!(same, one);
        let apart = Union::of(&[circle(500.0, 500.0, 192.0), circle(1500.0, 500.0, 192.0)]).count();
        assert_eq!(apart, 2 * one);
        let joined = Union::of(&[circle(500.0, 500.0, 192.0), circle(820.0, 500.0, 192.0)]);
        assert!(joined.count() > one && joined.count() < 2 * one, "{}", joined.count());
        assert_eq!(
            joined.overlap(Vec2::new(2000.0, 2000.0), Vec2::new(2024.0, 2024.0)),
            (0, 36)
        );
        let (inside, total) = joined.overlap(Vec2::new(500.0, 400.0), Vec2::new(1012.0, 424.0));
        assert_eq!(total, 128 * 6);
        assert!(inside > 0 && inside < total, "{inside} of {total}");
        assert_eq!(Union::of(&[]).count(), 0);
        assert_eq!(Union::of(&[]).overlap(Vec2::ZERO, Vec2::splat(24.0)), (0, 36));
    }
}
