//! The clouds' texels as plain data: over the Visible tiles only, merged where circles overlap, a release's flash
//! brighter and fading; and the box they cover.

use super::*;

/// A cloud of one Disciple's circle, 3 grid units, at the centre of tile (6, 6).
const CENTRE: Vec2 = Vec2::new(416.0, 416.0);
const RADIUS: f32 = 192.0;

/// A flat 20 × 20 map.
fn map() -> Map {
    Map::new(20, 20)
}

/// The texel at `point` of `data`, laid out over `area`.
fn texel(data: &[u8], area: Area, point: Vec2) -> [u8; 4] {
    let step = SU_PER_GU / TEXELS_PER_TILE as f32;
    let (corner, _) = area.bounds();
    let at = ((point - corner) / step).floor().as_ivec2();
    let i = ((at.y * area.w * TEXELS_PER_TILE + at.x) * 4) as usize;
    data[i..i + 4].try_into().unwrap()
}

/// A texel is as opaque as the circles cover it: wholly from half a texel inside the edge, half on it, not at all
/// from half a texel outside it.
#[test]
fn a_texel_s_coverage_softens_the_edge_over_a_texel() {
    let circles = [(CENTRE, RADIUS), (CENTRE + Vec2::new(500.0, 0.0), 100.0)];
    let at = |dx: f32| coverage(&circles, CENTRE + Vec2::new(dx, 0.0), 16.0);
    assert_eq!(
        [at(0.0), at(RADIUS - 8.0), at(RADIUS), at(RADIUS + 8.0)],
        [1.0, 1.0, 0.5, 0.0]
    );
    assert_eq!(at(RADIUS + 4.0), 0.25);
    assert_eq!(at(400.0), 0.5, "on the other circle's edge: the one it is deepest in");
}

#[test]
fn a_cloud_s_box_is_the_tiles_its_circles_touch_on_the_map() {
    let map = map();
    let area = Area::of(&map, &[(CENTRE, RADIUS)]);
    assert_eq!(area, Some(Area { x: 3, y: 3, w: 7, h: 7 }));
    let corner = Area::of(&map, &[(Vec2::new(64.0, 64.0), RADIUS)]);
    assert_eq!(corner, Some(Area { x: 0, y: 0, w: 5, h: 5 }), "cut at the map's edges");
    let two = [(CENTRE, RADIUS), (CENTRE + Vec2::new(320.0, 0.0), RADIUS)];
    assert_eq!(
        Area::of(&map, &two),
        Some(Area {
            x: 3,
            y: 3,
            w: 12,
            h: 7
        })
    );
    assert_eq!(Area::of(&map, &[]), None);
}

/// "the cloud is shown to **every player**, over the tiles that are Visible to them, so its edge can be seen even when
/// the Disciple can't": the haze over every tile of the circle that the viewer sees, nothing over the others; an enemy
/// who sees only its edge sees only the edge.
#[test]
fn a_cloud_shows_over_the_viewer_s_visible_tiles_only() {
    let map = map();
    let area = Area::of(&map, &[(CENTRE, RADIUS)]).unwrap();
    let clouds = [vec![(CENTRE, RADIUS)]];
    let (all, every) = texels(area, &clouds, &[], |_| true);
    let corners = [(3, 3), (9, 3), (3, 9), (9, 9)].map(|(x, y)| TilePos::new(x, y));
    assert_eq!(every.len(), 7 * 7 - 4, "the box but its corner tiles");
    assert!(corners.iter().all(|tile| !every.contains(tile)));
    assert!(every.contains(&TilePos::new(6, 6)));
    let (x, y) = (TilePos::new(4, 3), CENTRE + Vec2::new(-104.0, -168.0));
    assert!(
        every.contains(&x),
        "a sliver of the circle's soft edge, 5.6 su out at its nearest texel's centre"
    );
    assert!((1..CLOUD[3] / 2).contains(&texel(&all, area, y)[3]), "faint");
    assert_eq!(texel(&all, area, CENTRE), CLOUD);
    assert_eq!(
        texel(&all, area, CENTRE + Vec2::new(RADIUS - 20.0, 0.0)),
        CLOUD,
        "in, a texel inside the edge"
    );
    assert_eq!(texel(&all, area, CENTRE + Vec2::new(RADIUS + 20.0, 0.0))[3], 0, "out");
    // An enemy whose vision reaches the cloud's east edge, the tiles from column 9 on.
    let (edge, seen) = texels(area, &clouds, &[], |tile| tile.x >= 9);
    assert!(!seen.is_empty() && seen.iter().all(|tile| tile.x == 9), "{seen:?}");
    let rim = CENTRE + Vec2::new(RADIUS - 4.0, 0.0);
    assert!(texel(&edge, area, rim)[3] > 0, "its edge");
    assert_eq!(texel(&edge, area, rim), texel(&all, area, rim), "as its owner sees it");
    assert_eq!(texel(&edge, area, CENTRE)[3], 0, "not its middle");
    let (none, nothing) = texels(area, &clouds, &[], |_| false);
    assert!(nothing.is_empty() && none.iter().all(|&b| b == 0));
}

/// "Clouds are connected when their circles **touch or overlap**", "a connected cloud is simply one cloud": one haze,
/// no darker where its circles overlap.
#[test]
fn overlapping_circles_are_one_haze() {
    let map = map();
    let circles = vec![(CENTRE, RADIUS), (CENTRE + Vec2::new(256.0, 0.0), RADIUS)];
    let area = Area::of(&map, &circles).unwrap();
    let (data, _) = texels(area, &[circles], &[], |_| true);
    assert_eq!(texel(&data, area, CENTRE + Vec2::new(128.0, 0.0)), CLOUD, "the overlap");
    assert_eq!(texel(&data, area, CENTRE + Vec2::new(256.0, 0.0)), CLOUD);
}

/// A release flashes over its circles, brighter than the haze as it starts, fading to nothing over
/// [`RELEASE_TICKS`]; over the viewer's Visible tiles alike.
#[test]
fn a_release_flashes_over_its_circles_and_fades() {
    assert_eq!(flash_alpha(0), FLASH_ALPHA as u8);
    let alphas: Vec<u8> = (0..=RELEASE_TICKS).map(flash_alpha).collect();
    assert!(alphas.windows(2).all(|w| w[0] > w[1]), "{alphas:?}");
    assert_eq!(flash_alpha(RELEASE_TICKS), 0);
    let map = map();
    let circles = vec![(CENTRE, RADIUS)];
    let area = Area::of(&map, &circles).unwrap();
    let flash = [(circles.clone(), flash_alpha(0))];
    let (data, _) = texels(area, std::slice::from_ref(&circles), &flash, |_| true);
    assert_eq!(
        texel(&data, area, CENTRE),
        [FLASH[0], FLASH[1], FLASH[2], flash_alpha(0)]
    );
    let faint = [(circles.clone(), 10)];
    let (data, _) = texels(area, std::slice::from_ref(&circles), &faint, |_| true);
    assert_eq!(texel(&data, area, CENTRE), CLOUD, "a fading flash under the haze");
    let (data, shown) = texels(area, &[], &flash, |tile| tile.y >= 9);
    assert!(shown.iter().all(|tile| tile.y == 9) && !shown.is_empty());
    assert_eq!(
        texel(&data, area, CENTRE)[3],
        0,
        "not over a tile the viewer doesn't see"
    );
}
