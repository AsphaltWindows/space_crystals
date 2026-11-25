//! The remembered SupplyDeliveryStation (`core/vision.md > Explored Selectability`): only a GDO player may select it,
//! its frame draws over it, and its InfoPanel shows it as last seen but with the station's real delivery timer
//! (design change 0057).

use sc_sim::{ResourceState, TICKS_PER_SECOND};
use space_crystals::hud::info_panel::{InfoText, NO_DELIVERY};
use space_crystals::selection::markers::SelectionMarkers;
use space_crystals::selection::{ControlStates, Selection};

use super::*;

fn select(app: &mut App, id: u32) {
    let selection = Selection::new([(ObjectId(id), ObjectKind::SupplyDeliveryStation)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    app.update();
}

fn selected(app: &App) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    states.selection(P1).ids().map(|id| id.0).collect()
}

/// The InfoPanel's title and lines, while it shows something. (An own structure's activity area is tested in
/// `hud.rs`.)
fn info(app: &mut App) -> Option<(String, String)> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    let (mut title, mut body) = (None, None);
    for (which, text, node) in texts.iter(app.world()) {
        if node.display == Display::None {
            continue;
        }
        match which {
            InfoText::Title => title = Some(text.0.clone()),
            InfoText::Body => body = Some(text.0.clone()),
            InfoText::Activity => {}
        }
    }
    title.zip(body)
}

/// `core/vision.md > Explored Selectability`: a GDO player may select the station on Explored tiles, by its record,
/// and keeps it selected when it leaves; a click there picks it. Its InfoPanel shows the station as last seen,
/// without CurrentSupplies, and when that was, but with the live station's delivery countdown, not the one last seen
/// (`core/map.md > SupplyDeliveryStation > InfoPanel > Explored`, design change 0057).
#[test]
fn gdo_players_select_a_remembered_station() {
    let mut app = world(Faction::Gdo);
    select(&mut app, STATION);
    let (title, body) = info(&mut app).unwrap();
    assert_eq!(title, "Supply Delivery Station");
    assert!(body.starts_with("Neutral\nSupplies: 0\n"), "{body}");

    scout_to(&mut app, 20, 7);
    let left = tick(&app);
    run_until(&mut app, left + 40);
    assert_eq!(
        selectable(&app),
        [STATION, SCOUT],
        "the patch isn't selectable, the station is"
    );
    assert_eq!(selected(&app), [STATION]);
    let sim = app.world().resource::<SimRes>();
    let objects = Selectables::new(sim, &Sight::new(sim, P1, true), 1.0);
    let station_centre = TilePos::new(9, 11).corner();
    assert_eq!(objects.at(station_centre).map(|o| o.id), Some(ObjectId(STATION)));

    let record = sim.vision().player(P1).unwrap().record(ObjectId(STATION)).unwrap();
    let Some(ResourceState::SupplyStation {
        next_delivery_in: Some(then),
        ..
    }) = record.resource
    else {
        panic!("{record:?}");
    };
    let Some(ResourceState::SupplyStation {
        next_delivery_in: Some(now),
        ..
    }) = sim.object(ObjectId(STATION)).unwrap().resource
    else {
        panic!("the station counts down");
    };
    assert!(seconds(then) > seconds(now), "{then} {now}");
    let ago = seconds(sim.tick() - record.seen);
    let (title, body) = info(&mut app).unwrap();
    assert_eq!(title, "Supply Delivery Station");
    assert_eq!(
        body,
        format!(
            "Neutral\nDelivery size: 4\nDelivery interval: 60 s\nNext delivery in {} s\nLast seen {ago} s ago",
            seconds(now)
        )
    );
}

/// A selected structure's frame lies inside its footprint, which the structure's visual covers: it draws in front
/// of the structure, live or remembered, and behind a unit south of it.
#[test]
fn a_selected_stations_frame_draws_over_the_station() {
    let mut app = world(Faction::Gdo);
    select(&mut app, STATION);
    scout_to(&mut app, 8, 13);
    let z = |app: &App, entity| translation(app, entity).z;
    let marker = app.world().resource::<SelectionMarkers>().by_id[&ObjectId(STATION)];
    let live = app.world().resource::<ObjectVisuals>().by_id.clone();
    assert!(drawn(&app, STATION));
    assert!(z(&app, marker) > z(&app, live[&ObjectId(STATION)]), "over the station");
    assert!(
        z(&app, marker) < z(&app, live[&ObjectId(SCOUT)]),
        "under the scout south of it"
    );

    scout_to(&mut app, 20, 7);
    assert_eq!(remembered(&app), [PATCH, STATION]);
    let marker = app.world().resource::<SelectionMarkers>().by_id[&ObjectId(STATION)];
    let record = app.world().resource::<RememberedVisuals>().by_id[&ObjectId(STATION)];
    assert!(z(&app, marker) > z(&app, record), "over the remembered station");
}

/// A number of ticks in whole seconds, rounded up, as the InfoPanel shows them.
fn seconds(ticks: u32) -> u32 {
    ticks.div_ceil(TICKS_PER_SECOND)
}

/// The station's supplies and delivery countdown, live.
fn station_state(app: &App) -> (u32, Option<u32>) {
    let sim = app.world().resource::<SimRes>();
    match sim.object(ObjectId(STATION)).unwrap().resource {
        Some(ResourceState::SupplyStation {
            current_supplies,
            next_delivery_in,
            ..
        }) => (current_supplies, next_delivery_in),
        ref other => panic!("{other:?}"),
    }
}

