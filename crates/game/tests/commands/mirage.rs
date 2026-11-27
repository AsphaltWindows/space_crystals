//! (M6-CM) The Disciple's Mirage in the client (`factions/cults/units/disciple.md > Spells > Mirage`; `> Placeholder
//! stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **R: Shroud** and **F: Mirage**"): F casts it at
//! once, and its four illusions appear. Their owner reads each as "Disciple (illusion)", its energy line its parent's
//! (design change 0452), sees it washed out, its portrait too, and has its R and F greyed, "Illusions can't cast"; the
//! other player (the debug player switch, F2) sees five Disciples alike, on the map and in the InfoPanel ("**Seen by
//! enemies**: exactly like a Disciple, on the map and when selected; enemies can't tell them apart"). The panels as
//! plain data are the
//! unit tests' (`command_ui/interface/tests/cathedral.rs`), the spell the sim's (`crates/sim/tests/mirage`), the
//! computer players' View `sc_ai`'s (`view/tests.rs`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Ability, Faction, ObjectId, ObjectKind, PlayerId, SpawnAt};
use space_crystals::command_ui::feedback::ILLUSIONS_CANT_CAST_TEXT;
use space_crystals::fog::Sight;
use space_crystals::hud::info_panel::{InfoContent, InfoText, info_text};
use space_crystals::hud::selection_panel::{PortraitCell, SelectionPanelView};
use space_crystals::object_render::{ObjectVisuals, illusion_color, player_color};
use space_crystals::selection::Selection;

use super::armory::feedback;
use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Disciple, at the view's centre; its Mirage's illusions are 3 to 6, player 2's Militant being 2.
const DISCIPLE: u32 = 1;
const ILLUSIONS: [u32; 4] = [3, 4, 5, 6];

/// A game of player 1's Disciple and player 2's Militant, both of the Cults, on a flat 48 × 32 map, set up as [`game`]
/// does (the fog of war off).
fn mirage_game() -> App {
    let builder = ScenarioBuilder::new(&vec![".".repeat(48); 32].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(VIEW_CENTER))
        .spawn(
            ObjectKind::Militant,
            Some(P2),
            SpawnAt::Point(VIEW_CENTER + Vec2::new(400.0, 0.0)),
        );
    game_of(builder.build())
}

/// The InfoPanel's title, while it shows one.
fn info_title(app: &mut App) -> String {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    (texts.iter(app.world()))
        .find(|(which, _, node)| **which == InfoText::Title && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
        .unwrap_or_default()
}

/// The mesh object `id`'s visual is drawn with.
fn mesh(app: &App, id: u32) -> Handle<Mesh> {
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(id)];
    app.world().get::<Mesh2d>(entity).expect("a visual").0.clone()
}

/// The SelectionPanel's portraits, in their order: each one's object and its shape's colour.
fn portraits(app: &mut App) -> Vec<(u32, Color)> {
    let mut cells = app.world_mut().query::<(&PortraitCell, &Children)>();
    let mut shown: Vec<(usize, u32, Entity)> = Vec::new();
    let order = &app.world().resource::<SelectionPanelView>().cells;
    for (cell, children) in cells.iter(app.world()) {
        let PortraitCell::Object { id, .. } = *cell else {
            panic!("no overflow here");
        };
        let index = order.iter().position(|c| c == cell).expect("a cell of the view");
        shown.push((index, id.0, children[0]));
    }
    shown.sort();
    let colour = |shape: Entity| app.world().get::<BackgroundColor>(shape).expect("a shape").0;
    shown.into_iter().map(|(_, id, shape)| (id, colour(shape))).collect()
}

/// What `player`'s InfoPanel would show with object `id` alone selected, the fog off.
fn panel_of(app: &App, player: PlayerId, id: u32) -> InfoContent {
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, player, false);
    let selection = Selection::new([(ObjectId(id), ObjectKind::Disciple)]);
    info_text(sim, &sight, &selection).expect("a panel")
}

#[test]
fn f_casts_mirage_and_only_its_owner_tells_the_illusions_apart() {
    let mut app = mirage_game();
    select_units(&mut app, &[DISCIPLE]);
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(abilities(&app).last(), Some(&(P1, vec![DISCIPLE], Ability::Mirage)));
    updates(&mut app, 2);
    for id in ILLUSIONS {
        let o = app.world().resource::<SimRes>().object(ObjectId(id)).cloned();
        let o = o.unwrap_or_else(|| panic!("illusion #{id}"));
        assert_eq!(
            (o.kind, o.illusion_of),
            (ObjectKind::Disciple, Some(ObjectId(DISCIPLE)))
        );
    }
    assert_eq!(info_title(&mut app), "Disciple", "the parent");

    // As their owner sees them.
    pause(&mut app);
    let first = ILLUSIONS[0];
    let at = pos(&app, first);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [first]);
    assert_eq!(info_title(&mut app), "Disciple (illusion)");
    let own = panel_of(&app, P1, first);
    let parent = app.world().resource::<SimRes>().object(ObjectId(DISCIPLE)).cloned();
    let (energy, max) = parent.as_ref().and_then(sc_sim::energy::of).expect("the parent's pool");
    assert!(energy < 5.0, "its 50 spent: {energy}");
    let line = format!("Energy {} / {}", energy.floor(), max.floor());
    assert_eq!(own.body[1..], ["HP 40 / 40".to_string(), line], "its parent's energy");
    assert_eq!(own.body, panel_of(&app, P1, DISCIPLE).body, "its parent's lines");
    for id in ILLUSIONS {
        assert_ne!(mesh(&app, id), mesh(&app, DISCIPLE), "#{id} washed out");
        assert_eq!(panel_of(&app, P1, id).title, "Disciple (illusion)");
    }
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(feedback(&app), Some(ILLUSIONS_CANT_CAST_TEXT), "F greyed");
    assert_eq!(abilities(&app).len(), 1, "nothing cast");
    let five: Vec<u32> = [DISCIPLE].into_iter().chain(ILLUSIONS).collect();
    select_units(&mut app, &five);
    updates(&mut app, 1);
    let blue = player_color(Some(P1));
    let washed = five
        .iter()
        .map(|&id| (id, if id == DISCIPLE { blue } else { illusion_color(blue) }));
    assert_eq!(portraits(&mut app), washed.collect::<Vec<_>>(), "the portraits too");

    // As player 2 sees them, after the debug player switch: five Disciples.
    tap(&mut app, &[], KeyCode::F2);
    assert_eq!(local_player(&app), P2);
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    updates(&mut app, 2);
    let parent = panel_of(&app, P2, DISCIPLE);
    assert_eq!(parent.title, "Disciple");
    for id in ILLUSIONS {
        assert_eq!(mesh(&app, id), mesh(&app, DISCIPLE), "#{id} drawn as a Disciple");
        assert_eq!(panel_of(&app, P2, id), parent, "#{id} read as a Disciple");
    }
    pause(&mut app);
    let at = pos(&app, first);
    click(&mut app, at, &[]);
    assert_eq!(selected(&app), [first]);
    assert_eq!(info_title(&mut app), "Disciple", "selected");
}
