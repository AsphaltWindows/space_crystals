//! Supplies in the client (slice M4-I): an Agent's right-click on a Supply Delivery Station is Gather
//! (`factions/syndicate/units/agent.md > Right-Click Resolution`: "Supply Delivery Station → Gather supplies"), and
//! the placeholder notice of an empty station (design change 0256 point 3, `space_crystals::hud::notices`).

use space_crystals::hud::notices::{StationNotice, StationNoticeView};
use space_crystals::sim_driver::SimEventMessage;

use super::*;

/// Where the Agent of these tests stands: in the world view, west of the station.
const AGENT_AT: Vec2 = Vec2::new(1600.0, 760.0);

/// A selected Agent right-clicked on the station gathers there; the scenario's Peacekeepers move to it
/// (`right_click`).
#[test]
fn an_agent_right_clicked_on_a_station_gathers_supplies() {
    let mut app = game();
    let agent = spawn(&mut app, ObjectKind::Agent, Some(PlayerId(1)), AGENT_AT);
    pause(&mut app);
    click(&mut app, AGENT_AT, &[]);
    assert_eq!(selected(&app), [agent]);
    right_click(&mut app, at(STATION), &[]);
    let gather = OrderKind::Gather(ObjectId(STATION));
    assert_eq!(orders(&app).last(), Some(&given(&[agent], gather, false)));
}

/// Whether the notice shows.
fn notice_shown(app: &mut App) -> bool {
    let mut views = app.world_mut().query_filtered::<&Visibility, With<StationNoticeView>>();
    let shown: Vec<bool> = (views.iter(app.world())).map(|v| *v == Visibility::Inherited).collect();
    assert_eq!(shown.len(), 1, "one notice");
    shown[0]
}

/// The sim's report of the station's emptying for `owner`, handed to the client as the sim driver forwards its events,
/// in the tick that runs next.
fn station_empty(app: &mut App, owner: PlayerId) -> u32 {
    let tick = app.world().resource::<SimRes>().tick() + 1;
    let event = SimEvent::SupplyStationEmpty {
        station: ObjectId(STATION),
        owner,
    };
    app.world_mut()
        .resource_mut::<Messages<SimEventMessage>>()
        .write(SimEventMessage { tick, event });
    tick
}

/// Design change 0256 point 3: "the owner gets a notification", placeholder: the local player sees the notice of
/// their own empty station, for `FeedbackTuning::notice_ticks` from the tick it emptied; not another player's.
#[test]
fn the_empty_station_notice_shows_for_its_owner_for_a_while() {
    let mut app = game();
    assert!(!notice_shown(&mut app));
    station_empty(&mut app, PlayerId(2));
    app.update();
    assert!(!notice_shown(&mut app), "player 2's");
    let emptied = station_empty(&mut app, PlayerId(1));
    app.update();
    assert!(notice_shown(&mut app));
    assert_eq!(app.world().resource::<StationNotice>().since, Some(emptied));
    while app.world().resource::<SimRes>().tick() < emptied + space_crystals::tuning::feedback::BUILTIN.notice_ticks - 1
    {
        app.update();
        assert!(notice_shown(&mut app));
    }
    app.update();
    assert!(!notice_shown(&mut app), "gone after notice_ticks");
}
