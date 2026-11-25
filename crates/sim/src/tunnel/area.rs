//! The Tunnel Area (slice M4-B; `factions/syndicate/tunnels.md > Tunnel > Tunnel Area`): "a square underground build
//! zone centered on the Tunnel. The radius extends outward from the Tunnel's 4x4 footprint in each direction, forming
//! a square of (radius + 4 + radius) per side", the radius being the tier's (`factions/syndicate/tunnels.md > Tiers`:
//! 2, 3 and 4 grid units). This module is the one place that works a Tunnel Area out: Tunnel Space
//! (`crate::economy::army`, plan decision T33 as design change 0242 revised it), the placement rule of 0242
//! (`crate::placement`), an expansion's parent Tunnel (`Sim::parent_tunnel`, slice M4-C), and the slices that will
//! place Tunnels and expansions, keep Tunnel Areas apart and upgrade Tunnels (M4-E, M4-F, M4-G, M4-H, M4-J) all ask it.
//!
//! - **The area** ([`area`], [`of`]): the footprint grown by the radius on all four sides, a rectangle of cells that
//!   may reach past the map's edge, as a footprint may. It is never clamped to the map.
//! - **The parent Tunnel** ([`parent`], plan decision T32): an expansion's owner's Tunnel in operation whose current
//!   area fully contains it, derived and never stored.
//! - **The region** ([`region`]): `factions/syndicate/tunnels.md > Tunnel Space > Tunnel-region`, "a Tunnel's Tunnel
//!   Area, excluding the 4x4 grid units occupied by the Tunnel's own footprint": 48, 84 and 128 cells at Tiers 1, 2
//!   and 3. It counts every cell, on the map or off it, Drillable or not (0242: "So every tunnel-region cell is
//!   usable"; the earlier off-map and non-Drillable cases of design changes 0238 and 0241 are void).
//! - **The breaches** ([`breaches`]): `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area`
//!   (0242), "a Tunnel can only be placed if every cell of its Tunnel Area is on the map and Drillable (no Water or
//!   Mountain ...)": the cells that aren't. Drillable is the cell's preset's flag (`core/map.md > Tile`: "Any
//!   combination of properties is valid"), never its look.

use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::nav::TileRect;
use crate::network;
use crate::object::Object;
use crate::units::TilePos;

use super::{BASE_TIER, def_of, tier_def};

/// The Tunnel Area of a Tunnel whose footprint is `footprint`, at the Tunnel Area radius `radius`: the footprint grown
/// by `radius` cells on every side. It may reach past the map's edge; far from the map it saturates rather than
/// overflow.
pub fn area(footprint: TileRect, radius: u32) -> TileRect {
    let grow = |len: u32| len.saturating_add(radius.saturating_mul(2));
    let origin = TilePos::new(
        footprint.origin.x.saturating_sub_unsigned(radius),
        footprint.origin.y.saturating_sub_unsigned(radius),
    );
    TileRect::new(origin, grow(footprint.w), grow(footprint.h))
}

/// The Tunnel Area of `o` at its current tier ([`tier_def`]): `None` for an object that isn't a placed Tunnel. During
/// an upgrade it is the current tier's, as the tier changes only when the upgrade completes (slice M4-G).
pub fn of(o: &Object) -> Option<TileRect> {
    let placement = o.placement?;
    let radius = tier_def(o)?.area_radius;
    Some(area(TileRect::new(placement.origin, placement.w, placement.h), radius))
}

/// The parent Tunnel of `expansion`, an underground structure (plan decision T32: derived, never stored): "the
/// owner's Tunnel whose current area fully contains it" ([`of`], [`encloses`]), the first by id of `tunnels` (an
/// owner's Tunnel Areas never overlap, so there is at most one). `None` for an expansion without an owner or a
/// footprint, or outside the area of every one of its owner's Tunnels in operation (`crate::network::is_exit`).
pub fn parent<'a>(expansion: &Object, tunnels: impl IntoIterator<Item = &'a Object>) -> Option<ObjectId> {
    let owner = expansion.owner?;
    let p = expansion.placement?;
    let footprint = TileRect::new(p.origin, p.w, p.h);
    let contains = |t: &Object| of(t).is_some_and(|area| encloses(area, footprint));
    tunnels
        .into_iter()
        .find(|t| t.owner == Some(owner) && network::is_exit(t) && contains(t))
        .map(|t| t.id)
}

