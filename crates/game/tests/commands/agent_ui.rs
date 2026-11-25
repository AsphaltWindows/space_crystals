//! The Agent's UI in the client (slice M4-J1; design changes 0233, 0234, 0237, 0257 and 0260): its panel's letters, D
//! greyed with its reason; Build Tunnel's price and its refusal while the bank can't pay; D placing a Laser Turret
//! once the player owns an Optics Lab (slice M4-S1); Shift placing one ghost per Agent; a site's X Cancel Construction
//! alone; the lingering Tunnel ghosts, which only their owner sees; the Enter and Drop Off pointers; and the Supply an
//! Agent carries. The Tunnel ghost's verdicts are [`ghost`]'s (design change 0242).

mod ghost;

use sc_sim::defs::Resource;
use sc_sim::nav::TileRect;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{BuildingOrientation, ConstructionCommand, Faction, PlayerCommand as Sent, SU_PER_GU, SpawnSettings};
use space_crystals::command_ui::feedback::{
    BUSY_TEXT, NO_SUPPLIES_TEXT, NOTHING_CARRIED_TEXT, TargetFeedback, placement_text, requires_text,
};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::object_render::{LoadMarker, SiteProgress};
use space_crystals::placement::lingering::LingeringMesh;
use space_crystals::placement::{LingeringGhosts, PlacementGhost};

use super::*;
use KeyCode::{F2, KeyA, KeyD, KeyS, KeyX, ShiftLeft, Tab};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL_KIND: ObjectKind = ObjectKind::Tunnel;
const BUILD: PanelCommand = PanelCommand::Build(TUNNEL_KIND);
const TURRET_KIND: ObjectKind = ObjectKind::LaserTurret;
const BUILD_TURRET: PanelCommand = PanelCommand::Build(TURRET_KIND);

/// The objects of [`agent_game`], by id, and where they are (the map is flat: where they are drawn).
const TUNNEL: u32 = 1;
const AGENTS: [u32; 2] = [2, 3];
const GUARD: u32 = 4;
const STATION: u32 = 5;
const TUNNEL_AT: Vec2 = Vec2::new(1152.0, 1088.0); // tiles (16, 15) to (19, 18); Tier 1 area (14, 13) to (21, 20)
const AGENTS_AT: [Vec2; 2] = [Vec2::new(800.0, 760.0), Vec2::new(900.0, 760.0)];
const GUARD_AT: Vec2 = Vec2::new(1000.0, 650.0);
/// Sites clear of the Tunnel's area and of each other's (Tier 1 areas (24, 10) to (31, 17) and (32, 10) to (39, 17)),
/// far enough from the Agents that their Builds stay pending a while.
const SITES: [TilePos; 2] = [TilePos::new(26, 12), TilePos::new(34, 12)];
/// A site in the north of the world view, clear of the Tunnel's area (its own (22, 7) to (29, 14)), which an Agent
/// reaches soon.
const NEAR_SITE: TilePos = TilePos::new(24, 9);

/// The ghost's centre, where the cursor stands over a Tunnel's site at `origin` (4 × 4 tiles).
fn centre(origin: TilePos) -> Vec2 {
    (Vec2::new(origin.x as f32, origin.y as f32) + 2.0) * SU_PER_GU
}

/// A game of player 1, the Syndicate, on a flat 64 × 48 map, as [`game`] sets it up (no fog): their Tunnel, two
/// Agents in the west of the world view, a Guard, a stocked Supply Delivery Station by the Agents (tiles (12, 15) to
/// (13, 16)), and player 2's Tunnel far off, unseen. Player 1 has 10 Supplies: the next Tunnel costs 1 (design change
/// 0233: one per Tunnel owned).
fn agent_game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let stocked = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 6,
        delivery_size: 4,
        delivery_interval: 960,
    };
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL_KIND, Some(P1), tile(16, 15))
        .spawn(AGENT, Some(P1), SpawnAt::Point(AGENTS_AT[0]))
        .spawn(AGENT, Some(P1), SpawnAt::Point(AGENTS_AT[1]))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(GUARD_AT))
        .spawn_with(stocked, tile(12, 15))
        .spawn(TUNNEL_KIND, Some(P2), tile(56, 40));
    let mut app = game_of(builder.build());
    set_supplies(&mut app, 10);
    app
}

