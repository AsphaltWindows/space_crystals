//! (M5-X1b) Games on other rules than the built-in ones (`DebugCommand::SetRules`, plan decision T66, `sim/rules.rs`),
//! in a test program of their own: the rules a game reads are the process's (`sc_sim::ruleset`, the current
//! generation), and the `sim` program's tests all play at once on the built-in ones. Each test here holds
//! [`one_at_a_time`] for all of its games, so that no two games on different rules step at once.
//!
//! One duel throughout, `combat_duel.rs`'s: a Peacekeeper (id 1, player 1) and a Guard (id 2, player 2), 160 su apart,
//! ordered to attack each other in tick 2. On the built-in rules the Guard wins, taking the Peacekeeper's 50 HP in
//! three hits of 17, the last in tick 54, while the Peacekeeper lands three hits of 9 (its Damage 10, the Guard's
//! Armor 1).

use std::sync::{Mutex, MutexGuard, PoisonError};

use sc_sim::defs::{AttackAttrs, UnitBase};
use sc_sim::ruleset::RuleField;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Footprint, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, Replay,
    Ruleset, Sim, SimConfig, SimEvent, Tick, TickInputs, Vec2, energy,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PEACEKEEPER: ObjectId = ObjectId(1);
const GUARD: ObjectId = ObjectId(2);

/// The ticks a game here runs: past the end of the duel on the built-in rules.
const TICKS: Tick = 80;

/// 12 × 8 tiles of open ground.
const MAP: &str = "
    ............
    ............
    ............
    ............
    ............
    ............
    ............
    ............
";

/// Held by each test for all of its games (module docs). A test that failed holding it leaves it to the next.
fn one_at_a_time() -> MutexGuard<'static, ()> {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The duel (module docs), with a SetRules of `rules` in tick `at` if given.
fn duel(set: Option<(Tick, &Ruleset)>) -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 250.0))
        .spawn_unit_at(1, ObjectKind::Guard, P2, Vec2::new(460.0, 250.0))
        .order_at(2, P1, &[PEACEKEEPER], OrderKind::Attack(GUARD), false)
        .order_at(2, P2, &[GUARD], OrderKind::Attack(PEACEKEEPER), false);
    if let Some((at, rules)) = set {
        t.debug_at(at, set_rules(rules));
    }
    t
}

fn set_rules(rules: &Ruleset) -> DebugCommand {
    DebugCommand::SetRules(Box::new(rules.clone()))
}

/// `t` run [`TICKS`] ticks, with the state hash after each.
fn run(mut t: TestSim) -> (TestSim, Vec<u64>) {
    let hashes = (0..TICKS).map(|_| t.step().sim().state_hash()).collect();
    (t, hashes)
}

/// The built-in rules with the Peacekeeper's Damage 100: hits of 99 against the Guard, more than its 80 HP.
fn deadly_peacekeeper() -> Ruleset {
    let mut rules = Ruleset::builtin();
    attack(&mut rules, ObjectKind::Peacekeeper).damage = 100.0;
    rules
}

fn attack(rules: &mut Ruleset, kind: ObjectKind) -> &mut AttackAttrs {
    let unit = rules.defs.get_mut(kind).unit.as_mut().expect("a unit");
    unit.attack.as_mut().expect("an attack")
}

/// The hits `attacker` landed: their ticks and the HP each took.
fn hits(t: &TestSim, attacker: ObjectId) -> Vec<(Tick, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker: a, damage, ..
            } if a == attacker => Some((*tick, damage)),
            _ => None,
        })
        .collect()
}

/// The deaths: their ticks, and who died.
fn died(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Died { id, .. } => Some((*tick, *id)),
            _ => None,
        })
        .collect()
}

/// The ticks whose rules changed (`SimEvent::RulesChanged`).
fn changed(t: &TestSim) -> Vec<Tick> {
    (t.events().iter())
        .filter(|(_, e)| *e == SimEvent::RulesChanged)
        .map(|&(tick, _)| tick)
        .collect()
}

