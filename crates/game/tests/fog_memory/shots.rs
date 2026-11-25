//! A shot from the fog (design change 0177): its shooter is revealed for a while to the player whose object it hit,
//! and while it is, its shots at that player's objects show whole (design changes 0206 and 0244); any other shot
//! shows only over the player's Visible tiles, from the fog edge.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::tuning::vision::REVEAL_TICKS;
use sc_sim::{GameSetup, OrderKind, Slot};
use space_crystals::combat_effects::{CombatEffects, TRACER_TICKS, Tracer, shot_spans, visible_spans};

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
/// to level 0). The shooter is revealed to player 1, drawn and selectable, until `REVEAL_TICKS` after its last shot:
/// its first, as player 2 stops it then. And so player 1 sees the whole shot, from the shooter on, not out of the fog
/// at the east edge of tile (13, 2), the last tile player 1 sees that way (design change 0206, answering Q0056: "while
/// an attacker is revealed to you, its own shots are drawn along their whole path, from the attacker to the target, so
/// they visibly come from the unit you can see"), as the shot is at player 1's own unit (design change 0244); as with
/// the fog off, and for player 2. Auto-targeting is off, so that nobody fires on their own.
#[test]
fn a_shot_from_the_fog_reveals_the_shooter_whose_shot_then_shows_whole() {
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
        assert_eq!(tracer.target_owner, Some(P1));
        let spans = |player, fog| shot_spans(sim, &Sight::new(sim, player, fog), tracer);
        assert_eq!(
            spans(P1, true),
            [(0.0, 1.0)],
            "from the revealed shooter, not the fog edge a tenth of the way"
        );
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

/// A neutral Peacekeeper on the centre of tile (`x`, `y`), in the next tick.
fn spawn_neutral(app: &mut App, x: i32, y: i32) {
    let at = SpawnAt::Point(centre(x, y));
    let kind = ObjectKind::Peacekeeper;
    let orientation = Default::default();
    debug(
        app,
        DebugCommand::Spawn {
            kind,
            owner: None,
            at,
            orientation,
        },
    );
}

/// Design change 0206: "Other shots from the fog are still clipped to Visible tiles" (`core/vision.md > Attacks from
/// outside vision`: "Shots from attackers that aren't revealed to you are still drawn only over your Visible
/// tiles"). Player 2's Peacekeeper on the high ground (14, 2) shoots a neutral one on (10, 2), which player 1's on
/// (9, 2) sees: an attack on a neutral object reveals its attacker to nobody (`sc_sim::vision::reveal`), so player 1
/// sees the shot come out of the fog at the east edge of tile (13, 2), an eighth of the way from the shooter, which
/// stays hidden; player 2 sees the whole shot.
#[test]
fn a_shot_from_an_attacker_not_revealed_shows_from_the_fog_edge() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    spawn_neutral(&mut app, 10, 2);
    app.update();
    let (attacker, target) = (ObjectId(2), ObjectId(3));
    {
        let sim = app.world().resource::<SimRes>();
        let unit = |id| sim.object(id).unwrap();
        assert_eq!(unit(target).owner, None);
        assert!(!sim.is_visible(P1, unit(attacker)) && sim.is_visible(P1, unit(target)));
    }
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
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let tracer = &world.resource::<CombatEffects>().tracers[0];
    assert_eq!((tracer.attacker, tracer.target), (attacker, target));
    let sight = Sight::new(sim, P1, true);
    assert!(
        !sight.reveals(attacker),
        "a neutral object's attacker is revealed to nobody"
    );
    assert_eq!(shot_spans(sim, &sight, tracer), [(0.125, 1.0)], "from the fog edge");
    assert_eq!(
        shot_spans(sim, &Sight::new(sim, P2, true), tracer),
        [(0.0, 1.0)],
        "the shooter's own"
    );
    assert!(!drawn(&app, 2), "the shooter stays hidden");
}

/// Orders `attacker`, of `player`, to attack `target`, and runs until its shot at it takes effect: that shot.
fn shoot(app: &mut App, player: PlayerId, attacker: ObjectId, target: ObjectId) -> Tracer {
    let order = Command::Order {
        units: vec![attacker],
        kind: OrderKind::Attack(target),
        queue: false,
    };
    submit(app, player, order);
    let ordered = tick(app);
    loop {
        app.update();
        assert!(tick(app) < ordered + 30, "{attacker:?} never fired at {target:?}");
        let tracers = &app.world().resource::<CombatEffects>().tracers;
        let shot = |t: &&Tracer| t.attacker == attacker && t.target == target && t.tick > ordered;
        if let Some(tracer) = tracers.iter().find(shot) {
            return tracer.clone();
        }
    }
}

