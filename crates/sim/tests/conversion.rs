//! Weapon Conversion in the game (slice M3-B; `factions/colonists/units.md > Weapon Conversion`, plan decision T26):
//! its timing and what the unit keeps, who may convert, a mixed group, no cancel and no shelter, the orders that go on
//! (`> Orders are kept`, design change 0150), saves and replays. Its pieces are `conversion`'s unit tests. `TestSim`
//! checks the movement invariants after every tick.

use sc_sim::conversion::CONVERT_TICKS;
use sc_sim::shields::SHIELD_REGEN_PER_TICK;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, AttackPhase, Command, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    Replay, Sim, SimEvent, SpawnAt, Target, Tick, TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const RIFLE: ObjectKind = ObjectKind::RifleLegionnaire;
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn ids(ns: &[u32]) -> Vec<ObjectId> {
    ns.iter().map(|&n| id(n)).collect()
}

fn object(sim: &Sim, n: u32) -> &Object {
    sim.object(id(n)).expect("the object exists")
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// A 16 × 6 flat map for two Colonists players (plan decision T27).
fn builder() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(16, 6))
        .faction(P1, Faction::Colonists)
        .faction(P2, Faction::Colonists)
}

/// `Command::Ability` with `Ability::Convert` for units `ns`.
fn convert(ns: &[u32]) -> Command {
    Command::Ability {
        units: ids(ns),
        ability: Ability::Convert,
    }
}

/// What the gate says to player 1's Convert of units `ns` now.
fn check(sim: &Sim, ns: &[u32]) -> Result<(), Reject> {
    sim.validate(&PlayerCommand {
        player: P1,
        command: convert(ns),
    })
}

/// `sim` saved, with object `n`'s `field` set to `value`, and loaded: a state no command makes on demand.
fn edited(sim: &Sim, n: u32, field: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o[field] = value;
    serde_json::from_value(state).unwrap()
}

/// One tick of `sim` with player 1's `commands`.
fn step(sim: &mut Sim, commands: Vec<Command>) {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player: P1, command })
        .collect();
    sim.step(&TickInputs { commands });
}

/// The hits so far: tick, attacker, target.
fn hits(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Hit { attacker, target, .. } => Some((*tick, *attacker, *target)),
            _ => None,
        })
        .collect()
}

fn phase(sim: &Sim, n: u32) -> Option<AttackPhase> {
    object(sim, n).attack.as_ref().and_then(|attack| attack.phase)
}

/// The plan's acceptance: "A Sabre becomes a Rifle in 4 ticks, keeping HP and Shields" (`> Convert Command`: "The
/// unit converts in place over 4 frames"; `> HP/Shields`: "current HP/Shields values carry over unchanged"). Player
/// 1's Sabre, at 35 HP and 3 Shields, converts in tick 2: it is a Sabre converting, frame by frame, through tick 5,
/// and the same object is a Rifle from tick 6 on, with a `KindChanged` in that tick only. Its HP stays 35; its Shields
/// regenerate all along, 0.25 a tick. It stood still throughout, and can move again.
#[test]
fn a_sabre_becomes_a_rifle_in_four_ticks_keeping_its_hp_and_shields() {
    let mut t = builder().spawn(SABRE, Some(P1), point(300.0, 192.0)).start();
    t.run_until(1);
    let sim = edited(t.sim(), 1, "hp", serde_json::json!(35.0));
    let mut sim = edited(&sim, 1, "shields", serde_json::json!(3.0));
    let at = object(&sim, 1).pos;
    assert_eq!(check(&sim, &[1]), Ok(()));
    for tick in 2..=6 {
        step(&mut sim, if tick == 2 { vec![convert(&[1])] } else { Vec::new() });
        let o = object(&sim, 1);
        let frames = tick - 1;
        let kind_changed = sim.events().iter().any(|e| matches!(e, SimEvent::KindChanged { .. }));
        if frames <= CONVERT_TICKS {
            assert_eq!(o.kind, SABRE, "tick {tick}");
            assert_eq!(o.conversion.map(|c| c.frames_done()), Some(frames), "tick {tick}");
            assert!(!kind_changed, "tick {tick}");
        } else {
            assert_eq!((o.kind, o.conversion), (RIFLE, None), "tick {tick}");
            assert_eq!(
                sim.events(),
                [SimEvent::KindChanged {
                    id: id(1),
                    from: SABRE,
                    to: RIFLE,
                }]
            );
        }
        assert_eq!((o.pos, o.hp), (at, Some(35.0)), "tick {tick}");
        assert_eq!(
            o.shields,
            Some(3.0 + frames as f32 * SHIELD_REGEN_PER_TICK),
            "tick {tick}"
        );
    }
    let mover = object(&sim, 1).mover.as_ref().unwrap();
    assert!(mover.can_move(5.0), "at full speed again");
}

