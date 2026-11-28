//! Completion (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`; design changes 0229, 0231 and 0258):
//! after 480 frames the Tunnel is at the target tier, its HP rising with its MaxHP, its Tunnel Space the tier's; it
//! takes back a neutral expansion its new area now covers; and a game saved mid-upgrade goes on alike.

use sc_sim::{Replay, Sim};

use super::*;

/// `tunnel`'s HP and MaxHP.
fn hp(t: &TestSim, tunnel: ObjectId) -> (Option<f32>, Option<f32>) {
    let o = object(t, tunnel);
    (o.hp, o.max_hp())
}

/// Design change 0229's verification: "A Tunnel upgrade takes 30 s". Ordered in tick 2, [`T1`]'s upgrade does a frame
/// a tick from then, as a recipe (`docs/architecture/economy.md > Economy > Recipe timing`): the Tunnel is at Tier 1 through tick
/// 480 and at Tier 2 at the end of tick 481, the 480th; its MaxHP and HP rise together, 600/600 to 800/800
/// (`factions/syndicate/tunnels.md > Tunnel > Tiers`), and its Tunnel Space by 9 (its region from 48 cells to 84, a
/// quarter each). The same again to Tier 3: 1000/1000, and 32.
#[test]
fn an_upgrade_takes_480_frames_and_the_hp_and_tunnel_space_rise() {
    let mut t = started(base(), 20);
    assert_eq!(
        (hp(&t, T1), tunnel_space(&t, P1)),
        ((Some(600.0), Some(600.0)), (0, 12))
    );
    next(&mut t, P1, upgrade(T1));
    let ordered = t.sim().tick();
    assert_eq!(ordered, 2);
    while t.sim().tick() < ordered + FRAMES - 1 {
        let now = t.sim().tick();
        assert_eq!((tier(&t, T1), upgrading(&t, T1)), (1, Some(2)), "tick {now}");
        assert_eq!((hp(&t, T1), tunnel_space(&t, P1).1), ((Some(600.0), Some(600.0)), 12));
        t.step();
    }
    assert_eq!((tier(&t, T1), upgrading(&t, T1)), (2, None));
    assert_eq!(completions(&t), [(481, T1, 2)]);
    assert_eq!(
        (hp(&t, T1), tunnel_space(&t, P1)),
        ((Some(800.0), Some(800.0)), (0, 21))
    );

    next(&mut t, P1, upgrade(T1));
    assert_eq!(starts(&t)[1], (T1, 3, 3));
    t.run(FRAMES);
    assert_eq!(completions(&t)[1], (482 + FRAMES - 1, T1, 3));
    assert_eq!(
        (hp(&t, T1), tunnel_space(&t, P1)),
        ((Some(1000.0), Some(1000.0)), (0, 32))
    );
}

/// Design change 0231 point 4: "on completion, current HP rises with max HP (damage stays)". Player 2's Guard damages
/// [`T1`] and goes; the upgrade then takes [`T1`] from its HP to 200 more, out of 800. (The verification's own numbers,
/// 500/600 to 700/800, are the sim's unit test: `sim/upgrades.rs`.)
#[test]
fn the_damage_stays_through_the_upgrade() {
    let scenario = base()
        .spawn(ObjectKind::Guard, Some(P2), point(9, 14)) // 2
        .spawn(TUNNEL, Some(P2), tile(30, 2)); // 3, which keeps player 2 in the game
    let mut t = started(scenario, 20);
    let attack = Command::Order {
        units: vec![id(2)],
        kind: OrderKind::Attack(T1),
        queue: false,
    };
    next(&mut t, P2, attack);
    step_until(&mut t, 400, "the Guard hits the Tunnel", |t| {
        object(t, T1).hp.is_some_and(|hp| hp < 560.0)
    });
    next(&mut t, P2, remove(id(2)));
    let (damaged, _) = hp(&t, T1);
    let damaged = damaged.expect("HP");
    next(&mut t, P1, upgrade(T1));
    t.run(FRAMES);
    assert_eq!(tier(&t, T1), 2);
    assert_eq!(hp(&t, T1), (Some(damaged + 200.0), Some(800.0)));
}

