//! The Agents' Repair in the client (slice M5-N8; `factions/syndicate/units/agent.md > Repair`, design changes 0325 and
//! 0359): a right-click repairs a damaged own Laser Turret and enters a damaged own Tunnel, Enter winning; D repairs
//! the Tunnel, and is Inactive over an expansion, which no one repairs.

use space_crystals::command_ui::feedback::INVALID_TARGET_TEXT;

use super::*;

/// Where these tests' Laser Turret (1 × 1) stands, on the east edge of the Tunnel's Tier 1 area ((14, 13) to
/// (21, 20)), and where the cursor points at it; and where it points at the Optics Lab (2 × 2) at [`LAB_AT`].
const TURRET_AT: TilePos = TilePos::new(21, 16);
fn turret_at() -> Vec2 {
    (Vec2::new(TURRET_AT.x as f32, TURRET_AT.y as f32) + 0.5) * SU_PER_GU
}
fn lab_at() -> Vec2 {
    (Vec2::new(LAB_AT.x as f32, LAB_AT.y as f32) + 1.0) * SU_PER_GU
}

/// Spawns player 1's `kind` on `at` (a debug command), complete, and returns its id, two frames later.
fn spawn_own(app: &mut App, kind: ObjectKind, at: TilePos) -> u32 {
    let spawn = DebugCommand::Spawn {
        kind,
        owner: Some(P1),
        at: SpawnAt::Tile(at),
        orientation: Default::default(),
    };
    submit(app, P1, Command::Debug(spawn));
    updates(app, 2);
    let sim = app.world().resource::<SimRes>();
    let spawned = sim.objects().last().expect("spawned");
    assert_eq!((spawned.kind, spawned.owner), (kind, Some(P1)));
    spawned.id.0
}

/// `factions/syndicate/units/agent.md > Right-Click Resolution`: "Own Tunnel (not carrying resources) → Enter (also
/// when damaged: Enter wins over Repair)"; "Damaged own Laser Turret → Repair" (design change 0325). With player 1's
/// Tunnel at half its 600 HP and a Laser Turret at half its 300, the two Agents' pointer is Repair over the turret and
/// Enter over the Tunnel, and a right-click on each gives them that order.
#[test]
fn a_right_click_repairs_a_damaged_turret_and_enters_a_damaged_tunnel() {
    let mut app = agent_game();
    let turret = spawn_own(&mut app, TURRET_KIND, TURRET_AT);
    set_hp(&mut app, turret, 150.0);
    set_hp(&mut app, TUNNEL, 300.0);
    select_units(&mut app, &AGENTS);
    assert_eq!(pointer_over(&mut app, turret_at()), Some(PointerDisplay::Repair));
    assert_eq!(pointer_over(&mut app, TUNNEL_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, turret_at(), &[]);
    right_click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(
        orders(&app),
        [
            given(&AGENTS, OrderKind::Repair(ObjectId(turret)), false),
            given(&AGENTS, OrderKind::Enter(ObjectId(TUNNEL)), false),
        ]
    );
}

/// `factions/syndicate/units/agent.md > Repair`: "a Tunnel's right-click stays **Enter**, so repair it with **D:
/// Repair**"; Agents repair "**not expansions**, which regenerate instead". With player 1's Tunnel at half its 600 HP
/// and an Optics Lab at half its 400, D awaits a target: the pointer is Repair over the Tunnel and Inactive over the
/// Lab, where a click is an invalid target (no order, the message, D still awaiting one); a click on the Tunnel gives
/// the Agent its Repair, the panel back in DefaultState.
#[test]
fn d_repairs_a_damaged_tunnel_and_no_expansion() {
    let mut app = agent_game();
    let lab = spawn_own(&mut app, ObjectKind::OpticsLab, LAB_AT);
    set_hp(&mut app, lab, 200.0);
    set_hp(&mut app, TUNNEL, 300.0);
    click_on(&mut app, AGENTS[0]);
    tap(&mut app, &[], KeyD);
    let awaiting = InterfaceState::awaiting(REPAIR);
    assert_eq!(interface(&app), awaiting);
    assert_eq!(pointer_over(&mut app, TUNNEL_AT), Some(PointerDisplay::Repair));
    assert_eq!(pointer_over(&mut app, lab_at()), Some(PointerDisplay::Inactive));
    click(&mut app, lab_at(), &[]);
    assert_eq!(said(&app), Some(INVALID_TARGET_TEXT));
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), awaiting);
    click(&mut app, TUNNEL_AT, &[]);
    let repair = OrderKind::Repair(ObjectId(TUNNEL));
    assert_eq!(orders(&app), [given(&[AGENTS[0]], repair, false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
