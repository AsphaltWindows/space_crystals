//! A Tunnel's Upgrade in the client (slice M4-G; `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`,
//! `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`): Q names its price and sends the Upgrade, greyed
//! with the gate's reason, never what blocks it; the Tunnel's InfoPanel shows the upgrade's progress and its X cancels
//! it, refunding the price; once complete, the larger area in the overlay.

use sc_sim::nav::TileRect;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, PlayerCommand as Sent, TunnelCommand};
use space_crystals::command_ui::feedback::{
    NO_SUPPLIES_TEXT, OPERATION_UNDER_WAY_TEXT, TOP_TIER_TEXT, TargetFeedback, UPGRADE_BLOCKED_TEXT,
};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::hud::layout::CommandGrid;
use space_crystals::placement::TunnelAreaOverlay;
use space_crystals::platform::SurfaceSize;

use super::*;
use KeyCode::{KeyQ, KeyX};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UPGRADE: PanelCommand = PanelCommand::UpgradeTunnel;

/// The objects of [`upgrade_game`], by id, and where they are drawn (the map is flat).
const TUNNEL: u32 = 1;
const ENEMY_TUNNEL: u32 = 2;
const TUNNEL_AT: Vec2 = Vec2::new(1152.0, 1088.0); // tiles (16, 15) to (19, 18)
/// The Tunnel's Tier 1 area, and the Tier 2 area an upgrade gives it (`factions/syndicate/tunnels.md > Tunnel > Tiers`:
/// Tunnel Area Radius 2, then 3).
const TIER_1_AREA: TileRect = TileRect {
    origin: TilePos::new(14, 13),
    w: 8,
    h: 8,
};
const TIER_2_AREA: TileRect = TileRect {
    origin: TilePos::new(13, 12),
    w: 10,
    h: 10,
};

/// A game of player 1, the Syndicate, on a flat 64 × 48 map, as [`game`] sets it up (no fog): their Tunnel at tiles
/// (16, 15) to (19, 18), at Tier 1, and 10 Supplies; with `enemy_beside`, player 2's Tunnel at (24, 15) to (27, 18),
/// whose Tier 1 area, (22, 13) to (29, 20), touches player 1's Tier 1 area and takes the column a Tier 2 area needs, and
/// another of theirs far off, which keeps them in the game without it.
fn upgrade_game(enemy_beside: bool) -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(16, 15));
    if enemy_beside {
        builder = (builder.spawn(ObjectKind::Tunnel, Some(P2), tile(24, 15))).spawn(
            ObjectKind::Tunnel,
            Some(P2),
            tile(50, 36),
        );
    }
    let mut app = game_of(builder.build());
    let supplies = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, P1, Command::Debug(supplies));
    app.update();
    app
}

/// Player 1's Supplies.
fn supplies(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").supplies
}

/// Player 1's Tunnel's tier.
fn tier(app: &App) -> u8 {
    let sim = app.world().resource::<SimRes>();
    let tunnel = sim.object(ObjectId(TUNNEL)).and_then(|o| o.tunnel.as_ref());
    tunnel.expect("a Tunnel").tier
}

/// The HUD's button in `slot`: its command, look and price (in Supplies, an Upgrade's).
fn button(app: &mut App, slot: CommandSlot) -> Option<(PanelButton, ButtonLook, Option<u32>)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown = query.iter(app.world()).filter(|b| b.slot == slot);
    shown
        .next()
        .map(|b| (b.shown.button, b.shown.look, b.shown.price.map(|price| price.amount)))
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
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