/// Whether `sim` plays by the built-in rules: generation 0's (`Ruleset::BUILTIN`).
fn on_builtin(sim: &Sim) -> bool {
    std::ptr::eq(sim.rules(), Ruleset::builtin_ref())
}

/// The gate's answer to a SetRules of `rules` in `sim`.
fn check(sim: &Sim, rules: &Ruleset) -> Result<(), Reject> {
    sim.validate(&PlayerCommand {
        player: P1,
        command: Command::Debug(set_rules(rules)),
    })
}

/// A Peacekeeper's Damage of 100 counts from the tick after its SetRules: given in the tick before the Peacekeeper's
/// second hit, it makes that hit kill the Guard; given in the tick of the second hit, it leaves that hit at 9, the tick
/// playing out on the rules it started on, and the third kills. Up to the first hit by the new rules, the game is the
/// built-in one's, tick by tick: the state hash leaves the rules out, and the change touches nothing else. The rules
/// change once, in the tick after the command, and the game plays on by them.
#[test]
fn a_new_damage_counts_from_the_tick_after_the_command() {
    let _one = one_at_a_time();
    let (builtin, builtin_hashes) = run(duel(None));
    let landed = hits(&builtin, PEACEKEEPER);
    assert_eq!(landed.iter().map(|&(_, hp)| hp).collect::<Vec<f32>>(), [9.0; 3]);
    assert_eq!(died(&builtin), [(54, PEACEKEEPER)]);
    assert!(landed[2].0 < 54, "{landed:?}");
    assert!(changed(&builtin).is_empty());

    let rules = deadly_peacekeeper();
    for (at, killing) in [(landed[1].0 - 1, 1), (landed[1].0, 2)] {
        let (tuned, hashes) = run(duel(Some((at, &rules))));
        let kill = landed[killing].0;
        assert_eq!(changed(&tuned), [at + 1], "given in tick {at}");
        assert_eq!(
            hits(&tuned, PEACEKEEPER)[..killing],
            landed[..killing],
            "given in tick {at}"
        );
        assert_eq!(died(&tuned), [(kill, GUARD)], "given in tick {at}");
        let same = kill as usize - 1;
        assert_eq!(hashes[..same], builtin_hashes[..same], "given in tick {at}");
        assert_ne!(hashes[same], builtin_hashes[same], "given in tick {at}");
        assert_eq!(*tuned.sim().rules(), rules);
        assert_eq!(*sc_sim::rules(), rules);
    }
}