/// Centres the camera on the view point `view`, as far as the map lets it (a frame).
fn look_at(app: &mut App, view: Vec2) {
    app.world_mut().resource_mut::<CameraView>().center = view;
    app.update();
}

/// Sets player 1's Supplies, and no Space Crystals (a debug command, applied at the next frame's tick).
fn set_supplies(app: &mut App, supplies: u32) {
    set_bank(app, 0, supplies);
}

/// Sets player 1's Space Crystals and Supplies (a debug command, applied at the next frame's tick).
fn set_bank(app: &mut App, crystals: u32, supplies: u32) {
    let set = DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(app, P1, Command::Debug(set));
    app.update();
}

/// Player 1's Supplies.
fn supplies(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").supplies
}

/// Player 1's Space Crystals.
fn crystals(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(P1).expect("in the game").crystals
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

/// The feedback message that shows, and the tick of its cause.
fn feedback(app: &App) -> Option<(&'static str, u32)> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|since| (feedback.text, since))
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    feedback(app).map(|(text, _)| text)
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

/// A Build of a Tunnel at `origin`, unturned.
fn build_at(origin: TilePos) -> OrderKind {
    OrderKind::Build {
        kind: TUNNEL_KIND,
        origin,
        orientation: BuildingOrientation::default(),
    }
}

/// Player 1's order to Agent `agent`, unqueued, to build a Tunnel at `origin`, submitted for the next tick.
fn order_build(app: &mut App, agent: u32, origin: TilePos) {
    let build = Command::Order {
        units: vec![ObjectId(agent)],
        kind: build_at(origin),
        queue: false,
    };
    submit(app, P1, build);
}

/// Every Build given so far, as the sim received them.
fn builds(app: &App) -> Vec<Given> {
    let build = |given: &Given| matches!(given.kind, OrderKind::Build { .. });
    orders(app).into_iter().filter(build).collect()
}

/// The slots of the Agent's letters.
fn q() -> CommandSlot {
    slot(0, 0)
}
fn a() -> CommandSlot {
    slot(1, 0)
}
fn s() -> CommandSlot {
    slot(1, 1)
}
fn d() -> CommandSlot {
    slot(1, 2)
}
fn x() -> CommandSlot {
    slot(2, 1)
}

/// Selects `id` with a click on it.
fn click_on(app: &mut App, id: u32) {
    let at = pos(app, id);
    click(app, at, &[]);
    assert_eq!(selected(app), [id]);
}

/// Design change 0257 (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`): Q Move, A Build Tunnel, S
/// Drop Off Resources, D Build Laser Turret "(greyed until available)", X Stop, and no Attack, Hold, Patrol or
/// AttackMove. D and S, greyed, say why when pressed (design change 0162) and do nothing: D, the Laser Turret's Build
/// (slice M4-S1), that the player owns no Optics Lab. A's hint names the Tunnel's price as the sim would charge it
/// (design change 0233: 1 Supply with one Tunnel owned), D's the Laser Turret's.
#[test]
fn the_agents_panel_has_its_letters_d_greyed_with_its_reason() {
    use ButtonLook::{Common, Disabled};
    let mut app = agent_game();
    click_on(&mut app, AGENTS[0]);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(PanelCommand::Move), Common),
            (a(), c(BUILD), Common),
            (s(), c(PanelCommand::DropOff), Disabled),
            (d(), c(BUILD_TURRET), Disabled),
            (x(), c(PanelCommand::Stop), Common),
        ]
    );
    tap(&mut app, &[], KeyD);
    assert_eq!(said(&app), Some(requires_text(ObjectKind::OpticsLab)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyS);
    assert_eq!(said(&app), Some(NOTHING_CARRIED_TEXT));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(hint_over(&mut app, a()).as_deref(), Some("A: Build Tunnel (1 Supply)"));
    assert_eq!(
        hint_over(&mut app, d()).as_deref(),
        Some("D: Build Laser Turret (100 Space Crystals)")
    );
    assert_eq!(orders(&app), []);
}

