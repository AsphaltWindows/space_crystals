//! The Temple's panel in the client (slice M5-N3; `factions/cults/structures/temple.md > Placeholder stats`: "Panel:
//! **Q** Enhanced Constitution (next level), **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of
//! a kind (README > Choosing Recruits, Release and Eject All), **V: Eject All** (staff leave), **X: Cancel** (last
//! research). Recruits enter by right-click."): Recruits right-clicked into it staff it, Q researches Enhanced
//! Constitution Level 1, its key "In progress" until it completes and Level 2 after, greyed without a Cathedral; an
//! Enhanced Constitution Militant's MaxHP rises at once; A/S/D/F let one Recruit of their kind out, V the others, X
//! takes the last research back. The panel as plain data is the unit tests' (`command_ui/interface/tests/temple.rs`),
//! the research itself the sim's (`crates/sim/tests/research`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerId,
    ProductionCommand, RecruitKind, SpawnAt, Tech, TilePos,
};
use space_crystals::command_ui::feedback::{IN_PROGRESS_TEXT, NO_RECRUIT_TEXT, TargetFeedback};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PointerDisplay};
use space_crystals::hud::info_panel::InfoText;
use space_crystals::hud::info_panel::hint::CommandHint;

use super::armory::{buttons, holdings, inside, tick, until};
use super::*;
use KeyCode::{KeyA, KeyQ, KeyS, KeyV, KeyW, KeyX};
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const TEMPLE_KIND: ObjectKind = ObjectKind::Temple;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const EC1: Tech = Tech::EnhancedConstitutionL1;
const EC2: Tech = Tech::EnhancedConstitutionL2;
const IM1: Tech = Tech::ImprovedMetabolismL1;

/// The objects of [`temple_game`], by id, and where they are drawn (the map is flat). The Temple, unturned, has its
/// entrance, side B, to the east and its exit, side C, to the west (`factions/cults/structures/temple.md > Placeholder
/// stats`: "SymmetryType ABAC, **EntranceSide B, ExitSide C** (like the Armory)").
const TEMPLE: u32 = 1;
const TEMPLE_AT: Vec2 = Vec2::new(1120.0, 864.0); // tiles (16, 12) to (18, 14): x 1024 to 1216
/// Its staff: two Plain Recruits and one with Enhanced Constitution.
const RECRUITS: [u32; 3] = [2, 3, 4];
const CONSTITUTION_RECRUIT: u32 = 3;
/// A Militant with Enhanced Constitution, on the map south-east of the Temple.
const MILITANT: u32 = 5;
const MILITANT_AT: Vec2 = Vec2::new(1400.0, 1100.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: a Temple,
/// three Recruits, Recruit 3 with Enhanced Constitution (a debug command stands for the Altar), which a box selects and
/// a right-click on the Temple sends in (`factions/cults/structures/temple.md > Recruit Staffing`: "right-click own
/// Temple with Recruits selected → Enter Temple"; the Enter pointer over it), and a Militant with Enhanced
/// Constitution; returned once the three are inside.
fn temple_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(PlayerId(2), Faction::Cults)
        .spawn(TEMPLE_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(RECRUIT, Some(P1), point(1300.0, 800.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 864.0))
        .spawn(RECRUIT, Some(P1), point(1300.0, 928.0))
        .spawn(ObjectKind::Militant, Some(P1), point(MILITANT_AT.x, MILITANT_AT.y));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    let augmentation = Augmentation::of(Constitution);
    for id in [CONSTITUTION_RECRUIT, MILITANT] {
        let id = ObjectId(id);
        submit(
            &mut app,
            P1,
            Command::Debug(DebugCommand::SetAugmentation { id, augmentation }),
        );
    }
    app.update();
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, TEMPLE_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, TEMPLE_AT, &[]);
    let enter = OrderKind::Enter(ObjectId(TEMPLE));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
    until(&mut app, 300, |app| inside(app, TEMPLE).len() == RECRUITS.len());
    app
}

/// The lines of the InfoPanel's `area` (its body, or the activity area of an own structure), while it shows them.
pub(super) fn lines_of(app: &mut App, area: InfoText) -> Vec<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    (texts.iter(app.world()))
        .find(|(which, _, node)| **which == area && node.display != Display::None)
        .map(|(_, text, _)| text.0.lines().map(String::from).collect())
        .unwrap_or_default()
}

