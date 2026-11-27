//! The View's fog rule (`core/game_rules.md > Computer players (playtest tool)`: "a computer player sees only what its
//! own objects see and remembers what it has seen ... It never sees the whole map"): a View never shows an object of
//! another player that its player neither sees nor remembers, and shows every one it does.

use sc_sim::testing::TestSim;
const FOG_DELAY_TICKS: sc_sim::Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;
use sc_sim::{DebugCommand, ObjectKind, PlayerId, Sim, Vec2, gu, scenarios};

use super::{Known, Start, View};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// That `player`'s View shows exactly what the player sees and remembers: each object it shows of another player or
/// none is one the player sees now, as it is, or one it remembers out of its sight, as it last saw it; and each object
/// the player sees or remembers is shown. Returns how many objects of the other player it shows.
fn fair(sim: &Sim, starts: &[Start], player: PlayerId) -> usize {
    let tick = sim.tick();
    let view = View::new(sim, player, starts);
    let vision = sim.vision().player(player).expect("the player's vision");
    let known = view.known();
    for k in known {
        assert_ne!(
            k.owner,
            Some(player),
            "tick {tick}: player {} shown its own {:?}",
            player.0,
            k.id
        );
        if k.in_sight {
            let o = sim.object(k.id).expect("an object in sight is in the game");
            assert!(
                vision.sees(o),
                "tick {tick}: player {} shown {:?}, unseen",
                player.0,
                k.id
            );
            assert_eq!(
                (k.pos, k.hp, k.seen),
                (o.pos, o.hp, tick),
                "tick {tick}: {:?} as it is",
                k.id
            );
        } else {
            let record = vision.remembered_record(k.id);
            let record =
                record.unwrap_or_else(|| panic!("tick {tick}: player {} shown {:?}, not remembered", player.0, k.id));
            assert_eq!(
                (k.pos, k.hp, k.seen),
                (record.pos(), record.hp, record.seen),
                "tick {tick}: {:?} as last seen",
                k.id
            );
        }
    }
    let shown = |id| known.iter().any(|k| k.id == id);
    for o in sim.objects().filter(|o| o.owner != Some(player) && vision.sees(o)) {
        assert!(
            shown(o.id),
            "tick {tick}: player {} sees {:?}, not shown",
            player.0,
            o.id
        );
    }
    for r in vision.remembered() {
        assert!(
            shown(r.id),
            "tick {tick}: player {} remembers {:?}, not shown",
            player.0,
            r.id
        );
    }
    known.iter().filter(|k| k.owner.is_some_and(|p| p != player)).count()
}

/// On `dev_skirmish`, player 1's scout goes to player 2's base and is removed there: player 1 knows nothing of player
/// 2 until the scout sees the base, then its buildings and its unit as they are, and once the scout is gone only the
/// buildings, as they were when last seen, not the unit. Player 2 knows the scout only while it sees it. Checked for
/// both players at every tick.
#[test]
fn a_view_shows_only_what_its_player_sees_or_remembers() {
    let scenario = scenarios::dev_skirmish();
    let starts = Start::all(&scenario);
    assert_eq!(starts.len(), 2, "two players");
    let mut t = TestSim::new(scenario);
    let dc = |sim: &Sim, owner| {
        let dc = sim
            .objects()
            .find(|o| o.owner == Some(owner) && o.kind == ObjectKind::DeploymentCenter);
        dc.expect("a Deployment Center").pos
    };
    let theirs = dc(t.sim(), P2);
    // Player 2's Peacekeeper beside its base; then, in tick 3, player 1's scout 5 grid units off it.
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P2, theirs + Vec2::new(0.0, -gu(3.0)));
    t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
    let mut before = Vec::new();
    for _ in 0..2 {
        t.step();
        before.push((fair(t.sim(), &starts, P1), fair(t.sim(), &starts, P2)));
    }
    assert_eq!(before, [(0, 0), (0, 0)], "nobody knows the other before the scout");
    let view = View::new(t.sim(), P1, &starts);
    assert_eq!(
        view.patches().count(),
        16,
        "every patch the map placed is known from the start"
    );
    assert_eq!(view.enemy_structures().count(), 0);
    t.spawn_unit_at(3, ObjectKind::Peacekeeper, P1, theirs + Vec2::new(-gu(5.0), -gu(3.0)));
    t.run(3);
    let scout = t
        .sim()
        .objects()
        .find(|o| o.owner == Some(P1) && o.kind == ObjectKind::Peacekeeper);
    let scout = scout.expect("the scout").id;
    let seen = fair(t.sim(), &starts, P1);
    let view = View::new(t.sim(), P1, &starts);
    let structures: Vec<Known> = view.enemy_structures().cloned().collect();
    assert!(
        seen > structures.len() && !structures.is_empty(),
        "the scout sees the base and its unit: {seen}"
    );
    assert!(structures.iter().all(|k| k.in_sight));
    assert_eq!(fair(t.sim(), &starts, P2), 1, "player 2 sees the scout");
    let removed = t.sim().tick() + 1;
    t.debug_at(removed, DebugCommand::Remove { id: scout });
    for _ in 0..32 {
        t.step();
        fair(t.sim(), &starts, P1);
        fair(t.sim(), &starts, P2);
    }
    let view = View::new(t.sim(), P1, &starts);
    let remembered: Vec<Known> = view.enemy_structures().cloned().collect();
    assert_eq!(remembered.len(), structures.len(), "the buildings are remembered");
    // Last seen as the scout's sight faded: the tiles stay Visible for the fog delay after it is gone.
    let faded = removed + FOG_DELAY_TICKS;
    assert!(
        remembered
            .iter()
            .all(|k| !k.in_sight && (removed..=faded).contains(&k.seen)),
        "{remembered:?}"
    );
    assert_eq!(fair(t.sim(), &starts, P1), structures.len(), "but not the unit");
    assert_eq!(fair(t.sim(), &starts, P2), 0, "the scout is gone");
}

/// The start locations are the slots' starts, each player's its own slot's; the enemies' leave out the player's.
#[test]
fn every_player_knows_every_start() {
    let scenario = scenarios::dev_skirmish();
    let starts = Start::all(&scenario);
    let slots: Vec<_> = scenario.map.slots.iter().map(|s| s.start).collect();
    assert_eq!(starts.iter().map(|s| s.tile).collect::<Vec<_>>(), slots);
    let t = TestSim::new(scenario);
    let view = View::new(t.sim(), P2, &starts);
    assert_eq!(view.my_start(), Some(starts[1]));
    assert_eq!(view.enemy_starts().collect::<Vec<_>>(), [starts[0]]);
    assert_eq!(starts[0].point(), slots[0].center());
    assert!(view.in_game());
    let own: Vec<ObjectKind> = view.own().map(|o| o.kind).collect();
    assert!(own.contains(&ObjectKind::DeploymentCenter), "{own:?}");
    assert!(view.own_object(view.own().next().expect("an object").id).is_some());
    let theirs = t
        .sim()
        .objects()
        .find(|o| o.owner == Some(P1))
        .expect("player 1's object")
        .id;
    assert!(
        view.own_object(theirs).is_none(),
        "another player's object is not its own"
    );
    assert_eq!(
        view.me().map(|p| p.crystals),
        Some(100),
        "the GDO's Starting Conditions"
    );
}
