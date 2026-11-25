//! Attacks from outside vision (`core/vision.md > Attacks from outside vision`, design change 0177, answering question
//! Q0017 point 1): an attacker its victim's owner doesn't see is revealed to that player for [`REVEAL_TICKS`] after
//! each shot, so one that keeps firing stays Visible. Only the attacker is revealed, not its tiles; while revealed it
//! is a normal visible object: it can be ordered as a target, and auto-targeting picks it. The reveals are hashed,
//! saved and replayed. `TestSim` checks V1 after every tick.

use sc_sim::orders::{Auto, Behaviour};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::vision::REVEAL_TICKS;
use sc_sim::{
    Command, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Sim, SimEvent, Tick, TickInputs,
    TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// 24 × 5 tiles: level 0 in the west, a ramp on column 12, level 1 from column 13 on. From level 0, a unit's sight
/// reaches a grid unit less far onto level 1, and from level 1 its attack reaches a grid unit farther down
/// (`core/vision.md > ElevationModifier`).
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

/// The objects of [`step`], by id.
const VICTIM: u32 = 1;
const SNIPER: u32 = 2;

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// Player 1's `victim` stands at `at` on level 0; player 2's Peacekeeper, the sniper, on tile (14, 2), level 1,
/// beyond player 1's sight there (4 grid units from tile (9, 2)), but its attack reaches 5 grid units down. With
/// `auto_targeting` off, the sniper fires once ordered to ([`order_fire`]); on, it acquires the victim itself.
fn step(victim: ObjectKind, at: Vec2, auto_targeting: bool) -> TestSim {
    let mut t = ScenarioBuilder::new(STEP).start();
    if !auto_targeting {
        t.without_auto_targeting();
    }
    t.spawn_unit_at(1, victim, P1, at)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(14, 2))
        .run_until(1);
    assert!(!sees(&t, P1, SNIPER), "out of player 1's sight");
    assert!(sees(&t, P2, VICTIM));
    t
}

/// Orders the sniper to attack the victim in the next tick, and runs until its first shot lands.
fn order_fire(t: &mut TestSim) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P2, attack(P2, SNIPER, VICTIM).command);
    while !shot(t) {
        assert!(t.sim().tick() < 100, "the sniper never fired");
        t.step();
    }
}

fn sees(t: &TestSim, player: PlayerId, id: u32) -> bool {
    t.sim().is_visible(player, t.sim().object(ObjectId(id)).unwrap())
}

fn attack(player: PlayerId, unit: u32, target: u32) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: vec![ObjectId(unit)],
            kind: OrderKind::Attack(ObjectId(target)),
            queue: false,
        },
    }
}

/// Whether the sniper's shot landed in the latest tick.
fn shot(t: &TestSim) -> bool {
    let tick = t.sim().tick();
    t.events().iter().any(|(at, event)| {
        *at == tick && matches!(event, SimEvent::Attacked { attacker, .. } if *attacker == ObjectId(SNIPER))
    })
}

/// The acceptance's "an enemy on unseen high ground shoots your unit, and is visible to you for 24 ticks after each
/// shot; continuous fire keeps it visible; it disappears 24 ticks after the last shot". Player 2's Peacekeeper fires
/// at player 1's Guard from the fog, every few ticks; after its fourth shot it stops. Tick by tick, player 1 sees it
/// exactly while a shot landed within the last `REVEAL_TICKS` ticks, the shot's own included: all through the
/// firing, as the shots come closer together than that, and until `REVEAL_TICKS` after the last. Its tile stays
/// unseen, and an Attack on it is taken exactly while it is seen. Auto-targeting is off: the Guard doesn't react.
#[test]
fn an_attacker_on_unseen_high_ground_is_revealed_after_each_shot() {
    let mut t = step(ObjectKind::Guard, centre(9, 2), false);
    let tick = t.sim().tick() + 1;
    t.at(tick, P2, attack(P2, SNIPER, VICTIM).command);
    let mut shots: Vec<Tick> = Vec::new();
    let mut stop_at = None;
    loop {
        t.step();
        let tick = t.sim().tick();
        assert!(tick < 300, "the sniper never fired four times");
        if shot(&t) {
            shots.push(tick);
            if shots.len() == 4 {
                let stop = Command::Order {
                    units: vec![ObjectId(SNIPER)],
                    kind: OrderKind::Stop,
                    queue: false,
                };
                t.at(tick + 1, P2, stop);
                stop_at = Some(tick);
            }
        }
        let revealed = shots.last().is_some_and(|&last| tick - last < REVEAL_TICKS);
        assert_eq!(sees(&t, P1, SNIPER), revealed, "tick {tick}, shots {shots:?}");
        let player = t.sim().vision().player(P1).unwrap();
        assert_eq!(player.is_revealed(ObjectId(SNIPER)), revealed);
        assert_ne!(
            player.state(TilePos::new(14, 2)),
            VisibilityState::Visible,
            "only the attacker"
        );
        let verdict = if revealed {
            Ok(())
        } else {
            Err(Reject::NotVisible(ObjectId(SNIPER)))
        };
        assert_eq!(t.sim().validate(&attack(P1, VICTIM, SNIPER)), verdict, "tick {tick}");
        if stop_at.is_some_and(|stopped| tick > stopped + REVEAL_TICKS + 5) {
            break;
        }
    }
    assert_eq!(shots.len(), 4, "none after the Stop");
    assert!(
        shots.windows(2).all(|pair| pair[1] - pair[0] < REVEAL_TICKS),
        "continuous fire: {shots:?}"
    );
    assert!(t.sim().object(ObjectId(VICTIM)).is_some(), "the Guard took it");
}

