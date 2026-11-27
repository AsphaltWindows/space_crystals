//! The Radar Station in the client (slice M6-GR; `factions/gdo/structures/radar_station.md > Placeholder stats`:
//! "**Scan** (hotkey **Q**, targeted: left-click any point on the map): reveals a circle of radius **6 grid units**
//! for **8 seconds** (128 frames); **cooldown 60 seconds** (960 frames); free. Each Radar Station has its own
//! cooldown"): Q awaits any point on the map under the Cast pointer and the circle of the scan, out of sight too, and
//! scans there from the ActiveGroup's one station, which then cools down, greyed and saying so; two stations are two
//! SelectionGroups, Tab moving from one to the other (design change 0422). The panel and the clicks as plain data are
//! the unit tests' (`command_ui/resolution/tests/radar_station.rs`), the scans themselves the sim's
//! (`crates/sim/tests/radar`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{AreaAbility, Faction, ObjectId, ObjectKind, PlayerId, SpawnAt, TilePos};
use space_crystals::command_ui::feedback::COOLING_DOWN_TEXT;
use space_crystals::command_ui::indicators::cast;
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PointerDisplay};

use super::armory::buttons;
use super::shrine::casts;
use super::temple::said;
use super::*;
use KeyCode::{KeyQ, ShiftLeft, Tab};

const P1: PlayerId = PlayerId(1);
const SCAN: PanelCommand = PanelCommand::Cast(AreaAbility::Scan);

/// The objects of [`radar_game`], by id, and where they are drawn (the map is flat): player 1's two Radar Stations, on
/// tiles (16, 12) to (17, 13) and (20, 12) to (21, 13), and the two Generators that power them.
const S1: u32 = 1;
const S1_AT: Vec2 = Vec2::new(1088.0, 832.0);
const S2: u32 = 2;
const S2_AT: Vec2 = Vec2::new(1344.0, 832.0);
/// Two points on screen that none of player 1's objects sees: 16 and 14 grid units east of the stations.
const FAR: Vec2 = Vec2::new(2336.0, 1184.0);
const OTHER: Vec2 = Vec2::new(2208.0, 672.0);

/// A game of player 1, the GDO, on a flat 64 × 48 map, as [`game_of`] sets one up: two Radar Stations and two
/// Generators, at 40 generated for 40 consumed, so that nothing is slowed; player 2 a Generator far off.
fn radar_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .spawn(ObjectKind::RadarStation, Some(P1), tile(16, 12))
        .spawn(ObjectKind::RadarStation, Some(P1), tile(20, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(16, 16))
        .spawn(ObjectKind::Generator, Some(P1), tile(20, 16))
        .spawn(ObjectKind::Generator, Some(PlayerId(2)), tile(56, 40))
        .build();
    game_of(scenario)
}

/// The ActiveGroup's stations and the number of SelectionGroups.
fn groups(app: &App) -> (Vec<u32>, usize) {
    let states = app.world().resource::<ControlStates>();
    let selection = states.selection(P1);
    let active = selection.active_group().expect("an ActiveGroup");
    (
        active.instances.iter().map(|id| id.0).collect(),
        selection.groups().len(),
    )
}

/// Q as shown: its button and look.
fn q(app: &mut App) -> (PanelButton, ButtonLook) {
    let shown = buttons(app);
    let q = shown.iter().find(|b| b.0 == slot(0, 0)).expect("a Q");
    (q.1, q.2)
}

/// Whether player 1 sees the point `at` now, as the gate judges a point (`Sim::sees_point`).
fn sees(app: &App, at: Vec2) -> bool {
    app.world().resource::<SimRes>().sees_point(P1, at)
}

/// Design change 0422's Verification: "Select two Radar Stations: two groups. Q, click: one scans and cools down. Tab,
/// Q, click: the second scans." Two stations selected (Shift-click adds a building) are two SelectionGroups, the
/// first active. Q awaits a point, the scan's 6 grid units round the pointer (the cursor's circle): over a point that
/// no one of player 1's sees, the Cast pointer, and a click there scans from the first station alone, which reveals the
/// point; its Q is greyed then, pressed says "Cooling down", and awaits nothing. Tab moves to the second, whose Q scans
/// another point; Tab back, the first is still cooling down.
#[test]
fn two_stations_are_two_groups_and_q_tab_q_scans_two_places() {
    use ButtonLook::{Disabled, Group};
    let mut app = radar_game();
    click(&mut app, S1_AT, &[]);
    click(&mut app, S2_AT, &[ShiftLeft]);
    assert_eq!(selected(&app), [S1, S2]);
    assert_eq!(groups(&app), (vec![S1], 2), "Ungroupable: a group each");
    let c = PanelButton::Command;
    assert_eq!(q(&mut app), (c(SCAN), Group));
    assert!(!sees(&app, FAR) && !sees(&app, OTHER), "out of sight");

    tap(&mut app, &[], KeyQ);
    let awaiting = InterfaceState::AwaitingTarget {
        pending: SCAN,
        queued: false,
    };
    assert_eq!(interface(&app), awaiting);
    assert_eq!(cast::awaited(interface(&app)), Some(AreaAbility::Scan));
    assert_eq!(cast::radius(AreaAbility::Scan), 384.0, "6 grid units");
    assert_eq!(
        pointer_over(&mut app, FAR),
        Some(PointerDisplay::Cast),
        "any point on the map"
    );
    click(&mut app, FAR, &[]);
    let sent = casts(&app);
    let [(player, casters, ability, at)] = &sent[..] else {
        panic!("one scan: {sent:?}")
    };
    assert_eq!((*player, &casters[..], *ability), (P1, &[S1][..], AreaAbility::Scan));
    assert!(near(*at, FAR), "{at}");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 2);
    assert!(sees(&app, FAR), "the scan reveals it");
    let sim = app.world().resource::<SimRes>();
    assert!(sim.object(ObjectId(S1)).unwrap().cooldown.is_some());
    assert!(sim.object(ObjectId(S2)).unwrap().cooldown.is_none(), "its own cooldown");
    assert_eq!(q(&mut app), (c(SCAN), Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(COOLING_DOWN_TEXT));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    tap(&mut app, &[], Tab);
    assert_eq!(groups(&app), (vec![S2], 2));
    assert_eq!(q(&mut app), (c(SCAN), Group));
    tap(&mut app, &[], KeyQ);
    click(&mut app, OTHER, &[]);
    let sent = casts(&app);
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert_eq!((&sent[1].1[..], sent[1].2), (&[S2][..], AreaAbility::Scan));
    assert!(near(sent[1].3, OTHER), "{}", sent[1].3);
    updates(&mut app, 2);
    assert!(sees(&app, OTHER));
    tap(&mut app, &[], Tab);
    assert_eq!(groups(&app), (vec![S1], 2));
    assert_eq!(q(&mut app), (c(SCAN), Disabled), "still cooling down");
}
