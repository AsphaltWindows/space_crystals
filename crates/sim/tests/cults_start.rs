//! A Cults player's start (slice M5-B; `factions/cults/economy.md > Starting Conditions`): the banks, the objects of the
//! Starting Layout on `dev_skirmish` in either slot, the six Recruits belonging to the starting Recruitment Hall, so
//! that Unit Control starts at 6/20 and the Hall at once starts its seventh, and the camera's start on the Hall
//! (`core/camera.md > Starting Position`). `dev_skirmish`'s layout follows the design: the Hall's Recruitment Area on
//! the map, Recruitable and its own (`factions/cults/structures/recruitment_hall.md > Effectiveness`), the Storage a
//! 1-tile gap from the field (`factions/cults/structures/storage.md`). The map validates in each of the 16 pairings,
//! and an incomplete Cults layout lists what it lacks (`core/map_editor.md > Starting Layouts > Skirmish Default
//! restriction`; design change 0012: "Leave the Cults layout incomplete: the editor lists the missing Recruits, and
//! the map can't be started"). The lineage of a layout's units wherever its Hall stands, on a Custom map, and without
//! one. The Starting Conditions table is unit-tested in `scenario/starting.rs`, the Halls in `tests/recruitment/`, the
//! bar, the camera and the editor in the client.

use sc_sim::recruitment;
use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    ArmyCap, ArmyResource, Banks, Doorway, Faction, GameSetup, LayoutObject, MapDef, ObjectId, ObjectKind, PlayerId,
    SU_PER_GU, Scenario, ScenarioProblem, Sim, SimEvent, SlotId, SpawnAt, StartingConditions, TilePos, Vec2, scenarios,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const STORAGE: ObjectKind = ObjectKind::Storage;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
const CULTS: Faction = Faction::Cults;
const GDO: Faction = Faction::Gdo;
/// `factions/cults/structures/recruitment_hall.md > BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`.
const FRAMES: u32 = 192;

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// `dev_skirmish` with player 1 in slot 1 and player 2 in slot 2, playing `factions`.
fn dev_skirmish_as(factions: [Faction; 2]) -> Scenario {
    let map = scenarios::dev_skirmish_map();
    let setup = GameSetup::per_slot(&map, &factions, DEFAULT_SEED);
    Scenario { map, setup }
}

/// `player`'s objects: kind and position, in id order.
fn objects_of(sim: &Sim, player: PlayerId) -> Vec<(ObjectKind, Vec2)> {
    sim.objects()
        .filter(|o| o.owner == Some(player))
        .map(|o| (o.kind, o.pos))
        .collect()
}

/// The ids of `player`'s objects of `kind`, in id order.
fn ids(sim: &Sim, player: PlayerId, kind: ObjectKind) -> Vec<ObjectId> {
    (sim.objects())
        .filter(|o| o.owner == Some(player) && o.kind == kind)
        .map(|o| o.id)
        .collect()
}

/// The lineage of each of `player`'s Recruits, in id order.
fn lineages(sim: &Sim, player: PlayerId) -> Vec<Vec<ObjectId>> {
    (sim.objects())
        .filter(|o| o.owner == Some(player) && o.kind == RECRUIT)
        .map(|o| o.lineage.clone())
        .collect()
}

fn unit_control(used: u32, available: u32) -> Option<ArmyCap> {
    Some(ArmyCap {
        resource: ArmyResource::CultsUnitControl,
        used,
        available,
    })
}

fn layout_object(kind: ObjectKind, at: SpawnAt) -> LayoutObject {
    LayoutObject {
        kind,
        at,
        orientation: Default::default(),
    }
}

