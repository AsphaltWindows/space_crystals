//! The Tunnel's placeholder look by tier (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`, design
//! changes 0224 and 0248; drawing only, until the art direction, Q0001), over its structure body
//! (`super::structures`), on the sides its orientation turns them to (`sc_sim::sides`). Clockwise from the top in the
//! default orientation, A is the entrance and exit, B the crystal hatch, C the back wall and D the Supplies hatch
//! (0248, correcting 0224's C and D):
//! - **Side A**, "two paths: one for entering, one for exiting": two strips from the edge in, "narrow and
//!   shaggy-looking" at Tier 1 (ragged dirt), "wider and well paved" at Tiers 2 and 3. They lie on the half nearer B,
//!   the exit, and the half nearer D, the entry (0248), drawn alike, where the sim lets units out and in
//!   (`sc_sim::network::Paths`, slice M4-C).
//! - **Sides B and D**, each "a hatch": a framed square at the middle of the edge, B's in the crystals' cyan, D's in
//!   the Supplies' amber (the map resources' colours, `super::resources`). Side C, the back wall, has no mark. (Design
//!   change 0255 point 6 makes a hatch "1 grid square, near the middle of the side": slice M4-D picks the square.)
//! - **Tier 3**, "two horizontal circular sliding doors, one for flying units entering and one for flying units
//!   exiting": two split discs on the roof "over the matching path halves" (design change 0251 point 11), the exit's
//!   on the half nearer B and the entry's on the half nearer D, set back towards the back wall so that they cover
//!   neither the paths nor the tag. Slice M4-C gives them their roles.
//!
//! The tag in the middle names the tier (`crate::hud::selection_panel::object_label`); nothing here covers it.

use bevy::prelude::*;
use sc_sim::tunnel::TunnelState;
use sc_sim::{ObjectKind, Placement, SideLetter};

use super::structures::LOOK_Z;
use crate::shapes::ColorMesh;

/// What a Tunnel's look depends on: its tier, and the letters of its sides facing north, east, south and west as
/// placed (`sc_sim::sides`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TunnelLook {
    tier: u8,
    sides: [SideLetter; 4],
}

impl TunnelLook {
    /// The look of a `kind` object with tier state `tunnel` placed at `placement`, a live one's or a record's: `None`
    /// but for a Tunnel.
    pub(super) fn of(kind: ObjectKind, tunnel: Option<TunnelState>, placement: Option<&Placement>) -> Option<Self> {
        let sides = sc_sim::sides(kind, placement?.orientation())?;
        Some(Self {
            tier: tunnel?.tier,
            sides,
        })
    }
}

/// The view directions `sc_sim::sides` lists its letters in: north (+y in the view's world space), east, south, west.
const FACING: [Vec2; 4] = [Vec2::Y, Vec2::X, Vec2::NEG_Y, Vec2::NEG_X];

/// Side A's paths: how far apart their middles are (the sim's spots, `sc_sim::network::Paths`: where units go in and
/// come out, slice M4-C), how far in they reach from the edge, and how wide they are at Tier 1 and from Tier 2 on,
/// space units.
const PATH_SPACING_SU: f32 = 2.0 * sc_sim::network::PATH_OFFSET_SU;
const PATH_LENGTH_SU: f32 = 58.0;
const TRAIL_SU: f32 = 12.0;
const PAVED_SU: f32 = 24.0;
const TRAIL: Color = Color::srgb(0.42, 0.31, 0.19);
const PAVED: Color = Color::srgb(0.66, 0.66, 0.63);
const PAVED_EDGE: Color = Color::srgb(0.22, 0.22, 0.22);
/// A hatch's side and frame, su, and its colours: the frame's, B's (crystals) and D's (Supplies).
const HATCH_SU: f32 = 34.0;
const HATCH_FRAME_SU: f32 = 4.0;
const HATCH_FRAME: Color = Color::srgb(0.09, 0.10, 0.12);
const CRYSTAL_HATCH: Color = Color::srgb(0.36, 0.90, 0.98);
const SUPPLY_HATCH: Color = Color::srgb(0.85, 0.55, 0.13);
/// The Tier 3 air doors: their radius, how far apart their centres are, and their colours.
const DOOR_SU: f32 = 20.0;
const DOOR_SPACING_SU: f32 = 64.0;
const DOOR_RIM: Color = Color::srgb(0.10, 0.11, 0.13);
const DOOR: Color = Color::srgb(0.48, 0.52, 0.58);

