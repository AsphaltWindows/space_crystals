use super::*;
use crate::defs::Footprint;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;

/// A map of `w` × `h` tiles of Plane, Water on the rows `wet`.
fn map(w: usize, h: usize, wet: &[usize]) -> Map {
    let mut rows = vec![".".repeat(w); h];
    for &row in wet {
        rows[row] = "~".repeat(w);
    }
    Sim::new(&ScenarioBuilder::new(&rows.join("\n")).build()).map().clone()
}

/// A Hall's 4x4 footprint with its top-left tile at `(x, y)`.
fn at(x: i32, y: i32) -> Placement {
    Placement::new(TilePos::new(x, y), Footprint::Tiles { w: 4, h: 4 })
}

fn claimant(id: Option<u32>, (x, y): (i32, i32), claimer: Claimer, rank: Rank) -> Claimant {
    Claimant {
        id: id.map(ObjectId),
        kind: HALL,
        placement: at(x, y),
        claimer,
        rank,
    }
}

fn everywhere(_: TilePos) -> bool {
    true
}

/// What the preview of a Hall at `(x, y)` among `claimants` finds on `tile`.
fn found(preview: &Preview, x: i32, y: i32) -> PreviewTile {
    let tile = TilePos::new(x, y);
    (preview.tiles.iter().find(|(t, _)| *t == tile)).map_or_else(|| panic!("{tile:?} in the area"), |&(_, f)| f)
}

/// `factions/cults/structures/recruitment_hall.md > Recruitment Area display`: "the tiles it would claim [...] and its
/// Effectiveness in %": a lone Hall on Plane would claim its whole area, 100 tiles row by row, at 100%; a kind without
/// an area has no preview.
#[test]
fn a_lone_hall_would_claim_its_whole_area() {
    let map = map(40, 40, &[]);
    let preview = Preview::of(HALL, at(10, 10), &[], &map, everywhere).expect("a Hall");
    assert_eq!(preview.tiles.len(), 100);
    assert_eq!(
        (preview.tiles[0].0, preview.tiles[99].0),
        (TilePos::new(7, 7), TilePos::new(16, 16))
    );
    assert!(preview.tiles.iter().all(|(_, f)| *f == PreviewTile::Claimable));
    assert_eq!((preview.claimed(), preview.effectiveness_percent()), (100, 100));
    assert_eq!(
        Preview::of(ObjectKind::Storage, at(10, 10), &[], &map, everywhere),
        None
    );
}

/// "The tiles already claimed by your own Halls" and those of the other players' Halls the player knows of (design
/// changes 0350 and 0352: "the tiles they claim are shown as unclaimable") are taken, and the Effectiveness drops with
/// them: an own Hall 6 tiles west leaves the 4 columns its area shares with this one taken, an other player's 6 tiles
/// south the 4 rows, the corner where both reach the own Hall's or the other's by the tie rule.
#[test]
fn the_tiles_of_the_halls_known_are_taken() {
    let map = map(40, 40, &[]);
    let own = claimant(Some(1), (4, 10), Claimer::Own, Rank::Completed(0));
    let other = claimant(Some(2), (10, 16), Claimer::Other, Rank::Completed(0));
    let preview = Preview::of(HALL, at(10, 10), &[own, other], &map, everywhere).expect("a Hall");
    // The area is x 7..=16, y 7..=16; the own Hall's x 1..=10, the other's y 13..=22.
    assert_eq!(
        found(&preview, 7, 7),
        PreviewTile::Taken {
            claimer: Claimer::Own,
            by: 0
        }
    );
    assert_eq!(found(&preview, 11, 7), PreviewTile::Claimable);
    assert_eq!(
        found(&preview, 16, 16),
        PreviewTile::Taken {
            claimer: Claimer::Other,
            by: 1
        }
    );
    assert_eq!(found(&preview, 11, 12), PreviewTile::Claimable);
    // 6 columns by 6 rows are left.
    assert_eq!((preview.claimed(), preview.effectiveness_percent()), (36, 36));
}

