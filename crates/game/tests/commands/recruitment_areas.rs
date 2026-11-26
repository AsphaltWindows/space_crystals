//! The Recruitment Area display in the client (slice M5-J2; `factions/cults/structures/recruitment_hall.md >
//! Recruitment Area display`, design changes 0350, 0351 and 0352, answering Q0107), with the fog of war on: the
//! preview of a Hall placed beside an own Hall, an other player's Hall in sight, one never seen, one remembered and
//! destroyed unseen, and an own ghost Hall; its Effectiveness by the ghost; a selected own Hall's claimed tiles; and
//! never an enemy's area.

use sc_sim::recruitment::preview::{Claimer, PreviewTile};
use sc_sim::testing::ScenarioBuilder;
use sc_sim::tuning::vision::FOG_DELAY_TICKS;
use sc_sim::{Faction, SU_PER_GU};
use space_crystals::placement::RecruitmentAreaOverlay;
use space_crystals::placement::recruitment_areas::EffectivenessLabel;

use super::*;
use KeyCode::{KeyA, KeyQ, ShiftLeft};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HALL: ObjectKind = ObjectKind::RecruitmentHall;

/// The objects of [`field`], by id, and where the Halls stand (the map is flat: where they are drawn).
const OWN_HALL: u32 = 1;
const RECRUITS: [u32; 4] = [2, 3, 4, 5];
const SEEN_HALL: u32 = 6;
const SCOUT: u32 = 7;
const UNSEEN_HALL: u32 = 8;
/// Player 1's Hall's tiles (8, 8) to (11, 11), its area's (5, 5) to (14, 14); player 2's Hall in sight (26, 8) to
/// (29, 11), its area's (23, 5) to (32, 14); player 2's Hall never seen (46, 30) to (49, 33), its area's (43, 27) to
/// (52, 36).
const OWN_HALL_AT: TilePos = TilePos::new(8, 8);
const SEEN_HALL_AT: TilePos = TilePos::new(26, 8);
const UNSEEN_HALL_AT: TilePos = TilePos::new(46, 30);
/// Player 1's scout, 2 tiles south of player 2's Hall.
const SCOUT_AT: Vec2 = Vec2::new(1760.0, 864.0);

/// A game of player 1 against player 2, both the Cults, on a flat 64 × 48 map, with the fog of war on and no
/// auto-targeting: player 1's Hall in the north-west, four Recruits in the south-west, and a Militant scouting player
/// 2's Hall in the north; player 2's other Hall in the south-east, which no one of player 1's ever sees. Player 1 has
/// 1000 Space Crystals.
fn field() -> App {
    let tile = SpawnAt::Tile;
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(HALL, Some(P1), tile(OWN_HALL_AT));
    for x in [10.5, 11.5, 12.5, 13.5] {
        builder = builder.spawn(
            ObjectKind::Recruit,
            Some(P1),
            SpawnAt::Point(Vec2::new(x, 30.5) * SU_PER_GU),
        );
    }
    let scenario = builder
        .spawn(HALL, Some(P2), tile(SEEN_HALL_AT))
        .spawn(ObjectKind::Militant, Some(P1), SpawnAt::Point(SCOUT_AT))
        .spawn(HALL, Some(P2), tile(UNSEEN_HALL_AT))
        .build();
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, P1, Command::Debug(crystals));
    updates(&mut app, 3);
    app
}

/// The view point in the middle of a Hall's 4 × 4 footprint at `origin`.
fn middle(origin: TilePos) -> Vec2 {
    origin.corner() + Vec2::splat(2.0 * SU_PER_GU)
}

/// Centres the camera on the view point `view` (a frame).
fn look_at(app: &mut App, view: Vec2) {
    app.world_mut().resource_mut::<CameraView>().center = view;
    app.update();
}

/// Selects player 1's four Recruits and enters their Recruitment Hall's placement: A then Q
/// (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, slice M5-J1).
fn place_a_hall(app: &mut App) {
    look_at(app, pos(app, RECRUITS[1]));
    select_units(app, &RECRUITS);
    tap(app, &[], KeyA);
    tap(app, &[], KeyQ);
    assert!(matches!(interface(app), InterfaceState::AwaitingPlacement(_)));
}

/// Moves the Hall's ghost onto `origin`, the camera over it.
fn ghost_at(app: &mut App, origin: TilePos) {
    look_at(app, middle(origin));
    pointer_over(app, middle(origin));
}

fn overlay(app: &App) -> &RecruitmentAreaOverlay {
    app.world().resource::<RecruitmentAreaOverlay>()
}

