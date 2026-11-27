//! The Shrine in the client (slice M5-N5; `factions/cults/structures/shrine.md > Placeholder stats`: "**Health
//! regeneration** (hotkey Q)", "**Energy regeneration** (hotkey W)"; "Targeting: AwaitingTarget, left-click a point in
//! vision. **A/S/D/F** release one Recruit of a kind, **V: Eject All**"): Recruits right-clicked into it take part in
//! its ritual and charge its energy, which its InfoPanel shows with its participants; Q, greyed while it hasn't 75
//! energy, awaits a point in vision under the Cast pointer and the circle of the cast, refuses a point out of vision
//! ("Invalid target") and casts on one in vision, healing the wounded Militant there, which then shows its buff's mark;
//! W boosts the Disciple's energy; A lets one participant out, V the others. The panel as plain data is the unit
//! tests' (`command_ui/interface/tests/shrine.rs`), the casts themselves the sim's (`crates/sim/tests/shrine`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    AreaAbility, Command, Faction, HoldCommand, ObjectId, ObjectKind, PlayerId, RecruitKind, SpawnAt, TilePos,
};
use space_crystals::combat_effects::buffs::buff_marks;
use space_crystals::command_ui::feedback::{INVALID_TARGET_TEXT, NOT_ENOUGH_ENERGY_TEXT};
use space_crystals::command_ui::indicators::cast;
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, PointerDisplay};
use space_crystals::fog::Sight;
use space_crystals::hud::info_panel::InfoText;

use super::armory::{buttons, holdings, inside, tick, until};
use super::temple::{activity_lines, info_lines, said};
use super::*;
use AreaAbility::{EnergyRegeneration, HealthRegeneration};
use KeyCode::{KeyA, KeyQ, KeyV, KeyW};

const P1: PlayerId = PlayerId(1);
/// One energy point, in the sim's units (`sc_sim::energy::ONE`).
pub(super) const ONE: u32 = 65_536;

/// The objects of [`shrine_game`], by id, and where they are drawn (the map is flat). The Shrine, unturned, has its
/// entrance, side B, to the east and its exit, side C, to the west (`factions/cults/structures/shrine.md > Placeholder
/// stats`: "SymmetryType ABAC, **EntranceSide B, ExitSide C**").
const SHRINE: u32 = 1;
const SHRINE_AT: Vec2 = Vec2::new(1088.0, 832.0); // tiles (16, 12) to (17, 13): x 1024 to 1152
/// Its participants, east of it.
const RECRUITS: [u32; 8] = [2, 3, 4, 5, 6, 7, 8, 9];
/// A Militant and a Disciple, south-east of it, 60 su apart.
const MILITANT: u32 = 10;
const MILITANT_AT: Vec2 = Vec2::new(1400.0, 1100.0);
const DISCIPLE: u32 = 11;
const DISCIPLE_AT: Vec2 = Vec2::new(1460.0, 1100.0);
/// A point on screen that none of player 1's units sees.
const FAR: Vec2 = Vec2::new(2400.0, 600.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does: a Shrine, eight Recruits, which a
/// box selects and a right-click on the Shrine sends in (`factions/cults/structures/shrine.md > Ritual Participation`:
/// "right-click own Shrine with Recruits selected → Enter Shrine"; the Enter pointer over it), a Militant and a
/// Disciple; returned once the eight are inside, its energy set to `energy` points and the Militant's HP to `hp` (a
/// replaced state, as [`set_hp`] makes: nothing is selected then).
fn shrine_game(energy: u32, hp: f32) -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(PlayerId(2), Faction::Cults)
        .spawn(ObjectKind::Shrine, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)));
    for i in 0..8 {
        let (x, y) = (1300.0 + 50.0 * (i / 4) as f32, 760.0 + 60.0 * (i % 4) as f32);
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), point(x, y));
    }
    let builder = builder
        .spawn(ObjectKind::Militant, Some(P1), point(MILITANT_AT.x, MILITANT_AT.y))
        .spawn(ObjectKind::Disciple, Some(P1), point(DISCIPLE_AT.x, DISCIPLE_AT.y));
    let mut app = game_of(builder.build());
    select_units(&mut app, &RECRUITS);
    assert_eq!(pointer_over(&mut app, SHRINE_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, SHRINE_AT, &[]);
    let enter = sc_sim::OrderKind::Enter(ObjectId(SHRINE));
    assert_eq!(orders(&app).last(), Some(&given(&RECRUITS, enter, false)));
    until(&mut app, 600, |app| inside(app, SHRINE).len() == RECRUITS.len());
    set_energy(&mut app, SHRINE, energy);
    set_hp(&mut app, MILITANT, hp);
    app
}

