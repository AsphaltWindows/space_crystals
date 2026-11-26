//! The Recruitment Area's rule (slice M5-J1, `placement/recruitment_area.rs`): a Recruitment Hall's whole 10 × 10 area
//! on the map (design change 0341 point 1), in every mode, the map's edge being known.

use super::*;

use ObjectKind::{RecruitmentHall, Storage};
use WholeProblem::RecruitmentAreaOffMap;

/// [`check`] of a Recruitment Hall at `(x, y)` on `sim`'s map in `mode`, built from outside as the Recruits build it:
/// the problems as a whole.
fn hall_in(sim: &Sim, mode: Mode<'_>, x: i32, y: i32) -> Vec<WholeProblem> {
    let area = BuildArea::default();
    let site = Site {
        map: sim.map(),
        area: &area,
        mode,
        method: Method::WorkerBuilt,
    };
    check(&site, [], RecruitmentHall, t(x, y), BuildingOrientation::default()).whole
}

/// `factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "**The whole area must lie on the map** ...: a
/// Hall whose area would leave the map can't be placed (as Tunnels, 0242)". The area reaches 3 tiles past the Hall's 4
/// × 4 on every side (`recruitment::area`). With 2 tiles between the footprint and an edge, a row or a column of the
/// area lies off the map, and the Hall is refused as a whole, its own tiles being fine; with 3 tiles, the area just
/// reaches the edge and the Hall is valid (design change 0371: "0341's Verification example is off by one. A Hall 3
/// tiles from the edge is accepted; one 2 tiles from the edge is refused"; Tunnels count their gap so, design change
/// 0283).
/// Only a kind that recruits has the rule.
#[test]
fn a_halls_area_must_be_on_the_map() {
    let map = Map::new(20, 16);
    let area = BuildArea::default();
    let hall = |x, y| place(&map, &area, &[], RecruitmentHall, x, y);
    // 2 tiles from the west, north, east and south edges, and in the corner.
    for (x, y) in [(2, 6), (8, 2), (14, 6), (8, 10), (0, 0)] {
        let check = hall(x, y);
        assert_eq!(
            (check.tiles, check.whole),
            (vec![], vec![RecruitmentAreaOffMap]),
            "({x}, {y})"
        );
    }
    for (x, y) in [(3, 3), (13, 9), (3, 9), (13, 3)] {
        assert!(hall(x, y).is_valid(), "3 tiles from the edges: ({x}, {y})");
    }
    assert!(place(&map, &area, &[], Storage, 0, 0).is_valid());
    // Past the edge, the footprint's own tiles are off the map as well; far past it, nothing panics.
    let off = hall(18, 6);
    assert_eq!(off.whole, [RecruitmentAreaOffMap]);
    assert!(!off.tiles.is_empty() && off.tiles.iter().all(|&(_, p)| p == OffMap));
    for (x, y) in [(-1, 5), (i32::MAX - 1, 0), (i32::MIN, i32::MIN)] {
        assert_eq!(hall(x, y).whole, [RecruitmentAreaOffMap], "({x}, {y})");
    }
}

/// 0341 is "as Tunnels, 0242": the map's edge is known, so the ghost of a player who never explored the map
/// (`Mode::Known`) refuses a Hall 2 tiles from the edge, as the gate (`Mode::Command`) and the check on arrival
/// (`Mode::Arrival`) do, and passes one 3 tiles from it.
#[test]
fn every_mode_knows_the_edge() {
    let sim = Sim::new(&ScenarioBuilder::new(&vec![".".repeat(20); 16].join("\n")).build());
    assert_eq!(sim.visibility(PlayerId(1), t(2, 6)), VisibilityState::Unexplored);
    let known = Mode::Known {
        vision: sim.vision(),
        player: PlayerId(1),
    };
    let command = Mode::Command {
        vision: sim.vision(),
        player: PlayerId(1),
    };
    for mode in [known, Mode::Arrival, command] {
        assert_eq!(hall_in(&sim, mode, 2, 6), [RecruitmentAreaOffMap], "{mode:?}");
    }
    assert_eq!(hall_in(&sim, known, 3, 6), []);
    assert_eq!(hall_in(&sim, Mode::Arrival, 3, 6), []);
}