/// Design change 0351: "The preview treats own pending Halls as claimants, in their order of placement", after the
/// completed Halls, and before the Hall placed ("the overlap shows as taken"): whoever is nearer, a completed Hall
/// beats a pending one, the one placed first beats a later one, and every one of them the Hall placed.
#[test]
fn pending_halls_claim_after_the_completed_in_their_order_of_placement() {
    let map = map(40, 40, &[]);
    // A pending Hall 3 tiles west of the one placed, completed ones 6 tiles west and 6 east: on the tiles the pending
    // one shares with them it is nearer, and they have them all the same.
    let near = claimant(Some(9), (7, 10), Claimer::OwnPending, Rank::Pending(0));
    let west = claimant(Some(1), (4, 10), Claimer::Own, Rank::Completed(300));
    let east = claimant(Some(2), (16, 10), Claimer::Other, Rank::Completed(900));
    let preview = Preview::of(HALL, at(10, 10), &[near, west, east], &map, everywhere).expect("a Hall");
    assert_eq!(
        found(&preview, 8, 10),
        PreviewTile::Taken {
            claimer: Claimer::Own,
            by: 1
        }
    );
    assert_eq!(
        found(&preview, 11, 10),
        PreviewTile::Taken {
            claimer: Claimer::OwnPending,
            by: 0
        }
    );
    assert_eq!(
        found(&preview, 13, 10),
        PreviewTile::Taken {
            claimer: Claimer::Other,
            by: 2
        }
    );
    assert_eq!(
        found(&preview, 15, 10),
        PreviewTile::Taken {
            claimer: Claimer::Other,
            by: 2
        }
    );
    // The pending Hall's x 4..=13 and the other's 13..=22 cover the area's x 7..=16: nothing is left.
    assert_eq!(preview.claimed(), 0);
    // Of two pending Halls the one placed first has the tiles they share, though the other is nearer them.
    let first = claimant(None, (4, 10), Claimer::OwnPending, Rank::Pending(0));
    let second = claimant(Some(7), (7, 10), Claimer::OwnPending, Rank::Pending(1));
    for claimants in [[first, second], [second, first]] {
        let preview = Preview::of(HALL, at(10, 10), &claimants, &map, everywhere).expect("a Hall");
        let by = claimants
            .iter()
            .position(|c| c.rank == Rank::Pending(0))
            .expect("listed");
        assert_eq!(
            found(&preview, 9, 10),
            PreviewTile::Taken {
                claimer: Claimer::OwnPending,
                by
            }
        );
        assert_eq!(
            found(&preview, 12, 10),
            PreviewTile::Taken {
                claimer: Claimer::OwnPending,
                by: 1 - by
            }
        );
        assert_eq!(found(&preview, 14, 10), PreviewTile::Claimable);
        assert_eq!(preview.claimed(), 30, "x 14..=16 left");
    }
}

