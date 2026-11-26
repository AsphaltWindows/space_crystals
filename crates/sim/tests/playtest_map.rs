//! `maps/playtest.json`, the map to playtest on (slice M5-P3, `docs/playtest/README.md > Starting a pairing`):
//! `dev_skirmish`'s terrain, with each base beside the reference crystal field of the Cults' economy check
//! (`factions/cults/economy.md > Economy check`, `sc_sim::testing::cults::line_field`): eight patches of 1500 Space
//! Crystals in a line, and three Storage sites a 1-tile gap from it on each long side, the third turned
//! (`factions/cults/structures/storage.md`: "The best layout leaves a 1-tile gap between patch and Storage"). Slot
//! 1's field is the reference field moved by [`SHIFT`], slot 2's its image. The map is its editable part's
//! (`core/map_editor.md > Symmetry > Editing a symmetric map`) and starts in every pairing; in either slot each
//! Storage site but the starting Storage's is free for a Recruit's Build, and the Starting Layouts face the field.
//! The pairings play it (`tests/pairings/`, its ignored run), and the editor saves it unchanged (the client's
//! `editor::session` tests).

use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::testing::cults::{SITES, START_SITE};
use sc_sim::tunnel::{self, Resource};
use sc_sim::{
    Faction, GameSetup, MapDef, MapTransform, Object, ObjectKind, Placement, PlayerId, ResourceState, Rotation,
    SU_PER_GU, Scenario, Sim, TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;

/// Slot 1's field is the reference field moved by this many tiles: its line on tiles (3, 6) to (10, 6), the near
/// side south, toward the base.
const SHIFT: (i32, i32) = (-13, -6);

/// The map, as `--map maps/playtest.json` loads it.
fn map() -> MapDef {
    serde_json::from_str(include_str!("../../../maps/playtest.json")).unwrap()
}

/// The map with player 1 in slot 1 and player 2 in slot 2, playing `factions`.
fn playtest_as(factions: [Faction; 2]) -> Scenario {
    let map = map();
    let setup = GameSetup::per_slot(&map, &factions, DEFAULT_SEED);
    Scenario { map, setup }
}

/// Each player with the transform of their slot: the identity for slot 1, the half turn for slot 2.
fn slots(map: &MapDef) -> [(PlayerId, MapTransform); 2] {
    let transforms = map.transforms();
    assert_eq!(transforms.len(), 2);
    [(P1, transforms[0]), (P2, transforms[1])]
}

/// The reference field's tile `t` in the field of the slot `transform` maps to.
fn moved(transform: MapTransform, t: TilePos) -> TilePos {
    transform.tile(TilePos::new(t.x + SHIFT.0, t.y + SHIFT.1))
}

/// `player`'s only object of `kind`.
fn only(sim: &Sim, player: PlayerId, kind: ObjectKind) -> &Object {
    let mut theirs = sim.objects().filter(|o| o.owner == Some(player) && o.kind == kind);
    let o = theirs.next().unwrap_or_else(|| panic!("{player:?} has no {kind:?}"));
    assert!(theirs.next().is_none(), "{player:?} has more than one {kind:?}");
    o
}

/// The tiles of the patches of the field of the slot `transform` maps to: the reference line's, (16, 12) to (23, 12)
/// (`testing::cults::fields`), moved; west to east in slot 1.
fn field(transform: MapTransform) -> Vec<TilePos> {
    (16..24).map(|x| moved(transform, TilePos::new(x, 12))).collect()
}

/// The map is its editable part's: generating the rest again changes nothing, so slot 2's layouts and the east half
/// are the images of slot 1's and the west half's, and no layout is outdated. Every pairing of the four factions
/// validates and starts (`core/map_editor.md > Map Types > Skirmish Default`).
#[test]
fn the_map_is_generated_and_starts_in_every_pairing() {
    let map = map();
    assert_eq!(map.name, "playtest");
    let mut generated = map.clone();
    generated.regenerate();
    assert!(
        generated == map,
        "slot 2 and the east half are generated from slot 1 and the west half"
    );
    assert!(map.first_asymmetry().is_none());
    assert_eq!(map.outdated_layouts(), []);
    for &a in Faction::ALL {
        for &b in Faction::ALL {
            let scenario = playtest_as([a, b]);
            assert_eq!(scenario.validate(), [], "{a:?}, {b:?}");
            assert!(Sim::try_new(&scenario).is_ok(), "{a:?}, {b:?}");
        }
    }
}

/// Each slot's field is the reference field's line, eight patches of 1500 Space Crystals, the map's only patches. With
/// the Cults in both slots, the starting Storage stands on the reference field's starting site, a Recruit may be
/// ordered to build a Storage on each of the five others, and the Hall's Recruitment Area is wholly its own and
/// Recruitable, at 100% effectiveness (`factions/cults/structures/recruitment_hall.md > Effectiveness`).
#[test]
fn each_slot_has_the_reference_field_and_its_storage_sites() {
    let scenario = playtest_as([Faction::Cults, Faction::Cults]);
    let mut t = sc_sim::testing::TestSim::new(scenario.clone());
    t.run(1);
    let sim = t.sim();
    let patches: Vec<(TilePos, Option<u32>)> = (sim.objects())
        .filter(|o| o.kind == PATCH)
        .map(|o| match o.resource {
            Some(ResourceState::CrystalPatch { remaining }) => (o.placement.unwrap().origin, Some(remaining)),
            _ => (o.placement.unwrap().origin, None),
        })
        .collect();
    assert_eq!(patches.len(), 16);
    for (player, transform) in slots(&scenario.map) {
        for tile in field(transform) {
            let remaining = patches.iter().find(|(at, _)| *at == tile).and_then(|(_, r)| *r);
            assert_eq!(remaining, Some(1500), "{player:?} {tile:?}");
        }

        let storage = only(sim, player, ObjectKind::Storage);
        let recruit = (sim.objects())
            .find(|o| o.owner == Some(player) && o.kind == ObjectKind::Recruit)
            .unwrap();
        for (i, site) in SITES.iter().enumerate() {
            // `factions/cults/structures/storage.md > Size - 3x2 grid units`, turned a quarter at the line's ends.
            let (w, h) = if site.turned { (2, 3) } else { (3, 2) };
            let reference = Placement {
                origin: TilePos::new(site.origin.x + SHIFT.0, site.origin.y + SHIFT.1),
                w,
                h,
                rotation: if site.turned { Rotation::R90 } else { Rotation::R0 },
                flip_h: false,
                flip_v: false,
            };
            let at = transform.placement(&reference);
            if i == START_SITE {
                assert_eq!(storage.placement, Some(at), "{player:?}: the starting Storage's site");
                continue;
            }
            let check = sim.build_check(recruit.id, ObjectKind::Storage, at.origin, at.orientation(), false);
            let check = check.unwrap();
            assert!(check.is_valid(), "{player:?} site {i} at {:?}: {check:?}", at.origin);
        }

        let hall = only(sim, player, ObjectKind::RecruitmentHall);
        assert_eq!(sim.claims().tiles(hall.id), 100, "{player:?}");
        assert_eq!(
            sim.hall_state(hall.id).unwrap().effectiveness_percent(),
            100,
            "{player:?}"
        );
    }
}

/// The other factions' Starting Layouts face the field too, in either slot:
/// - the GDO's Deployment Center sees every patch by its own sight, and its build area holds them all, so a plate can
///   go onto each uncovered one at once (design change 0095; `factions/gdo/economy.md > Opening Economy Baseline >
///   Reference starting base`), two of them already plated;
/// - the Syndicate's crystal hatch lies 3 grid units edge to edge from the field's nearest patch, straight across
///   three free rows (design change 0294: "The reference layouts and `dev_skirmish` put the patch 3 GU from the
///   hatch");
/// - the Colonists' starting Drones stay at least 1 grid unit clear of the Egress Node, where the gatherers drop off,
///   and of the Robo Portal, whose doors its Drones come out of (question Q0041 point 3).
#[test]
fn the_starting_layouts_face_the_field() {
    let scenario = playtest_as([Faction::Gdo, Faction::Gdo]);
    let sim = Sim::try_new(&scenario).unwrap();
    let plated: Vec<_> = sim.objects().filter_map(|o| o.extractor.map(|e| e.patch)).collect();
    assert_eq!(plated.len(), 4);
    for (player, transform) in slots(&scenario.map) {
        let dc = only(&sim, player, ObjectKind::DeploymentCenter);
        let area = sim.build_area(player);
        for tile in field(transform) {
            let patch = sim
                .objects()
                .find(|o| o.kind == PATCH && o.placement.unwrap().origin == tile);
            let patch = patch.unwrap();
            let distance = patch.pos.distance(dc.pos) / SU_PER_GU;
            assert!(distance <= 6.0, "{player:?} {tile:?}: {distance} grid units");
            assert_eq!(
                sim.visibility(player, tile),
                VisibilityState::Visible,
                "{player:?} {tile:?}"
            );
            assert!(area.contains(tile), "{player:?} {tile:?}");
            let check = sim.placement_check(player, PLATE, tile, Default::default());
            assert_eq!(
                check.is_valid(),
                !plated.contains(&patch.id),
                "{player:?} {tile:?}: {check:?}"
            );
        }
    }

    let sim = Sim::try_new(&playtest_as([Faction::Syndicate, Faction::Syndicate])).unwrap();
    for (player, transform) in slots(&scenario.map) {
        let hatch = tunnel::hatch_square(only(&sim, player, ObjectKind::Tunnel), Resource::Crystals).unwrap();
        // Edge to edge, in tiles, across and along.
        let gap = |p: TilePos| (((p.x - hatch.x).abs() - 1).max(0), ((p.y - hatch.y).abs() - 1).max(0));
        let nearest = field(transform).into_iter().map(gap).min_by_key(|(x, y)| x * x + y * y);
        assert_eq!(nearest, Some((0, 3)), "{player:?}: the hatch {hatch:?}");
    }

    let sim = Sim::try_new(&playtest_as([Faction::Colonists, Faction::Colonists])).unwrap();
    for (player, _) in slots(&scenario.map) {
        let owned = || sim.objects().filter(move |o| o.owner == Some(player));
        let buildings: Vec<(Vec2, Vec2)> = owned()
            .filter_map(|o| {
                let p = o.placement?;
                let min = Vec2::new(p.origin.x as f32, p.origin.y as f32) * SU_PER_GU;
                Some((min, min + Vec2::new(p.w as f32, p.h as f32) * SU_PER_GU))
            })
            .collect();
        assert_eq!(buildings.len(), 2, "{player:?}: the Egress Node and the Robo Portal");
        let drones: Vec<&Object> = owned().filter(|o| o.kind == ObjectKind::Drone).collect();
        assert_eq!(drones.len(), 4, "{player:?}");
        for drone in drones {
            let half = Vec2::splat(18.0); // `factions/colonists/units/drone.md > Silhouette - 36x36`
            for &(min, max) in &buildings {
                let gap = (min - (drone.pos + half)).max((drone.pos - half) - max).max(Vec2::ZERO);
                assert!(
                    gap.length() >= SU_PER_GU,
                    "{player:?}'s Drone {:?} is {gap} from a building",
                    drone.id
                );
            }
        }
    }
}
