//! Aiming by the hull (`factions/cults/units/hand_of_heaven.md > TurretAttributes`: "TurnAngle: **minimal** ...: the
//! launcher barely turns, so to aim at most targets the vehicle must **stop and turn its hull** to face them";
//! "Turret: TurnAngle **20°** total; TurnRate 4°/frame"). Its attack comes from its launcher, a turret
//! (`systems/combat.md > TurretSource`), whose sequence starts only at a target within its arc
//! (`systems/controls/command_behaviors.md > AttackingObject` 2: "within Range, beyond MinRange, and within turret arc
//! (turret units)"). (M5-N10) In range but out of the arc it stands, and its behaviour asks the movement step, phase
//! 5d, to turn its hull toward the target (`combat::turret::aims_by_hull`, `Mover::turn_to`), which spins it by the
//! SpinRate of a TrackedVehicle at rest (`systems/units.md > SpeedTurnRadiusMovement`, slice M5-V3), 1/3 radian a
//! tick, as far as its Silhouette stays free: once the target is within ±10° of its heading, it aims. Out of range it
//! drives into range as any attacker does ("resume MovingToObject", step 4).

use sc_sim::math::{self, PI};
use sc_sim::mover::{MoveGoal, Orientation};
use sc_sim::testing::invariants::Invariants;
use sc_sim::{DebugCommand, Sim, TickInputs};

use super::*;

/// The Hand of Heaven's SpinRate (2 × 5 ÷ 30 radians a tick) and its launcher's half arc.
const SPIN: f32 = 1.0 / 3.0;
const HALF_ARC: f32 = 10.0 * PI / 180.0;

/// Player 1's Hand of Heaven (1) at the centre of tile (10, 4), facing east, and player 2's Chop Shop (2) on tiles (9,
/// 15) to (11, 17), its centre 12 tiles due south; player 1's Recruit (3) beside the Chop Shop, its spotter, and player
/// 2's Storage (4), which keeps it in the game.
fn south() -> TestSim {
    let mut t = cults(24, 24)
        .spawn(HAND, Some(P1), point(centre(10, 4)))
        .spawn(ObjectKind::ChopShop, Some(P2), tile(9, 15))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(14, 16)))
        .spawn(ObjectKind::Storage, Some(P2), tile(20, 1))
        .start();
    t.without_auto_targeting();
    t
}

/// The angle off unit 1's heading of `at`, radians, unsigned.
fn off_heading(t: &TestSim, at: Vec2) -> f32 {
    let to = at - t.pos(1);
    math::wrap_angle(math::vector_angle(to) - facing(t, 1)).abs()
}

/// The brief's acceptance: "a stopped Hand of Heaven facing 90° away from a building in range turns its hull, then
/// aims, then fires". Ordered to attack the Chop Shop due south, a quarter turn off its heading, in range, the Hand of
/// Heaven stands, and phase 5d spins its hull at its SpinRate (`movement::turn_in_place`, slice M5-N10), four full
/// ticks and the rest of the quarter in a fifth, not a space unit moved; in that fifth tick, the Chop Shop within its
/// launcher's ±10°, it starts aiming, for 48 ticks, and fires: the missile leaves from where it stood at the order.
#[test]
fn facing_90_degrees_away_it_spins_its_hull_then_aims_then_fires() {
    let mut t = south();
    let (start, at) = (t.pos(1), t.pos(2));
    assert_eq!(at - start, Vec2::new(0.0, TWELVE_TILES));
    assert!((off_heading(&t, at) - PI / 2.0).abs() < 1e-6, "a quarter turn off");
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    let mut turned = Vec::new();
    let mut aiming_from = None;
    while aiming_from.is_none() {
        let was = facing(&t, 1);
        t.step();
        turned.push(math::wrap_angle(facing(&t, 1) - was));
        if matches!(phase(&t, 1), Some(AttackPhase::Aiming { .. })) {
            aiming_from = Some(t.sim().tick());
        }
        assert!(t.sim().tick() < 30, "turned {turned:?}, {:?}", phase(&t, 1));
    }
    let aiming_from = aiming_from.unwrap();
    assert_eq!(aiming_from, 5, "turned {turned:?}");
    assert!(off_heading(&t, at) <= HALF_ARC, "within the arc once aiming");
    assert!(
        turned.iter().all(|turn| (0.0..=SPIN + 1e-6).contains(turn)),
        "clockwise, at most SpinRate a tick: {turned:?}"
    );
    assert!(turned[..4].iter().all(|&turn| (turn - SPIN).abs() < 1e-6), "{turned:?}");
    assert!(
        (turned.iter().sum::<f32>() - PI / 2.0).abs() < 1e-4,
        "a quarter: {turned:?}"
    );
    assert_eq!(t.pos(1), start, "on the spot");
    until(&mut t, 80, "the launch", |t| !launches(t).is_empty());
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!(launch.tick, aiming_from + AIM + FIRING - 1, "48 ticks aimed, 2 fired");
    assert_eq!((launch.from, launch.to), (start, at));
}