/// Plan acceptance: "`cargo run -- --factions cults,gdo` starts with the objects and 0 SC, the camera on the Hall".
/// A Cults player starts with nothing in the bank (`factions/cults/economy.md > Starting Conditions`: "**0 Space
/// Crystals**"), with the Recruitment Hall, the Storage and the six Recruits where `dev_skirmish`'s layout puts them,
/// in either slot (the second slot's layout is the first's 180° image), each Recruit of the Hall's lineage ("all
/// belonging to the starting Recruitment Hall"). At tick 0 the Hall claims its whole area: 100% effectiveness, its
/// usage 6 of its 20, and the HUD's Unit Control 6/20 (`factions/cults/structures/recruitment_hall.md > Cults Unit
/// Control HUD Display`). The camera starts on the Hall (`core/camera.md > Starting Position`: "Cults: starting
/// Recruitment Hall"). The GDO beside them starts as ever.
#[test]
fn a_cults_player_starts_with_their_starting_conditions() {
    let scenario = dev_skirmish_as([CULTS, GDO]);
    let sim = Sim::new(&scenario);
    assert_eq!(sim.player(P1).unwrap().banks(), Banks::default());
    // Slot 1: the Hall on (8, 8) to (11, 11), the Storage on (5, 4) to (6, 5), the Recruits south of the Storage.
    let slot_1 = [
        (HALL, point(10.0, 10.0)),
        (STORAGE, point(6.0, 5.0)),
        (RECRUIT, point(5.0, 7.0)),
        (RECRUIT, point(6.0, 7.0)),
        (RECRUIT, point(7.0, 7.0)),
        (RECRUIT, point(5.0, 8.0)),
        (RECRUIT, point(6.0, 8.0)),
        (RECRUIT, point(7.0, 8.0)),
    ];
    assert_eq!(objects_of(&sim, P1), slot_1);
    let hall = ids(&sim, P1, HALL)[0];
    assert_eq!(lineages(&sim, P1), vec![vec![hall]; 6]);
    let state = sim.hall_state(hall).unwrap();
    assert_eq!(
        (state.claimed, state.tiles, state.usage, state.capacity.floor()),
        (100, 100, 6, 20)
    );
    assert_eq!(sim.army_cap(P1), unit_control(6, 20));
    assert_eq!(scenario.primary_structure(P1), Some(point(10.0, 10.0)), "the Hall");
    let hundred = Banks {
        crystals: 100,
        ..Banks::default()
    };
    assert_eq!(sim.player(P2).unwrap().banks(), hundred);
    assert_eq!(ids(&sim, P2, ObjectKind::DeploymentCenter).len(), 1);

    let scenario = dev_skirmish_as([GDO, CULTS]);
    let sim = Sim::new(&scenario);
    let image = |(kind, at): (ObjectKind, Vec2)| (kind, point(64.0, 48.0) - at);
    assert_eq!(objects_of(&sim, P2), slot_1.map(image));
    let hall = ids(&sim, P2, HALL)[0];
    assert_eq!(lineages(&sim, P2), vec![vec![hall]; 6]);
    assert_eq!(
        (sim.player(P2).unwrap().banks(), sim.army_cap(P2)),
        (Banks::default(), unit_control(6, 20))
    );
    assert_eq!(scenario.primary_structure(P2), Some(point(54.0, 38.0)));
}