/// Sets object `id`'s energy to `points`, in a copy of the game's state that replaces it, as [`set_hp`] does.
pub(super) fn set_energy(app: &mut App, id: u32, points: u32) {
    let mut state = serde_json::to_value(app.world().resource::<SimRes>().sim()).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let object = objects.iter_mut().find(|o| o["id"] == id).expect("on the map");
    object["energy"] = (points * ONE).into();
    let sim = serde_json::from_value(state).unwrap();
    app.world_mut().resource_mut::<SimRes>().replace(sim);
    app.update();
}

/// Object `id`'s energy now, in the sim's units.
pub(super) fn energy(app: &App, id: u32) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).and_then(|o| o.energy).expect("an energy pool")
}

/// The abilities of the buffs object `id` has now.
pub(super) fn buffs(app: &App, id: u32) -> Vec<AreaAbility> {
    let sim = app.world().resource::<SimRes>();
    let o = sim.object(ObjectId(id)).expect("on the map");
    o.buffs.iter().map(|buff| buff.ability).collect()
}

/// The buffs' marks drawn now, by the units they are beside: those of the abilities of each unit's buffs.
pub(super) fn marks(app: &App) -> Vec<(u32, AreaAbility)> {
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let sight = Sight::new(sim, P1, false);
    let marks = buff_marks(sim, &sight, 1.0, &space_crystals::tuning::ClientTuning::BUILTIN);
    (marks.into_iter())
        .map(|mark| {
            let nearest = (sim.objects())
                .min_by(|a, b| a.pos.distance(mark.center).total_cmp(&b.pos.distance(mark.center)))
                .expect("an object");
            (nearest.id.0, mark.ability)
        })
        .collect()
}

/// Every cast sent so far, as the sim received them (like [`orders`]): player, casters, ability, point.
pub(super) fn casts(app: &App) -> Vec<(PlayerId, Vec<u32>, AreaAbility, Vec2)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Cast { casters, ability, at } => {
                Some((c.player, casters.iter().map(|id| id.0).collect(), *ability, *at))
            }
            _ => None,
        })
        .collect()
}

/// The Shrine's Q and W keys as shown: their buttons and looks.
fn keys(app: &mut App) -> Vec<(PanelButton, ButtonLook)> {
    buttons(app)[..2].iter().map(|b| (b.1, b.2)).collect()
}

/// N5's acceptance: "With 8 Recruits inside, 2 energy a second"; "too little energy greys the command"; "Q heals each
/// own Cults infantry unit inside the circle"; "a click outside vision gives "invalid target" feedback"; "the cast
/// cursor with its ring, the InfoPanel's energy, and a buffed unit". At 40 energy, the Shrine's InfoPanel shows its
/// energy and its eight participants, and its energy rises by a quarter point a second for each; Q and W are greyed
/// below their costs, saying so; at 75 Q is offered and awaits its point: the Cast pointer over a point in vision, the
/// Inactive one over a point out of vision, where a click is an invalid target that keeps the target awaited. A click
/// on the wounded Militant casts there, ending the targeting; it pays 75 and the Militant and the Disciple beside it
/// get the buff, marked beside them, the Militant healing 2 HP a second.
#[test]
fn recruits_charge_the_shrine_and_q_heals_at_a_point_in_vision() {
    use ButtonLook::{Common, Disabled};
    let mut app = shrine_game(40, 20.0);
    click(&mut app, SHRINE_AT, &[]);
    assert_eq!(selected(&app), [SHRINE]);
    let lines = info_lines(&mut app);
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("Energy 4") && line.ends_with(" / 200")),
        "{lines:?}"
    );
    let lines = activity_lines(&mut app);
    assert!(lines.iter().any(|line| line == "Participants: 8 / 8"), "{lines:?}");
    let (heal, boost) = (
        PanelCommand::Cast(HealthRegeneration),
        PanelCommand::Cast(EnergyRegeneration),
    );
    let c = PanelButton::Command;
    let points = energy(&app, SHRINE) / ONE;
    assert!(points < 50, "{points}");
    assert_eq!(keys(&mut app), [(c(heal), Disabled), (c(boost), Disabled)]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(NOT_ENOUGH_ENERGY_TEXT));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let (from, since) = (energy(&app, SHRINE), tick(&app));
    updates(&mut app, 16);
    let ticks = tick(&app) - since;
    assert_eq!(
        energy(&app, SHRINE) - from,
        ticks as u32 * 8 * ONE / 64,
        "8 × 0.25 a second"
    );
    app.update();
    let shown = format!("Energy {} / 200", energy(&app, SHRINE) / ONE);
    assert!(info_lines(&mut app).contains(&shown), "{shown}");

    until(&mut app, 400, |app| energy(app, SHRINE) >= 75 * ONE);
    app.update();
    assert_eq!(keys(&mut app), [(c(heal), Common), (c(boost), Common)]);
    tap(&mut app, &[], KeyQ);
    let awaiting = InterfaceState::AwaitingTarget {
        pending: heal,
        queued: false,
    };
    assert_eq!(interface(&app), awaiting);
    assert_eq!(cast::awaited(interface(&app)), Some(HealthRegeneration));
    assert_eq!(cast::radius(HealthRegeneration), 256.0, "4 grid units");
    assert_eq!(pointer_over(&mut app, FAR), Some(PointerDisplay::Inactive));
    assert_eq!(pointer_over(&mut app, MILITANT_AT), Some(PointerDisplay::Cast));
    click(&mut app, FAR, &[]);
    assert_eq!(said(&app), Some(INVALID_TARGET_TEXT));
    assert_eq!(interface(&app), awaiting, "still awaited");
    assert_eq!(casts(&app), []);
    let before = energy(&app, SHRINE);
    click(&mut app, MILITANT_AT, &[]);
    let sent = casts(&app);
    let [(player, casters, ability, at)] = &sent[..] else {
        panic!("one cast: {sent:?}")
    };
    assert_eq!(
        (*player, &casters[..], *ability),
        (P1, &[SHRINE][..], HealthRegeneration)
    );
    assert!(near(*at, MILITANT_AT), "{at}");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(energy(&app, SHRINE) < before - 70 * ONE, "75 paid");
    assert_eq!(buffs(&app, MILITANT), [HealthRegeneration]);
    assert_eq!(buffs(&app, DISCIPLE), [HealthRegeneration], "Cults infantry too");
    let mut shown = marks(&app);
    shown.sort_unstable_by_key(|&(id, _)| id);
    assert_eq!(shown, [(MILITANT, HealthRegeneration), (DISCIPLE, HealthRegeneration)]);
    let healed = hp(&app, MILITANT).unwrap();
    assert!(healed > 20.0 && healed < 23.0, "{healed}");
    updates(&mut app, 16);
    let after = hp(&app, MILITANT).unwrap();
    assert!(
        (after - healed - 2.0).abs() < 0.2,
        "2 HP a second: {healed} then {after}"
    );
}