/// The plan's acceptance: "The command is rejected in Firing and Cooldown, and accepted while Aiming, Reloading,
/// moving or idle" (`> Eligibility`: "interrupting that phase immediately"). Player 1's Rifle 1 shoots player 2's
/// Peacekeeper 2, which stands by, through a whole attack sequence: the gate takes a Convert of it when it is idle,
/// Aiming or Reloading, and refuses it in Firing and Cooldown (`Reject::CannotConvert`); a group with an idle Sabre 3
/// goes to the Sabre only then (question Q0042 point 9). Converted while Reloading, its reload ends at once.
#[test]
fn convert_is_refused_while_firing_or_cooling_down() {
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 192.0)) // 1
        .spawn(PK, Some(P2), point(400.0, 192.0)) // 2, in range
        .spawn(SABRE, Some(P1), point(200.0, 300.0)) // 3
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    t.run_until(1);
    assert_eq!(check(t.sim(), &[1]), Ok(()), "idle");
    let mut seen = Vec::new();
    while seen.len() < 4 {
        assert!(t.sim().tick() < 40, "a whole sequence: {seen:?}");
        t.step();
        let Some(phase) = phase(t.sim(), 1) else {
            continue;
        };
        let expected = match phase {
            AttackPhase::Aiming { .. } | AttackPhase::Reloading => Ok(()),
            AttackPhase::Firing { .. } | AttackPhase::Cooldown => Err(Reject::CannotConvert),
        };
        assert_eq!(check(t.sim(), &[1]), expected, "{phase:?}");
        assert_eq!(check(t.sim(), &[1, 3]), Ok(()), "{phase:?}: the Sabre converts anyway");
        if expected.is_err() {
            // The group's Convert leaves the Rifle out.
            let mut sim = t.sim().clone();
            step(&mut sim, vec![convert(&[1, 3])]);
            let converting = [1, 3].map(|n| object(&sim, n).conversion.is_some());
            assert_eq!(converting, [false, true], "{phase:?}");
        }
        let name = format!("{phase:?}")
            .chars()
            .take_while(|c| c.is_alphabetic())
            .collect::<String>();
        if !seen.contains(&name) {
            seen.push(name);
        }
    }
    assert_eq!(seen, ["Aiming", "Firing", "Cooldown", "Reloading"]);
    // In Reloading now: the group's Convert goes to both, and the reload ends.
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, convert(&[1, 3])).step();
    let sim = t.sim();
    assert!(object(sim, 1).conversion.is_some() && object(sim, 3).conversion.is_some());
    assert_eq!(phase(sim, 1), None, "the reload is over");
}

/// `> Eligibility`: "it may be issued freely while the unit is [...] moving". The Convert of a moving Sabre is taken.
#[test]
fn a_moving_legionnaire_may_convert() {
    let mut t = builder().spawn(SABRE, Some(P1), point(100.0, 192.0)).start();
    let to = Vec2::new(900.0, 192.0);
    t.order_at(2, P1, &[id(1)], OrderKind::Move(Target::Point(to)), false)
        .run_until(5);
    assert_ne!(t.pos(1), Vec2::new(100.0, 192.0), "on its way");
    assert_eq!(check(t.sim(), &[1]), Ok(()));
}