/// The Hall starts its seventh Recruit at once (`factions/cults/structures/recruitment_hall.md > Production Behavior`:
/// "automatically and continuously produces Recruits at no resource cost until LocalUnitControlUsage reaches
/// LocalUnitControlCapacity"): in production from the first tick, it counts from then on (design change 0342: "the
/// Recruit in production counts toward LocalUnitControlUsage from the start of its production"), so the HUD shows 7/20,
/// and comes out 192 ticks later (`factions/cults/structures/recruitment_hall.md > BaseRecruitProductionTime - 12
/// seconds (192 frames) per Recruit`), out of side B, the Hall's one exit (0342: "the exit on B"), the west side in the
/// first slot and the east in its image, the starting Recruits in its way in neither. The eighth is in production at
/// once: 8/20. Nothing in the bank moves: Recruits cost nothing.
#[test]
fn the_starting_hall_starts_its_seventh_recruit_at_once() {
    let mut t = TestSim::new(dev_skirmish_as([CULTS, CULTS]));
    let hall = ids(t.sim(), P1, HALL)[0];
    assert_eq!(
        t.sim().production_status(hall),
        None,
        "tick 0: nothing in production yet"
    );
    t.run(1);
    let status = t.sim().production_status(hall).expect("in production");
    assert_eq!(status.progress, 100, "100 tiles a frame");
    t.run_until(FRAMES - 1);
    assert_eq!(t.sim().army_cap(P1), unit_control(7, 20), "the seventh in production");
    t.run_until(FRAMES);
    for (player, outward) in [(P1, Vec2::NEG_X), (P2, Vec2::X)] {
        let hall = ids(t.sim(), player, HALL)[0];
        let made: Vec<_> = (t.events().iter())
            .filter_map(|(tick, e)| match *e {
                SimEvent::UnitProduced { producer, unit } if producer == hall => Some((*tick, unit)),
                _ => None,
            })
            .collect();
        assert_eq!(made.len(), 1, "{player:?}: {made:?}");
        let (tick, seventh) = made[0];
        assert_eq!(tick, FRAMES);
        let o = t.sim().object(seventh).unwrap();
        assert_eq!((o.kind, o.lineage.as_slice()), (RECRUIT, [hall].as_slice()));
        let doorway = Doorway::of(t.sim().object(hall).unwrap()).unwrap();
        assert_eq!(doorway.doors().count(), 1, "side B alone");
        let door = doorway.door_at(o.pos);
        assert_eq!(door.outward(), outward, "{player:?}: out of side B");
        assert_eq!(
            t.sim().army_cap(player),
            unit_control(8, 20),
            "{player:?}: the eighth in production"
        );
    }
    assert_eq!(t.sim().player(P1).unwrap().banks(), Banks::default());
}

/// The gap in whole tiles between two tile rectangles, `(x, y, w, h)`: the tiles strictly between them along the axis
/// they are farther apart on, 0 for rectangles that touch, even at a corner.
fn gap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> i32 {
    let along = |a0: i32, a_len: i32, b0: i32, b_len: i32| (b0 - (a0 + a_len)).max(a0 - (b0 + b_len)).max(0);
    along(a.0, a.2, b.0, b.2).max(along(a.1, a.3, b.1, b.3))
}

/// `dev_skirmish`'s Cults layout follows the design, in either slot:
/// - the Hall's Recruitment Area is all on the map and Recruitable, and no other Hall's
///   (`factions/cults/structures/recruitment_hall.md > Effectiveness`, map design guideline: "balancing starts from the
///   assumption that the whole map is Recruitable"; "A hall on fully Recruitable terrain with no competing halls
///   therefore reaches 100% effectiveness"), with the Cults in both slots;
/// - the Storage stands a 1-tile gap from the nearest patches of its field (`factions/cults/structures/storage.md`:
///   "The best layout leaves a 1-tile gap between patch and Storage"), five of the eight, and touches none;
/// - the Recruits stand beside the Storage, within 2 grid units of its footprint, and at least 2 grid units from the
///   middle of the Hall's side B, the one door its Recruits come out of (design change 0342: "the exit on B").
#[test]
fn dev_skirmish_cults_layout_follows_the_design() {
    let sim = Sim::new(&dev_skirmish_as([CULTS, CULTS]));
    let map = scenarios::dev_skirmish_map();
    let half = map.terrain.width() as f32 * SU_PER_GU / 2.0;
    let def = recruitment::def_of(HALL).unwrap();
    let claims = sim.claims();
    for player in [P1, P2] {
        let hall = sim.object(ids(&sim, player, HALL)[0]).unwrap();
        let area: Vec<TilePos> = recruitment::area(&hall.placement.unwrap(), def).collect();
        assert_eq!(area.len(), 100);
        for t in &area {
            let preset = map.terrain.tile_preset(*t);
            assert!(preset.is_some_and(|p| p.recruitable), "{player:?} {t:?}");
        }
        assert_eq!(claims.tiles(hall.id), 100, "{player:?}: uncontested");
        assert_eq!(sim.hall_state(hall.id).unwrap().effectiveness_percent(), 100);

        let storage = sim.object(ids(&sim, player, STORAGE)[0]).unwrap();
        let rect = |o: &sc_sim::Object| {
            let p = o.placement.unwrap();
            (p.origin.x, p.origin.y, p.w as i32, p.h as i32)
        };
        let field: Vec<i32> = (sim.objects())
            .filter(|o| o.kind == PATCH && (o.pos.x < half) == (player == P1))
            .map(|patch| gap(rect(patch), rect(storage)))
            .collect();
        assert_eq!(field.len(), 8, "{player:?}");
        assert_eq!(field.iter().min(), Some(&1), "{player:?}: {field:?}");
        assert_eq!(field.iter().filter(|&&g| g == 1).count(), 5, "{player:?}: {field:?}");

        let doors: Vec<Vec2> = Doorway::of(hall).unwrap().doors().map(|d| d.middle()).collect();
        assert_eq!(doors.len(), 1, "side B");
        let (x, y, w, h) = rect(storage);
        let (min, max) = (point(x as f32, y as f32), point((x + w) as f32, (y + h) as f32));
        for id in ids(&sim, player, RECRUIT) {
            let pos = sim.object(id).unwrap().pos;
            let beside = pos.distance(pos.clamp(min, max)) / SU_PER_GU;
            assert!(beside <= 2.0, "{player:?} {id:?}: {beside} grid units from the Storage");
            for door in &doors {
                let away = pos.distance(*door) / SU_PER_GU;
                assert!(away >= 2.0, "{player:?} {id:?}: {away} grid units from a door");
            }
        }
    }
}