/// The Tunnel Area radius of a new object of `kind`, at [`BASE_TIER`]: Tier 1's 2 for the Tunnel, what a Tunnel being
/// placed is checked at (design change 0242). `None` for a kind without a tier table.
pub fn base_radius(kind: ObjectKind) -> Option<u32> {
    Some(def_of(kind)?.tier(BASE_TIER)?.area_radius)
}

/// The cells of `rect`, row by row from the north-west. Those whose coordinates would overflow are left out: the
/// rectangle then lies far past the map's edge.
pub fn cells(rect: TileRect) -> impl Iterator<Item = TilePos> {
    (0..rect.h).flat_map(move |dy| {
        (0..rect.w).filter_map(move |dx| {
            let x = rect.origin.x.checked_add_unsigned(dx)?;
            let y = rect.origin.y.checked_add_unsigned(dy)?;
            Some(TilePos::new(x, y))
        })
    })
}

/// Whether `rect` holds `cell`, in 64 bits so that no rectangle overflows.
pub fn contains(rect: TileRect, cell: TilePos) -> bool {
    let within = |start: i32, len: u32, at: i32| {
        let (start, at) = (i64::from(start), i64::from(at));
        start <= at && at < start + i64::from(len)
    };
    within(rect.origin.x, rect.w, cell.x) && within(rect.origin.y, rect.h, cell.y)
}

/// Whether `a` and `b` share a cell, in 64 bits.
pub fn overlaps(a: TileRect, b: TileRect) -> bool {
    let span = |start: i32, len: u32| (i64::from(start), i64::from(start) + i64::from(len));
    let apart = |(a0, a1): (i64, i64), (b0, b1): (i64, i64)| a1 <= b0 || b1 <= a0;
    !(apart(span(a.origin.x, a.w), span(b.origin.x, b.w)) || apart(span(a.origin.y, a.h), span(b.origin.y, b.h)))
}

/// Whether every cell of `inner` lies in `outer` (an empty `inner` always does), in 64 bits.
pub fn encloses(outer: TileRect, inner: TileRect) -> bool {
    let span = |start: i32, len: u32| (i64::from(start), i64::from(start) + i64::from(len));
    let inside = |(o0, o1): (i64, i64), (i0, i1): (i64, i64)| i0 == i1 || (o0 <= i0 && i1 <= o1);
    inside(span(outer.origin.x, outer.w), span(inner.origin.x, inner.w))
        && inside(span(outer.origin.y, outer.h), span(inner.origin.y, inner.h))
}

/// The region of a Tunnel whose footprint is `footprint`, at the radius `radius` (`factions/syndicate/tunnels.md >
/// Tunnel Space > Tunnel-region`): the cells of its [`area`] that the footprint doesn't cover, row by row from the
/// north-west. Every one counts, whatever the map holds there (module docs).
pub fn region(footprint: TileRect, radius: u32) -> impl Iterator<Item = TilePos> {
    cells(area(footprint, radius)).filter(move |&cell| !contains(footprint, cell))
}

/// How a cell of a Tunnel Area breaks design change 0242's rule.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breach {
    /// It lies past the map's edge: "no Tunnel near the map edge (closer than its area reaches)".
    OffMap,
    /// Its preset isn't Drillable: of the default presets, Water and Mountain (`core/map.md > DefaultTilePresets`).
    NotDrillable,
}

