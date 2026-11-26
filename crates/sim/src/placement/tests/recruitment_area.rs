//! A Recruitment Hall's placement by its footprint alone (slice M5-S4; design change 0387, which replaced slice M5-J1's
//! rule, 0341 point 1's "The whole 10x10 area must lie on the map"): right up to the map's edge and in its corners, its
//! Recruitment Area reaching off the map, in every mode.

use super::*;

use ObjectKind::RecruitmentHall;

const P1: PlayerId = PlayerId(1);

/// [`check`] of a Recruitment Hall at `(x, y)` on `sim`'s map in `mode`, built from outside as the Recruits build it.
fn hall_in(sim: &Sim, mode: Mode<'_>, x: i32, y: i32) -> PlacementCheck {
    let area = BuildArea::default();
    let site = Site {
        map: sim.map(),
        area: &area,
        mode,
        method: Method::WorkerBuilt,
    };
    check(&site, [], RecruitmentHall, t(x, y), BuildingOrientation::default())
}

/// Design change 0387's Verification: "Place a Hall flush against the map edge: it is accepted"
/// (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "a Hall can be placed anywhere its 4x4 footprint
/// fits on the map, including right at the edge"). On a 20 × 16 map, a Hall flush against each edge, 3 rows or columns
/// of its area off the map, and in each corner, both edges cutting its area, is valid; so is one 2 tiles from an edge,
/// which slice M5-J1 refused (design change 0371: "one 2 tiles from the edge is refused"). Only its footprint counts: a
/// Hall a column past the edge has that column's tiles off the map, its one problem.
#[test]
fn a_hall_may_stand_right_up_to_the_map_edge() {
    let map = Map::new(20, 16);
    let area = BuildArea::default();
    let hall = |x, y| place(&map, &area, &[], RecruitmentHall, x, y);
    let edges = [(0, 6), (8, 0), (16, 6), (8, 12)];
    let corners = [(0, 0), (16, 0), (0, 12), (16, 12)];
    for (x, y) in edges.into_iter().chain(corners).chain([(2, 6)]) {
        assert_eq!(hall(x, y), PlacementCheck::default(), "({x}, {y})");
    }
    let past = hall(17, 6);
    assert_eq!(past.whole, []);
    assert_eq!(past.tiles, (6..10).map(|y| (t(20, y), OffMap)).collect::<Vec<_>>());
}

/// Design change 0387: "A Hall can be placed wherever its 4x4 footprint fits on the map", whoever checks it: a Hall
/// flush against the north edge, in player 1's sight, is valid in every mode, the Recruits' ghost's (`Mode::Known`),
/// the check on arrival (`Mode::Arrival`), a Starting Layout's (`Mode::Layout`) and a game's direct placement
/// (`Mode::Command`). Out of sight, a direct placement's one problem is its own tiles' (`TileProblem::NotVisible`),
/// nothing of the whole.
#[test]
fn every_mode_takes_a_hall_at_the_edge() {
    let rows = vec![".".repeat(20); 16].join("\n");
    let sim = Sim::new(
        &ScenarioBuilder::new(&rows)
            .spawn(RecruitmentHall, Some(P1), crate::SpawnAt::Tile(t(8, 4)))
            .build(),
    );
    let footprint = |x0: i32| (0..4).flat_map(move |y| (x0..x0 + 4).map(move |x| t(x, y)));
    assert!(footprint(8).all(|tile| sim.visibility(P1, tile) == VisibilityState::Visible));
    let known = Mode::Known {
        vision: sim.vision(),
        player: P1,
    };
    let command = Mode::Command {
        vision: sim.vision(),
        player: P1,
    };
    for mode in [known, Mode::Arrival, Mode::Layout, command] {
        assert_eq!(hall_in(&sim, mode, 8, 0), PlacementCheck::default(), "{mode:?}");
    }
    assert!(footprint(16).all(|tile| sim.visibility(P1, tile) != VisibilityState::Visible));
    let unseen = hall_in(&sim, command, 16, 0);
    assert_eq!(unseen.whole, []);
    assert_eq!(
        unseen.tiles,
        footprint(16).map(|tile| (tile, NotVisible)).collect::<Vec<_>>()
    );
}