/// Plan acceptance: "Every Skirmish Default map validates" and every pairing starts. `dev_skirmish` validates with
/// each of the four factions in either slot, 16 pairings, and a game starts on it with each player's Starting
/// Conditions (`StartingConditions::of`): their banks and their objects' kinds; no layout is outdated, and its file
/// is the built-in map.
#[test]
fn dev_skirmish_validates_in_every_pairing() {
    let mut pairings = 0;
    for &a in Faction::ALL {
        for &b in Faction::ALL {
            let scenario = dev_skirmish_as([a, b]);
            assert_eq!(scenario.validate(), [], "{a:?}, {b:?}");
            let sim = Sim::try_new(&scenario).unwrap();
            for (player, faction) in [(P1, a), (P2, b)] {
                let start = StartingConditions::of(faction);
                let banks = sim.player(player).unwrap().banks();
                assert_eq!((banks.crystals, banks.supplies), (start.crystals, start.supplies));
                for &(kind, n) in start.objects {
                    assert_eq!(ids(&sim, player, kind).len(), n as usize, "{a:?}, {b:?}: {kind:?}");
                }
                let owned = sim.objects().filter(|o| o.owner == Some(player)).count();
                assert_eq!(owned, start.objects.iter().map(|&(_, n)| n as usize).sum::<usize>());
            }
            pairings += 1;
        }
    }
    assert_eq!(pairings, 16);
    assert_eq!(scenarios::dev_skirmish_map().outdated_layouts(), []);
    let file: MapDef = serde_json::from_str(include_str!("../../../maps/dev_skirmish.json")).unwrap();
    assert_eq!(
        file,
        scenarios::dev_skirmish_map(),
        "maps/dev_skirmish.json is the built-in map"
    );
}