/// A 4 × 4 building straight behind it, in range: the Hand of Heaven turns on the spot until the building's centre is
/// within its launcher's arc, and fires, never driving. Its path to the building would lead to the tile off the
/// building's corner, 14.7° off its centre, outside the ±10° arc all the way in: before slice M5-N10 it followed that
/// path (`systems/controls/command_behaviors.md > AttackingObject` 4: "If target moves out of range or arc: resume
/// MovingToObject"), drove past the building and never fired; now it stands and its hull turns
/// (`factions/cults/units/hand_of_heaven.md`: "the vehicle must **stop and turn its hull** to face them"). Player 2's
/// Deployment Center (1) on tiles (5, 7) to (8, 10), player 1's Recruit (2) north of it on Hold, its spotter, and its
/// Hand of Heaven (3) 12 tiles east of the Deployment Center's centre, facing east.
#[test]
fn a_building_behind_it_off_its_path_it_turns_to_on_the_spot_and_fires_never_driving() {
    let mut t = cults(40, 18)
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(5, 7))
        .spawn(ObjectKind::Recruit, Some(P1), point(Vec2::new(544.0, 352.0)))
        .spawn(HAND, Some(P1), point(Vec2::new(1216.0, 576.0)))
        .start();
    t.without_auto_targeting();
    let (start, at) = (t.pos(3), t.pos(1));
    order(&mut t, 1, P1, &[2], OrderKind::Hold);
    order(&mut t, 4, P1, &[3], OrderKind::Attack(id(1)));
    let mut aiming_from = None;
    while launches(&t).is_empty() {
        t.step();
        assert_eq!(t.pos(3), start, "tick {}: on the spot", t.sim().tick());
        if aiming_from.is_none() && matches!(phase(&t, 3), Some(AttackPhase::Aiming { .. })) {
            aiming_from = Some(t.sim().tick());
            let off = math::wrap_angle(math::vector_angle(at - start) - facing(&t, 3));
            assert!(off.abs() <= HALF_ARC, "the centre within the arc: {off}");
        }
        assert!(t.sim().tick() < 80, "{:?}", phase(&t, 3));
    }
    let aiming_from = aiming_from.expect("it aimed");
    assert!(
        aiming_from <= 4 + 8,
        "π less the half arc at 19.1° a tick: aiming from {aiming_from}"
    );
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!(launch.tick, aiming_from + AIM + FIRING - 1);
    assert_eq!((launch.target, launch.from, launch.to), (Some(id(1)), start, at));
}

/// Player 1's Hand of Heaven (1) centred on tile (6, 4), facing east, its side 12 su from a row of mountains on row 3,
/// and player 2's Chop Shop (2) on tiles (5, 11) to (7, 13), its centre 8 tiles due south: a quarter turn, of which
/// the wall allows the Hand of Heaven only 26° where it stands (its corners 37.7 su from its centre, 32 from the
/// wall's face). Player 1's Recruit (3) beside the Chop Shop spots it, and player 2's Storage (4) keeps player 2 in the
/// game. The Hand of Heaven is ordered to attack the Chop Shop in tick 1.
fn beside_a_wall() -> TestSim {
    let mut rows = vec![".".repeat(20); 16];
    rows[3] = "M".repeat(20);
    let mut t = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(HAND, Some(P1), point(centre(6, 4)))
        .spawn(ObjectKind::ChopShop, Some(P2), tile(5, 11))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(9, 10)))
        .spawn(ObjectKind::Storage, Some(P2), tile(15, 6))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    t
}