/// HP follows a MaxHP the rules change, the damage taken staying (`transform::follow_max`). Given in the tick after
/// the duel's first hits, the Peacekeeper's MaxHP of 40 (from 50) takes its 33 HP, 17 taken, to 23, and the Guard's
/// of 100 (from 80) its 71, 9 taken, to 91; a Peacekeeper out of the fight, unhurt, stays full at 40. Shields follow
/// theirs the same way: a Sabre Legionnaire's MaxShields of 5, from 10, leaves it full at 5. Energy stays within its
/// MaxEnergy: a Disciple's 50 and a little, of 200, comes down to a MaxEnergy of 30. The tick of the command plays out
/// as it started; the next hits take from what the change left: the Guard's second leaves the Peacekeeper at 6.
#[test]
fn hp_follows_a_new_max_hp_with_the_damage_kept() {
    let _one = one_at_a_time();
    let mut t = duel(None);
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(64.0, 64.0))
        .spawn_unit_at(1, ObjectKind::SabreLegionnaire, P1, Vec2::new(64.0, 448.0))
        .spawn_unit_at(1, ObjectKind::Disciple, P2, Vec2::new(704.0, 448.0));
    let hp = |t: &TestSim, id: u32| t.sim().object(ObjectId(id)).and_then(|o| o.hp);
    t.run_until(1);
    while hp(&t, 1) == Some(50.0) || hp(&t, 2) == Some(80.0) {
        t.step();
    }
    assert_eq!((hp(&t, 1), hp(&t, 2), hp(&t, 3)), (Some(33.0), Some(71.0), Some(50.0)));

    let mut rules = Ruleset::builtin();
    rules.defs.get_mut(ObjectKind::Peacekeeper).max_hp = Some(40.0);
    rules.defs.get_mut(ObjectKind::Guard).max_hp = Some(100.0);
    rules.defs.get_mut(ObjectKind::SabreLegionnaire).max_shields = Some(5.0);
    (rules.defs.get_mut(ObjectKind::Disciple).energy.as_mut())
        .expect("a pool")
        .max = 30.0;
    let at = t.sim().tick() + 1;
    t.debug_at(at, set_rules(&rules)).step();
    assert_eq!((hp(&t, 1), hp(&t, 2), hp(&t, 3)), (Some(33.0), Some(71.0), Some(50.0)));
    let disciple = |t: &TestSim| energy::of(t.sim().object(ObjectId(5)).expect("the Disciple")).expect("a pool");
    assert!(disciple(&t).0 > 50.0 && disciple(&t).1 == 200.0, "{:?}", disciple(&t));

    t.step();
    assert_eq!((hp(&t, 1), hp(&t, 2), hp(&t, 3)), (Some(23.0), Some(91.0), Some(40.0)));
    let shields = t.sim().object(ObjectId(4)).and_then(|o| o.shields);
    assert_eq!(shields, Some(5.0));
    assert_eq!(disciple(&t), (30.0, 30.0));
    let max_hp = |t: &TestSim, id: u32| (t.sim().object(ObjectId(id))).and_then(|o| o.max_hp_with(t.sim().research()));
    assert_eq!((max_hp(&t, 1), max_hp(&t, 2)), (Some(40.0), Some(100.0)));

    while hp(&t, 1) == Some(23.0) {
        t.step();
    }
    assert_eq!(hp(&t, 1), Some(6.0));
}

/// A tuned game plays back from its replay file, the SetRules in it with its rules: the same state tick by tick, on
/// the same rules, the Guard dying as it did. A replay of the same commands but the SetRules is the built-in game.
#[test]
fn a_replay_plays_a_tuned_game_back() {
    let _one = one_at_a_time();
    let rules = deadly_peacekeeper();
    let (tuned, hashes) = run(duel(Some((20, &rules))));
    assert!(died(&tuned).iter().any(|&(_, id)| id == GUARD), "{:?}", died(&tuned));

    let json = serde_json::to_string(tuned.replay()).expect("a replay file");
    let replay: Replay = serde_json::from_str(&json).expect("the replay file read back");
    assert_eq!(&replay, tuned.replay());
    let mut sim = replay.start();
    assert!(on_builtin(&sim));
    let mut played = Vec::new();
    while replay.step(&mut sim) {
        played.push(sim.state_hash());
    }
    assert_eq!(played, hashes);
    assert_eq!(&sim, tuned.sim());
    assert_eq!(*sim.rules(), rules);

    let (builtin, builtin_hashes) = run(duel(None));
    let mut without = replay.clone();
    for inputs in &mut without.inputs {
        inputs
            .commands
            .retain(|c| !matches!(c.command, Command::Debug(DebugCommand::SetRules(_))));
    }
    assert_eq!(&without.run(), builtin.sim());
    assert_ne!(builtin_hashes, hashes);
}