/// `factions/syndicate/tunnels.md > Losing Tunnels` (design change 0258): "**Reclaiming happens** the moment the
/// covering Tunnel, or its upgrade, **finishes**". Player 1 loses [`T1`]: its Hideout (2), on (8, 8) to (9, 9), turns
/// neutral. Player 1's Tunnel 4, spawned on (11, 8) to (14, 11), covers only the Hideout's east column with its Tier 1
/// area, (9, 6) to (16, 13): the Hideout stays neutral, and decays, through the upgrade ("a Tunnel that is
/// mid-upgrade counts at its current (pre-upgrade) tier"). Its Tier 2 area, (8, 5) to (17, 14), covers it all: in the
/// tick the upgrade completes the Hideout is player 1's again, and decays no more.
#[test]
fn an_upgrade_now_covering_a_neutral_expansion_takes_it_back_as_it_completes() {
    let scenario = base()
        .spawn(ObjectKind::Hideout, Some(P1), tile(8, 8)) // 2
        .spawn(TUNNEL, Some(P1), tile(30, 8)); // 3, which keeps player 1 in the game
    let mut t = started(scenario, 20);
    let hideout = id(2);
    next(&mut t, P1, remove(T1));
    assert_eq!(object(&t, hideout).owner, None, "neutral");
    let spawn = DebugCommand::Spawn {
        kind: TUNNEL,
        owner: Some(P1),
        at: tile(11, 8),
        orientation: BuildingOrientation::default(),
    };
    next(&mut t, P1, Command::Debug(spawn));
    let covering = id(4);
    assert_eq!(object(&t, covering).kind, TUNNEL);
    t.run(2);
    assert_eq!(object(&t, hideout).owner, None, "partly covered");
    next(&mut t, P1, upgrade(covering));
    let ordered = t.sim().tick();
    while t.sim().tick() < ordered + FRAMES - 1 {
        let h = object(&t, hideout);
        assert_eq!((h.owner, h.decay.is_some()), (None, true), "tick {}", t.sim().tick());
        t.step();
    }
    assert_eq!(completions(&t), [(ordered + FRAMES - 1, covering, 2)]);
    let h = object(&t, hideout);
    assert_eq!((h.owner, h.decay), (Some(P1), None));
}

/// A game with upgrades under way, one cancelled and one completed, goes on alike after a save at every stage, tick by
/// tick, and its replay plays back to the same state (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`).
#[test]
fn a_game_saved_mid_upgrade_goes_on_alike_and_replays() {
    let mut t = started(base().spawn(TUNNEL, Some(P1), tile(24, 8)), 20);
    t.at(2, P1, upgrade(T1))
        .at(2, P1, upgrade(id(2)))
        .at(200, P1, cancel(id(2)));
    let mut saves: Vec<(Tick, Sim)> = Vec::new();
    while t.sim().tick() < 500 {
        if [1, 2, 150, 200, 480].contains(&t.sim().tick()) {
            let saved = serde_json::to_string(t.sim()).expect("saved");
            let loaded: Sim = serde_json::from_str(&saved).expect("loaded");
            assert_eq!(&loaded, t.sim(), "tick {}", t.sim().tick());
            saves.push((t.sim().tick(), loaded));
        }
        t.step();
        for (since, loaded) in &mut saves {
            loaded.step(t.replay().inputs.last().expect("a tick's inputs"));
            assert_eq!(
                loaded.state_hash(),
                t.sim().state_hash(),
                "tick {}, saved in {since}",
                t.sim().tick()
            );
        }
    }
    assert_eq!(completions(&t), [(481, T1, 2)]);
    assert_eq!(cancels(&t), [(id(2), 2, 4)]);
    let json = serde_json::to_string(t.replay()).expect("saved");
    let replay: Replay = serde_json::from_str(&json).expect("loaded");
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