/// What the preview shows: how many of its tiles are claimable, taken by an own completed Hall, by an own Hall not
/// completed yet and by another player's Hall, and not Recruitable; and its Effectiveness.
fn counts(app: &App) -> ([usize; 5], u32) {
    let shown = overlay(app).preview.as_ref().expect("a preview");
    let tiles = &shown.preview.tiles;
    let count = |wanted: &dyn Fn(PreviewTile) -> bool| tiles.iter().filter(|&&(_, found)| wanted(found)).count();
    let by = |claimer| move |found| matches!(found, PreviewTile::Taken { claimer: c, .. } if c == claimer);
    let counts = [
        count(&|found| found == PreviewTile::Claimable),
        count(&by(Claimer::Own)),
        count(&by(Claimer::OwnPending)),
        count(&by(Claimer::Other)),
        count(&|found| found == PreviewTile::NotRecruitable),
    ];
    (counts, shown.preview.effectiveness_percent())
}

/// The text of the Effectiveness label by the ghost.
fn label(app: &mut App) -> String {
    let mut query = app.world_mut().query_filtered::<&Text2d, With<EffectivenessLabel>>();
    let texts: Vec<String> = query.iter(app.world()).map(|text| text.0.clone()).collect();
    assert_eq!(texts.len(), 1, "one label");
    texts[0].clone()
}

/// The tiles of the area of a Hall at `origin`, row by row.
fn area_of(origin: TilePos) -> Vec<TilePos> {
    let (x0, y0) = (origin.x - 3, origin.y - 3);
    (y0..y0 + 10)
        .flat_map(|y| (x0..x0 + 10).map(move |x| TilePos::new(x, y)))
        .collect()
}

/// `factions/cults/structures/recruitment_hall.md > Recruitment Area display`: "Placing a Hall shows its 10x10 area:
/// the tiles it would claim, the tiles already claimed by your own Halls [...] and its Effectiveness in %", that by
/// the ghost; and design change 0350's Verification: "With an enemy Hall in sight, the preview next to it shows its
/// claimed tiles as unavailable, and the Effectiveness drops accordingly". Beside the own Hall, the 4 columns their
/// areas share are the own Hall's: 60%; beside player 2's Hall in sight, the 4 columns its area shares are its: 60%;
/// beside player 2's Hall never seen, nothing is taken (0350: "Unseen enemy Halls aren't counted"): 100%, though that
/// Hall claims the same 4 columns in the game. Only the ghost's own area is drawn ("Enemy Halls' areas are never drawn
/// as areas of their own").
#[test]
fn the_preview_counts_the_own_halls_and_those_seen() {
    let mut app = field();
    place_a_hall(&mut app);
    let beside_own = TilePos::new(14, 8);
    ghost_at(&mut app, beside_own);
    assert_eq!(counts(&app), ([60, 40, 0, 0, 0], 60));
    assert_eq!(label(&mut app), "Effectiveness: 60%");
    assert!(overlay(&app).visible);

    let beside_seen = TilePos::new(20, 8);
    ghost_at(&mut app, beside_seen);
    assert_eq!(counts(&app), ([60, 0, 0, 40, 0], 60));
    assert_eq!(label(&mut app), "Effectiveness: 60%");
    let shown = overlay(&app).preview.as_ref().expect("a preview");
    let drawn: Vec<TilePos> = shown.preview.tiles.iter().map(|&(t, _)| t).collect();
    assert_eq!(drawn, area_of(beside_seen), "the ghost's area only");
    assert!(overlay(&app).selected.is_empty());

    let beside_unseen = TilePos::new(40, 30);
    ghost_at(&mut app, beside_unseen);
    assert_eq!(counts(&app), ([100, 0, 0, 0, 0], 100));
    assert_eq!(label(&mut app), "Effectiveness: 100%");
    let sim = app.world().resource::<SimRes>();
    let claimed = sim.claimed_tiles(ObjectId(UNSEEN_HALL));
    assert!(
        claimed.contains(&TilePos::new(43, 30)),
        "player 2's Hall claims it in the game"
    );

    tap(&mut app, &[], KeyCode::Escape);
    app.update();
    assert_eq!(overlay(&app).preview, None);
    assert_eq!(label(&mut app), "", "emptied, not hidden");
    assert!(!overlay(&app).visible);
}