/// The InfoPanel's lines, but for the last one (the placeholder note of when the station was last seen).
fn remembered_info(app: &mut App) -> Vec<String> {
    let (_, body) = info(app).expect("the station is selected");
    let (lines, last) = body.rsplit_once('\n').unwrap();
    assert!(last.starts_with("Last seen "), "{body}");
    lines.lines().map(str::to_string).collect()
}

/// Empties the station, as an enemy picking its supplies up will (supply pickup doesn't exist yet): the sim is
/// replaced by an edited state dump of itself. Its next tick starts the countdown. A replaced sim starts the
/// selection over, so this selects the station again.
fn empty_station(app: &mut App) {
    let mut state = serde_json::to_value(app.world().resource::<SimRes>().sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let station = objects.iter_mut().find(|o| o["id"] == STATION).unwrap();
    station["resource"]["SupplyStation"]["current_supplies"] = 0.into();
    let sim = serde_json::from_value(state).unwrap();
    app.world_mut().resource_mut::<SimRes>().replace(sim);
    app.update();
    select(app, STATION);
}

/// Design change 0057's verification (`core/map.md > SupplyDeliveryStation > InfoPanel > Explored`): a GDO player
/// sees a stocked station, which runs no countdown, and leaves. Out of sight, someone empties it: the panel switches
/// from no delivery scheduled to the live countdown, which runs down to the actual delivery; then no delivery is
/// scheduled again. CurrentSupplies stays hidden all along. (Other factions can't select it there:
/// `other_factions_cannot_select_a_remembered_station`.)
#[test]
fn a_gdo_player_sees_a_remembered_stations_real_delivery_timer() {
    const INTERVAL: u32 = 64;
    let station = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 4,
        delivery_size: 4,
        delivery_interval: INTERVAL,
    };
    let mut app = world_with(Faction::Gdo, station);
    select(&mut app, STATION);
    let stocked = ["Neutral", "Delivery size: 4", "Delivery interval: 4 s", NO_DELIVERY];
    let (_, body) = info(&mut app).unwrap();
    assert_eq!(
        body,
        format!("Neutral\nSupplies: 4\nDelivery size: 4\nDelivery interval: 4 s\n{NO_DELIVERY}")
    );

    scout_to(&mut app, 20, 7);
    let left = tick(&app);
    run_until(&mut app, left + 20);
    assert_eq!(remembered(&app), [PATCH, STATION]);
    assert_eq!(remembered_info(&mut app), stocked);

    empty_station(&mut app);
    assert_eq!(remembered(&app), [PATCH, STATION], "still out of sight");
    assert_eq!(selected(&app), [STATION]);
    let record = app
        .world()
        .resource::<SimRes>()
        .vision()
        .player(P1)
        .unwrap()
        .record(ObjectId(STATION));
    assert!(
        matches!(
            record.unwrap().resource,
            Some(ResourceState::SupplyStation {
                current_supplies: 4,
                next_delivery_in: None,
                ..
            })
        ),
        "the record still has it stocked"
    );
    let (supplies, Some(countdown)) = station_state(&app) else {
        panic!("the emptied station counts down");
    };
    assert_eq!(supplies, 0);
    assert!(countdown < INTERVAL, "running: {countdown}");
    let due = tick(&app) + countdown;
    while tick(&app) < due {
        let (_, Some(countdown)) = station_state(&app) else {
            panic!("no delivery before {due}: {}", tick(&app));
        };
        let mut counting = stocked.map(str::to_string).to_vec();
        counting[3] = format!("Next delivery in {} s", seconds(countdown));
        assert_eq!(remembered_info(&mut app), counting, "tick {}", tick(&app));
        app.update();
    }
    assert_eq!(station_state(&app), (4, None), "delivered when the panel said");
    assert_eq!(remembered_info(&mut app), stocked);
}

/// Other factions may not select a remembered station: it leaves the selection when its tiles turn Explored.
#[test]
fn other_factions_cannot_select_a_remembered_station() {
    let mut app = world(Faction::Syndicate);
    select(&mut app, STATION);
    assert_eq!(selected(&app), [STATION], "Visible: selectable by anyone");
    scout_to(&mut app, 20, 7);
    assert_eq!(remembered(&app), [PATCH, STATION], "still drawn");
    assert_eq!(selectable(&app), [SCOUT]);
    assert_eq!(selected(&app), [0_u32; 0]);
    assert_eq!(info(&mut app), None);
}

/// `core/vision.md > Explored`: the removal of a station that a GDO player remembers out of sight tells them nothing
/// (the filter of events gives no player the removal of what they remember out of sight, slice M4-V): it stays
/// selected, by its record, which still shows where it was. Back in sight, the record goes, and the selection with it.
#[test]
fn a_remembered_station_removed_out_of_sight_stays_selected_until_its_tiles_are_seen() {
    let mut app = world(Faction::Gdo);
    select(&mut app, STATION);
    scout_to(&mut app, 20, 7);
    assert_eq!(selected(&app), [STATION]);
    debug(&mut app, DebugCommand::Remove { id: ObjectId(STATION) });
    app.update();
    let removed = app.world().resource::<SimRes>().object(ObjectId(STATION)).is_none();
    assert!(removed, "the station has left the game");
    assert_eq!(remembered(&app), [PATCH, STATION], "unseen");
    assert_eq!(selected(&app), [STATION], "the player didn't learn of it");
    scout_to(&mut app, 5, 7);
    assert_eq!(remembered(&app), [0_u32; 0]);
    assert_eq!(selected(&app), [0_u32; 0], "seen gone");
}
