//! A Tunnel's expansions in the client (slice M4-F; design changes 0254, 0257, 0290 and 0291): W opens the ExpandMenu,
//! nine expansion slots and then the Laser Turret; an entry's placement, its ghost held within the Tunnel Area while
//! the area shows, green over a standing unit and red over a crystal patch; a click that builds, one the bank can't
//! pay; Z back to the menu; the Tunnel's operation in its InfoPanel, its menu greyed meanwhile, the site's X alone with
//! its progress bar, and the Tunnel's X, which cancels the operation.

use sc_sim::nav::TileRect;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{BuildingOrientation, Faction, PlayerCommand as Sent, SU_PER_GU, TunnelCommand};
use space_crystals::command_ui::feedback::{
    NO_CRYSTALS_TEXT, NOT_YET_TEXT, OPERATION_UNDER_WAY_TEXT, TargetFeedback, placement_text, requires_text,
};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand, Placing, Planned};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::object_render::SiteProgress;
use space_crystals::placement::{PlacementGhost, TunnelAreaOverlay};

use super::*;
use KeyCode::{Escape, KeyC, KeyQ, KeyR, KeyW, KeyX, KeyZ};

const P1: PlayerId = PlayerId(1);
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const OPTICS_LAB: ObjectKind = ObjectKind::OpticsLab;
const CHEMICAL_PLANT: ObjectKind = ObjectKind::ChemicalPlant;
const TURRET: ObjectKind = ObjectKind::LaserTurret;

/// The objects of [`expansion_game`], by id, and where they are (the map is flat: where they are drawn).
const TUNNEL: u32 = 1;
const GUARD: u32 = 2;
const TUNNEL_AT: Vec2 = Vec2::new(1152.0, 1088.0); // tiles (16, 15) to (19, 18)
/// The Tunnel's Tier 1 area.
const AREA: TileRect = TileRect {
    origin: TilePos::new(14, 13),
    w: 8,
    h: 8,
};
/// A Hideout's site in the area's north-west corner, where the Guard stands.
const UNDER_GUARD: TilePos = TilePos::new(14, 13);
/// A free site in the area, west of the Tunnel.
const FREE: TilePos = TilePos::new(14, 15);
/// The crystal patch in the area's north-east corner.
const PATCH_AT: TilePos = TilePos::new(20, 13);
/// The ground east of the area, in the world view: the ghost stays at the area's edge, on the patch.
const EAST: Vec2 = Vec2::new(2000.0, 900.0);

/// A game of player 1, the Syndicate, on a flat 64 × 48 map, as [`game`] sets it up (no fog), with 1000 Space
/// Crystals: their Tunnel, its Tier 1 area (14, 13) to (21, 20), a Guard of theirs standing in its north-west corner
/// and a crystal patch in its north-east corner.
fn expansion_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(16, 15))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(centre(UNDER_GUARD)))
        .spawn(ObjectKind::CrystalPatch, None, tile(PATCH_AT.x, PATCH_AT.y));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    app
}

/// The centre of a Hideout's site at `origin` (2 × 2 tiles), where the cursor stands over its ghost.
fn centre(origin: TilePos) -> Vec2 {
    (Vec2::new(origin.x as f32, origin.y as f32) + 1.0) * SU_PER_GU
}

/// The buttons the HUD shows: slot, button and look.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = (query.iter(app.world()))
        .map(|b| (b.slot, b.shown.button, b.shown.look))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// Player 1's Space Crystals.
fn crystals(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").crystals
}

/// Every Tunnel command given so far: its Tunnel and the command.
fn tunnel_commands(app: &App) -> Vec<(u32, TunnelCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    let tunnel = |sent: &Sent| match &sent.command {
        Command::Tunnel { tunnel, command } => Some((tunnel.0, *command)),
        _ => None,
    };
    commands.filter_map(tunnel).collect()
}

/// The Expand of a Hideout at `origin`, unturned.
fn expand_at(origin: TilePos) -> TunnelCommand {
    TunnelCommand::Expand {
        kind: HIDEOUT,
        origin,
        orientation: BuildingOrientation::default(),
    }
}

/// The ghost that shows, if one does.
fn ghost(app: &App) -> Option<space_crystals::placement::Ghost> {
    app.world().resource::<PlacementGhost>().shown.clone()
}

