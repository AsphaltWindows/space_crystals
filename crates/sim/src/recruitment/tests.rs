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

/// Design change 0341 point 2 (`factions/cults/economy.md > RecruitmentHall > TileClaiming`, Claiming details:
/// "LocalUnitControlCapacity is rounded down everywhere (production and HUD): 33 claimed tiles of 100 gives 6.6, so
/// 6"): a Hall of 33 tiles makes 6 Recruits and no 7th; 50 tiles give 10 (`factions/cults/economy.md > RecruitmentHall
/// > BaseUnitControlCapacity - 20`: "at 50% effectiveness, capacity = 10"); under 5 tiles, under 1, none.
#[test]
fn a_capacity_is_rounded_down_everywhere() {
    let def = hall_def();
    let (third, half, full) = (Capacity::of(def, 33), Capacity::of(def, 50), Capacity::of(def, 100));
    assert_eq!((third.floor(), half.floor(), full.floor()), (6, 10, 20));
    assert!(third.has_room(5) && !third.has_room(6), "6.6 makes 6");
    assert!(half.has_room(9) && !half.has_room(10));
    assert!(full.has_room(19) && !full.has_room(20));
    assert!(!Capacity::of(def, 0).has_room(0), "no tiles, no Recruit");
    assert!(!Capacity::of(def, 4).has_room(0), "0.8 is 0");
    assert!(Capacity::of(def, 5).has_room(0) && !Capacity::of(def, 5).has_room(1));
}

/// `factions/cults/economy.md > RecruitmentHall > TileClaiming` and `factions/cults/economy.md > RecruitmentHall >
/// Effectiveness`: a lone Hall on Plane claims its whole area; Water isn't Recruitable, the map's edge cuts the area
/// short (**Placeholder (Q0099 point 1)**: the tiles off the map count among the 100 and can't be claimed), and
/// buildings, its own footprint and another player's, take nothing from it ("Buildings do not affect recruitability").
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

/// `factions/cults/economy.md > RecruitmentHall > TileClaiming` (design change 0341): of two Halls whose areas
/// overlap, the one completed first keeps the shared tiles ("Priority goes to the Hall completed first"), whoever owns
/// it (0341 point 5: "Different players' Halls compete for tiles"). Of two completed in the same tick, as a map's are,
/// each shared tile goes to the Hall whose centre is nearer: the shared columns split down the middle, 3 each. The
/// claims follow the completion ticks, not the ids.
#[test]
fn the_first_hall_completed_claims_the_shared_tiles() {
    // Areas x 5..=14 and 9..=18: 6 columns of 10 tiles shared, the centres at x 10 and 14.
    let sim = sim(
        &plane(24, 20),
        &[
            (ObjectKind::RecruitmentHall, Some(P1), 8, 8),
            (ObjectKind::RecruitmentHall, Some(P2), 12, 8),
        ],
    );
    let claims = sim.claims();
    assert_eq!(
        claims.iter().collect::<Vec<_>>(),
        [(ObjectId(1), 70), (ObjectId(2), 70)]
    );
    // The first one completed later: the second keeps them all; and the other way round.
    let (mut first, mut second) = (object(&sim, 1), object(&sim, 2));
    first.production.as_mut().expect("a producer").completed_at = 5;
    let claims = Claims::of([&first, &second], sim.map());
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (40, 100));
    first.production.as_mut().expect("a producer").completed_at = 0;
    second.production.as_mut().expect("a producer").completed_at = 5;
    let claims = Claims::of([&first, &second], sim.map());
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (100, 40));
    // A Hall that isn't operational claims nothing (0341 point 4: "A Hall under construction claims nothing, provides
    // no capacity and produces nothing until it is complete"): here a site.
    let mut site = object(&sim, 1);
    let tunnel_site = site_def(ObjectKind::Tunnel).expect("a site");
    site.construction = Some(Construction::new(tunnel_site, Builder::Held(ObjectId(9))));
    let claims = Claims::of([&site, &object(&sim, 2)], sim.map());
    assert_eq!((claims.tiles(ObjectId(1)), claims.tiles(ObjectId(2))), (0, 100));
    assert_eq!(claims.tiles(ObjectId(9)), 0, "not a Hall");
}

/// Design change 0341 point 3's tie rule (`factions/cults/economy.md > RecruitmentHall > TileClaiming`: "each
/// contested tile goes to the Hall whose centre is nearer; at exactly equal distance, to the Hall with the lower object
/// id"): Halls completed in the same tick at tiles (13, 8) and (8, 8), the lower id the eastern one. Their areas, x
/// 10..=19 and 5..=14, share the 5 columns 10..=14; column 12 is as far from both centres (x 15 and 10), and goes to
/// the lower id, so it has 3 of the shared columns and the other 2: contiguous parts.
#[test]
fn a_tile_as_near_two_halls_completed_together_goes_to_the_lower_id() {
    let sim = sim(
        &plane(24, 20),
        &[
            (ObjectKind::RecruitmentHall, Some(P1), 13, 8),
            (ObjectKind::RecruitmentHall, Some(P2), 8, 8),
        ],
    );
    let claims = sim.claims();
    assert_eq!(
        claims.iter().collect::<Vec<_>>(),
        [(ObjectId(1), 80), (ObjectId(2), 70)]
    );
    // Off the line through the centres, the same: the rows are the same for both, so only the columns count.
    let (east, west) = (object(&sim, 1), object(&sim, 2));
    let placement = |o: &Object| o.placement.expect("placed");
    let column = |x: i32| (5..=14).map(move |y| TilePos::new(x, y));
    for tile in column(12) {
        assert_eq!(
            distance_squared(&placement(&east), tile),
            distance_squared(&placement(&west), tile),
            "{tile:?}"
        );
    }
    assert!(
        distance_squared(&placement(&west), TilePos::new(11, 5))
            < distance_squared(&placement(&east), TilePos::new(11, 5))
    );
}

/// `factions/cults/economy.md > RecruitmentHall > Unit Control Tracking` (the Director's amendment to T56): each entry
/// of a lineage is a point at its Hall, a Jackal's two perhaps at the same Hall; an entry whose Hall is gone is its
/// owner's orphaned point, and a Cults unit without a lineage is orphaned whole (design change 0344: "Recruits no Hall
/// produced (custom maps, debug) are orphans from the start"). Another faction's unit costs nothing.
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

/// Design change 0342 point 3 (`factions/cults/economy.md > RecruitmentHall > Production Behavior`: "the Recruit in
/// production counts toward LocalUnitControlUsage from the start of its production"): a Hall's queued Recruit is a
/// point of its usage before any unit has its lineage, and isn't anyone's orphan.
#[test]
fn a_halls_recruit_in_production_counts_toward_its_usage() {
    let sim = sim(&plane(24, 20), &[(ObjectKind::RecruitmentHall, Some(P1), 8, 8)]);
    let mut hall = object(&sim, 1);
    assert_eq!(Lineages::of([&hall]).usage(ObjectId(1)), 0);
    hall.production.as_mut().expect("a producer").queue = vec![ObjectKind::Recruit];
    let lineages = Lineages::of([&hall]);
    assert_eq!((lineages.usage(ObjectId(1)), lineages.orphaned(P1)), (1, 0));
}
