//! Units in the map editor: the palette's unit types go at a point, owned by a slot on a Custom map and neutral on a
//! Skirmish Default map, apart from other units (`docs/movement.md > 3`). Units on blocked ground are listed as
//! problems, which keep the map from being play-tested.

use bevy::prelude::*;
use sc_sim::{
    MapSettings, MapType, ObjectKind, PlayerId, ScenarioProblem, SlotId, Spawn, SpawnAt, SpawnProblem, Symmetry,
    scenarios,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::objects::palette_groups;
use space_crystals::editor::state::{Selection, StatusLevel};
use space_crystals::editor::tools::Tool;

use super::{
    act, assert_warned, created, doc, editor, problems, selection, sim, status, temp_maps, tools, visual_count,
};

const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su

fn unit(kind: ObjectKind, owner: Option<SlotId>, p: Vec2) -> Spawn {
    Spawn::new(kind, owner, SpawnAt::Point(p))
}

fn mode(app: &App) -> ClientMode {
    *app.world().resource::<ClientMode>()
}

/// Both Peacekeeper types (design change 0173), like the Legionnaires' two forms: a map may place a Peacekeeper
/// (Shielded), its shield up from the start; the Field Engineer after them, in the order of the kinds (slice M5-N9).
/// The Syndicate's Agent follows its Guard (slice M4-A), and its Grenadier the Agent (slice M4-U), in the order of the
/// kinds. The Cults' come between the Syndicate's and the Colonists', in `Faction::ALL`'s order: slice M5-A's three,
/// then slice M5-G2's five, which a map may place, the Hand of Heaven and the Prayer-mobile though no building makes
/// them yet (every type is in the palette: `the_palette_lists_every_object_type_once`).
#[test]
fn the_palette_lists_the_units_of_each_faction() {
    let units: Vec<(String, Vec<ObjectKind>)> = palette_groups()
        .into_iter()
        .filter(|g| g.title.ends_with("units"))
        .map(|g| (g.title, g.kinds))
        .collect();
    assert_eq!(
        units,
        [
            (
                "GDO units".to_string(),
                vec![PK, ObjectKind::PeacekeeperShielded, ObjectKind::FieldEngineer]
            ),
            (
                "Syndicate units".to_string(),
                vec![GUARD, ObjectKind::Agent, ObjectKind::Grenadier]
            ),
            (
                "Cults units".to_string(),
                vec![
                    ObjectKind::Recruit,
                    ObjectKind::Militant,
                    ObjectKind::Gunner,
                    ObjectKind::Disciple,
                    ObjectKind::Seeker,
                    ObjectKind::Sparrow,
                    ObjectKind::HandOfHeaven,
                    ObjectKind::PrayerMobile
                ]
            ),
            (
                "Colonists units".to_string(),
                vec![
                    ObjectKind::Drone,
                    ObjectKind::SabreLegionnaire,
                    ObjectKind::RifleLegionnaire
                ]
            )
        ]
    );
}

/// On a Custom map a unit belongs to the chosen slot's player. A drag places units only where they don't overlap the
/// last one; a unit can't go on another, but may touch it; the Select tool moves it; the sim shows each edit.
#[test]
fn units_are_placed_moved_and_play_tested() {
    // A Custom map of 16 × 12 tiles: 1024 × 768 su.
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("units"));
    let p = Vec2::new(300.0, 420.0); // away from the start markers, on tiles (4, 3) and (11, 8)
    act(
        &mut app,
        [
            A::SetOwner(Some(SlotId(2))),
            A::SetKind(GUARD),
            A::Hover(Some(p)),
            A::Press(p),
            A::Drag(p + Vec2::new(5.0, 3.0)),
            A::Drag(p + Vec2::new(36.0, 0.0)), // touches the first
            A::Release,
        ],
    );
    let second = p + Vec2::new(36.0, 0.0);
    assert_eq!(
        doc(&app).spawns,
        [unit(GUARD, Some(SlotId(2)), p), unit(GUARD, Some(SlotId(2)), second)]
    );
    assert_eq!(problems(&app), []);
    let shown: Vec<_> = sim(&app).objects().map(|o| (o.kind, o.pos, o.owner)).collect();
    assert_eq!(
        shown,
        [(GUARD, p, Some(PlayerId(2))), (GUARD, second, Some(PlayerId(2)))]
    );
    assert_eq!(visual_count(&mut app), 2);

    // On a unit: refused. Below the first Guard, touching it: placed.
    act(
        &mut app,
        [A::SetKind(PK), A::Press(p + Vec2::new(0.0, 29.0)), A::Release],
    );
    assert_warned(&app, "would overlap spawn #0");
    assert_eq!(doc(&app).spawns.len(), 2);
    let below = p + Vec2::new(0.0, 30.0);
    act(&mut app, [A::Press(below), A::Release]);
    assert_eq!(doc(&app).spawns[2], unit(PK, Some(SlotId(2)), below));

    // The Select tool moves it, but not onto a Guard.
    let q = Vec2::new(600.0, 300.0);
    act(
        &mut app,
        [
            A::SetTool(Tool::Select),
            A::Press(below),
            A::Drag(second),
            A::Drag(q),
            A::Release,
        ],
    );
    assert_eq!(selection(&app), Some(Selection::Spawn(2)));
    assert_eq!(doc(&app).spawns[2].at, SpawnAt::Point(q));
    assert_eq!(sim(&app).objects().nth(2).map(|o| o.pos), Some(q));
    act(
        &mut app,
        [A::Press(q), A::Drag(second + Vec2::new(10.0, 0.0)), A::Release],
    );
    assert_eq!(doc(&app).spawns[2].at, SpawnAt::Point(q), "it stays");

    // Across the map's edge: placed, but listed as a problem, and the map can't be play-tested.
    act(&mut app, [A::SetKind(PK), A::Press(Vec2::new(5.0, 300.0)), A::Release]);
    assert_eq!(
        problems(&app),
        [ScenarioProblem::InvalidSpawn {
            spawn: 3,
            kind: PK,
            problem: SpawnProblem::Blocked
        }]
    );
    act(&mut app, [A::PlayTest]);
    app.update();
    let refused = status(&app);
    assert_eq!(refused.level, StatusLevel::Error);
    assert!(refused.message.starts_with("Can't play-test"), "{}", refused.message);
    assert_eq!(mode(&app), ClientMode::Editor);

    act(&mut app, [A::Undo, A::PlayTest]);
    app.update();
    assert_eq!(mode(&app), ClientMode::PlayTest);
    for _ in 0..8 {
        app.update();
    }
    let units = sim(&app).objects().filter(|o| o.orders.is_some() && o.mover.is_some());
    assert_eq!(units.count(), 3);
}

