//! (M5-N5) The Shrine's panel as plain data (`factions/cults/structures/shrine.md > Placeholder stats`: "**Health
//! regeneration** (hotkey Q)", "**Energy regeneration** (hotkey W)"; "No cooldowns. Targeting: AwaitingTarget,
//! left-click a point in vision. **A/S/D/F** release one Recruit of a kind, **V: Eject All**"): its grid, its casts
//! greyed by the gate's answer, its releases' counts, and who its commands go to. Its clicks are
//! `crate::command_ui::resolution`'s tests; in a game: `crates/game/tests/commands/shrine.rs`.

use sc_sim::{AreaAbility, HoldCommand, RecruitKind};

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::{NOT_ENOUGH_ENERGY_TEXT, unavailable_text};

const SHRINE: ObjectKind = ObjectKind::Shrine;
const HEAL: PanelCommand = PanelCommand::Cast(AreaAbility::HealthRegeneration);
const BOOST: PanelCommand = PanelCommand::Cast(AreaAbility::EnergyRegeneration);

/// The Shrine's DefaultState: Q and W its two casts, labelled with their names' first words and named after the
/// abilities, each awaiting its point (AwaitingTarget), Q greyed here as the gate refuses it for want of energy (N5's
/// acceptance: "too little energy greys the command"), saying so when pressed and pressing nothing; A, S, D and F the
/// releases of one participant of each kind, in the kind buttons' order, with its count, greyed where there are none;
/// V Eject All. No C, no X and no Z: no rally point, no queue.
#[test]
fn the_shrines_panel_casts_releases_and_ejects() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{EjectAll, ReleaseOne};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let shrine = Selection::new([(id(1), SHRINE)]);
    let conditions = Conditions::default()
        .with_pool([6, 2, 0, 0])
        .refusing(HEAL, Reject::NotEnoughEnergy)
        .refusing(ReleaseOne(Metabolism), Reject::NoRecruit(Metabolism))
        .refusing(ReleaseOne(Both), Reject::NoRecruit(Both));
    let shown = panel(&shrine, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (q, w, v) = (slot(0, 0), slot(0, 1), slot(2, 3));
    assert_eq!(
        counted(&shown),
        [
            (q, command(HEAL), Disabled, None),
            (w, command(BOOST), Common, None),
            (slot(1, 0), command(ReleaseOne(Plain)), Common, Some(6)),
            (slot(1, 1), command(ReleaseOne(Constitution)), Common, Some(2)),
            (slot(1, 2), command(ReleaseOne(Metabolism)), Disabled, Some(0)),
            (slot(1, 3), command(ReleaseOne(Both)), Disabled, Some(0)),
            (v, command(EjectAll), Common, None),
        ]
    );
    assert_eq!(InterfaceTemplate::of(SHRINE), Some(InterfaceTemplate::Shrine));
    let labels: Vec<&str> = shown.slots().filter_map(|(_, s)| Some(s?.label())).collect();
    assert_eq!(
        labels,
        ["Health", "Energy", "Plain", "Const.", "Metab.", "Both", "Eject"]
    );
    let names: Vec<String> = shown.slots().filter_map(|(slot, _)| shown.name(slot)).collect();
    assert_eq!(
        names,
        [
            "Health regeneration",
            "Energy regeneration",
            "Release one Plain Recruit",
            "Release one Constitution Recruit",
            "Release one Metabolism Recruit",
            "Release one Both Recruit",
            "Eject All",
        ]
    );
    let why = |at| shown.get(at).and_then(|s| s.why).map(unavailable_text);
    assert_eq!(why(q), Some(NOT_ENOUGH_ENERGY_TEXT));
    assert_eq!(press(&shown, q), SlotPress::Nothing, "greyed");
    assert_eq!(press(&shown, w), SlotPress::AwaitTarget(BOOST));
    assert_eq!(
        press(&shown, slot(1, 0)),
        SlotPress::Hold(ReleaseOne(Plain), HoldCommand::ReleaseOne(Plain))
    );
    assert_eq!(press(&shown, v), SlotPress::Hold(EjectAll, HoldCommand::EjectAll));
    let free = [
        CommandSlot::BACK,
        CommandSlot::RALLY_POINT,
        CommandSlot::CANCEL_PRODUCTION,
    ];
    for free in free.into_iter().chain([slot(0, 2), slot(0, 3)]) {
        assert_eq!(shown.get(free), None, "{free:?}");
    }
    let with_energy = Conditions::default().with_pool([8, 0, 0, 0]);
    let shown = panel(&shrine, InterfaceState::DefaultState, &with_energy);
    assert_eq!(press(&shown, q), SlotPress::AwaitTarget(HEAL), "with the energy");
}

/// "Targeting: AwaitingTarget, left-click a point in vision": a cast awaits its point and is no order: Shift doesn't
/// queue it (`PanelCommand::queues`: a structure has no order queue, plan decision T76; **Placeholder**). "Groupable
/// **false** (user, 2026-11-26; Q0118: each press casts from one Shrine)": two Shrines are two groups, and a cast, a
/// release and an Eject All go to the one Tabbed to, never to both. The Shrine offers no rally point.
#[test]
fn a_cast_awaits_its_point_and_goes_to_the_one_shrine_tabbed_to() {
    use PanelCommand::{EjectAll, ReleaseOne};
    for cast in [HEAL, BOOST] {
        assert!(
            cast.needs_target() && cast.instance_only() && !cast.queues(),
            "{cast:?}"
        );
        assert!(cast.has_sim_command());
        assert!(offers(SHRINE, cast) && !offers(ObjectKind::Temple, cast), "{cast:?}");
    }
    let mut two = Selection::new([(id(1), SHRINE), (id(2), SHRINE)]);
    for shrine in [id(1), id(2)] {
        assert!(two.in_active_group(shrine));
        for command in [HEAL, BOOST, ReleaseOne(RecruitKind::Plain), EjectAll] {
            assert_eq!(recipients(&two, command), [shrine], "{command:?}");
            assert!(!is_common(&two, command), "{command:?}");
        }
        two.cycle(true);
    }
    assert!(!offers(SHRINE, PanelCommand::SetRallyPoint));
    assert!(!offers(SHRINE, PanelCommand::CancelProduction));
}