/// The cells of `area` that break design change 0242's rule ("every cell of its Tunnel Area is on the map and
/// Drillable"), each with how, row by row from the north-west. Read as written, a Tier 1 Tunnel whose footprint lies
/// 2 tiles from the map's edge has an area that just reaches the edge, and breaks nothing: 0242's Verification ("A
/// Tunnel 2 tiles from the map edge is red") is one tile off from its own rule. Placeholder: Q0078.
pub fn breaches(map: &Map, area: TileRect) -> impl Iterator<Item = (TilePos, Breach)> + '_ {
    cells(area).filter_map(move |cell| match map.tile_preset(cell) {
        _ if !map.in_bounds(cell) => Some((cell, Breach::OffMap)),
        Some(preset) if preset.drillable => None,
        _ => Some((cell, Breach::NotDrillable)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Tile, TilePreset, TilePresetId};
    use crate::scenario::SpawnAt;
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;

    const TUNNEL: ObjectKind = ObjectKind::Tunnel;

    fn footprint(x: i32, y: i32) -> TileRect {
        TileRect::new(TilePos::new(x, y), 4, 4)
    }

    /// `factions/syndicate/tunnels.md > Tiers` (radius 2, 3, 4) and `factions/syndicate/tunnels.md > Tunnel Space >
    /// Tunnel-region`: "a Tier 1 Tunnel has an 8x8 = 64 grid unit Tunnel Area, minus its own 16 grid unit footprint =
    /// 48 grid units of tunnel-region"; 10x10 less 16 is 84, 12x12 less 16 is 128.
    #[test]
    fn each_tier_has_its_area_and_region() {
        let tiers = def_of(TUNNEL).unwrap().tiers;
        let radii: Vec<u32> = tiers.iter().map(|t| t.area_radius).collect();
        assert_eq!(radii, [2, 3, 4]);
        let sizes: Vec<(u32, u32, usize)> = radii
            .iter()
            .map(|&r| {
                let a = area(footprint(10, 10), r);
                (a.w, a.h, region(footprint(10, 10), r).count())
            })
            .collect();
        assert_eq!(sizes, [(8, 8, 48), (10, 10, 84), (12, 12, 128)]);
        assert_eq!((base_radius(TUNNEL), base_radius(ObjectKind::Hideout)), (Some(2), None));
        // Grown on every side, the footprint left out.
        assert_eq!(area(footprint(4, 4), 2), TileRect::new(TilePos::new(2, 2), 8, 8));
        assert!(region(footprint(4, 4), 2).all(|c| !contains(footprint(4, 4), c)));
        assert_eq!(region(footprint(4, 4), 2).next(), Some(TilePos::new(2, 2)));
    }

    /// A placed Tunnel's area is its tier's; nothing else has one.
    #[test]
    fn a_tunnels_area_follows_its_tier() {
        let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
        let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 16].join("\n"))
            .spawn(TUNNEL, None, tile(6, 6))
            .spawn(ObjectKind::Hideout, None, tile(2, 2))
            .build();
        let sim = Sim::new(&scenario);
        let mut tunnel = sim.object(ObjectId(1)).unwrap().clone();
        assert_eq!(of(&tunnel), Some(TileRect::new(TilePos::new(4, 4), 8, 8)));
        assert!(super::super::set_tier(&mut tunnel, 3));
        assert_eq!(of(&tunnel), Some(TileRect::new(TilePos::new(2, 2), 12, 12)));
        assert_eq!(of(sim.object(ObjectId(2)).unwrap()), None);
    }

    /// Plan decision T32: an expansion's parent is its owner's Tunnel whose current area wholly holds it. Player 1's
    /// Tunnel (1) on (6, 6) has its Tier 1 area on (4, 4) to (11, 11), player 2's (2) on (14, 6) to (17, 9). The
    /// Hideout inside player 1's area has it for its parent; half outside, or player 2's in that area, or a neutral
    /// one, has none; at Tier 3 the area reaches the Hideout half outside.
    #[test]
    fn an_expansions_parent_is_its_owners_tunnel_whose_area_holds_it() {
        let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
        let (p1, p2) = (Some(crate::PlayerId(1)), Some(crate::PlayerId(2)));
        let scenario = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
            .spawn(TUNNEL, p1, tile(6, 6)) // 1
            .spawn(TUNNEL, p2, tile(14, 6)) // 2
            .spawn(ObjectKind::Hideout, p1, tile(10, 4)) // 3: inside
            .spawn(ObjectKind::Hideout, p1, tile(11, 10)) // 4: half outside
            .spawn(ObjectKind::Hideout, p2, tile(4, 10)) // 5: another player's
            .spawn(ObjectKind::Hideout, None, tile(4, 4)) // 6: neutral
            .build();
        let sim = Sim::new(&scenario);
        let object = |n| sim.object(ObjectId(n)).unwrap();
        let parents = |tunnels: &[Object]| (3..=6).map(|n| parent(object(n), tunnels)).collect::<Vec<_>>();
        let mut tunnels: Vec<Object> = [1, 2].map(|n| object(n).clone()).to_vec();
        assert_eq!(parents(&tunnels), [Some(ObjectId(1)), None, None, None]);
        assert!(super::super::set_tier(&mut tunnels[0], 3));
        assert_eq!(parents(&tunnels), [Some(ObjectId(1)), Some(ObjectId(1)), None, None]);
    }

    /// Design change 0242: the cells off the map and those whose preset isn't Drillable breach, whatever their look;
    /// Rugged and Cliff are Drillable (`core/map.md > DefaultTilePresets`).
    #[test]
    fn off_map_and_undrillable_cells_breach() {
        // Plane, but for a column each of Water (x = 3), Mountain (x = 4), Rugged (x = 5) and Cliff (x = 6).
        let map = Map::from_ascii(&vec!["...~Mrc.........".to_string(); 16].join("\n")).unwrap();
        let found = |rect| breaches(&map, rect).collect::<Vec<_>>();
        assert_eq!(found(area(footprint(9, 6), 2)), [], "Plane only");
        assert_eq!(found(area(footprint(7, 6), 2)), [], "Rugged and Cliff are Drillable");
        let lake = found(area(footprint(5, 6), 2));
        assert_eq!(lake.len(), 16, "two columns of 8: {lake:?}");
        assert!(
            lake.iter()
                .all(|&(c, b)| b == Breach::NotDrillable && (c.x == 3 || c.x == 4))
        );
        // 2 tiles from the east edge, the area (x 8 to 15) just reaches it (Placeholder: Q0078); 1 tile
        // from it, a column of 8 lies off the map.
        assert_eq!(found(area(footprint(10, 6), 2)), []);
        let off = found(area(footprint(11, 6), 2));
        assert_eq!(off.len(), 8);
        assert!(off.iter().all(|&(c, b)| b == Breach::OffMap && c.x == 16));
        // In the south-east corner, 2 rows and 2 columns off: 64 cells less the 6x6 on the map.
        let corner = found(area(footprint(12, 12), 2));
        assert_eq!(corner.len(), 64 - 36);
        assert!(corner.iter().all(|&(_, b)| b == Breach::OffMap));
    }

    /// A custom preset that isn't Drillable breaches, whatever its look (`core/map.md > Tile`).
    #[test]
    fn a_custom_undrillable_preset_breaches() {
        let mut map = Map::new(12, 12);
        let plane = map.preset(TilePresetId::PLANE).clone();
        let id = map.add_preset(TilePreset {
            name: "Hard plane".into(),
            drillable: false,
            ..plane
        });
        map.set_tile(TilePos::new(9, 9), Tile::new(id));
        assert_eq!(
            breaches(&map, area(footprint(4, 4), 2)).collect::<Vec<_>>(),
            [(TilePos::new(9, 9), Breach::NotDrillable)]
        );
    }

    /// Far from the map nothing overflows: the cells that would are left out, and the rest lie off the map.
    #[test]
    fn nothing_overflows_far_off_the_map() {
        let map = Map::new(8, 8);
        for origin in [TilePos::new(i32::MAX - 1, 0), TilePos::new(i32::MIN, i32::MIN)] {
            let far = area(TileRect::new(origin, 4, 4), 4);
            assert!(cells(far).count() <= 144);
            assert!(breaches(&map, far).all(|(_, b)| b == Breach::OffMap));
            assert!(!contains(far, TilePos::new(0, 0)));
        }
        assert!(encloses(footprint(0, 0), TileRect::new(TilePos::new(3, 3), 1, 1)));
        assert!(!encloses(footprint(0, 0), TileRect::new(TilePos::new(3, 3), 2, 1)));
        assert!(!encloses(footprint(0, 0), TileRect::new(TilePos::new(-1, 0), 2, 1)));
        assert!(encloses(
            footprint(0, 0),
            TileRect::new(TilePos::new(i32::MAX, 0), 0, 0)
        ));
        assert!(overlaps(footprint(0, 0), footprint(3, 3)) && !overlaps(footprint(0, 0), footprint(4, 0)));
        assert!(!overlaps(footprint(i32::MAX, 0), footprint(i32::MIN, 0)));
    }
}