/// The InfoPanel's activity area, while it shows.
fn activity(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    texts
        .iter(app.world())
        .find(|(which, _, node)| **which == InfoText::Activity && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
}

/// Player 1's site, if one stands.
fn site_of(app: &App) -> Option<u32> {
    let sim = app.world().resource::<SimRes>();
    let site = sim.objects().find(|o| o.is_site() && o.owner == Some(P1));
    site.map(|o| o.id.0)
}

/// The share of its construction done that site `id`'s progress bar shows.
fn site_progress(app: &mut App, id: u32) -> f32 {
    let mut query = app.world_mut().query::<&SiteProgress>();
    let shown: Vec<f32> = (query.iter(app.world()))
        .filter(|fill| fill.id == ObjectId(id))
        .map(|fill| fill.shown)
        .collect();
    assert_eq!(shown.len(), 1, "one progress bar");
    shown[0]
}

/// The Tunnel selected with a click on it, and its ExpandMenu opened (W).
fn open_menu(app: &mut App) {
    click(app, TUNNEL_AT, &[]);
    assert_eq!(selected(app), [TUNNEL]);
    tap(app, &[], KeyW);
    assert_eq!(interface(app), InterfaceState::ExpandMenu);
}

/// The Hideout's placement, as Q in the ExpandMenu enters it.
fn placing_hideout() -> InterfaceState {
    InterfaceState::AwaitingPlacement(Placing::expansion(ObjectId(TUNNEL), HIDEOUT))
}

/// Design change 0291's verification, "The ExpandMenu shows nine expansion slots and then the turret": W opens it
/// (design change 0257), the Hideout in Q, the Optics Lab in W and the Chemical Plant in E (slice M4-K), the six
/// expansions not in the game yet greyed in R to X, Z being Back, and the Laser Turret in C (slice M4-S2), greyed
/// by its gate with its prerequisite: this player owns no Optics Lab. A greyed entry says why and leaves the menu open; Z goes back to DefaultState, and so do Escape and a
/// right-click.
#[test]
fn w_opens_the_expand_menu_nine_expansions_then_the_turret() {
    use ButtonLook::{Back, Common, Unavailable};
    use PanelCommand::{Expand, Planned as P};
    use Planned::*;
    let mut app = expansion_game();
    open_menu(&mut app);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Expand(HIDEOUT)), Common),
            (slot(0, 1), c(Expand(OPTICS_LAB)), Common),
            (slot(0, 2), c(Expand(CHEMICAL_PLANT)), Common),
            (slot(0, 3), c(P(Vault)), Unavailable),
            (slot(1, 0), c(P(VehicleAssembly)), Unavailable),
            (slot(1, 1), c(P(Forge)), Unavailable),
            (slot(1, 2), c(P(NullChamber)), Unavailable),
            (slot(1, 3), c(P(AerospaceAssembly)), Unavailable),
            (CommandSlot::BACK, PanelButton::Back, Back),
            (slot(2, 1), c(P(OrbitalCommand)), Unavailable),
            (slot(2, 2), c(Expand(TURRET)), Unavailable),
        ]
    );
    let optics_lab = requires_text(OPTICS_LAB);
    for (key, says) in [(KeyR, NOT_YET_TEXT), (KeyC, optics_lab)] {
        tap(&mut app, &[], key);
        assert_eq!(said(&app), Some(says), "{key:?}");
        assert_eq!(interface(&app), InterfaceState::ExpandMenu);
    }
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyW);
    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyW);
    right_click(&mut app, EAST, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(selected(&app), [TUNNEL]);
    assert_eq!(tunnel_commands(&app), []);
}

/// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`: "A ghost preview of the expansion follows the
/// cursor within the Tunnel Area, snapped to the grid", while the area shows (`factions/syndicate/tunnels.md > Tunnel
/// Area display`: "**Placing an expansion** (from a selected Tunnel) shows that Tunnel's area too"; design change 0300,
/// answering Q0086 point 4: "as built"). Design change 0290's verification, "Place a Hideout under a standing Guard: it
/// is accepted": the ghost is green over the Guard. East of the area the ghost stays at its edge, on the crystal patch:
/// red, and a click there says why and sends nothing. Z goes back to the ExpandMenu ("**Z**: returns to ExpandMenu"), Q
/// places again, and a click over the Guard builds the Hideout there (one `TunnelCommand::Expand`) and returns to
/// DefaultState: paid at once (design change 0254), the Guard where it stood.
#[test]
fn the_ghost_stays_in_the_area_and_a_click_builds_the_hideout() {
    let mut app = expansion_game();
    open_menu(&mut app);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), placing_hideout());
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    assert!(overlay.visible, "the area shows");
    let areas: Vec<TileRect> = overlay.selected.iter().map(|shown| shown.area).collect();
    assert_eq!(
        (areas, overlay.rings.len()),
        (vec![AREA], 0),
        "the Tunnel's area, no ring"
    );

    pointer_over(&mut app, centre(UNDER_GUARD));
    let over_guard = ghost(&app).expect("a ghost");
    assert_eq!(over_guard.origin, UNDER_GUARD);
    assert!(over_guard.is_valid(), "a unit doesn't block it: {:?}", over_guard.check);

    pointer_over(&mut app, EAST);
    let at_edge = ghost(&app).expect("a ghost");
    assert_eq!(at_edge.origin, PATCH_AT, "held at the area's east edge");
    assert!(!at_edge.is_valid(), "the patch");
    let why = placement_text(at_edge.check.first_problem());
    click(&mut app, EAST, &[]);
    assert_eq!(said(&app), Some(why));
    assert_eq!(tunnel_commands(&app), []);
    assert_eq!(interface(&app), placing_hideout(), "placement goes on");

    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::ExpandMenu);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), placing_hideout());
    let guard_at = pos(&app, GUARD);
    click(&mut app, centre(UNDER_GUARD), &[]);
    assert_eq!(tunnel_commands(&app), [(TUNNEL, expand_at(UNDER_GUARD))]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let site = site_of(&app).expect("the Hideout's site");
    let sim = app.world().resource::<SimRes>();
    let placement = sim.object(ObjectId(site)).and_then(|o| o.placement);
    assert_eq!(placement.map(|p| (p.origin, p.w, p.h)), Some((UNDER_GUARD, 2, 2)));
    assert_eq!(crystals(&app), 1000 - 200);
    assert_eq!(pos(&app, GUARD), guard_at, "the Guard stands where it stood");
}

