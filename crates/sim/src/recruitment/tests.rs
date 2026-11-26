use super::*;
use crate::construction::site::{Builder, Construction, site_def};
use crate::defs::{Footprint, ObjectKind};
use crate::ids::PlayerId;
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The Recruitment Hall's own: a 10x10 area and a capacity of 20 (`factions/cults/economy.md > RecruitmentHall`).
fn hall_def() -> &'static RecruitmentDef {
    def_of(ObjectKind::RecruitmentHall).expect("the Hall recruits")
}

/// A Cults game on `ascii` with `spawns`, Halls and units at tiles, as player 1's or player 2's.
fn sim(ascii: &str, spawns: &[(ObjectKind, Option<PlayerId>, i32, i32)]) -> Sim {
    let mut builder = ScenarioBuilder::new(ascii)
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults);
    for &(kind, owner, x, y) in spawns {
        let at = match kind.def().tile_size() {
            Some(_) => SpawnAt::Tile(TilePos::new(x, y)),
            None => SpawnAt::Point(TilePos::new(x, y).center()),
        };
        builder = builder.spawn(kind, owner, at);
    }
    Sim::new(&builder.build())
}

fn plane(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn object(sim: &Sim, id: u32) -> Object {
    sim.object(ObjectId(id)).expect("in the game").clone()
}

/// `factions/cults/economy.md > RecruitmentHall > RecruitmentArea`: "A 10x10 grid unit area centered on the 4x4
/// building": 3 tiles on every side of the footprint, row by row.
#[test]
fn the_area_is_ten_tiles_square_around_the_footprint() {
    let placement = Placement::new(TilePos::new(5, 7), Footprint::Tiles { w: 4, h: 4 });
    let tiles: Vec<TilePos> = area(&placement, hall_def()).collect();
    assert_eq!(tiles.len(), 100);
    assert_eq!(
        (tiles[0], tiles[9], tiles[99]),
        (TilePos::new(2, 4), TilePos::new(11, 4), TilePos::new(11, 13))
    );
    let set: BTreeSet<TilePos> = tiles.iter().copied().collect();
    assert_eq!(set.len(), 100, "no tile twice");
    assert!(placement.tiles().all(|t| set.contains(&t)), "the footprint is in it");
}

/// **Placeholder (Q0099 point 2)**: a capacity is compared exactly and shown rounded down: 33 tiles give 6.6, so a
/// Hall makes a 7th Recruit and shows 6; 50 tiles give 10 (`factions/cults/economy.md > RecruitmentHall >
/// BaseUnitControlCapacity - 20`: "at 50% effectiveness, capacity = 10"). Sums are exact too.
#[test]
fn a_capacity_is_exact_and_shown_rounded_down() {
    let def = hall_def();
    let (third, half, full) = (Capacity::of(def, 33), Capacity::of(def, 50), Capacity::of(def, 100));
    assert!(third.has_room(6) && !third.has_room(7));
    assert_eq!(third.floor(), 6);
    assert!(half.has_room(9) && !half.has_room(10));
    assert_eq!((half.floor(), full.floor()), (10, 20));
    assert!(!Capacity::of(def, 0).has_room(0), "no tiles, no Recruit");
    assert_eq!(third.plus(third).floor(), 13, "6.6 + 6.6");
    assert_eq!(Capacity::ZERO.plus(half), half);
    // Denominators of their own: 1/3 + 1/6 = 1/2, and twice that is 1.
    let (a, b) = (Capacity { num: 1, den: 3 }, Capacity { num: 1, den: 6 });
    assert_eq!(a.plus(b), Capacity { num: 1, den: 2 });
    assert_eq!(a.plus(b).plus(a.plus(b)).floor(), 1);
}

/// `factions/cults/economy.md > RecruitmentHall > TileClaiming` and `factions/cults/economy.md > RecruitmentHall >
/// Effectiveness`: a lone Hall on Plane claims its whole area; Water isn't Recruitable, the map's edge cuts the area
/// of a Hall spawned where none can be placed short (design change 0341 point 1 refuses its placement; **Placeholder
/// (not in the design: 0341 rules such a Hall out)**: the tiles off the map count among the 100 and can't be
/// claimed), and buildings, its own footprint and another player's, take nothing from it ("Buildings do not affect
/// recruitability").
#[test]
fn a_hall_claims_the_recruitable_tiles_of_its_area() {
    let lone = sim(&plane(20, 20), &[(ObjectKind::RecruitmentHall, Some(P1), 8, 8)]);
    assert_eq!(lone.claims().tiles(ObjectId(1)), 100);
    // Two rows of Water in the area, the Hall's own row of Plane kept.
    let mut rows = vec![".".repeat(20); 20];
    rows[5] = "~".repeat(20);
    rows[15] = "~".repeat(20);
    let wet = sim(&rows.join("\n"), &[(ObjectKind::RecruitmentHall, Some(P1), 8, 8)]);
    assert_eq!(
        wet.claims().tiles(ObjectId(1)),
        90,
        "row 15 is out of the area (5..=14)"
    );
    // In the corner: rows and columns 0..=6 of the area's -3..=6 are on the map.
    let corner = sim(&plane(20, 20), &[(ObjectKind::RecruitmentHall, Some(P1), 0, 0)]);
    assert_eq!(corner.claims().tiles(ObjectId(1)), 49);
    let hall = |sim: &Sim| sim.hall_state(ObjectId(1)).expect("a Hall");
    assert_eq!((hall(&corner).tiles, hall(&corner).effectiveness_percent()), (100, 49));
    // A building in the area, its own player's or another's, takes no tile from it.
    let built = sim(
        &plane(24, 20),
        &[
            (ObjectKind::RecruitmentHall, Some(P1), 8, 8),
            (ObjectKind::Storage, Some(P1), 5, 5),
            (ObjectKind::Generator, Some(P2), 12, 12),
        ],
    );
    assert_eq!(built.claims().tiles(ObjectId(1)), 100);
}

/// `factions/cults/economy.md > RecruitmentHall > TileClaiming`: of two Halls whose areas overlap, the one completed
/// first keeps the shared tiles, whoever owns it (**Placeholder (Q0099 point 5)**), and of two completed in the same
/// tick, the one placed first (**Placeholder (Q0099 point 3)**). The claims follow the completion ticks, not the ids.
#[test]
fn the_first_hall_completed_claims_the_shared_tiles() {
    // Areas x 5..=14 and 9..=18: 6 columns of 10 tiles shared.
    let sim = sim(
        &plane(24, 20),
        &[
            (ObjectKind::RecruitmentHall, Some(P1), 8, 8),
            (ObjectKind::RecruitmentHall, Some(P2), 12, 8),
        ],
    );
    let claims = sim.claims();
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (100, 40));
    assert_eq!(
        claims.iter().collect::<Vec<_>>(),
        [(ObjectId(1), 100), (ObjectId(2), 40)]
    );
    // The first one completed later: the second keeps them.
    let (mut first, second) = (object(&sim, 1), object(&sim, 2));
    first.production.as_mut().expect("a producer").completed_at = 5;
    let claims = Claims::of([&first, &second], sim.map());
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (40, 100));
    // A Hall that isn't operational claims nothing (**Placeholder (Q0099 point 4)**): here a site.
    let mut site = object(&sim, 1);
    let tunnel_site = site_def(ObjectKind::Tunnel).expect("a site");
    site.construction = Some(Construction::new(tunnel_site, Builder::Held(ObjectId(9))));
    let claims = Claims::of([&site, &second], sim.map());
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (0, 100));
    assert_eq!(claims.tiles(ObjectId(9)), 0, "not a Hall");
}