/// Design change 0352's Verification: "Scout an enemy Hall, leave, and have it destroyed unseen: the preview still
/// treats its tiles as taken until you see the spot again". The scout sees player 2's Hall and goes; the Hall goes
/// while no one of player 1's sees its place, and the preview beside it still has its 4 columns taken, from player
/// 1's memory (`core/vision.md > Explored`): "The preview reads the player's memory of enemy Halls, not the true
/// state". A new scout sees the place without it, and the preview has them back.
#[test]
fn a_remembered_hall_counts_until_its_place_is_seen_without_it() {
    let mut app = field();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove { id: ObjectId(SCOUT) }),
    );
    updates(&mut app, 2 + FOG_DELAY_TICKS as usize);
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove {
            id: ObjectId(SEEN_HALL),
        }),
    );
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.object(ObjectId(SEEN_HALL)).is_none(), "gone");
    let record = sim.vision().player(P1).and_then(|v| v.record(ObjectId(SEEN_HALL)));
    assert!(record.is_some(), "remembered");

    place_a_hall(&mut app);
    let beside = TilePos::new(20, 8);
    ghost_at(&mut app, beside);
    assert_eq!(counts(&app), ([60, 0, 0, 40, 0], 60), "remembered, though gone");
    assert_eq!(label(&mut app), "Effectiveness: 60%");

    let scout = spawn(&mut app, ObjectKind::Militant, Some(P1), SCOUT_AT);
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.object(ObjectId(scout)).is_some());
    let record = sim.vision().player(P1).and_then(|v| v.record(ObjectId(SEEN_HALL)));
    assert!(record.is_none(), "seen gone");
    assert_eq!(counts(&app), ([100, 0, 0, 0, 0], 100));
    assert_eq!(label(&mut app), "Effectiveness: 100%");
}

/// Design change 0351's Verification: "Place a ghost Hall, then preview a second one overlapping it: the overlap
/// shows as taken". With Shift held the Recruits queue a Hall, whose lingering ghost then has the 4 columns its area
/// shares with the next one's: 60%. Without Shift, the click would replace the Recruits' orders and their ghost with
/// them, so it doesn't count (as their placement check, `Sim::group_build_check`): 100%; holding Shift again, it does.
#[test]
fn a_ghost_hall_placed_takes_its_tiles_from_the_next() {
    let mut app = field();
    place_a_hall(&mut app);
    let (first, second) = (TilePos::new(20, 36), TilePos::new(26, 36));
    look_at(&mut app, middle(first));
    press_keys(&mut app, &[ShiftLeft]);
    click(&mut app, middle(first), &[]);
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    let pending: Vec<TilePos> = sim.pending_builds(P1).iter().map(|p| p.site.origin).collect();
    assert_eq!(pending, [first], "one lingering ghost");
    ghost_at(&mut app, second);
    assert_eq!(counts(&app), ([60, 0, 40, 0, 0], 60));
    let (tile, taken) = overlay(&app).preview.as_ref().expect("a preview").preview.tiles[3];
    assert_eq!(tile, TilePos::new(26, 33));
    assert!(matches!(
        taken,
        PreviewTile::Taken {
            claimer: Claimer::OwnPending,
            ..
        }
    ));
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "letting go of Shift ends placement"
    );

    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyQ);
    ghost_at(&mut app, second);
    assert_eq!(counts(&app), ([100, 0, 0, 0, 0], 100), "the click would replace it");
    press_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(counts(&app), ([60, 0, 40, 0, 0], 60));
    assert_eq!(label(&mut app), "Effectiveness: 60%");
}

/// `factions/cults/structures/recruitment_hall.md > Recruitment Area display`: "A selected own Hall shows its claimed
/// tiles: its **true** claims, matching its InfoPanel's Effectiveness and what it produces" (design change 0373,
/// answering Q0133), those it claims in the game: player 1's Hall alone claims its whole area. A selected enemy Hall
/// shows nothing: "Enemy Halls' areas are never drawn as areas of their own".
#[test]
fn a_selected_own_hall_shows_its_claimed_tiles_and_an_enemy_none() {
    let mut app = field();
    look_at(&mut app, middle(OWN_HALL_AT));
    click(&mut app, middle(OWN_HALL_AT), &[]);
    assert_eq!(selected(&app), [OWN_HALL]);
    let shown = &overlay(&app).selected;
    assert_eq!(shown.len(), 1);
    let mut area = area_of(OWN_HALL_AT);
    area.sort();
    assert_eq!((shown[0].hall, &shown[0].tiles), (ObjectId(OWN_HALL), &area));
    assert!(overlay(&app).visible && overlay(&app).preview.is_none());
    assert_eq!(label(&mut app), "");

    look_at(&mut app, middle(SEEN_HALL_AT));
    click(&mut app, middle(SEEN_HALL_AT), &[]);
    assert_eq!(selected(&app), [SEEN_HALL]);
    assert!(overlay(&app).selected.is_empty());
    assert!(!overlay(&app).visible);
}