/// `> Attacks from outside vision`: "While revealed it is a normal visible object for you: it can be seen, selected
/// and targeted". Right after the first shot, player 1 orders the Guard to attack the sniper: the gate takes it, and
/// the Guard sets off up the ramp after it.
#[test]
fn a_revealed_attacker_can_be_ordered_as_a_target() {
    let mut t = step(ObjectKind::Guard, centre(9, 2), false);
    order_fire(&mut t);
    let order = attack(P1, VICTIM, SNIPER);
    assert_eq!(t.sim().validate(&order), Ok(()));
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order.command).step();
    let guard = t.sim().object(ObjectId(VICTIM)).unwrap();
    let current = guard.orders.as_ref().unwrap().current.as_ref().unwrap();
    assert_eq!(current.order.kind, OrderKind::Attack(ObjectId(SNIPER)));
    assert!(matches!(&current.state, Behaviour::Attack(state) if state.chasing && state.seen));
    let from = guard.pos;
    t.run(5);
    assert!(t.sim().object(ObjectId(VICTIM)).unwrap().pos.x > from.x, "up the ramp");
}

/// The acceptance's "your units' auto-targeting can pick it while revealed" (design change 0177: "Auto-targeting /
/// acquisition can use the revealed attacker like any visible unit"). Player 1's idle Peacekeeper stands 268 su
/// from the sniper, within its idle scan (SightRange 4 grid units up to level 1, edge to edge) but not with the
/// sniper's tile in sight (centre to centre). It doesn't acquire the sniper until the sniper, which acquired it,
/// fires: then it does, in the next tick, and goes after it.
#[test]
fn auto_targeting_picks_a_revealed_attacker() {
    let mut t = step(ObjectKind::Peacekeeper, Vec2::new(660.0, centre(9, 2).y), true);
    let auto = |t: &TestSim| t.sim().object(ObjectId(VICTIM)).unwrap().orders.as_ref().unwrap().auto;
    while !shot(&t) {
        assert!(t.sim().tick() < 100, "the sniper never fired");
        assert!(matches!(auto(&t), Auto::Watching { .. }), "nothing it sees");
        t.step();
    }
    assert!(sees(&t, P1, SNIPER));
    t.step();
    assert!(
        matches!(auto(&t), Auto::Engaging { pursuit, .. } if pursuit.target == ObjectId(SNIPER)),
        "{:?}",
        auto(&t)
    );
}

/// The reveals are state: a game saved while one runs loads equal, hashes the same and plays on the same, tick for
/// tick, through the reveal's end; a replay ends the same.
#[test]
fn saves_and_replays_keep_the_reveals() {
    let mut t = step(ObjectKind::Guard, centre(9, 2), false);
    order_fire(&mut t);
    let json = serde_json::to_string(t.sim()).unwrap();
    let ends = t.sim().tick() + REVEAL_TICKS;
    assert!(
        json.contains(&format!("\"revealed\":[{{\"id\":2,\"ends\":{ends}}}]")),
        "{json}"
    );
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert!(loaded.is_visible(P1, loaded.object(ObjectId(SNIPER)).unwrap()));
    for _ in 0..2 * REVEAL_TICKS {
        t.step();
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}