/// Design change 0244, answering Q0075: "A revealed attacker's shots at anything else (a neutral, a third player's
/// object) are drawn only over your Visible tiles, like any shot from the fog." Player 2's Peacekeeper on the high
/// ground (14, 2) shoots player 1's on (9, 2), from beyond player 1's sight, and is revealed to player 1: that shot,
/// at player 1's own unit, shows whole. Then, revealed still, it shoots a neutral Peacekeeper on (10, 2), which
/// player 1 sees: that shot shows from the fog edge, an eighth of the way, as one from an attacker not revealed
/// ([`a_shot_from_an_attacker_not_revealed_shows_from_the_fog_edge`]), and whole to player 2.
#[test]
fn a_revealed_attackers_shot_at_a_neutral_shows_from_the_fog_edge() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    spawn_neutral(&mut app, 10, 2);
    app.update();
    let (own, attacker, neutral) = (ObjectId(1), ObjectId(2), ObjectId(3));
    let first = shoot(&mut app, P2, attacker, own);
    {
        let sim = app.world().resource::<SimRes>();
        let sight = Sight::new(sim, P1, true);
        assert!(sight.reveals(attacker));
        assert_eq!(first.target_owner, Some(P1));
        assert_eq!(shot_spans(sim, &sight, &first), [(0.0, 1.0)], "at player 1's own unit");
    }
    let second = shoot(&mut app, P2, attacker, neutral);
    assert!(second.tick < first.tick + REVEAL_TICKS);
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let sight = Sight::new(sim, P1, true);
    assert!(sight.reveals(attacker), "revealed still");
    assert!(drawn(&app, 2), "and drawn");
    assert_eq!(second.target_owner, None);
    assert_eq!(shot_spans(sim, &sight, &second), [(0.125, 1.0)], "from the fog edge");
    assert_eq!(
        shot_spans(sim, &Sight::new(sim, P2, true), &second),
        [(0.0, 1.0)],
        "the shooter's own"
    );
}

/// [`STEP`] with three rows more: 24 × 8 tiles.
const STEP_TALL: &str = "
    ........................
    ........................
    ........................
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
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
";

const P3: PlayerId = PlayerId(3);

/// **Design change 0244's Verification**: "In a three-player game, a revealed enemy then shoots a third player's unit
/// hidden in the fog. Only the part of the shot inside your vision is drawn." Player 2's Peacekeeper on the high
/// ground (15, 2) shoots player 1's on (10, 2), from beyond player 1's sight, and is revealed to player 1. Then it
/// shoots player 3's Peacekeeper on the high ground (13, 5), which player 1 doesn't see (4.2 grid units up to level
/// 1, beyond its 4): player 1 sees the shot only over tile (13, 4), the one tile of its path that player 1's unit
/// sees (3.6 grid units), from three quarters of the way to five sixths, neither where the shooter stands nor where
/// the target does. Player 3, whose unit sees the shooter, sees the whole shot.
#[test]
fn a_revealed_attackers_shot_at_a_third_players_hidden_unit_shows_only_over_your_vision() {
    let mut scenario = ScenarioBuilder::new(STEP_TALL).build();
    scenario.map.slots.push(Slot::new(TilePos::new(20, 6)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    let mut app = headless(AppConfig::new(scenario));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 10, 2);
    spawn_unit(&mut app, P2, 15, 2);
    spawn_unit(&mut app, P3, 13, 5);
    app.update();
    let (own, attacker, third) = (ObjectId(1), ObjectId(2), ObjectId(3));
    {
        let sim = app.world().resource::<SimRes>();
        assert_eq!(sim.players().len(), 3, "three players");
        let unit = |id| sim.object(id).unwrap();
        assert_eq!(unit(third).owner, Some(P3));
        assert!(!sim.is_visible(P1, unit(attacker)) && !sim.is_visible(P1, unit(third)));
    }
    let first = shoot(&mut app, P2, attacker, own);
    {
        let sim = app.world().resource::<SimRes>();
        let sight = Sight::new(sim, P1, true);
        assert_eq!(shot_spans(sim, &sight, &first), [(0.0, 1.0)], "at player 1's own unit");
    }
    let second = shoot(&mut app, P2, attacker, third);
    assert!(second.tick < first.tick + REVEAL_TICKS);
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let sight = Sight::new(sim, P1, true);
    assert!(sight.reveals(attacker), "revealed still");
    assert!(!sim.is_visible(P1, sim.object(third).unwrap()), "the target hidden");
    assert_eq!(second.target_owner, Some(P3));
    let spans = shot_spans(sim, &sight, &second);
    assert_eq!(spans, visible_spans(&sight, second.from, second.to));
    assert_eq!(spans.len(), 1, "{spans:?}");
    let (start, end) = spans[0];
    assert!(
        start == 0.75 && (end - 5.0 / 6.0).abs() < 1e-6,
        "over tile (13, 4) only: {spans:?}"
    );
    assert_eq!(
        shot_spans(sim, &Sight::new(sim, P3, true), &second),
        [(0.0, 1.0)],
        "player 3 sees it all"
    );
}

/// The shot that kills player 1's unit is still one at player 1's own object (design change 0244): the shooter of
/// [`a_shot_from_the_fog_reveals_the_shooter_whose_shot_then_shows_whole`] fires on until player 1's Peacekeeper
/// dies, and each of its shots, the last one too, shows whole to player 1, although the unit it killed is gone by the
/// time the client takes the shot (its owner is its death's, `SimEvent::Died`).
#[test]
fn a_revealed_attackers_killing_shot_at_your_unit_shows_whole() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    app.update();
    let (own, attacker) = (ObjectId(1), ObjectId(2));
    shoot(&mut app, P2, attacker, own);
    let mut shots = 1;
    loop {
        assert!(tick(&app) < 400, "the unit never died");
        app.update();
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let sight = Sight::new(sim, P1, true);
        let tracers = &world.resource::<CombatEffects>().tracers;
        let Some(tracer) = tracers.iter().find(|t| t.tick == sim.tick()) else {
            continue;
        };
        shots += 1;
        assert_eq!(tracer.target_owner, Some(P1));
        assert_eq!(shot_spans(sim, &sight, tracer), [(0.0, 1.0)], "shot {shots}");
        if sim.object(own).is_none() {
            break;
        }
    }
    assert!(shots >= 3, "{shots} shots");
}