/// `factions/cults/structures/recruitment_hall.md > TileClaiming`, Claiming details (design change 0341): of Halls
/// completed in the same tick "each contested tile goes to the Hall whose centre is nearer; at exactly equal distance,
/// to the Hall with the lower object id", as the game's claims split them: two Halls 5 tiles apart are equally far
/// from the column between them, which goes to the lower id, whoever's it is; with different ticks, the first one
/// completed has it all.
#[test]
fn completed_halls_split_their_tiles_by_the_tie_rule() {
    let map = map(40, 40, &[]);
    let preview_with = |ids: (u32, u32), ticks: (Tick, Tick)| {
        let own = claimant(Some(ids.0), (10, 10), Claimer::Own, Rank::Completed(ticks.0));
        let other = claimant(Some(ids.1), (15, 10), Claimer::Other, Rank::Completed(ticks.1));
        Preview::of(HALL, at(12, 14), &[own, other], &map, everywhere).expect("a Hall")
    };
    let taken = |claimer: Claimer, by: usize| PreviewTile::Taken { claimer, by };
    // The placed area is x 9..=18, y 11..=20; the two Halls' y 7..=16, x 7..=16 and 12..=21.
    let lower_own = preview_with((1, 2), (0, 0));
    assert_eq!(found(&lower_own, 13, 11), taken(Claimer::Own, 0), "nearer the own Hall");
    assert_eq!(
        found(&lower_own, 14, 11),
        taken(Claimer::Own, 0),
        "equally far: the lower id"
    );
    assert_eq!(found(&lower_own, 15, 16), taken(Claimer::Other, 1), "nearer the other");
    let lower_other = preview_with((2, 1), (0, 0));
    assert_eq!(
        found(&lower_other, 14, 11),
        taken(Claimer::Other, 1),
        "equally far: the lower id"
    );
    assert_eq!(found(&lower_other, 13, 11), taken(Claimer::Own, 0));
    let own_first = preview_with((2, 1), (5, 6));
    assert_eq!(found(&own_first, 15, 16), taken(Claimer::Own, 0), "completed first");
    for preview in [lower_own, lower_other, own_first] {
        assert_eq!(preview.claimed(), 40, "rows 17..=20 are left");
        assert_eq!(found(&preview, 9, 17), PreviewTile::Claimable);
    }
    // The game's claims split them the same way.
    let ascii = vec![".".repeat(40); 40].join("\n");
    let game = Sim::new(
        &ScenarioBuilder::new(&ascii)
            .faction(crate::ids::PlayerId(1), crate::player::Faction::Cults)
            .faction(crate::ids::PlayerId(2), crate::player::Faction::Cults)
            .spawn(
                HALL,
                Some(crate::ids::PlayerId(1)),
                crate::scenario::SpawnAt::Tile(TilePos::new(10, 10)),
            )
            .spawn(
                HALL,
                Some(crate::ids::PlayerId(2)),
                crate::scenario::SpawnAt::Tile(TilePos::new(15, 10)),
            )
            .build(),
    );
    let first = crate::recruitment::claimed_tiles(game.objects(), game.map(), ObjectId(1));
    assert!(first.contains(&TilePos::new(14, 11)) && !first.contains(&TilePos::new(15, 16)));
    assert_eq!(first.len(), 80, "its area's columns up to the middle one: 8 of 10");
    assert_eq!(game.claims().tiles(ObjectId(1)), 80);
}

/// The tiles of the area off the map count among its tiles, and can't be claimed (`super::super::area`: placement
/// refuses such a Hall, design change 0341 point 1, but its ghost may stand there): in the corner, 49 of 100.
#[test]
fn off_map_tiles_count_and_cannot_be_claimed() {
    let map = map(20, 20, &[]);
    let preview = Preview::of(HALL, at(0, 0), &[], &map, everywhere).expect("a Hall");
    assert_eq!(preview.tiles.len(), 100);
    assert_eq!(found(&preview, -3, -3), PreviewTile::NotRecruitable);
    assert_eq!(found(&preview, 6, -1), PreviewTile::NotRecruitable);
    assert_eq!(found(&preview, 0, 0), PreviewTile::Claimable);
    assert_eq!((preview.claimed(), preview.effectiveness_percent()), (49, 49));
}

/// "The non-Recruitable tiles" (`TilePreset::recruitable`: Water) as far as the player knows the ground: known Water
/// can't be claimed; ground they never explored passes, as in the placement checks (design change 0120: "Never-explored
/// tiles pass"), so the preview may be optimistic there, never pessimistic.
#[test]
fn known_water_is_not_recruitable() {
    let map = map(40, 40, &[8, 30]);
    let preview = Preview::of(HALL, at(10, 10), &[], &map, everywhere).expect("a Hall");
    assert_eq!(found(&preview, 12, 8), PreviewTile::NotRecruitable);
    assert_eq!(found(&preview, 12, 9), PreviewTile::Claimable);
    assert_eq!(preview.effectiveness_percent(), 90);
    let unexplored = Preview::of(HALL, at(10, 10), &[], &map, |t| t.x < 12).expect("a Hall");
    assert_eq!(found(&unexplored, 11, 8), PreviewTile::NotRecruitable);
    assert_eq!(found(&unexplored, 12, 8), PreviewTile::Claimable, "never explored");
    assert_eq!(unexplored.effectiveness_percent(), 95);
}

/// The Effectiveness is rounded down (`crate::recruitment::effectiveness_percent`; design change 0341 point 2, as the
/// capacity), though the Hall's 100 tiles make every count a whole percent.
#[test]
fn the_effectiveness_is_rounded_down() {
    assert_eq!(effectiveness_percent(2, 3), 66);
    assert_eq!(effectiveness_percent(49, 100), 49);
    assert_eq!(effectiveness_percent(0, 0), 0);
}
