//! (M4-O) Area attacks in the fog (plan decision T52; `core/vision.md > Attacks from outside vision`): a projectile
//! shows only over the player's Visible tiles, a revealed shooter's at the player's own objects too (design change
//! 0293, answering Q0083 point 4: "No exceptions"); a blast only where the player sees its centre; and a victim hidden
//! from the player, a Hideout under a blast, shows nothing: no HP bar, no death marker, no hit. No type of the game
//! throws grenades before slice U, so the projectiles' and the blasts' events are handed to the client as the sim
//! driver would hand them; the reveal and the damage are the sim's own.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::vision::Hiding;
use sc_sim::{OrderKind, SimEvent, VisibilityState};
use space_crystals::combat_effects::{CombatEffects, Flight, hp_bars, shows_blast, shows_death, shows_flight};
use space_crystals::sim_driver::SimEventMessage;

use super::*;

/// 24 × 5 tiles: level 0 in the west, a ramp on column 12, level 1 from column 13 on (as `shots.rs` has it).
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

/// Gives the client `event` as the sim's of the tick that runs next, as the sim driver forwards them, and runs that
/// tick.
fn hand(app: &mut App, event: SimEvent) {
    let tick = tick(app) + 1;
    app.world_mut()
        .resource_mut::<Messages<SimEventMessage>>()
        .write(SimEventMessage { tick, event });
    app.update();
}

fn order(app: &mut App, player: PlayerId, unit: ObjectId, kind: OrderKind) {
    let units = vec![unit];
    submit(
        app,
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    );
}

/// `core/vision.md > Attacks from outside vision`: "a shot, tracer or projectile is visible to a player only along the
/// part of its path that crosses tiles Visible to that player", and design change 0293's verification, "A revealed
/// attacker's shots appear only from the edge of your vision" (answering Q0083 point 4, which M4-O built as drawing
/// these whole). Player 2's Peacekeeper on the high ground (14, 2) shoots player 1's on (9, 2) from beyond player 1's
/// sight, and is revealed to player 1 (`shots.rs`). Then two projectiles leave it for player 1's unit: one locked on
/// the unit, one at the ground there. Player 1 sees each only over its Visible tiles, from the fog edge on, as one from
/// a shooter not revealed: not a twentieth of the way, over tile (13, 2), which it doesn't see; nineteen twentieths of
/// the way, over (9, 2), which it does. Player 2 sees both whole, and neither shows once it has landed.
#[test]
fn a_revealed_shooters_projectile_at_your_object_shows_only_over_your_visible_tiles() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    app.update();
    let (own, shooter) = (ObjectId(1), ObjectId(2));
    order(&mut app, P2, shooter, OrderKind::Attack(own));
    let ordered = tick(&app);
    while app.world().resource::<CombatEffects>().tracers.is_empty() {
        assert!(tick(&app) < ordered + 20, "player 2 never fired");
        app.update();
    }
    order(&mut app, P2, shooter, OrderKind::Stop);
    let launched = tick(&app) + 1;
    let lands_at = launched + 20;
    let (from, to) = (centre(14, 2), centre(9, 2));
    for target in [Some(own), None] {
        let event = SimEvent::ProjectileLaunched {
            shooter,
            target,
            from,
            to,
            lands_at,
            homing: false,
        };
        let tick = tick(&app) + 1;
        app.world_mut()
            .resource_mut::<Messages<SimEventMessage>>()
            .write(SimEventMessage { tick, event });
    }
    app.update();
    {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let flights: &[Flight] = &world.resource::<CombatEffects>().flights;
        assert_eq!(flights.len(), 2);
        let (at_unit, at_ground) = (&flights[0], &flights[1]);
        assert_eq!((at_unit.tick, at_unit.owner), (launched, Some(P2)));
        assert_eq!((at_unit.target, at_ground.target), (Some(own), None));
        assert_eq!(at_unit.progress(sim, 1.0), 0.0, "where it left, in its own tick");
        let p1 = Sight::new(sim, P1, true);
        assert!(p1.reveals(shooter));
        // A twentieth of the way: over tile (13, 2), which player 1 doesn't see; 19 in 20: over (9, 2), which it does.
        let shows = |sight: &Sight, flight, progress| shows_flight(sim, sight, flight, progress);
        for flight in [at_unit, at_ground] {
            assert!(
                !shows(&p1, flight, 0.05) && shows(&p1, flight, 0.95),
                "clipped: {flight:?}"
            );
        }
        let p2 = Sight::new(sim, P2, true);
        assert!(
            shows(&p2, at_unit, 0.05) && shows(&p2, at_ground, 0.05),
            "the shooter's own"
        );
    }
    run_until(&mut app, lands_at);
    assert!(app.world().resource::<CombatEffects>().flights.is_empty(), "landed");
}