/// A unit's images go to the image slots; a unit can't overlap its own image, nor go in the locked part. A Skirmish
/// Default map takes neutral units only.
#[test]
fn symmetric_and_skirmish_maps_take_units_with_their_images() {
    // 24 × 24 tiles: 1536 × 1536 su, the west half editable.
    let image = |p: Vec2| Vec2::splat(1536.0) - p;
    let mut app = editor(created(Symmetry::Rotate180, 24, 24, 2), &temp_maps("unit_images"));
    let p = Vec2::new(300.0, 400.0);
    act(
        &mut app,
        [A::SetOwner(Some(SlotId(1))), A::SetKind(PK), A::Press(p), A::Release],
    );
    assert_eq!(
        doc(&app).spawns,
        [unit(PK, Some(SlotId(1)), p), unit(PK, Some(SlotId(2)), image(p))]
    );
    assert_eq!(doc(&app).first_asymmetry(), None);
    assert_eq!(problems(&app), []);
    act(&mut app, [A::Press(Vec2::new(760.0, 768.0)), A::Release]);
    assert_warned(&app, "its own mirror image");
    act(&mut app, [A::Press(Vec2::new(1000.0, 400.0)), A::Release]);
    assert_warned(&app, "Locked");
    assert_eq!(doc(&app).spawns.len(), 2);

    let skirmish = MapSettings {
        map_type: MapType::SkirmishDefault,
        width: 24,
        height: 24,
        slots: 2,
        symmetry: Symmetry::Rotate180,
        name: "test".into(),
        description: String::new(),
    }
    .create()
    .unwrap();
    let mut app = editor(skirmish, &temp_maps("unit_skirmish"));
    act(
        &mut app,
        [A::SetOwner(Some(SlotId(1))), A::SetKind(GUARD), A::Press(p), A::Release],
    );
    assert_eq!(tools(&app).owner, None, "neutral objects only");
    assert_eq!(doc(&app).spawns, [unit(GUARD, None, p), unit(GUARD, None, image(p))]);
    // Only the Starting Layouts, which a new map's are, lack anything (design changes 0009 and 0012).
    let problems = problems(&app);
    assert!(
        !problems.is_empty()
            && problems
                .iter()
                .all(|p| matches!(p, ScenarioProblem::LayoutMissing { .. })),
        "{problems:?}"
    );
}
