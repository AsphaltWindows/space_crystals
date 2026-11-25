//! A shot from the fog (design change 0177): it shows from the fog edge, its shooter revealed for a while.

use sc_sim::OrderKind;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::vision::REVEAL_TICKS;
use space_crystals::combat_effects::{CombatEffects, TRACER_TICKS, shot_spans};

use super::*;

/// 24 × 5 tiles: level 0 in the west, a ramp on column 12, level 1 from column 13 on.
const STEP: &str = "
    ........................
    ........................
    ........................
    ........................
    ........................

    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
";

/// Design change 0177 (`core/vision.md > Attacks from outside vision`), which replaces Q0017 point 1's assumption (a
/// flash of the unit hit, no tracer). Player 2's Peacekeeper on the high ground (tile (14, 2), level 1) shoots player
/// 1's on (9, 2) from beyond player 1's sight (4 grid units up to level 1) but within its own range (5 grid units down
/// to level 0). Player 1 sees the shot come out of the fog at the east edge of tile (13, 2), the last tile it sees
/// that way, a tenth of the way from the shooter; with the fog off, and for player 2, the whole shot shows. The
/// shooter is revealed to player 1, drawn and selectable, until `REVEAL_TICKS` after its last shot: its first, as
/// player 2 stops it then. Auto-targeting is off, so that nobody fires on their own.
#[test]
fn a_shot_from_the_fog_shows_from_the_fog_edge_and_reveals_the_shooter() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    app.update();
    let (target, attacker) = (ObjectId(1), ObjectId(2));
    {
        let sim = app.world().resource::<SimRes>();
        let unit = |id| sim.object(id).unwrap();
        assert!(!sim.is_visible(P1, unit(attacker)) && sim.is_visible(P2, unit(target)));
    }
    assert!(!drawn(&app, 2));
    assert_eq!(selectable(&app), [1]);
    let order = |kind| Command::Order {
        units: vec![attacker],
        kind,
        queue: false,
    };
    submit(&mut app, P2, order(OrderKind::Attack(target)));
    let ordered = tick(&app);
    while app.world().resource::<CombatEffects>().tracers.is_empty() {
        assert!(tick(&app) < ordered + 20, "player 2 never fired");
        app.update();
    }
    let shot = tick(&app);
    submit(&mut app, P2, order(OrderKind::Stop));
    {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let tracer = &world.resource::<CombatEffects>().tracers[0];
        assert_eq!((tracer.target, tracer.tick), (target, shot));
        let spans = |player, fog| shot_spans(sim, &Sight::new(sim, player, fog), tracer);
        assert_eq!(spans(P1, true), [(0.1, 1.0)], "from the fog edge");
        assert_eq!(spans(P1, false), [(0.0, 1.0)], "the fog off");
        assert_eq!(spans(P2, true), [(0.0, 1.0)], "the shooter's own");
    }
    assert!(drawn(&app, 2), "revealed");
    assert_eq!(selectable(&app), [1, 2]);

    run_until(&mut app, shot + TRACER_TICKS);
    {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let mut tracers = world.resource::<CombatEffects>().tracers.iter();
        let sight = Sight::new(sim, P1, true);
        assert!(tracers.all(|t| shot_spans(sim, &sight, t).is_empty()), "faded");
    }
    run_until(&mut app, shot + REVEAL_TICKS - 1);
    assert!(drawn(&app, 2), "still revealed");
    assert_eq!(selectable(&app), [1, 2]);
    app.update();
    assert!(!drawn(&app, 2), "hidden again");
    assert_eq!(selectable(&app), [1]);
}