/// Draws `look` over a Tunnel body whose drawing reaches `half` from its centre (`super::structures::inner_half`).
pub(super) fn draw(mesh: &mut ColorMesh, look: TunnelLook, half: Vec2) {
    let facing = |letter| (look.sides.iter().position(|&l| l == letter)).map(|i| FACING[i]);
    // From the centre to the edge the side `n` faces.
    let reach = |n: Vec2| (half * n).length();
    if let (Some(n), Some(exit), Some(entry)) = (facing(SideLetter::A), facing(SideLetter::B), facing(SideLetter::D)) {
        paths(mesh, n, reach(n), [exit, entry], look.tier);
    }
    for (letter, color) in [(SideLetter::B, CRYSTAL_HATCH), (SideLetter::D, SUPPLY_HATCH)] {
        if let Some(n) = facing(letter) {
            hatch(mesh, n * (reach(n) - HATCH_SU / 2.0), n, color);
        }
    }
    if look.tier >= 3
        && let (Some(n), Some(exit), Some(entry)) =
            (facing(SideLetter::C), facing(SideLetter::B), facing(SideLetter::D))
    {
        for towards in [exit, entry] {
            air_door(mesh, n * reach(n) * 0.5 + towards * DOOR_SPACING_SU / 2.0, n);
        }
    }
}

/// Side A's two paths, from its edge (`reach` from the centre along `n`, its outward direction) inwards, on its halves
/// towards `halves`: the directions the crystal hatch (the exit's) and the Supplies hatch (the entry's) face.
fn paths(mesh: &mut ColorMesh, n: Vec2, reach: f32, halves: [Vec2; 2], tier: u8) {
    for towards in halves {
        let at = towards * PATH_SPACING_SU / 2.0;
        let (inner, outer) = (at + n * (reach - PATH_LENGTH_SU), at + n * reach);
        if tier >= 2 {
            mesh.segment(inner, outer, PAVED_SU + 4.0, LOOK_Z, PAVED_EDGE);
            mesh.segment(inner, outer, PAVED_SU, LOOK_Z + 0.001, PAVED);
        } else {
            // Shaggy: pieces of uneven width.
            let pieces = 6;
            for i in 0..pieces {
                let (a, b) = (i as f32 / pieces as f32, (i + 1) as f32 / pieces as f32);
                let width = if i % 2 == 0 { TRAIL_SU } else { TRAIL_SU * 0.6 };
                mesh.segment(inner.lerp(outer, a), inner.lerp(outer, b), width, LOOK_Z, TRAIL);
            }
        }
    }
}

/// A hatch centred on `center`, on the side facing `n`: a framed square with a seam across.
fn hatch(mesh: &mut ColorMesh, center: Vec2, n: Vec2, color: Color) {
    let half = Vec2::splat(HATCH_SU / 2.0);
    let inner = half - HATCH_FRAME_SU;
    mesh.rect(center - half, center + half, LOOK_Z, HATCH_FRAME);
    mesh.rect(center - inner, center + inner, LOOK_Z + 0.001, color);
    let seam = n.perp() * inner.x;
    mesh.segment(center - seam, center + seam, 3.0, LOOK_Z + 0.002, HATCH_FRAME);
}