/// The plan's acceptance: "A mixed group converts each unit to its other form" (`> Convert Command`: "units may be
/// mixed in selection; each converts to its respective other form"). Player 1's Sabre 1, Rifle 2 and Drone 3 are
/// sent a Convert: the Legionnaires swap forms, the Drone, which has no other form, is left out. A Convert to
/// Drones alone is refused (`Reject::LacksAbility`), and so is one to another player's Legionnaire.
#[test]
fn a_mixed_group_converts_each_unit_to_its_other_form() {
    let mut t = builder()
        .spawn(SABRE, Some(P1), point(100.0, 100.0)) // 1
        .spawn(RIFLE, Some(P1), point(300.0, 100.0)) // 2
        .spawn(ObjectKind::Drone, Some(P1), point(500.0, 100.0)) // 3
        .spawn(SABRE, Some(P2), point(700.0, 300.0)) // 4
        .start();
    t.run_until(1);
    assert_eq!(check(t.sim(), &[3]), Err(Reject::LacksAbility));
    assert_eq!(check(t.sim(), &[1, 4]), Err(Reject::NotOwned(id(4))));
    t.at(2, P1, convert(&[1, 2, 3])).run_until(1 + CONVERT_TICKS);
    let kinds = |sim: &Sim| [1, 2, 3].map(|n| object(sim, n).kind);
    assert_eq!(kinds(t.sim()), [SABRE, RIFLE, ObjectKind::Drone], "converting");
    t.step();
    assert_eq!(kinds(t.sim()), [RIFLE, SABRE, ObjectKind::Drone]);
    assert_eq!(object(t.sim(), 3).conversion, None);
}

/// The plan's acceptance: "A conversion can't be cancelled, and the unit is attackable throughout" (`> Completion`:
/// "runs to completion over its 4 frames and cannot be cancelled"; `> Vulnerability`: "no invulnerability window").
/// Player 2's Peacekeeper 2 shoots player 1's Sabre 1; the Sabre converts two ticks before a hit lands, which lands
/// while it converts. A second Convert is refused (`Reject::CannotConvert`), and a Stop, which is taken, changes
/// nothing: the Sabre is a Rifle 4 ticks after the first Convert.
#[test]
fn a_conversion_cannot_be_cancelled_and_the_unit_stays_attackable() {
    let game = || {
        let mut t = builder()
            .spawn(SABRE, Some(P1), point(200.0, 192.0)) // 1
            .spawn(PK, Some(P2), point(400.0, 192.0)) // 2
            .start();
        t.without_auto_targeting()
            .order_at(2, P2, &[id(2)], OrderKind::Attack(id(1)), false);
        t
    };
    let mut dry = game();
    dry.run_until(60);
    let landed: Vec<Tick> = hits(&dry).iter().map(|&(tick, ..)| tick).collect();
    assert!(landed.len() >= 2, "the Peacekeeper shoots: {landed:?}");
    let start = landed[1] - 2;

    let mut t = game();
    t.run_until(start - 1);
    t.at(start, P1, convert(&[1])).step();
    assert_eq!(check(t.sim(), &[1]), Err(Reject::CannotConvert), "no second Convert");
    t.order_at(start + 1, P1, &[id(1)], OrderKind::Stop, false);
    for tick in start + 1..start + CONVERT_TICKS {
        t.step();
        assert_eq!(object(t.sim(), 1).kind, SABRE, "tick {tick}: still converting");
        assert!(object(t.sim(), 1).conversion.is_some(), "tick {tick}");
    }
    assert!(
        hits(&t).contains(&(landed[1], id(2), id(1))),
        "hit while converting, in tick {}",
        landed[1]
    );
    t.step();
    assert_eq!(object(t.sim(), 1).kind, RIFLE, "4 ticks after the Convert");
    assert_eq!(object(t.sim(), 1).conversion, None);
}