/// The InfoPanel's activity area, while it shows.
fn activity(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    texts
        .iter(app.world())
        .find(|(which, _, node)| **which == InfoText::Activity && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
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

/// The areas of the selected Tunnels the Tunnel Area display shows: Tunnel, tier and area, while it shows.
fn overlay(app: &App) -> Option<Vec<(u32, u8, TileRect)>> {
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    let shown = overlay
        .selected
        .iter()
        .map(|shown| (shown.tunnel.0, shown.tier, shown.area));
    overlay.visible.then(|| shown.collect())
}

/// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257): "**Q** Upgrade Tunnel", its
/// hint naming the price (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: 2 Supplies to Tier 2 for a
/// player with no other Tunnel at Tier 2 or above). Q sends the Upgrade and the Supplies go; meanwhile the Tunnel's
/// InfoPanel shows "Upgrading to Tier 2" and the progress (placeholder wording), Q is greyed ("A Tunnel can only
/// perform one operation at a time") and X is not. X cancels the upgrade, refunding the price in full (design change
/// 0284: the current top price), and Q is back.
#[test]
fn q_upgrades_the_tunnel_and_x_cancels_it_with_a_refund() {
    let mut app = upgrade_game(false);
    click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(selected(&app), [TUNNEL]);
    let c = PanelButton::Command;
    assert_eq!(
        button(&mut app, slot(0, 0)),
        Some((c(UPGRADE), ButtonLook::Common, Some(2)))
    );
    assert_eq!(
        hint_over(&mut app, slot(0, 0)).as_deref(),
        Some("Q: Upgrade Tunnel (2 Supplies)")
    );
    let x = |app: &mut App| button(app, CommandSlot::CANCEL_PRODUCTION).map(|(_, look, _)| look);
    assert_eq!(x(&mut app), Some(ButtonLook::Disabled));
    assert_eq!(activity(&mut app), None, "no operation");

    tap(&mut app, &[], KeyQ);
    assert_eq!(tunnel_commands(&app), [(TUNNEL, TunnelCommand::Upgrade)]);
    updates(&mut app, 2);
    assert_eq!(supplies(&app), 8);
    let line = activity(&mut app).expect("the Tunnel's activity");
    assert!(line.starts_with("Upgrading to Tier 2: "), "{line}");
    assert_eq!(
        button(&mut app, slot(0, 0)).map(|(_, look, _)| look),
        Some(ButtonLook::Disabled)
    );
    assert_eq!(x(&mut app), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(OPERATION_UNDER_WAY_TEXT));
    assert_eq!(tunnel_commands(&app).len(), 1, "a greyed Q sends nothing");

    tap(&mut app, &[], KeyX);
    assert_eq!(tunnel_commands(&app).last(), Some(&(TUNNEL, TunnelCommand::Cancel)));
    updates(&mut app, 2);
    assert_eq!(supplies(&app), 10, "refunded");
    assert_eq!(tier(&app), 1);
    assert_eq!(activity(&mut app), None, "cancelled");
    assert_eq!(
        button(&mut app, slot(0, 0)),
        Some((c(UPGRADE), ButtonLook::Common, Some(2)))
    );
}

/// `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**480 frames (30 seconds)**". The Tunnel's area
/// shows at Tier 1 while it upgrades (it "keeps the old tier" until complete), at Tier 2 once complete; Q then prices
/// the Tier 3 upgrade (3 Supplies), and at Tier 3 it is greyed: "Unavailable if already Tier 3".
#[test]
fn the_upgrade_completes_and_the_area_grows() {
    let mut app = upgrade_game(false);
    click(&mut app, TUNNEL_AT, &[]);
    tap(&mut app, &[], KeyQ);
    updates(&mut app, 2);
    assert_eq!(overlay(&app), Some(vec![(TUNNEL, 1, TIER_1_AREA)]));
    // The exact frame is the sim's to test (`crates/sim/tests/upgrades`); here, about 480 updates, one tick each.
    let mut ticks = 2;
    while tier(&app) == 1 {
        assert!(ticks < 482, "the upgrade is over in 480 frames");
        app.update();
        ticks += 1;
    }
    assert!(ticks > 470, "not before: {ticks}");
    app.update();
    assert_eq!(overlay(&app), Some(vec![(TUNNEL, 2, TIER_2_AREA)]));
    assert_eq!(activity(&mut app), None, "complete");
    let c = PanelButton::Command;
    assert_eq!(
        button(&mut app, slot(0, 0)),
        Some((c(UPGRADE), ButtonLook::Common, Some(3)))
    );

    let set_tier = DebugCommand::SetTier {
        id: ObjectId(TUNNEL),
        tier: 3,
    };
    submit(&mut app, P1, Command::Debug(set_tier));
    updates(&mut app, 2);
    assert_eq!(
        button(&mut app, slot(0, 0)),
        Some((c(UPGRADE), ButtonLook::Disabled, None))
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(TOP_TIER_TEXT));
}

/// `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`: "the upgrade check uses the truth; the upgrading
/// Tunnel's owner learns that something blocks it". Player 2's Tunnel takes the column player 1's Tier 2 area needs:
/// Q is greyed and says only that something blocks the upgrade (placeholder wording); with the other Tunnel gone it is
/// back. Without the Supplies it is greyed for them.
#[test]
fn something_blocks_the_upgrade_and_the_owner_learns_only_that() {
    let mut app = upgrade_game(true);
    click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(selected(&app), [TUNNEL]);
    let look = |app: &mut App| button(app, slot(0, 0)).map(|(_, look, _)| look);
    assert_eq!(look(&mut app), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(UPGRADE_BLOCKED_TEXT));
    assert!(tunnel_commands(&app).is_empty(), "a greyed Q sends nothing");

    let remove = DebugCommand::Remove {
        id: ObjectId(ENEMY_TUNNEL),
    };
    submit(&mut app, P1, Command::Debug(remove));
    updates(&mut app, 2);
    assert_eq!(look(&mut app), Some(ButtonLook::Common));

    let broke = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 1,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, P1, Command::Debug(broke));
    updates(&mut app, 2);
    assert_eq!(look(&mut app), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyQ);
    assert_eq!(said(&app), Some(NO_SUPPLIES_TEXT));
}