/// A flat 24 × 12 map: player 2 (the Syndicate) has a Hideout on (10, 4)–(11, 5) and a Peacekeeper on (13, 5); player
/// 1 a Peacekeeper on (8, 5), whose sight takes in the Hideout's tiles, which player 1 doesn't see for all that
/// (plan decision T32). Auto-targeting off. The Hideout is 2, the Peacekeepers 1 and 3.
fn hideout_world() -> App {
    let map = vec![".".repeat(24); 12].join("\n");
    let scenario = ScenarioBuilder::new(&map).faction(P2, Faction::Syndicate).build();
    let mut app = headless(AppConfig::new(scenario));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 8, 5);
    debug(
        &mut app,
        DebugCommand::Spawn {
            kind: ObjectKind::Hideout,
            owner: Some(P2),
            at: SpawnAt::Tile(TilePos::new(10, 4)),
            orientation: Default::default(),
        },
    );
    spawn_unit(&mut app, P2, 13, 5);
    app.update();
    app
}

/// Plan decision T52: "Nothing is drawn for a hidden victim"; Acceptance: "The shooter's owner sees the blast where
/// they have vision, and nothing about the hidden Hideout it damaged: no hit marker, no death marker, no HP bar."
/// Player 2's own Peacekeeper damages its Hideout (a force-attack), as a blast would. Player 1, whose unit sees the
/// tiles all round it, sees a blast over the Hideout, and of the Hideout no HP bar, no death marker and no hit;
/// player 2 sees the blast, the HP bar and the death marker. One more blast, in the fog east of player 1's sight,
/// shows to nobody: player 2 has no sight there either.
#[test]
fn a_hidden_hideout_under_a_blast_shows_nothing_to_the_other_players() {
    let mut app = hideout_world();
    let (hideout, own_shooter) = (ObjectId(2), ObjectId(3));
    order(&mut app, P2, own_shooter, OrderKind::Attack(hideout));
    let ordered = tick(&app);
    let hp = |app: &App| app.world().resource::<SimRes>().object(hideout).and_then(|o| o.hp);
    while hp(&app) == Some(400.0) {
        assert!(tick(&app) < ordered + 40, "the Hideout took no damage");
        app.update();
    }
    order(&mut app, P2, own_shooter, OrderKind::Stop);
    let (at, hiding) = {
        let o = app.world().resource::<SimRes>().object(hideout).unwrap();
        (o.pos, Hiding::of(o))
    };
    let blasted = tick(&app) + 1;
    hand(
        &mut app,
        SimEvent::Blast {
            at,
            radius: 40.0,
            attacker: None,
        },
    );
    hand(
        &mut app,
        SimEvent::Blast {
            at: centre(22, 5),
            radius: 40.0,
            attacker: None,
        },
    );
    let tracers_before = app.world().resource::<CombatEffects>().tracers.len();
    hand(
        &mut app,
        SimEvent::Hit {
            attacker: ObjectId(1),
            target: hideout,
            damage: 92.6,
            shield_damage: 0.0,
        },
    );
    hand(
        &mut app,
        SimEvent::Died {
            id: hideout,
            kind: ObjectKind::Hideout,
            owner: Some(P2),
            pos: at,
            hiding,
        },
    );
    {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let effects = world.resource::<CombatEffects>();
        assert_eq!(effects.tracers.len(), tracers_before, "no hit is drawn");
        let (p1, p2) = (Sight::new(sim, P1, true), Sight::new(sim, P2, true));
        assert_eq!(p1.at(at), VisibilityState::Visible);
        assert!(!p1.sees(sim.object(hideout).unwrap()), "hidden from player 1");
        let (over, east) = (&effects.blasts[0], &effects.blasts[1]);
        assert_eq!(over.tick, blasted);
        assert!(
            shows_blast(sim, &p1, over, &space_crystals::tuning::ClientTuning::BUILTIN)
                && shows_blast(sim, &p2, over, &space_crystals::tuning::ClientTuning::BUILTIN)
        );
        assert!(
            !shows_blast(sim, &p1, east, &space_crystals::tuning::ClientTuning::BUILTIN)
                && !shows_blast(sim, &p2, east, &space_crystals::tuning::ClientTuning::BUILTIN),
            "in the fog"
        );
        let bars = |sight: &Sight| {
            hp_bars(sim, sight, 1.0, &space_crystals::tuning::ClientTuning::BUILTIN, |_| {
                false
            })
            .len()
        };
        assert_eq!(
            (bars(&p1), bars(&p2)),
            (0, 1),
            "the Hideout's HP bar, to its owner only"
        );
        let death = effects.deaths.last().expect("the death is collected");
        assert_eq!(death.kind, ObjectKind::Hideout);
        assert!(!shows_death(&p1, death), "no death marker for a hidden victim");
        assert!(shows_death(&p2, death), "its owner's");
        assert!(shows_death(&Sight::new(sim, P1, false), death), "the fog off");
    }
    // The blasts fade `FeedbackTuning::blast_ticks` after their ticks, the second a tick after the first.
    run_until(
        &mut app,
        blasted + space_crystals::tuning::feedback::BUILTIN.blast_ticks,
    );
    assert_eq!(app.world().resource::<CombatEffects>().blasts.len(), 1);
    app.update();
    assert!(app.world().resource::<CombatEffects>().blasts.is_empty(), "faded");
}