/// Plan acceptance: "An incomplete Cults layout lists what it lacks" (design change 0012: "Leave the Cults layout
/// incomplete: the editor lists the missing Recruits, and the map can't be started"). Two Recruits short in the
/// first slot, and so in its image, the map lacks them in both slots (`ScenarioProblem::LayoutMissing`), flags the
/// layouts as outdated, and no game starts on it, a GDO one included; without its Hall it lacks that too, and a
/// Recruit too many is beyond the Starting Conditions (`ScenarioProblem::LayoutExtra`).
#[test]
fn an_incomplete_cults_layout_lists_what_it_lacks() {
    let mut map = scenarios::dev_skirmish_map();
    map.slots[0].layouts.get_mut(CULTS).truncate(6);
    map.regenerate();
    let lacks = |slot, kind, missing| ScenarioProblem::LayoutMissing {
        slot: SlotId(slot),
        faction: CULTS,
        kind,
        missing,
    };
    let expected = [lacks(1, RECRUIT, 2), lacks(2, RECRUIT, 2)];
    assert_eq!(map.validate(), expected);
    assert_eq!(
        expected[0].to_string(),
        "slot 1 Cults layout lacks 2 Recruit of the Starting Conditions: a Skirmish Default map is played with every \
         layout complete"
    );
    assert_eq!(map.outdated_layouts(), [(SlotId(1), CULTS), (SlotId(2), CULTS)]);
    assert_eq!(Sim::try_new(&Scenario::new(map.clone())).err(), Some(expected.to_vec()));

    map.slots[0].layouts.get_mut(CULTS).remove(0);
    map.regenerate();
    let expected = [
        lacks(1, HALL, 1),
        lacks(1, RECRUIT, 2),
        lacks(2, HALL, 1),
        lacks(2, RECRUIT, 2),
    ];
    assert_eq!(map.validate(), expected);

    let mut map = scenarios::dev_skirmish_map();
    let extra = layout_object(RECRUIT, SpawnAt::Point(point(8.0, 7.0)));
    map.slots[0].layouts.get_mut(CULTS).push(extra);
    map.regenerate();
    let beyond = |slot| ScenarioProblem::LayoutExtra {
        slot: SlotId(slot),
        faction: CULTS,
        index: 8,
        kind: RECRUIT,
        allowed: 6,
    };
    assert_eq!(map.validate(), [beyond(1), beyond(2)]);
    assert_eq!(
        beyond(1).to_string(),
        "slot 1 Cults layout object 8 (Recruit): beyond the 6 of the Cults Starting Conditions: a Skirmish Default \
         layout holds only those"
    );
}

/// The lineage of a layout's units (`Scenario::start_objects`, `Object::lineage`): each Cults unit embodies a Recruit
/// of the first Recruitment Hall of its layout, wherever the layout lists it, before its units or after them; a
/// second Hall gets none. A unit of another faction has no lineage. A Custom map's layout without a Hall (which a
/// Custom map allows, `core/map_editor.md > Map Types > Custom`) gives its Recruits none: no Hall made them, and their
/// cost is orphaned (design change 0344: "Recruits no Hall produced (custom maps, debug) are orphans from the start"),
/// used Unit Control with none available.
#[test]
fn a_layouts_units_belong_to_its_first_hall() {
    let flat = vec![".".repeat(40); 24].join("\n");
    let mut s = ScenarioBuilder::new(&flat).faction(P1, CULTS).build();
    let recruit = |x, y| layout_object(RECRUIT, SpawnAt::Point(point(x, y)));
    let tile = |kind, x, y| layout_object(kind, SpawnAt::Tile(TilePos::new(x, y)));
    *s.map.slots[0].layouts.get_mut(CULTS) = vec![
        recruit(2.0, 2.0),
        layout_object(ObjectKind::Peacekeeper, SpawnAt::Point(point(2.0, 4.0))),
        tile(HALL, 10, 10),
        recruit(3.0, 2.0),
        tile(HALL, 20, 10),
        recruit(4.0, 2.0),
    ];
    let sim = Sim::try_new(&s).unwrap();
    let halls = ids(&sim, P1, HALL);
    assert_eq!(halls.len(), 2);
    assert_eq!(lineages(&sim, P1), vec![vec![halls[0]]; 3]);
    let peacekeeper = ids(&sim, P1, ObjectKind::Peacekeeper)[0];
    assert!(sim.object(peacekeeper).unwrap().lineage.is_empty());
    assert_eq!(sim.hall_state(halls[0]).unwrap().usage, 3);
    assert_eq!(sim.hall_state(halls[1]).unwrap().usage, 0);

    s.map.slots[0].layouts.get_mut(CULTS).retain(|o| o.kind != HALL);
    let sim = Sim::try_new(&s).unwrap();
    assert_eq!(lineages(&sim, P1), vec![Vec::<ObjectId>::new(); 3]);
    assert_eq!(sim.army_cap(P1), unit_control(3, 0), "orphaned");
}