/// `factions/cults/structures/shrine.md > Placeholder stats`: "**Energy regeneration** (hotkey W): 50 energy [...]
/// affected Disciples' energy regeneration is **+1 per second for 20 seconds**"; "**A/S/D/F** release one Recruit of a
/// kind, **V: Eject All** (participants leave)". At 60 energy, W awaits its point and a click on the Disciple casts
/// there: the Disciple's energy rises by 1.5 a second, its own half and the buff's one, and the Militant beside it gets
/// nothing (not a Disciple); Q is greyed with 10 left. A lets one Plain participant out of side C, V the others.
#[test]
fn w_boosts_the_disciples_energy_and_a_and_v_let_the_participants_out() {
    use ButtonLook::Disabled;
    let mut app = shrine_game(60, 60.0);
    click(&mut app, SHRINE_AT, &[]);
    tap(&mut app, &[], KeyW);
    let boost = PanelCommand::Cast(EnergyRegeneration);
    assert_eq!(cast::awaited(interface(&app)), Some(EnergyRegeneration));
    click(&mut app, DISCIPLE_AT, &[]);
    assert_eq!(casts(&app).len(), 1);
    assert_eq!(casts(&app)[0].2, EnergyRegeneration);
    assert_eq!(buffs(&app, DISCIPLE), [EnergyRegeneration]);
    assert_eq!(buffs(&app, MILITANT), [], "not a Disciple");
    let (from, since) = (energy(&app, DISCIPLE), tick(&app));
    updates(&mut app, 16);
    let ticks = tick(&app) - since;
    assert_eq!(
        energy(&app, DISCIPLE) - from,
        ticks as u32 * 3 * ONE / 32,
        "1.5 a second"
    );
    let heal = PanelCommand::Cast(HealthRegeneration);
    let c = PanelButton::Command;
    assert_eq!(keys(&mut app)[0], (c(heal), Disabled));
    assert_eq!(keys(&mut app)[1].0, c(boost));

    tap(&mut app, &[], KeyA);
    let release = HoldCommand::ReleaseOne(RecruitKind::Plain);
    assert_eq!(holdings(&app).last(), Some(&(P1, vec![SHRINE], release)));
    until(&mut app, 60, |app| inside(app, SHRINE).len() == 7);
    let out = *RECRUITS
        .iter()
        .find(|&&id| !inside(&app, SHRINE).contains(&id))
        .unwrap();
    app.update();
    assert!(
        pos(&app, out).x <= 1024.0,
        "out of side C, its west edge: {}",
        pos(&app, out)
    );
    tap(&mut app, &[], KeyV);
    assert_eq!(holdings(&app).last(), Some(&(P1, vec![SHRINE], HoldCommand::EjectAll)));
    until(&mut app, 200, |app| inside(app, SHRINE).is_empty());
    let lines = super::temple::lines_of(&mut app, InfoText::Activity);
    assert!(lines.iter().any(|line| line == "Participants: 0 / 8"), "{lines:?}");
}