/// `> Weapon Conversion > Orders are kept` (design change 0150): "The unit holds still for the 4 frames, then carries
/// on with its current orders using the other weapon". Player 1's Sabre 1, walking east, converts in tick 10: it stands
/// from tick 10 to 13 with its Move order still on, then walks on as a Rifle from tick 14 and arrives. Player 1's
/// Rifle, shooting player 2's Peacekeeper from within its 4 grid units, converts while Reloading: as a Sabre it closes
/// in on the same target and strikes it (Damage 15 less PointArmor 1), its Attack order unchanged.
#[test]
fn orders_stay_and_the_unit_carries_on_with_its_other_weapon() {
    let to = Vec2::new(900.0, 100.0);
    let mut t = builder().spawn(SABRE, Some(P1), point(100.0, 100.0)).start();
    t.order_at(2, P1, &[id(1)], OrderKind::Move(Target::Point(to)), false)
        .run_until(9);
    let stood = t.pos(1);
    t.at(10, P1, convert(&[1]));
    for tick in 10..=13 {
        t.step();
        assert_eq!(t.pos(1), stood, "tick {tick}: holds still");
        let order = object(t.sim(), 1)
            .orders
            .as_ref()
            .unwrap()
            .current
            .as_ref()
            .map(|a| a.order.kind);
        assert_eq!(order, Some(OrderKind::Move(Target::Point(to))), "tick {tick}");
    }
    t.step();
    assert_eq!(object(t.sim(), 1).kind, RIFLE);
    assert_ne!(t.pos(1), stood, "walks on");
    while t.pos(1) != to {
        assert!(t.sim().tick() < 400, "arrives");
        t.step();
    }

    // The Rifle: wait for its reload after the first shot, then convert.
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 300.0)) // 1
        .spawn(PK, Some(P2), point(420.0, 300.0)) // 2
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    while phase(t.sim(), 1) != Some(AttackPhase::Reloading) {
        assert!(t.sim().tick() < 60, "the Rifle shoots and reloads");
        t.step();
    }
    let rifle_hits = hits(&t).len();
    assert!(rifle_hits >= 1);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, convert(&[1])).run_until(tick + CONVERT_TICKS);
    assert_eq!(object(t.sim(), 1).kind, SABRE);
    let order = object(t.sim(), 1)
        .orders
        .as_ref()
        .unwrap()
        .current
        .as_ref()
        .map(|a| a.order.kind);
    assert_eq!(order, Some(OrderKind::Attack(id(2))), "the same order");
    while hits(&t).len() == rifle_hits {
        assert!(t.sim().tick() < tick + 80, "the Sabre strikes");
        t.step();
    }
    let strike = t.events().iter().rev().find_map(|(_, e)| match e {
        SimEvent::Hit { attacker, damage, .. } if *attacker == id(1) => Some(*damage),
        _ => None,
    });
    assert_eq!(strike, Some(14.0), "a Sabre's strike");
}

/// A game saved in the middle of a conversion loads and goes on exactly as the running one, and the whole run, a
/// mixed group's conversions and a fight included, replays to the same state, also after a JSON round trip.
#[test]
fn conversions_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = builder()
        .spawn(SABRE, Some(P1), point(200.0, 192.0)) // 1
        .spawn(RIFLE, Some(P1), point(200.0, 300.0)) // 2
        .spawn(PK, Some(P2), point(420.0, 250.0)) // 3
        .start();
    t.order_at(2, P1, &[id(1), id(2)], OrderKind::Attack(id(3)), false)
        .at(12, P1, convert(&[1, 2]))
        .run_until(13);
    assert!(object(t.sim(), 1).conversion.is_some(), "saved mid-conversion");
    let mut loaded: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    t.at(30, P1, convert(&[1, 2]));
    for _ in 0..40 {
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