/// Where the Agent of [`with_an_optics_lab_d_places_a_laser_turret`] builds its Laser Turret (1 × 1), north of the
/// Tunnel's area, and the ghost's centre there.
const TURRET_SITE: TilePos = TilePos::new(16, 11);
const TURRET_SITE_AT: Vec2 = Vec2::new(16.5 * SU_PER_GU, 11.5 * SU_PER_GU);
/// Where its Optics Lab (2 × 2) is spawned, in the Tunnel's area by the Tunnel.
const LAB_AT: TilePos = TilePos::new(14, 13);

/// Slice M4-S1 (`factions/syndicate/structures/laser_turret.md > Construction`; design change 0297 point 4: paid at the
/// order): once player 1 owns an Optics Lab and the bank holds the Laser Turret's 100 Space Crystals, D is lit and its
/// hint names that price. D enters the placement of a 1 × 1 ghost; a click on a clear tile sends the Agent its Build
/// and ends placement, the order pays the 100 Space Crystals, and the lingering ghost is the turret's footprint with no
/// Tunnel Area (it has none).
#[test]
fn with_an_optics_lab_d_places_a_laser_turret() {
    let mut app = agent_game();
    let lab = DebugCommand::Spawn {
        kind: ObjectKind::OpticsLab,
        owner: Some(P1),
        at: SpawnAt::Tile(LAB_AT),
        orientation: Default::default(),
    };
    submit(&mut app, P1, Command::Debug(lab));
    set_bank(&mut app, 100, 10);
    let sim = app.world().resource::<SimRes>();
    let lab = sim
        .objects()
        .find(|o| o.kind == ObjectKind::OpticsLab)
        .map(|o| (o.owner, o.placement));
    assert_eq!(
        lab.map(|(owner, at)| (owner, at.map(|at| at.origin))),
        Some((Some(P1), Some(LAB_AT)))
    );
    click_on(&mut app, AGENTS[0]);
    let lit = buttons(&mut app).into_iter().find(|(slot, ..)| *slot == d());
    let build = PanelButton::Command(BUILD_TURRET);
    assert_eq!(lit, Some((d(), build, ButtonLook::Common)));
    assert_eq!(
        hint_over(&mut app, d()).as_deref(),
        Some("D: Build Laser Turret (100 Space Crystals)")
    );
    tap(&mut app, &[], KeyD);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
    pointer_over(&mut app, TURRET_SITE_AT);
    let ghost = app.world().resource::<PlacementGhost>().shown.clone().expect("a ghost");
    assert_eq!(
        (ghost.kind, ghost.origin, ghost.size),
        (TURRET_KIND, TURRET_SITE, (1, 1))
    );
    assert!(ghost.is_valid(), "{:?}", ghost.check);

    click(&mut app, TURRET_SITE_AT, &[]);
    let order = OrderKind::Build {
        kind: TURRET_KIND,
        origin: TURRET_SITE,
        orientation: BuildingOrientation::default(),
    };
    assert_eq!(builds(&app), [given(&[AGENTS[0]], order, false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 2);
    assert_eq!((crystals(&app), supplies(&app)), (0, 10));
    let shown = app.world().resource::<LingeringGhosts>();
    let footprints: Vec<_> = (shown.shown.iter()).map(|site| (site.origin, site.w, site.h)).collect();
    assert_eq!(footprints, [(TURRET_SITE, 1, 1)]);
    assert_eq!(shown.areas, []);
}

/// Design change 0233: "a Build order that can't be paid is refused" (0162: with the unavailable-command feedback).
/// With 0 Supplies, A is greyed: pressing it says so, shows no ghost, and a click on the map then sends no Build. With
/// 1 Supply, A shows the ghost; a click once the bank no longer pays (the Supplies spent meanwhile) says so too, sends
/// nothing, and placement goes on.
#[test]
fn a_build_the_bank_cant_pay_says_so_and_sends_nothing() {
    let mut app = agent_game();
    set_supplies(&mut app, 0);
    click_on(&mut app, AGENTS[0]);
    let greyed = buttons(&mut app).into_iter().find(|(slot, ..)| *slot == a());
    assert_eq!(greyed.map(|(.., look)| look), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyA);
    let (text, first) = feedback(&app).expect("a message");
    assert_eq!(text, NO_SUPPLIES_TEXT);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(app.world().resource::<PlacementGhost>().shown, None);
    click(&mut app, centre(SITES[0]), &[]);
    assert_eq!(builds(&app), []);

    set_supplies(&mut app, 1);
    click_on(&mut app, AGENTS[0]);
    tap(&mut app, &[], KeyA);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
    set_supplies(&mut app, 0);
    click(&mut app, centre(SITES[0]), &[]);
    let (text, then) = feedback(&app).expect("a message");
    assert_eq!(text, NO_SUPPLIES_TEXT);
    assert!(then > first, "said again");
    assert_eq!(builds(&app), []);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
}

/// `systems/controls/selection_and_hud.md > AwaitingTarget > Shift while placing`: "each Agent takes exactly one
/// placement", and placement ends at a Shift-click as at a plain one; to place with the next Agent, the player presses
/// Tab, then A. Two Shift-clicks with two Agents leave two lingering ghosts with their Tier 1 areas, each paid as it
/// was given (1, then 2: the first ghost counts, design change 0233). The second Agent's ghost is red where its area
/// would overlap the first ghost's, and a Shift-click there says why and sends nothing, placement going on (design
/// change 0289: "Shift-queue two Tunnels whose areas overlap: the second ghost is red"). A third Shift-click with the
/// first Agent again, on a valid site, is refused by the gate's one placement per Agent and sends nothing.
#[test]
fn shift_places_several_ghosts_one_agent_each() {
    use sc_sim::{PlacementProblem, WholeProblem::TunnelAreaOverlap};
    let mut app = agent_game();
    select_units(&mut app, &AGENTS);
    for (agent, site) in AGENTS.into_iter().zip(SITES) {
        tap(&mut app, &[], KeyA);
        assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
        if agent == AGENTS[1] {
            let overlapping = TilePos::new(SITES[0].x + 4, SITES[0].y);
            pointer_over(&mut app, centre(overlapping));
            let ghost = app.world().resource::<PlacementGhost>().shown.clone().expect("a ghost");
            assert_eq!(ghost.origin, overlapping);
            assert_eq!(ghost.check.whole, [TunnelAreaOverlap]);
            click(&mut app, centre(overlapping), &[ShiftLeft]);
            assert_eq!(
                said(&app),
                Some(placement_text(Some(PlacementProblem::Whole(TunnelAreaOverlap))))
            );
            assert_eq!(builds(&app).len(), 1);
            assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
        }
        click(&mut app, centre(site), &[ShiftLeft]);
        assert_eq!(builds(&app).last(), Some(&given(&[agent], build_at(site), true)));
        assert_eq!(interface(&app), InterfaceState::DefaultState, "placement ends");
        tap(&mut app, &[], Tab);
    }
    assert_eq!(builds(&app).len(), 2);
    assert_eq!(supplies(&app), 10 - 1 - 2);
    let shown = app.world().resource::<LingeringGhosts>();
    let footprints: Vec<TilePos> = shown.shown.iter().map(|site| site.origin).collect();
    assert_eq!(footprints, SITES);
    let areas = SITES.map(|site| TileRect::new(TilePos::new(site.x - 2, site.y - 2), 8, 8));
    assert_eq!(shown.areas, areas);

    // Tabbed back to the first Agent.
    let third = TilePos::new(40, 30);
    tap(&mut app, &[], KeyA);
    look_at(&mut app, centre(third));
    click(&mut app, centre(third), &[ShiftLeft]);
    assert_eq!(said(&app), Some(BUSY_TEXT));
    assert_eq!(builds(&app).len(), 2);
}

/// Design change 0237: a Tunnel site "does nothing but Cancel: [...] the only command is X Cancel Construction". The
/// site an Agent starts is selected alone, shows X alone and its progress, has no right-click of its own, and X
/// cancels it: the Agent comes out, and the Supply it cost comes back (the current top price, design change 0233).
#[test]
fn a_site_offers_x_cancel_construction_only() {
    let mut app = agent_game();
    order_build(&mut app, AGENTS[0], NEAR_SITE);
    let site = run_until_site(&mut app);
    assert_eq!(supplies(&app), 9);
    updates(&mut app, 4);
    click(&mut app, centre(NEAR_SITE), &[]);
    assert_eq!(selected(&app), [site]);
    let cancel = PanelCommand::CancelConstruction;
    assert_eq!(
        buttons(&mut app),
        [(x(), PanelButton::Command(cancel), ButtonLook::Common)]
    );
    assert_eq!(pointer_over(&mut app, GUARD_AT), Some(PointerDisplay::Inactive));
    let progress = site_progress(&mut app, site);
    assert!(progress > 0.0 && progress < 1.0, "{progress}");

    tap(&mut app, &[], KeyX);
    let sent = construction_commands(&app);
    assert_eq!(sent, [(vec![site], ConstructionCommand::CancelConstruction)]);
    updates(&mut app, 4);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.object(ObjectId(site)).is_none(), "cancelled");
    assert!(sim.object(ObjectId(AGENTS[0])).is_some(), "the Agent came out");
    assert_eq!(supplies(&app), 10);
}

/// Runs frames until one of player 1's sites stands, 400 at most, and returns its id.
fn run_until_site(app: &mut App) -> u32 {
    for _ in 0..400 {
        if let Some(site) = site_of(app) {
            return site;
        }
        app.update();
    }
    panic!("no site within 400 frames");
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

/// Every construction command given so far: its objects and the command.
fn construction_commands(app: &App) -> Vec<(Vec<u32>, ConstructionCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    let construction = |sent: &Sent| match &sent.command {
        Command::Construction { objects, command } => Some((objects.iter().map(|id| id.0).collect(), *command)),
        _ => None,
    };
    commands.filter_map(construction).collect()
}

/// `core/entities.md > Placement Validation > Lingering build ghosts` (design change 0125): "Visible to its owner.
/// Enemies never see it." Player 1's pending Build shows its ghost with its Tier 1 area to player 1; switched to player
/// 2 (F2, the debug player switch: the per-player view the client draws), none shows; back to player 1, it does again.
#[test]
fn the_owner_sees_the_lingering_ghost_and_the_enemy_doesnt() {
    let mut app = agent_game();
    order_build(&mut app, AGENTS[0], SITES[1]);
    updates(&mut app, 2);
    let lingering = |app: &mut App| {
        let ghosts = app.world().resource::<LingeringGhosts>();
        let shown = (ghosts.shown.len(), ghosts.areas.len());
        let mut query = app.world_mut().query_filtered::<&Visibility, With<LingeringMesh>>();
        let visible = query.single(app.world()).is_ok_and(|v| *v != Visibility::Hidden);
        (shown, visible)
    };
    assert_eq!(lingering(&mut app), ((1, 1), true));
    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), P2);
    assert_eq!(lingering(&mut app), ((0, 0), false), "never the enemy's");
    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), P1);
    assert_eq!(lingering(&mut app), ((1, 1), true));
}