/// Design change 0254: "**Paid when the ghost is placed**": a click once the bank no longer pays (the crystals spent
/// since Q was pressed) says so, sends nothing, and placement goes on; with too few crystals, Q is greyed and says so.
#[test]
fn an_expansion_the_bank_cant_pay_says_so() {
    let mut app = expansion_game();
    open_menu(&mut app);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), placing_hideout());
    set_crystals(&mut app, 199);
    click(&mut app, centre(FREE), &[]);
    assert_eq!(said(&app), Some(NO_CRYSTALS_TEXT));
    assert_eq!(tunnel_commands(&app), []);
    assert_eq!(interface(&app), placing_hideout());

    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::ExpandMenu);
    let q = buttons(&mut app).into_iter().find(|(shown, ..)| *shown == slot(0, 0));
    assert_eq!(q.map(|(.., look)| look), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(NO_CRYSTALS_TEXT));
    assert_eq!(interface(&app), InterfaceState::ExpandMenu);
}

/// `factions/syndicate/tunnels.md > Expansions under construction` (design change 0254): while the Tunnel builds a
/// Hideout, "**The Tunnel meanwhile** shows the progress" (its InfoPanel's activity), its ExpandMenu's entries are
/// greyed ("A Tunnel can only perform one operation at a time") and its X is no longer greyed. The site "offers only
/// **X Cancel Construction**" and shows its progress bar. "The Tunnel's X cancels whatever operation it is running":
/// the site goes, "refunding **75%** of its cost".
#[test]
fn the_tunnel_shows_its_operation_and_x_cancels_it() {
    use ButtonLook::{Common, Disabled};
    let mut app = expansion_game();
    submit(
        &mut app,
        P1,
        Command::Tunnel {
            tunnel: ObjectId(TUNNEL),
            command: expand_at(FREE),
        },
    );
    updates(&mut app, 40);
    let site = site_of(&app).expect("the Hideout's site");
    assert_eq!(crystals(&app), 800);

    click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(selected(&app), [TUNNEL]);
    let activity_line = activity(&mut app).expect("the Tunnel's activity");
    assert!(activity_line.starts_with("Building Hideout: "), "{activity_line}");
    let x = buttons(&mut app)
        .into_iter()
        .find(|(slot, ..)| *slot == CommandSlot::CANCEL_PRODUCTION);
    assert_eq!(x.map(|(.., look)| look), Some(Common));
    tap(&mut app, &[], KeyW);
    let q = buttons(&mut app).into_iter().find(|(shown, ..)| *shown == slot(0, 0));
    assert_eq!(q.map(|(.., look)| look), Some(Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(OPERATION_UNDER_WAY_TEXT));
    assert_eq!(interface(&app), InterfaceState::ExpandMenu);
    tap(&mut app, &[], KeyZ);

    click(&mut app, centre(FREE), &[]);
    assert_eq!(selected(&app), [site]);
    let cancel = PanelCommand::CancelConstruction;
    assert_eq!(
        buttons(&mut app),
        [(CommandSlot::CANCEL_PRODUCTION, PanelButton::Command(cancel), Common)]
    );
    let progress = site_progress(&mut app, site);
    assert!(progress > 0.0 && progress < 1.0, "{progress}");

    click(&mut app, TUNNEL_AT, &[]);
    tap(&mut app, &[], KeyX);
    assert_eq!(tunnel_commands(&app).last(), Some(&(TUNNEL, TunnelCommand::Cancel)));
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.object(ObjectId(site)).is_none(), "cancelled");
    assert_eq!(crystals(&app), 800 + 150);
    assert_eq!(activity(&mut app), None, "no operation");
}