/// A round sliding door on the roof centred on `center`: a rimmed disc, split along `n`.
fn air_door(mesh: &mut ColorMesh, center: Vec2, n: Vec2) {
    let disc = |radius: f32| -> Vec<Vec2> {
        (0..20)
            .map(|i| center + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 20.0) * radius)
            .collect()
    };
    mesh.polygon(&disc(DOOR_SU), LOOK_Z, DOOR_RIM);
    mesh.polygon(&disc(DOOR_SU - 4.0), LOOK_Z + 0.001, DOOR);
    let seam = n * (DOOR_SU - 4.0);
    mesh.segment(center - seam, center + seam, 2.0, LOOK_Z + 0.002, DOOR_RIM);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{Rotation, TilePos};

    fn look(tier: u8, rotation: Rotation) -> Option<TunnelLook> {
        let placement = Placement {
            rotation,
            ..Placement::new(TilePos::new(0, 0), ObjectKind::Tunnel.def().footprint)
        };
        TunnelLook::of(ObjectKind::Tunnel, Some(TunnelState::at(tier)), Some(&placement))
    }

    /// Design change 0224: a Tunnel's look follows its tier and its sides; nothing else has one.
    #[test]
    fn only_a_tunnel_has_a_look_by_tier_and_sides() {
        let (t1, t3) = (look(1, Rotation::R0), look(3, Rotation::R0));
        assert!(t1.is_some() && t1 != t3);
        let unturned = t1.unwrap();
        assert_eq!(
            unturned.sides,
            [SideLetter::A, SideLetter::B, SideLetter::C, SideLetter::D]
        );
        assert_ne!(look(1, Rotation::R90), t1, "turned, its sides face elsewhere");
        let placement = Placement::new(TilePos::new(0, 0), ObjectKind::Hideout.def().footprint);
        assert_eq!(TunnelLook::of(ObjectKind::Hideout, None, Some(&placement)), None);
        // More is drawn from Tier 2 on (paved paths with edges), and most at Tier 3 (the air doors).
        let count = |look: Option<TunnelLook>| {
            let mut mesh = ColorMesh::triangles();
            draw(&mut mesh, look.unwrap(), Vec2::splat(115.0));
            mesh.vertex_count()
        };
        let (one, two, three) = (count(t1), count(look(2, Rotation::R0)), count(t3));
        assert!(one != two && two < three, "{one} {two} {three}");
    }

    /// Design change 0248: unturned, with A on top, the hatches are on Sides B (east) and D (west), and Side C, the
    /// back wall at the bottom, has no mark; Side A's paths lie on its halves towards B and D, and the air doors over
    /// the same halves (0251 point 11), towards C.
    #[test]
    fn the_hatches_are_on_sides_b_and_d() {
        let half = Vec2::splat(115.0);
        let drawn = |tier| {
            let mut mesh = ColorMesh::triangles();
            draw(&mut mesh, look(tier, Rotation::R0).unwrap(), half);
            let mesh = mesh.into_mesh();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("no positions");
            };
            points.iter().map(|p| Vec2::new(p[0], p[1])).collect::<Vec<_>>()
        };
        for tier in [1, 3] {
            let points = drawn(tier);
            let any = |test: &dyn Fn(Vec2) -> bool| points.iter().any(|&p| test(p));
            assert!(any(&|p| p.x > 80.0 && p.y.abs() < 20.0), "B's hatch, Tier {tier}");
            assert!(any(&|p| p.x < -80.0 && p.y.abs() < 20.0), "D's hatch, Tier {tier}");
            assert!(!any(&|p| p.y < -80.0), "nothing on C, Tier {tier}");
            assert!(
                any(&|p| p.y > 100.0 && p.x > 0.0) && any(&|p| p.y > 100.0 && p.x < 0.0),
                "A's two paths"
            );
            assert!(!any(&|p| p.y > 100.0 && p.x.abs() > 60.0), "A's paths only");
        }
        let doors = |points: &[Vec2]| points.iter().filter(|p| p.y < -30.0).count();
        assert_eq!(
            (doors(&drawn(1)), doors(&drawn(3)) > 0),
            (0, true),
            "the air doors at Tier 3"
        );
        let door_on = |points: &[Vec2], east: bool| points.iter().any(|p| p.y < -30.0 && (p.x > 10.0) == east);
        let t3 = drawn(3);
        assert!(door_on(&t3, true) && door_on(&t3, false), "one door over each half");
    }
}
