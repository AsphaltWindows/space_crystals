//! A right-click in the Recruits' Construct menu and in its placement (design change 0400;
//! `systems/controls/interface.md > CommandPanel > Standard Slot Assignments > Right-click in multi-stage menus`:
//! "right-click **doesn't reset a menu**"). In the menu it is the Recruits' normal right-click and the menu stays open.
//! In placement, "the next step is a **left-click on a location**", it goes "**straight back to DefaultState**". Escape
//! and Z "still go back one step".

use space_crystals::command_ui::Placing;

use super::*;
use KeyCode::{Escape, KeyA};

/// Bare ground north-east of the Recruits, in the world view.
const GROUND: Vec2 = Vec2::new(1400.0, 800.0);
/// Where the Armory's ghost is hovered: the Armory site of [`super::placing`].
const ARMORY_SITE: TilePos = TilePos::new(20, 9);

/// 0400's Verification 1: "Select Recruits, open Basic Construct and right-click ground: the Recruits move and the
/// build menu stays open." W opens Basic Construct (design change 0398). The pointer previews the right-click as in
/// DefaultState, Move (**reading**: the pointer rules list no menu, `space_crystals::command_ui::pointer_display`). The
/// right-click gives the four Recruits their Move there, one order, and the menu stays open while they walk.
#[test]
fn a_right_click_in_basic_construct_moves_the_recruits_and_keeps_the_menu_open() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyW);
    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    assert_eq!(interface(&app), menu);
    assert_eq!(pointer_over(&mut app, GROUND), Some(PointerDisplay::Move));
    right_click(&mut app, GROUND, &[]);
    assert_eq!(interface(&app), menu, "the menu stays open");
    let given = orders(&app);
    assert!(
        matches!(&given[..], [order] if order.units == RECRUITS && near(point_of(order.kind), GROUND)),
        "{given:?}"
    );
    let before = pos(&app, RECRUITS[0]).distance(GROUND);
    updates(&mut app, 30);
    assert!(pos(&app, RECRUITS[0]).distance(GROUND) < before - 32.0, "on its way");
    assert_eq!(interface(&app), menu, "still open");
}

/// 0400's Verification 2: "Pick a building and right-click during placement: the panel is back at DefaultState." The
/// Armory's placement, entered with W then A, its ghost over the ground: the right-click orders nothing, builds
/// nothing and leaves the placement for DefaultState, not for the menu.
#[test]
fn a_right_click_in_placement_goes_straight_back_to_default_state() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyW);
    tap(&mut app, &[], KeyA);
    let placing = InterfaceState::AwaitingPlacement(Placing::recruits(ARMORY_KIND));
    assert_eq!(interface(&app), placing);
    pointer_over(&mut app, middle(ARMORY_SITE, (3, 2)));
    assert!(ghost(&app).is_some());
    right_click(&mut app, middle(ARMORY_SITE, (3, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(
        (orders(&app), builds(&app)),
        (Vec::new(), Vec::new()),
        "the right-click ordered nothing"
    );
    assert_eq!(ghost(&app), None);
    assert_eq!(crystals(&app), 300, "nothing paid");
}

/// 0400's Verification 3: "Escape during placement goes back to the build menu." So does Z (Back), and from the
/// menu, Escape goes on back to DefaultState.
#[test]
fn escape_or_z_in_placement_goes_back_to_basic_construct() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    let placing = InterfaceState::AwaitingPlacement(Placing::recruits(ARMORY_KIND));
    tap(&mut app, &[], KeyW);
    for back in [Escape, KeyZ] {
        tap(&mut app, &[], KeyA);
        assert_eq!(interface(&app), placing);
        pointer_over(&mut app, middle(ARMORY_SITE, (3, 2)));
        tap(&mut app, &[], back);
        assert_eq!(interface(&app), menu, "{back:?}");
        assert_eq!(ghost(&app), None);
    }
    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), []);
}