/// A save keeps the rules a game plays by, and those of a SetRules its next tick starts on: as the rules themselves,
/// left out while they are the built-in ones. Saved between the tick of the command and the next, the game loads into
/// the same state and changes its rules in the next tick, as the running game does, and plays on the same, tick by
/// tick; saved after, it loads on the rules it played by. A save whose rules change more than numbers from this
/// build's doesn't load.
#[test]
fn a_save_keeps_the_rules() {
    let _one = one_at_a_time();
    let rules = deadly_peacekeeper();
    let mut t = duel(Some((20, &rules)));
    t.run_until(19);
    let builtin = serde_json::to_value(t.sim()).expect("a save");
    assert!(builtin.get("rules").is_none() && builtin.get("next_rules").is_none());

    t.step();
    let saved = serde_json::to_value(t.sim()).expect("a save");
    let rules_json = serde_json::to_value(&rules).expect("the rules");
    assert_eq!((saved.get("rules"), saved.get("next_rules")), (None, Some(&rules_json)));
    let mut loaded: Sim = serde_json::from_value(saved).expect("the save loaded");
    assert_eq!(&loaded, t.sim());
    let mut running = t.sim().clone();
    for i in 0..TICKS {
        running.step(&TickInputs::default());
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), running.state_hash(), "tick {}", running.tick());
        assert_eq!(loaded.events(), running.events(), "tick {}", running.tick());
        assert_eq!(
            i == 0,
            running.events().contains(&SimEvent::RulesChanged),
            "tick {}",
            running.tick()
        );
    }
    assert_eq!(*loaded.rules(), rules);

    let saved = serde_json::to_value(&running).expect("a save");
    assert_eq!((saved.get("rules"), saved.get("next_rules")), (Some(&rules_json), None));
    let loaded: Sim = serde_json::from_value(saved.clone()).expect("the save loaded");
    assert_eq!(loaded, running);
    assert_eq!(*loaded.rules(), rules);

    let mut other = saved;
    let mut silhouette = rules.clone();
    silhouette.defs.get_mut(ObjectKind::Peacekeeper).footprint = Footprint::Silhouette { w: 30.0, h: 30.0 };
    other["rules"] = serde_json::to_value(&silhouette).expect("the rules");
    let error = serde_json::from_value::<Sim>(other).expect_err("a save of other rules' structure");
    assert!(error.to_string().contains("Peacekeeper's w"), "{error}");
}

/// Rules that change more than numbers are refused (`Reject::RulesStructural`), naming the first change: a
/// Peacekeeper's Silhouette of 30 × 30 su, a Mech that doesn't crush. So are rules with a number that isn't finite
/// (`Reject::RulesNotFinite`): a Guard's Damage of NaN. The game goes on as if it had never been given them, tick by
/// tick. Numbers alone pass, and the game's own rules too, which change nothing: no `RulesChanged`. With debug
/// commands off, none passes (`Reject::DebugDisabled`).
#[test]
fn rules_that_change_more_than_numbers_are_refused() {
    let _one = one_at_a_time();
    let mut silhouette = Ruleset::builtin();
    silhouette.defs.get_mut(ObjectKind::Peacekeeper).footprint = Footprint::Silhouette { w: 30.0, h: 30.0 };
    let mut crushing = Ruleset::builtin();
    crushing.bases.get_mut(UnitBase::Mech).crushes = false;
    let numbers = deadly_peacekeeper();
    let t = duel(None);
    let change = |entry, field| Err(Reject::RulesStructural(RuleField { entry, field }));
    assert_eq!(check(t.sim(), &silhouette), change("Peacekeeper", "w"));
    assert_eq!(check(t.sim(), &crushing), change("Mech", "crushes"));
    assert_eq!(check(t.sim(), &numbers), Ok(()));
    assert_eq!(check(t.sim(), &Ruleset::builtin()), Ok(()));
    let mut nan = Ruleset::builtin();
    attack(&mut nan, ObjectKind::Guard).damage = f32::NAN;
    let not_finite = Reject::RulesNotFinite(RuleField {
        entry: "Guard",
        field: "damage",
    });
    assert_eq!(check(t.sim(), &nan), Err(not_finite));

    let (builtin, builtin_hashes) = run(duel(None));
    for rules in [silhouette, crushing, nan, Ruleset::builtin()] {
        let (refused, hashes) = run(duel(Some((20, &rules))));
        assert_eq!(hashes, builtin_hashes);
        assert_eq!(refused.events(), builtin.events());
        assert!(on_builtin(refused.sim()));
    }

    let off = Sim::new(&ScenarioBuilder::new(MAP).config(SimConfig::default()).build());
    assert_eq!(check(&off, &numbers), Err(Reject::DebugDisabled));
}
