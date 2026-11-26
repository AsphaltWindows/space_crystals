//! The Recruit's panel and its two Construct menus in the client (slice M5-J1; design changes 0322, 0327 and 0398): its
//! letters, W and E opening the menus and A attacking; each menu's buildings in their fixed slots, greyed until they
//! can be built.

use super::*;
use KeyCode::{Escape, KeyE};

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands` (design change 0398):
/// "**Q** Move, **W** Basic Construct, **E** Advanced Construct, **A** Attack, **S** Assist, **D** Repair, **X** Stop",
/// each hint naming it in full. 0398's Verification: "Recruits: W opens Basic Construct, E opens Advanced Construct, A
/// attacks": W and E open the two menus, and A, "as BasicCombatUnitInterfaceState's Attack", awaits its target, a click
/// on an enemy Peacekeeper giving all four an Attack on it. D (slice M5-N8) awaits its target, and gives no order until
/// it has one.
#[test]
fn the_recruits_panel_shows_every_letter() {
    use ButtonLook::Common;
    let mut app = recruit_game(0);
    let enemy = spawn(&mut app, ObjectKind::Peacekeeper, Some(P2), Vec2::new(1000.0, 800.0));
    select_units(&mut app, &RECRUITS);
    let cmd = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), cmd(PanelCommand::Move), Common, "Move"),
            (w(), cmd(PanelCommand::BuildMenu(BuildMenu::Basic)), Common, "Basic"),
            (e(), cmd(PanelCommand::BuildMenu(BuildMenu::Advanced)), Common, "Adv."),
            (a(), cmd(PanelCommand::Attack), Common, "Attack"),
            (s(), cmd(PanelCommand::Assist), Common, "Assist"),
            (d(), cmd(PanelCommand::Repair), Common, "Repair"),
            (x(), cmd(PanelCommand::Stop), Common, "Stop"),
        ]
    );
    let hints = [
        (q(), "Q: Move"),
        (w(), "W: Basic Construct"),
        (e(), "E: Advanced Construct"),
        (a(), "A: Attack"),
        (s(), "S: Assist Construction"),
        (d(), "D: Repair"),
        (x(), "X: Stop"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    for (key, menu) in [(KeyW, BuildMenu::Basic), (KeyE, BuildMenu::Advanced)] {
        tap(&mut app, &[], key);
        assert_eq!(interface(&app), InterfaceState::BuildMenu(menu), "{key:?}");
        tap(&mut app, &[], KeyZ);
    }
    tap(&mut app, &[], KeyD);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Repair));
    assert_eq!(orders(&app), []);
    tap(&mut app, &[], Escape);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    let at = pos(&app, enemy);
    click(&mut app, at, &[]);
    let attack = OrderKind::Attack(ObjectId(enemy));
    assert_eq!(orders(&app), [given(&RECRUITS, attack, false)]);
}

/// Design change 0327 (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`,
/// BuildMenu[Basic] and BuildMenu[Advanced]: "Fixed slots: every building keeps its key and is greyed until its
/// TechPrerequisite is met, so no hotkey ever moves"; "**Z** returns to DefaultState"). With 250 Space Crystals: W
/// (design change 0398) opens the Basic menu, Q Recruitment Hall greyed for the bank, W Storage, A Armory (a Storage
/// stands), S Gunworks (an Armory stands; slice M5-G2), D Garrison (an Armory stands; slice M5-N2) and X Altar (a
/// Storage stands; slice M5-N1b), whose placement X enters and Z leaves for the menu; each Build's hint names its
/// price. Z goes back; E opens the Advanced menu: S Machine Shop and F Scrap Hangar (slice M5-G2), X Temple (slice
/// M5-N3), C Cathedral (slice M5-N4a) and V Shrine (slice M5-N5) greyed by the gate for their TechPrerequisites, a Chop
/// Shop, an Airstrip, an Altar and a Temple, which they name when pressed; A Chop Shop and D Airstrip greyed as not in
/// the game yet (0327's Verification: "W then A places a Chop Shop (greyed without an Armory)", E then A since 0398:
/// its Build comes with its slice).
#[test]
fn the_construct_menus_keep_their_slots_greying_what_cant_be_built() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::Build;
    let mut app = recruit_game(250);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    let cmd = PanelButton::Command;
    let planned = |entry| cmd(PanelCommand::Planned(entry));
    assert_eq!(
        buttons(&mut app),
        [
            (q(), cmd(Build(HALL_KIND)), Disabled, "RH"),
            (w(), cmd(Build(STORAGE_KIND)), Common, "SE"),
            (a(), cmd(Build(ARMORY_KIND)), Common, "AY"),
            (s(), cmd(Build(ObjectKind::Gunworks)), Common, "GS"),
            (d(), cmd(Build(ObjectKind::Garrison)), Common, "GA"),
            (z(), PanelButton::Back, Back, "Back"),
            (x(), cmd(Build(ObjectKind::Altar)), Common, "AR"),
        ]
    );
    let hints = [
        (q(), "Q: Build Recruitment Hall (300 Space Crystals)"),
        (w(), "W: Build Storage (100 Space Crystals)"),
        (a(), "A: Build Armory (200 Space Crystals)"),
        (s(), "S: Build Gunworks (150 Space Crystals)"),
        (d(), "D: Build Garrison (150 Space Crystals)"),
        (x(), "X: Build Altar (150 Space Crystals)"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyX);
    let placing = interface(&app).placing().map(|placing| placing.kind);
    assert_eq!(placing, Some(ObjectKind::Altar));
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Basic));
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    tap(&mut app, &[], KeyE);
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    let entries = [
        (a(), planned(Planned::ChopShop), Unavailable, "CS"),
        (s(), cmd(Build(ObjectKind::MachineShop)), Disabled, "MS"),
        (d(), planned(Planned::Airstrip), Unavailable, "AP"),
        (f(), cmd(Build(ObjectKind::ScrapHangar)), Disabled, "SH"),
        (x(), cmd(Build(ObjectKind::Temple)), Disabled, "TE"),
        (c(), cmd(Build(ObjectKind::Cathedral)), Disabled, "CL"),
        (v(), cmd(Build(ObjectKind::Shrine)), Disabled, "SR"),
    ];
    let mut expected = entries.to_vec();
    expected.insert(4, (z(), PanelButton::Back, Back, "Back"));
    assert_eq!(buttons(&mut app), expected);
    assert_eq!(hint_over(&mut app, a()).as_deref(), Some("A: Build Chop Shop"));
    let hints = [
        (s(), "S: Build Machine Shop (150 Space Crystals)"),
        (f(), "F: Build Scrap Hangar (200 Space Crystals)"),
        (x(), "X: Build Temple (200 Space Crystals)"),
        (c(), "C: Build Cathedral (250 Space Crystals)"),
        (v(), "V: Build Shrine (200 Space Crystals)"),
    ];
    for (at, hint) in hints {
        assert_eq!(hint_over(&mut app, at).as_deref(), Some(hint));
    }
    tap(&mut app, &[], KeyA);
    assert_eq!(said(&app), Some(NOT_YET_TEXT));
    tap(&mut app, &[], KeyS);
    assert_eq!(said(&app), Some("Requires a Chop Shop"));
    tap(&mut app, &[], KeyF);
    assert_eq!(said(&app), Some("Requires an Airstrip"));
    tap(&mut app, &[], KeyX);
    assert_eq!(said(&app), Some("Requires an Altar"));
    tap(&mut app, &[], KeyC);
    assert_eq!(said(&app), Some("Requires a Temple"));
    tap(&mut app, &[], KeyV);
    assert_eq!(said(&app), Some("Requires a Temple"));
    assert_eq!(interface(&app), InterfaceState::BuildMenu(BuildMenu::Advanced));
    assert_eq!(orders(&app), []);
}