/// Beside a wall, its spin is cut where a corner meets it (`systems/units.md > UnitCollision > Collision Response
/// (Un-sticking)`: "the blocked part is discarded and the unit never ends up overlapping"), and it doesn't freeze there
/// (design change 0035: "Blocked units don't freeze; they creep and turn out of the contact"): backing off along its
/// heading frees nothing, so it creeps toward the target, one tick's Acceleration (0.5 su) a tick, as far as it lies
/// free, and spins on as it gets room (`Tracked::spin`): 6 su in 12 ticks, of which 5.7 free its corners. It aims from
/// tick 15, where in the open it would from tick 5, and fires. The invariants, I2 among them, hold every tick
/// (`TestSim`).
#[test]
fn beside_a_wall_its_spin_is_cut_and_it_creeps_off_toward_the_target_then_fires() {
    let mut t = beside_a_wall();
    let (at, target) = (t.pos(1), t.pos(2));
    assert_eq!(target - at, Vec2::new(0.0, 8.0 * SU_PER_GU));
    let mut aiming_from = None;
    while aiming_from.is_none() {
        let was = facing(&t, 1);
        t.step();
        let turned = math::wrap_angle(facing(&t, 1) - was);
        assert!(
            (0.0..=SPIN + 1e-6).contains(&turned),
            "tick {}: {turned}",
            t.sim().tick()
        );
        if matches!(phase(&t, 1), Some(AttackPhase::Aiming { .. })) {
            aiming_from = Some(t.sim().tick());
        }
        assert!(t.sim().tick() < 60, "{}", t.ascii(Some(t.pos(1))));
    }
    assert_eq!(aiming_from, Some(15));
    assert_eq!(t.pos(1) - at, Vec2::new(0.0, 6.0), "crept toward it");
    assert!(off_heading(&t, target) <= HALF_ARC, "within the arc once aiming");
    let from = t.pos(1);
    until(&mut t, 80, "the launch", |t| !launches(t).is_empty());
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!((launch.target, launch.from, launch.to), (Some(id(2)), from, target));
}

/// **Negative test** of I5/I7's exception for the hull aim (`docs/movement.md > 8`): a unit without a goal that turned
/// in place toward its aim in a tick, its Orientation Turning, may move in it by one tick's Acceleration at most. The
/// Hand of Heaven beside the wall creeps 0.5 su in tick 2, its first spin cut short, and the invariants hold; edited
/// to have moved 1 su, or to be Maintaining its facing, it breaks I5/I7. The state is edited, as in
/// `movement_separate.rs`: the movement step never does this.
#[test]
fn only_a_unit_turning_to_its_aim_creeps_and_only_by_its_acceleration() {
    let mut t = beside_a_wall();
    t.run_until(1);
    let (before, at) = (t.sim().clone(), t.pos(1));
    t.step();
    assert_eq!(t.pos(1) - at, Vec2::new(0.0, 0.5), "it creeps in tick 2");
    let edited = |edit: fn(&mut serde_json::Value)| {
        let mut json = serde_json::to_value(t.sim()).expect("a state serializes");
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let hand = objects
            .iter_mut()
            .find(|o| o["id"] == 1)
            .expect("the Hand of Heaven is in the state");
        edit(hand);
        let after: Sim = serde_json::from_value(json).expect("the edited state loads");
        let result = Invariants::new(&before).check(&after, &TickInputs::default());
        result.err().map(|violation| violation.invariant)
    };
    assert_eq!(edited(|_| {}), None, "as the step left it");
    assert_eq!(
        edited(|hand| hand["pos"][1] = (hand["pos"][1].as_f64().unwrap() + 0.5).into()),
        Some("I5/I7"),
        "twice its Acceleration"
    );
    assert_eq!(
        edited(|hand| hand["mover"]["orientation"] = "Maintaining".into()),
        Some("I5/I7"),
        "not turning"
    );
}

/// (M5-Mb, found by the soak sweep) I5/I7's exception for the hull aim holds for a unit a defeat makes neutral in the
/// tick it creeps: the defeat check ends the economy phase, after the movement (`core/game_rules.md > On Defeat`:
/// "their orders are cleared and they stay idle where they are"), so its Orientation is Maintaining as the tick ends.
/// [`beside_a_wall`]'s game with a Storage of player 1's (5), removed in tick 2: player 1 is defeated in the tick the
/// Hand of Heaven creeps 0.5 su, its first spin cut short, and the invariants hold (`TestSim`).
#[test]
fn a_unit_that_creeps_in_the_tick_its_owner_is_defeated_keeps_the_invariants() {
    let mut rows = vec![".".repeat(20); 16];
    rows[3] = "M".repeat(20);
    let mut t = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(HAND, Some(P1), point(centre(6, 4)))
        .spawn(ObjectKind::ChopShop, Some(P2), tile(5, 11))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(9, 10)))
        .spawn(ObjectKind::Storage, Some(P2), tile(15, 6))
        .spawn(ObjectKind::Storage, Some(P1), tile(15, 12))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    t.debug_at(2, DebugCommand::Remove { id: id(5) });
    t.run_until(1);
    let at = t.pos(1);
    t.step();
    assert!(
        t.sim().player(P1).is_some_and(|p| p.is_defeated()),
        "player 1 is defeated in tick 2"
    );
    let hand = t.sim().object(id(1)).expect("the Hand of Heaven stays");
    assert_eq!(hand.owner, None, "neutral");
    assert_eq!(
        t.pos(1) - at,
        Vec2::new(0.0, 0.5),
        "it crept in tick 2, as it does with an owner"
    );
    let mover = hand.mover.as_ref().expect("a unit moves");
    assert_eq!(
        (mover.goal, mover.orientation),
        (MoveGoal::None, Orientation::Maintaining),
        "stopped"
    );
}