/// The InfoPanel's body lines.
pub(super) fn info_lines(app: &mut App) -> Vec<String> {
    lines_of(app, InfoText::Body)
}

/// The InfoPanel's activity lines: what an own structure is doing.
pub(super) fn activity_lines(app: &mut App) -> Vec<String> {
    lines_of(app, InfoText::Activity)
}

/// The feedback message that shows.
pub(super) fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// What the InfoPanel's command hint says over the button of `slot`, a frame after the pointer moved onto it.
fn hint_over(app: &mut App, slot: CommandSlot) -> Option<String> {
    let surface = app.world().resource::<SurfaceSize>().logical();
    input(app).cursor = Some(CommandGrid::on(surface).slot_rect(slot).center());
    app.update();
    let mut query = app.world_mut().query_filtered::<(&Text, &Node), With<CommandHint>>();
    let (text, node) = query.single(app.world()).expect("one hint");
    (node.display != Display::None).then(|| text.0.clone())
}

/// Object `id`'s HP and MaxHP, with its Augmentations at its owner's research, wherever it is.
fn hp_of(app: &App, id: u32) -> (f32, f32) {
    let sim = app.world().resource::<SimRes>();
    let held = sim.held_in(ObjectId(TEMPLE));
    let o = (sim.object(ObjectId(id)))
        .or_else(|| held.iter().map(|held| &held.object).find(|o| o.id == ObjectId(id)))
        .expect("in the game");
    let research = sim.research();
    (o.hp.expect("HP"), o.max_hp_with(research).expect("MaxHP"))
}

/// `factions/cults/structures/temple.md > Placeholder stats`: "Research (BaseResearchTime = with 1 Recruit):
/// Enhanced Constitution L1 [...]: **100 SC, 960 frames (60 s)**"; `core/tech_and_research.md > Research > Faction
/// Exceptions > Cults — research is staffed by Recruits`: "3 reduce it to a third". The Temple's DefaultState shows Q
/// and W each line's Level 1, A/S/D/F its staff of each kind, greyed where it has none, X greyed with nothing to cancel
/// and V Eject All; the InfoPanel its staff. Q queues Enhanced Constitution Level 1 for 100 Space Crystals; its key
/// shows "In progress", greyed, and says so when pressed, while W stays; the InfoPanel shows the research. 320 frames
/// later it completes: the Militant with Enhanced Constitution goes from 60 to 80 MaxHP at once, its HP with it
/// (design change 0336), and so does the Recruit inside; Q shows Level 2, greyed "Requires a Cathedral".
#[test]
fn q_researches_enhanced_constitution_in_progress_then_level_2() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, ReleaseOne, Research, Researching};
    let mut app = temple_game();
    click(&mut app, MILITANT_AT, &[]);
    assert_eq!(selected(&app), [MILITANT]);
    assert_eq!(
        info_lines(&mut app).get(1).map(String::as_str),
        Some("HP 60 / 60"),
        "before"
    );
    click(&mut app, TEMPLE_AT, &[]);
    assert_eq!(selected(&app), [TEMPLE]);
    let c = PanelButton::Command;
    let (q, w, x, v) = (slot(0, 0), slot(0, 1), CommandSlot::CANCEL_PRODUCTION, slot(2, 3));
    assert_eq!(
        buttons(&mut app),
        [
            (q, c(Research(EC1)), Common, None),
            (w, c(Research(IM1)), Common, None),
            (slot(1, 0), c(ReleaseOne(Plain)), Common, Some(2)),
            (slot(1, 1), c(ReleaseOne(Constitution)), Common, Some(1)),
            (slot(1, 2), c(ReleaseOne(Metabolism)), Disabled, Some(0)),
            (slot(1, 3), c(ReleaseOne(Both)), Disabled, Some(0)),
            (x, c(CancelProduction), Disabled, None),
            (v, c(EjectAll), Common, None),
        ]
    );
    let hint = "Q: Research Enhanced Constitution Level 1 (100 Space Crystals)";
    assert_eq!(hint_over(&mut app, q).as_deref(), Some(hint));
    let lines = activity_lines(&mut app);
    assert!(lines.iter().any(|line| line == "Staff: 3 / 6"), "{lines:?}");

    tap(&mut app, &[], KeyQ);
    let research = ProductionCommand::Research(EC1);
    assert_eq!(productions(&app).last(), Some(&(P1, vec![TEMPLE], research)));
    assert_eq!(bank(&app), 1000 - 100);
    let keys = |app: &mut App| buttons(app)[..2].to_vec();
    assert_eq!(
        keys(&mut app),
        [
            (q, c(Researching(EC1)), Disabled, None),
            (w, c(Research(IM1)), Common, None)
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(IN_PROGRESS_TEXT));
    assert_eq!(productions(&app).len(), 1, "nothing more sent");
    let lines = activity_lines(&mut app);
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("Researching Enhanced Constitution Level 1: ")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Queue: Enhanced Constitution Level 1 (1 / 5)"),
        "{lines:?}"
    );

    let queued = tick(&app);
    until(&mut app, 330, |app| hp_of(app, MILITANT).1 == 80.0);
    assert!(tick(&app) <= queued + 320, "a third of 960 frames with three Recruits");
    assert_eq!(hp_of(&app, MILITANT), (80.0, 80.0), "its HP rises by 20");
    assert_eq!(
        hp_of(&app, CONSTITUTION_RECRUIT),
        (80.0, 80.0),
        "the Recruit inside too"
    );
    app.update();
    assert_eq!(keys(&mut app)[0], (q, c(Research(EC2)), Disabled, None));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some("Requires a Cathedral"));
    let hint = "Q: Research Enhanced Constitution Level 2 (150 Space Crystals)";
    assert_eq!(hint_over(&mut app, q).as_deref(), Some(hint));
    let militant_at = pos(&app, MILITANT);
    click(&mut app, militant_at, &[]);
    assert_eq!(
        info_lines(&mut app).get(1).map(String::as_str),
        Some("HP 80 / 80"),
        "after"
    );
}

/// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "pressing one sends **one**
/// Recruit of that kind out of the exit at once (no queue)"; "it is greyed if there are none"; `factions/cults/
/// structures/temple.md > Placeholder stats`: "**V: Eject All** (staff leave), **X: Cancel** (last research)". S lets
/// the Constitution Recruit out of side C, its button then greyed at 0, saying so when pressed; A lets the Plain
/// Recruit that went in first out, the count dropping to 1; V lets the last one out. W queues Improved Metabolism
/// Level 1 and X takes it back, refunded in full.
#[test]
fn asdf_release_one_recruit_of_a_kind_v_ejects_and_x_cancels() {
    use ButtonLook::Disabled;
    use PanelCommand::ReleaseOne;
    let mut app = temple_game();
    click(&mut app, TEMPLE_AT, &[]);
    let staff = inside(&app, TEMPLE);
    let first_plain = *staff.iter().find(|&&id| id != CONSTITUTION_RECRUIT).unwrap();
    tap(&mut app, &[], KeyS);
    let release = HoldCommand::ReleaseOne(Constitution);
    assert_eq!(holdings(&app).last(), Some(&(P1, vec![TEMPLE], release)));
    until(&mut app, 30, |app| !inside(app, TEMPLE).contains(&CONSTITUTION_RECRUIT));
    app.update();
    let out = pos(&app, CONSTITUTION_RECRUIT);
    assert!(out.x <= 1024.0, "out of side C, its west edge: {out}");
    let s = slot(1, 1);
    let shown = |app: &mut App, at| buttons(app).into_iter().find(|b| b.0 == at).unwrap();
    assert_eq!(
        shown(&mut app, s),
        (s, PanelButton::Command(ReleaseOne(Constitution)), Disabled, Some(0))
    );
    tap(&mut app, &[], KeyS);
    assert_eq!(said(&app), Some(NO_RECRUIT_TEXT));
    assert_eq!(holdings(&app).len(), 1, "nothing more sent");

    tap(&mut app, &[], KeyA);
    until(&mut app, 30, |app| !inside(app, TEMPLE).contains(&first_plain));
    app.update();
    assert_eq!(shown(&mut app, slot(1, 0)).3, Some(1));
    tap(&mut app, &[], KeyV);
    assert_eq!(holdings(&app).last(), Some(&(P1, vec![TEMPLE], HoldCommand::EjectAll)));
    until(&mut app, 30, |app| inside(app, TEMPLE).is_empty());

    tap(&mut app, &[], KeyW);
    assert_eq!(bank(&app), 1000 - 100);
    tap(&mut app, &[], KeyX);
    let cancel = ProductionCommand::CancelProduction;
    assert_eq!(productions(&app).last(), Some(&(P1, vec![TEMPLE], cancel)));
    app.update();
    assert_eq!(bank(&app), 1000, "refunded in full");
}
