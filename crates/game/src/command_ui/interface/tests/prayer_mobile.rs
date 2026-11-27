//! (M5-N11) The Prayer-mobile's panel as plain data (`factions/cults/units/prayer_mobile.md > Placeholder stats`:
//! "ObjectInterfaceState: BasicCombatUnitInterfaceState without attack commands, plus **R: Inspire** and **F:
//! Suppress**"; "**Inspire** (hotkey **R**, instant, no target): 75 energy", "**Suppress** (hotkey **F**, instant, no
//! target): 100 energy"; "Energy: MaxEnergy 200; starts at 50"): its grid, its casts greyed by the gate's answer until
//! it has their energy, and who they go to. In a game: `crates/game/tests/commands/prayer_mobile.rs`.

use bevy::math::Vec2;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{AreaAbility, Faction, PlayerId, SpawnAt};

use super::*;
use crate::command_ui::feedback::{NOT_ENOUGH_ENERGY_TEXT, unavailable_text};

const PRAYER_MOBILE: ObjectKind = ObjectKind::PrayerMobile;
const INSPIRE: PanelCommand = PanelCommand::Cast(AreaAbility::Inspire);
const SUPPRESS: PanelCommand = PanelCommand::Cast(AreaAbility::Suppress);

/// Read from a game with two Prayer-mobiles. Q Move, W Reverse (it CanReverse, as the Seeker it drives as), E Hold
/// Position, S Patrol and X Stop; no A, as no selected unit can attack (design change 0399; "AttackAttributes - None"),
/// and no D, not even unavailable as the Disciple's is ("without attack commands": `PanelTemplate::without_attacks`).
/// R is Inspire and F Suppress, labelled "Insp." and "Suppr." and named after the abilities, each cast at once (no
/// target: `SlotPress::Cast`, from where the vehicle stands). With the 50 energy it starts with both are greyed, "Not
/// enough energy", and press nothing; 50 s in (75 energy) R casts and F is still greyed; 100 s in (100 energy) F casts
/// too.
#[test]
fn its_panel_has_no_attack_commands_and_casts_inspire_with_r_and_suppress_with_f() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{HoldPosition, Move, Patrol, Reverse, Stop};
    let player = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(16); 16].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(PRAYER_MOBILE, Some(player), SpawnAt::Point(Vec2::new(300.0, 300.0)))
        .spawn(PRAYER_MOBILE, Some(player), SpawnAt::Point(Vec2::new(700.0, 300.0)))
        .start();
    let one = Selection::new([(id(1), PRAYER_MOBILE)]);
    let shown_in = |t: &sc_sim::testing::TestSim| {
        let conditions = Conditions::of(t.sim(), player, &one);
        panel(&one, InterfaceState::DefaultState, &conditions)
    };
    let c = PanelButton::Command;
    let (r, f) = (slot(0, 3), slot(1, 3));
    let start = shown_in(&t);
    assert_eq!(
        shown(&start),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Common),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(INSPIRE), Disabled),
            (slot(1, 1), c(Patrol), Common),
            (f, c(SUPPRESS), Disabled),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(
        InterfaceTemplate::of(PRAYER_MOBILE),
        Some(InterfaceTemplate::BasicCombatUnit)
    );
    let d = slot_index(slot(1, 2));
    assert_eq!(grid(PRAYER_MOBILE)[d], None, "no D");
    assert_eq!(
        grid(ObjectKind::Disciple)[d],
        Some(PanelCommand::AttackGround),
        "the Disciple's"
    );
    assert!(!offers(PRAYER_MOBILE, PanelCommand::AttackGround));
    let labels = [r, f].map(|at| start.get(at).unwrap().label());
    assert_eq!(labels, ["Insp.", "Suppr."]);
    assert_eq!(
        [r, f].map(|at| start.name(at)),
        [Some("Inspire".into()), Some("Suppress".into())]
    );
    let why = |panel: &Panel, at| panel.get(at).and_then(|shown| shown.why).map(unavailable_text);
    assert_eq!([why(&start, r), why(&start, f)], [Some(NOT_ENOUGH_ENERGY_TEXT); 2]);
    assert_eq!([press(&start, r), press(&start, f)], [SlotPress::Nothing; 2], "greyed");

    t.run_until(800);
    let inspired = shown_in(&t);
    let looks = [r, f].map(|at| inspired.get(at).map(|shown| shown.look));
    assert_eq!(looks, [Some(Common), Some(Disabled)], "75 energy");
    assert_eq!(press(&inspired, r), SlotPress::Cast(INSPIRE, AreaAbility::Inspire));
    assert_eq!(why(&inspired, f), Some(NOT_ENOUGH_ENERGY_TEXT));
    t.run_until(1600);
    let both = shown_in(&t);
    assert_eq!(why(&both, f), None, "100 energy");
    assert_eq!(press(&both, f), SlotPress::Cast(SUPPRESS, AreaAbility::Suppress));
}

/// "instant, no target": a cast is issued at once, and is no order: Shift doesn't queue it (`PanelCommand::queues`, as
/// the Shrine's: **Placeholder**). "**Groupable false** (one cast spends one vehicle's energy, as for the Disciple)":
/// two Prayer-mobiles are two groups, and a cast goes to the one Tabbed to, never to both; a Move goes to both.
#[test]
fn a_cast_is_issued_at_once_from_the_one_prayer_mobile_tabbed_to() {
    for cast in [INSPIRE, SUPPRESS] {
        assert!(
            !cast.needs_target() && cast.instance_only() && !cast.queues(),
            "{cast:?}"
        );
        assert!(cast.has_sim_command());
        assert!(
            offers(PRAYER_MOBILE, cast) && !offers(ObjectKind::Shrine, cast),
            "{cast:?}"
        );
    }
    let heal = PanelCommand::Cast(AreaAbility::HealthRegeneration);
    assert!(heal.needs_target(), "the Shrine's, at a point");
    let mut two = Selection::new([(id(1), PRAYER_MOBILE), (id(2), PRAYER_MOBILE)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    assert_eq!(recipients(&two, PanelCommand::Move), [id(1), id(2)]);
    for vehicle in [id(1), id(2)] {
        for cast in [INSPIRE, SUPPRESS] {
            assert_eq!(recipients(&two, cast), [vehicle], "{cast:?}");
            assert!(!is_common(&two, cast), "{cast:?}");
        }
        two.cycle(true);
    }
}