/// [`south`]'s game with `player`'s unit of `kind` (5) as well, idle against the front half of the Hand of Heaven's
/// south side, 1 su off it: in the way of its corner as it turns toward the Chop Shop.
fn beside(kind: ObjectKind, player: PlayerId) -> TestSim {
    let at = centre(10, 4);
    let mut t = cults(24, 24)
        .spawn(HAND, Some(P1), point(at))
        .spawn(ObjectKind::ChopShop, Some(P2), tile(9, 15))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(14, 16)))
        .spawn(ObjectKind::Storage, Some(P2), tile(20, 1))
        .spawn(kind, Some(player), point(at + Vec2::new(20.0, 33.0)))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    t
}

/// The crushes so far (`SimEvent::Crushed`): their tick, the crushed and the crusher.
fn crushes(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Crushed { id, crusher, .. } => Some((*tick, id, crusher)),
            _ => None,
        })
        .collect()
}

/// Its hull's spin crushes (`systems/units.md > UnitCollision > Crushing`; design change 0418: "Silhouette overlap
/// while Moving, Reversing **or Stationary + Turning**", slice M5-V4): the turn that aims it is a turn in place.
/// Player 2's Militant (5), idle against its south side, dies in its first tick of turning toward the Chop Shop and
/// doesn't stop it: it aims from tick 5, as in the open.
#[test]
fn turning_its_hull_to_aim_it_crushes_an_enemy_infantryman_its_hull_swings_into() {
    let mut t = beside(ObjectKind::Militant, P2);
    until(&mut t, 10, "aiming", |t| {
        matches!(phase(t, 1), Some(AttackPhase::Aiming { .. }))
    });
    assert_eq!(t.sim().tick(), 5);
    assert_eq!(crushes(&t), [(1, id(5), id(1))]);
}

/// "No friendly crushing" (`systems/units.md > UnitCollision > Crushing`): its own infantryman stops its spin at
/// contact ("its spin too, which stops at contact", slice M5-V4), and is asked to make way as one a turn on a path
/// meets is (`systems/units.md > Ground Collision > Friendly yielding`). Player 1's idle Recruit (5), where the
/// Militant above stood, steps 24 su aside, east, and the Hand of Heaven turns on and aims from tick 11, nobody
/// crushed.
#[test]
fn its_own_infantryman_in_the_way_of_its_hull_stops_it_then_steps_aside() {
    let mut t = beside(ObjectKind::Recruit, P1);
    let at = t.pos(5);
    t.step();
    assert!(facing(&t, 1) < SPIN / 4.0, "cut at contact: {}", facing(&t, 1));
    until(&mut t, 60, "aiming", |t| {
        matches!(phase(t, 1), Some(AttackPhase::Aiming { .. }))
    });
    assert_eq!(t.sim().tick(), 11);
    let moved = t.pos(5) - at;
    assert!(moved.x > 20.0 && moved.y.abs() < 1e-3, "stepped aside: {moved}");
    assert_eq!(crushes(&t), []);
}

