//! The Prayer-mobile in the client (slice M5-N11; `factions/cults/units/prayer_mobile.md > Placeholder stats`:
//! "**Inspire** (hotkey **R**, instant, no target): 75 energy; every own infantry unit within **4 grid units** of the
//! vehicle [...]", "**Suppress** (hotkey **F**, instant, no target): 100 energy; an arc of **90° total** in front of
//! the vehicle's current facing, **5 grid units** long"): R casts Inspire at once, from where the vehicle stands, and
//! the Recruits round it show Inspire's mark and the cast its ring; F casts Suppress ahead of it, the units in its arc,
//! the enemy's and its own alike, show Suppress's mark and the cast its wedge; with too little energy left, each is
//! greyed and says so. The panel as plain data is the unit tests' (`command_ui/interface/tests/prayer_mobile.rs`), the
//! casts themselves the sim's (`crates/sim/tests/prayer_mobile`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{AreaAbility, Faction, ObjectKind, PlayerId, SpawnAt};
use space_crystals::combat_effects::{CombatEffects, shows_cast};
use space_crystals::command_ui::feedback::NOT_ENOUGH_ENERGY_TEXT;
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::fog::Sight;

use super::armory::{buttons, tick, until};
use super::shrine::{ONE, buffs, casts, energy, marks, set_energy};
use super::temple::said;
use super::*;
use AreaAbility::{Inspire, Suppress};
use KeyCode::{KeyF, KeyR};

const P1: PlayerId = PlayerId(1);

/// The objects of [`prayer_mobile_game`], by id, and where they stand (the map is flat). The Prayer-mobile faces east,
/// as every unit spawned does.
const PRAYER_MOBILE: u32 = 1;
const PRAYER_MOBILE_AT: Vec2 = Vec2::new(1400.0, 900.0);
/// Player 1's Recruits: two within 4 grid units of it (100 su off, behind it and beside it), one 350 su off.
const NEAR_RECRUITS: [u32; 2] = [2, 3];
const FAR_RECRUIT: u32 = 4;
/// Player 2's Militants: two in the arc ahead (200 su east; 269 su off, 22° to the right), one 200 su to its left, 90°
/// off its facing.
const AHEAD: [u32; 2] = [5, 6];
const ASIDE: u32 = 7;
/// One of player 1's Militants in the arc, 283 su ahead, 8° to the left: Suppress takes the own units in too (and
/// Inspire, within 256 su, doesn't reach it).
const OWN_AHEAD: u32 = 8;

/// A game of player 1, the Cults, against player 2, the Cults, on a flat 64 × 48 map, set up as [`game`] does (no
/// fight: auto-targeting is off), with the Prayer-mobile's energy set to `points`.
fn prayer_mobile_game(points: u32) -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let p2 = PlayerId(2);
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(p2, Faction::Cults)
        .spawn(
            ObjectKind::PrayerMobile,
            Some(P1),
            point(PRAYER_MOBILE_AT.x, PRAYER_MOBILE_AT.y),
        )
        .spawn(ObjectKind::Recruit, Some(P1), point(1300.0, 900.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(1400.0, 1000.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(1400.0, 1250.0))
        .spawn(ObjectKind::Militant, Some(p2), point(1600.0, 900.0))
        .spawn(ObjectKind::Militant, Some(p2), point(1650.0, 1000.0))
        .spawn(ObjectKind::Militant, Some(p2), point(1400.0, 700.0))
        .spawn(ObjectKind::Militant, Some(P1), point(1680.0, 860.0));
    let mut app = game_of(builder.build());
    set_energy(&mut app, PRAYER_MOBILE, points);
    app
}

/// The casts drawn now: each one's ability, point and facing, while it shows to player 1.
fn drawn_casts(app: &App) -> Vec<(AreaAbility, Vec2, f32)> {
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, P1, false);
    let effects = app.world().resource::<CombatEffects>();
    (effects.casts.iter())
        .filter(|cast| shows_cast(sim, &sight, cast, &space_crystals::tuning::ClientTuning::BUILTIN))
        .map(|cast| (cast.ability, cast.at, cast.facing))
        .collect()
}

/// The R and F keys as shown: their buttons and looks.
fn r_and_f(app: &mut App) -> [(PanelButton, ButtonLook); 2] {
    let shown = buttons(app);
    [slot(0, 3), slot(1, 3)].map(|at| {
        let (_, button, look, _) = *shown.iter().find(|b| b.0 == at).expect("shown");
        (button, look)
    })
}

