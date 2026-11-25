//! Weapon Conversion in the game (slice M3-B; `factions/colonists/units.md > Weapon Conversion`, plan decision T26):
//! its timing and what the unit keeps, who may convert, a mixed group, no cancel and no shelter, the orders that go on
//! (`> Orders are kept`, design change 0150), saves and replays; `mid_attack`: a Convert pressed mid-attack, which
//! waits for the Cooldown's end (design change 0153); `target`: a press flips the unit's target form (design change
//! 0198, slice M3-T), its type switching at the press and its form, its weapon, when its conversion ends. Its pieces
//! are `conversion`'s unit tests. `TestSim` checks the movement invariants after every tick.

use sc_sim::combat;
use sc_sim::conversion;
use sc_sim::defs::CONVERT_TICKS;
use sc_sim::shields::SHIELD_REGEN_PER_TICK;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, AttackPhase, Command, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    Replay, Sim, SimEvent, SpawnAt, Target, Tick, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const RIFLE: ObjectKind = ObjectKind::RifleLegionnaire;
const PK: ObjectKind = ObjectKind::Peacekeeper;

mod mid_attack;
mod target;

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

/// `o`'s conversion: `Some(true)` while it waits for its Cooldown's end (design change 0153), its type its target
/// form already (design change 0198), `Some(false)` under way, `None` without one.
fn waits(o: &Object) -> Option<bool> {
    match conversion::waits_to_convert(o) {
        true => Some(true),
        false => o.conversion.map(|_| false),
    }
}

/// The plan's acceptance: "A Sabre becomes a Rifle in 4 ticks, keeping HP and Shields" (`> Convert Command`: "The
/// unit converts in place over 4 frames"; `> HP/Shields`: "current HP/Shields values carry over unchanged"). Player
/// 1's Sabre, at 35 HP and 3 Shields, converts in tick 2: its type is the Rifle Legionnaire from that tick on, with a
/// `KindChanged` in that tick only (design change 0198: "The unit's type, and so its SelectionGroup, switches at the
/// press"); its form is the Sabre's, converting frame by frame, through tick 5, and the Rifle's from tick 6 on. Its HP
/// stays 35; its Shields regenerate all along, 0.25 a tick. It stood still throughout, and can move again.
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
        assert_eq!(o.kind, RIFLE, "tick {tick}: its type from the press");
        if tick == 2 {
            assert_eq!(
                sim.events(),
                [SimEvent::KindChanged {
                    id: id(1),
                    from: SABRE,
                    to: RIFLE,
                }]
            );
        } else {
            assert!(!kind_changed, "tick {tick}");
        }
        if frames <= CONVERT_TICKS {
            assert_eq!(conversion::form(o), SABRE, "tick {tick}");
            assert!(conversion::is_converting(o), "tick {tick}");
            assert_eq!(o.conversion.and_then(|c| c.frames_done()), Some(frames), "tick {tick}");
        } else {
            assert_eq!((conversion::form(o), o.conversion), (RIFLE, None), "tick {tick}");
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
/// sent a Convert: the Legionnaires swap forms, their types at once and their weapons 4 ticks later, the Drone, which
/// has no other form, is left out. A Convert to Drones alone is refused (`Reject::LacksAbility`), and so is one to
/// another player's Legionnaire.
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
    let forms = |sim: &Sim| [1, 2, 3].map(|n| conversion::form(object(sim, n)));
    assert_eq!(
        kinds(t.sim()),
        [RIFLE, SABRE, ObjectKind::Drone],
        "the types at the press"
    );
    assert_eq!(forms(t.sim()), [SABRE, RIFLE, ObjectKind::Drone], "converting");
    t.step();
    assert_eq!(forms(t.sim()), [RIFLE, SABRE, ObjectKind::Drone]);
    assert_eq!(kinds(t.sim()), forms(t.sim()));
    assert_eq!(object(t.sim(), 3).conversion, None);
}