/// Idle, its auto-targeting acquires an enemy for its base and locks its turret on it (`systems/controls/behaviors.md >
/// BaseAutoTargeting`: "For turret units, the base sets TurretCommandState.LockedTarget"), and engages it as an
/// AttackingObject does: player 2's Militant (2), on Hold 6 tiles due south, a quarter turn off its heading, within
/// its sight and range. The Hand of Heaven turns its hull to it on the spot and fires at it.
#[test]
fn idle_it_turns_its_hull_to_an_enemy_its_auto_targeting_acquires_and_fires() {
    let at = centre(10, 4);
    let mut t = cults(24, 16)
        .spawn(HAND, Some(P1), point(at))
        .spawn(ObjectKind::Militant, Some(P2), point(centre(10, 10)))
        .start();
    order(&mut t, 1, P2, &[2], OrderKind::Hold);
    let target = t.pos(2);
    until(&mut t, 80, "a launch", |t| !launches(t).is_empty());
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!((launch.target, launch.from, launch.to), (Some(id(2)), at, target));
    assert!(
        launch.tick <= 1 + 5 + AIM + FIRING,
        "a quarter turn, then 48 aimed: {}",
        launch.tick
    );
    assert!(off_heading(&t, target) <= HALF_ARC, "turned to it");
    assert_eq!(t.pos(1), at, "on the spot");
}

/// Its hull aims only at what its base engages, standing (`orders::engage`): on Hold, a turret unit's base engages
/// nothing (`systems/controls/command_behaviors.md > HoldingPosition` 2: "For turret units: TurretCommandState is
/// unchanged — turret continues autonomous scanning and engaging targets within range and arc"). Attacking player 2's
/// Militant (2) 6 tiles due east, the Hand of Heaven is put on Hold, and the Militant walks off north: its launcher
/// keeps the lock while it can fire (`systems/controls/commands.md > TurretCommandState > When a lock ends`) and
/// follows the Militant to the edge of its arc, then drops it there, and the hull never turns.
#[test]
fn on_hold_its_hull_stays_put_as_its_target_walks_out_of_the_arc() {
    let at = centre(10, 10);
    let mut t = cults(30, 20)
        .spawn(HAND, Some(P1), point(at))
        .spawn(ObjectKind::Militant, Some(P2), point(centre(16, 10)))
        .start();
    t.without_auto_targeting();
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    order(&mut t, 2, P1, &[1], OrderKind::Hold);
    order(
        &mut t,
        2,
        P2,
        &[2],
        OrderKind::Move(sc_sim::Target::Point(centre(16, 2))),
    );
    let mut edge = false;
    while t.sim().tick() < 80 {
        t.step();
        let tick = t.sim().tick();
        assert_eq!((t.pos(1), facing(&t, 1)), (at, 0.0), "tick {tick}: the hull stays put");
        let turret = t
            .sim()
            .object(id(1))
            .and_then(|o| o.attack.as_ref()?.turret)
            .unwrap_or_default();
        edge |= (turret.facing + HALF_ARC).abs() < 1e-6;
    }
    assert!(edge, "the launcher followed the Militant to the edge of its arc");
    assert!(off_heading(&t, t.pos(2)) > HALF_ARC, "the Militant out of the arc");
    let locked = t.sim().object(id(1)).and_then(|o| o.attack.as_ref()?.turret?.locked);
    assert_eq!(locked, None, "the lock dropped");
}

/// "it can also be fired at the ground (CanTargetGround, Attack Ground)", and `systems/controls/command_behaviors.md >
/// AttackingLocation`: "Range/arc check is against the target location". Facing east, ordered to Attack Ground at a
/// point 10 tiles due west, behind it, it spins on the spot, then aims at the point and fires there, again and again
/// (design change 0293: "the behavior repeats its attack cycle at the location until the player gives another
/// order"): its first missile leaves at most 10 ticks of spin (π at a third of a radian a tick) and 49 of the
/// sequence after the order, and it never leaves the spot.
#[test]
fn an_attack_ground_behind_it_spins_it_round_then_it_fires_there_again_and_again() {
    let at = centre(20, 6);
    let mut t = cults(32, 12).spawn(HAND, Some(P1), point(at)).start();
    t.without_auto_targeting();
    let ground = at - Vec2::new(10.0 * SU_PER_GU, 0.0);
    order(&mut t, 1, P1, &[1], OrderKind::AttackGround(ground));
    t.run_until(1 + 12 + 2 * (AIM + FIRING + COOLDOWN + RELOAD));
    let fired = launches(&t);
    assert!(fired.len() >= 2, "again and again: {fired:?}");
    assert!(fired[0].tick < 1 + 10 + AIM + FIRING, "{fired:?}");
    for launch in &fired {
        assert_eq!((launch.target, launch.from, launch.to), (None, at, ground));
    }
    assert_eq!(t.pos(1), at, "on the spot");
    assert_eq!(
        fired[1].tick - fired[0].tick,
        AIM + FIRING + COOLDOWN + RELOAD,
        "one every 150 ticks"
    );
}