/// `factions/cults/economy.md > RecruitmentHall > Unit Control Tracking` (the Director's amendment to T56): each entry
/// of a lineage is a point at its Hall, a Jackal's two perhaps at the same Hall; an entry whose Hall is gone is its
/// owner's orphaned point, and a Cults unit without a lineage is orphaned whole (**Placeholder (Q0101 point 2)**).
/// Another faction's unit costs nothing.
#[test]
fn each_entry_of_a_lineage_is_a_point_at_its_hall() {
    let sim = sim(
        &plane(24, 20),
        &[
            (ObjectKind::RecruitmentHall, Some(P1), 8, 8),
            (ObjectKind::Recruit, Some(P1), 1, 1),
            (ObjectKind::Militant, Some(P1), 2, 1),
            (ObjectKind::Recruit, Some(P2), 3, 1),
            (ObjectKind::Peacekeeper, Some(P1), 4, 1),
        ],
    );
    let (hall, mut recruit, mut militant, theirs, peacekeeper) = (
        object(&sim, 1),
        object(&sim, 2),
        object(&sim, 3),
        object(&sim, 4),
        object(&sim, 5),
    );
    let all = |objects: &[&Object]| Lineages::of(objects.iter().copied());
    let none = all(&[&hall, &recruit, &militant, &theirs, &peacekeeper]);
    assert_eq!(
        (none.usage(ObjectId(1)), none.orphaned(P1), none.orphaned(P2)),
        (0, 2, 1)
    );
    recruit.lineage = vec![ObjectId(1)];
    militant.lineage = vec![ObjectId(1), ObjectId(1), ObjectId(77)];
    let some = all(&[&hall, &recruit, &militant, &theirs, &peacekeeper]);
    assert_eq!(
        (some.usage(ObjectId(1)), some.orphaned(P1), some.orphaned(P2)),
        (3, 1, 1)
    );
    // The Hall gone: its points are orphaned.
    let gone = all(&[&recruit, &militant, &theirs, &peacekeeper]);
    assert_eq!((gone.usage(ObjectId(1)), gone.orphaned(P1)), (0, 4));
}
