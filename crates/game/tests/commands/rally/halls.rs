//! Two Recruitment Halls' rally point (design change 0395; `factions/cults/structures/recruitment_hall.md`:
//! "Groupable - true (user, 2026-11-27; was false: selected Halls form one group, and a rally point set with several
//! selected applies to all of them)").

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, SimEvent};

use super::*;
use KeyCode::KeyC;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;
/// Player 1's two Halls, on tiles (16, 10) to (19, 13) and (32, 10) to (35, 13): their 10 × 10 Recruitment Areas
/// don't meet, so each recruits at its full Effectiveness.
const HALLS: [u32; 2] = [1, 2];
const HALLS_AT: [TilePos; 2] = [TilePos::new(16, 10), TilePos::new(32, 10)];
/// Bare ground south of the Halls, in the world view.
const RALLY_AT: Vec2 = Vec2::new(1664.0, 1150.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map of Recruitable ground, set up as [`game`] does: the two Halls
/// of [`HALLS`], recruiting from the first tick.
fn halls_game() -> App {
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(HALL, Some(P1), SpawnAt::Tile(HALLS_AT[0]))
        .spawn(HALL, Some(P1), SpawnAt::Tile(HALLS_AT[1]));
    game_of(builder.build())
}

/// The view point in the middle of the Hall whose top-left tile is `origin`.
fn middle(origin: TilePos) -> Vec2 {
    origin.corner() + Vec2::splat(128.0)
}

/// 0395's Verification: "Box-select 2 Halls and set a rally point: both send new Recruits there." A box selects one
/// own building (`systems/controls/selection.md > BoxSelection > Priority tiers`: "Own buildings — single-select"), so
/// the second Hall joins the first with a Shift + box. The two are one SelectionGroup, whose C, Set Rally Point, and
/// its click on the ground give both the rally point, one command; each Hall's next Recruit walks there.
#[test]
fn two_selected_halls_both_send_their_recruits_to_the_rally_point() {
    let mut app = halls_game();
    click(&mut app, middle(HALLS_AT[0]), &[]);
    add_box(&mut app, middle(HALLS_AT[1]));
    assert_eq!(selected(&app), HALLS);
    let states = app.world().resource::<ControlStates>();
    let selection = states.selection(P1);
    let active = selection.active_group().expect("an ActiveGroup");
    assert_eq!(selection.groups().len(), 1, "one SelectionGroup");
    assert_eq!(active.instances, HALLS.map(ObjectId));

    tap(&mut app, &[], KeyC);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::SetRallyPoint));
    click(&mut app, RALLY_AT, &[]);
    let given = rallies_since(&app, 0);
    assert!(
        matches!(given[..], [(ref producers, Target::Point(p))] if producers == &HALLS && near(p, RALLY_AT)),
        "{given:?}"
    );
    for hall in HALLS {
        assert!(
            rallies_at(rally(&app, hall), RALLY_AT),
            "{hall}: {:?}",
            rally(&app, hall)
        );
    }

    let events = run_collecting(&mut app, 400);
    for hall in HALLS {
        let recruit = events.iter().find_map(|event| match *event {
            SimEvent::UnitProduced { producer, unit } if producer == ObjectId(hall) => Some(unit.0),
            _ => None,
        });
        let recruit = recruit.unwrap_or_else(|| panic!("Hall {hall} recruited no one"));
        assert_eq!(object_kind(&app, recruit), ObjectKind::Recruit);
        until(&mut app, 400, |app| pos(app, recruit).distance(RALLY_AT) < 96.0);
    }
}

/// The kind of object `id`.
fn object_kind(app: &App, id: u32) -> ObjectKind {
    app.world()
        .resource::<SimRes>()
        .object(ObjectId(id))
        .expect("in the game")
        .kind
}

/// Runs frames until `done`, `frames` at most.
fn until(app: &mut App, frames: usize, done: impl Fn(&App) -> bool) {
    for _ in 0..frames {
        if done(app) {
            return;
        }
        app.update();
    }
    assert!(done(app), "not within {frames} frames");
}