/// `systems/controls/selection_and_hud.md > PointerDisplayType`: "Cursor over own Tunnel (Syndicate unit, tier
/// sufficient): **Enter**", for an Agent that carries nothing and for a Guard; an Agent that carries a Supply shows
/// ReturnResources there instead (`factions/syndicate/units/agent.md > Right-Click Resolution`: "Own Tunnel (carrying
/// resources) → Drop off resources"), with the Supply's crate on its body and no crystal shard, and its S, no longer
/// greyed, sends a Drop Off to the Tunnel clicked. Over a site, neither pointer shows: Move.
#[test]
fn the_enter_and_drop_off_pointers_and_the_carried_supply() {
    let mut app = agent_game();
    click_on(&mut app, AGENTS[0]);
    assert_eq!(pointer_over(&mut app, TUNNEL_AT), Some(PointerDisplay::Enter));
    click_on(&mut app, GUARD);
    assert_eq!(pointer_over(&mut app, TUNNEL_AT), Some(PointerDisplay::Enter));

    // Agent 2 starts a site; Agent 3 picks up a Supply at the station, and stops there with it (other orders keep the
    // load, design change 0129).
    let carrier = AGENTS[1];
    order_build(&mut app, AGENTS[0], NEAR_SITE);
    let gather = Command::Order {
        units: vec![ObjectId(carrier)],
        kind: OrderKind::Gather(ObjectId(STATION)),
        queue: false,
    };
    submit(&mut app, P1, gather);
    let mut stopped = false;
    for _ in 0..600 {
        if !stopped && carried(&app, carrier).is_some() {
            let stop = Command::Order {
                units: vec![ObjectId(carrier)],
                kind: OrderKind::Stop,
                queue: false,
            };
            submit(&mut app, P1, stop);
            stopped = true;
        }
        if stopped && site_of(&app).is_some() {
            break;
        }
        app.update();
    }
    updates(&mut app, 2);
    assert_eq!(carried(&app, carrier), Some(Resource::Supplies));
    let site = site_of(&app).expect("a site");
    use Resource::{Crystals, Supplies};
    assert_eq!(markers_shown(&mut app, carrier), [(Crystals, false), (Supplies, true)]);
    assert_eq!(markers_shown(&mut app, GUARD), [], "not a worker");

    let site_at = pos(&app, site);
    click_on(&mut app, GUARD);
    assert_eq!(pointer_over(&mut app, site_at), Some(PointerDisplay::Move));
    click_on(&mut app, carrier);
    assert_eq!(pointer_over(&mut app, site_at), Some(PointerDisplay::Move));
    assert_eq!(pointer_over(&mut app, TUNNEL_AT), Some(PointerDisplay::ReturnResources));
    tap(&mut app, &[], KeyS);
    let awaiting = InterfaceState::AwaitingTarget {
        pending: PanelCommand::DropOff,
        queued: false,
    };
    assert_eq!(interface(&app), awaiting);
    click(&mut app, TUNNEL_AT, &[]);
    let drop_off = OrderKind::DropOff(ObjectId(TUNNEL));
    assert_eq!(orders(&app).last(), Some(&given(&[carrier], drop_off, false)));
}

/// What worker `id` carries, if anything.
fn carried(app: &App, id: u32) -> Option<Resource> {
    let sim = app.world().resource::<SimRes>();
    let worker = sim.object(ObjectId(id))?.worker?;
    (worker.load > 0).then_some(worker.resource)
}

/// Which of object `id`'s load markers show, by resource.
fn markers_shown(app: &mut App, id: u32) -> Vec<(Resource, bool)> {
    let mut query = app.world_mut().query::<(&LoadMarker, &Visibility)>();
    let mut shown: Vec<(Resource, bool)> = (query.iter(app.world()))
        .filter(|(marker, _)| marker.0 == ObjectId(id))
        .map(|(marker, visibility)| (marker.1, *visibility != Visibility::Hidden))
        .collect();
    shown.sort();
    shown
}