/// The plan's acceptance: "A conversion can't be cancelled, and the unit is attackable throughout" (`> Completion`:
/// "runs to completion over its 4 frames and cannot be cancelled"; `> Vulnerability`: "no invulnerability window").
/// Player 2's Peacekeeper 2 shoots player 1's Sabre 1; the Sabre converts two ticks before a hit lands, which lands
/// while it converts. A Stop, which is taken, changes nothing: the Sabre has its rifle 4 ticks after the Convert. (A
/// second Convert doesn't cancel it either: the unit converts back once it is done, `target`.)
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
    t.order_at(start + 1, P1, &[id(1)], OrderKind::Stop, false);
    for tick in start + 1..start + CONVERT_TICKS {
        t.step();
        let o = object(t.sim(), 1);
        assert_eq!(conversion::form(o), SABRE, "tick {tick}: still converting");
        assert!(conversion::is_converting(o), "tick {tick}");
    }
    assert!(
        hits(&t).contains(&(landed[1], id(2), id(1))),
        "hit while converting, in tick {}",
        landed[1]
    );
    t.step();
    let o = object(t.sim(), 1);
    assert_eq!(
        (o.kind, conversion::form(o)),
        (RIFLE, RIFLE),
        "4 ticks after the Convert"
    );
    assert_eq!(o.conversion, None);
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
    assert_eq!(conversion::form(object(t.sim(), 1)), RIFLE);
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
    assert_eq!(conversion::form(object(t.sim(), 1)), SABRE);
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

/// Design change 0152's Verification (`> Orders are kept`: "New weapon starts ready"): "Rifle fires, convert during
/// Reloading: the Sabre begins Aiming as soon as conversion ends and a target is in reach". Player 1's Rifle 1 shoots
/// player 2's Peacekeeper 2, whose edge is 6 su from its own, within a Sabre's reach too. Converted while Reloading, it
/// is a Sabre 4 ticks later and Aiming at the Peacekeeper in that very tick: no reload is left to wait for.
#[test]
fn the_new_weapon_starts_ready() {
    let mut t = builder()
        .spawn(RIFLE, Some(P1), point(200.0, 300.0)) // 1
        .spawn(PK, Some(P2), point(236.0, 300.0)) // 2: a gap of 36 - 18 - 12 = 6 su
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    while phase(t.sim(), 1) != Some(AttackPhase::Reloading) {
        assert!(t.sim().tick() < 60, "the Rifle shoots and reloads");
        t.step();
    }
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, convert(&[1])).run_until(tick + CONVERT_TICKS - 1);
    assert_eq!(conversion::form(object(t.sim(), 1)), RIFLE, "converting");
    assert_eq!(phase(t.sim(), 1), None, "converting");
    t.step();
    assert_eq!(
        conversion::form(object(t.sim(), 1)),
        SABRE,
        "tick {}",
        tick + CONVERT_TICKS
    );
    let phase = phase(t.sim(), 1);
    assert!(matches!(phase, Some(AttackPhase::Aiming { .. })), "{phase:?}");
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

/// A state saved before design change 0198 had a conversion under way as its frames left alone, its unit's type
/// being the form it converted from: it loads without it, the unit keeping that form (`conversion::saved`), and plays
/// on.
#[test]
fn a_conversion_saved_before_the_target_form_loads_as_none() {
    let mut t = builder().spawn(SABRE, Some(P1), point(200.0, 192.0)).start();
    t.run_until(2);
    let mut loaded = edited(t.sim(), 1, "conversion", serde_json::json!({ "left": 2 }));
    assert_eq!((object(&loaded, 1).kind, object(&loaded, 1).conversion), (SABRE, None));
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    step(&mut loaded, vec![convert(&[1])]);
    assert_eq!(
        object(&loaded, 1).kind,
        ObjectKind::RifleLegionnaire,
        "a press goes on as ever"
    );
}