/// The plan's acceptance (slice M5-N11): "Inspire: a ring of 4 GU at the cast, and a mark on the units inspired;
/// Suppress: a 90° wedge of 5 GU at the cast, and a mark on the units suppressed". At 200 energy, selected, the
/// Prayer-mobile's R and F are lit. R sends one cast of Inspire from the one vehicle, at its own point, at once: it
/// pays 75, and its two Recruits within 4 grid units get the buff, each marked beside it, and the cast's ring shows
/// where it stood for a second; the Recruit 350 su off gets nothing. F sends Suppress the same way: it pays 100, and
/// the three Militants in its arc, the enemy's two and player 1's one, are suppressed and marked, the one at its side
/// isn't, nor are the Recruits behind it; its wedge shows facing east. With 25 energy left both keys are greyed and
/// say "Not enough energy", sending nothing.
#[test]
fn r_inspires_the_recruits_round_it_and_f_suppresses_the_units_ahead_of_it() {
    use ButtonLook::{Common, Disabled};
    let mut app = prayer_mobile_game(200);
    click(&mut app, PRAYER_MOBILE_AT, &[]);
    assert_eq!(selected(&app), [PRAYER_MOBILE]);
    let c = PanelButton::Command;
    let (inspire, suppress) = (PanelCommand::Cast(Inspire), PanelCommand::Cast(Suppress));
    assert_eq!(r_and_f(&mut app), [(c(inspire), Common), (c(suppress), Common)]);

    tap(&mut app, &[], KeyR);
    let sent = casts(&app);
    let [(player, casters, ability, at)] = &sent[..] else {
        panic!("one cast: {sent:?}")
    };
    assert_eq!((*player, &casters[..], *ability), (P1, &[PRAYER_MOBILE][..], Inspire));
    assert!(near(*at, PRAYER_MOBILE_AT), "from where it stands: {at}");
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no target awaited");
    updates(&mut app, 1);
    assert_eq!(energy(&app, PRAYER_MOBILE) / ONE, 125, "75 paid");
    for recruit in NEAR_RECRUITS {
        assert_eq!(buffs(&app, recruit), [Inspire], "{recruit}");
    }
    assert_eq!(buffs(&app, FAR_RECRUIT), [], "350 su off");
    let mut shown = marks(&app);
    shown.sort_unstable_by_key(|&(id, _)| id);
    assert_eq!(shown, NEAR_RECRUITS.map(|id| (id, Inspire)));
    let rings = drawn_casts(&app);
    assert!(
        matches!(rings[..], [(Inspire, at, _)] if near(at, PRAYER_MOBILE_AT)),
        "{rings:?}"
    );

    tap(&mut app, &[], KeyF);
    let sent = casts(&app);
    assert_eq!(sent.len(), 2);
    assert_eq!((sent[1].1.as_slice(), sent[1].2), (&[PRAYER_MOBILE][..], Suppress));
    updates(&mut app, 1);
    assert_eq!(energy(&app, PRAYER_MOBILE) / ONE, 25, "100 paid");
    for militant in AHEAD.into_iter().chain([OWN_AHEAD]) {
        assert_eq!(buffs(&app, militant), [Suppress], "{militant}");
    }
    assert_eq!(buffs(&app, ASIDE), [], "90° off its facing");
    assert_eq!(buffs(&app, NEAR_RECRUITS[0]), [Inspire], "behind it");
    let wedges: Vec<_> = (drawn_casts(&app).into_iter())
        .filter(|cast| cast.0 == Suppress)
        .collect();
    assert!(
        matches!(wedges[..], [(Suppress, at, facing)] if near(at, PRAYER_MOBILE_AT) && facing.abs() < 1e-3),
        "{wedges:?}"
    );
    let mut suppressed: Vec<u32> = (marks(&app).into_iter())
        .filter(|mark| mark.1 == Suppress)
        .map(|mark| mark.0)
        .collect();
    suppressed.sort_unstable();
    assert_eq!(suppressed, [AHEAD[0], AHEAD[1], OWN_AHEAD]);

    assert_eq!(r_and_f(&mut app), [(c(inspire), Disabled), (c(suppress), Disabled)]);
    tap(&mut app, &[], KeyR);
    assert_eq!(said(&app), Some(NOT_ENOUGH_ENERGY_TEXT));
    tap(&mut app, &[], KeyF);
    assert_eq!(said(&app), Some(NOT_ENOUGH_ENERGY_TEXT));
    assert_eq!(casts(&app).len(), 2, "nothing more sent");
    let since = tick(&app);
    until(&mut app, 40, |app| {
        tick(app) > since + space_crystals::tuning::feedback::BUILTIN.cast_ticks
    });
    assert_eq!(drawn_casts(&app), [], "faded");
}
