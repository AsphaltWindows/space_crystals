//! The Raptor in a fight (`factions/gdo/units/raptor.md`): "a missile launcher turret", "TurnAngle: **60°** full arc
//! (±30° from the vehicle's facing)", "Turret TurnRate 10°/frame", "AttackType: HeadDisjointed (homing missiles —
//! track the target, cannot miss)", "Damage 14; Range 6 grid units; MinRange 0; ProjectileSpeed 14 su/frame; Aim 4,
//! Firing 2, Cooldown 4, Reload 22 frames (one missile per 32 frames)", on a WheeledVehicle's FixedTurnRadiusMovement
//! ("MinimumTurnRadius 80 su; ForwardMaxSpeed **10 su/frame**"). On an open field, auto-targeting off: player 1's
//! Raptor attacks player 2's Peacekeeper (`factions/gdo/units/peacekeeper.md`: MaxHP 50, PointArmor 1), which a
//! missile hits for 13, four killing it. Question Q0184's cases, the Assault Bike's (`tests/vehicle_assembly/
//! mortar.rs`), are pinned for the Raptor too, the second wheeled vehicle with a narrow turret
//! (`orders::engage`'s `rests_in_arc`, slice M6-SV's), apart in [`close_range`].

mod close_range;

use sc_sim::combat::turret_facing;
use sc_sim::math;

use super::*;

/// The Raptor and the Peacekeeper of [`field`].
const RAPTOR_ID: ObjectId = ObjectId(3);
const PK_ID: ObjectId = ObjectId(4);
/// A missile's hit on a Peacekeeper: its Damage 14 less the Peacekeeper's PointArmor 1.
const HIT: f32 = 13.0;
/// Half the turret's "TurnAngle: **60°** full arc", radians.
const HALF_ARC: f32 = 30.0 * math::PI / 180.0;
/// One degree, radians: `tuning::combat::Combat::facing_tolerance`.
const DEGREE: f32 = math::PI / 180.0;
/// "Range 6 grid units", space units: the gap between the two shapes (`combat::within_range`).
const RANGE: f32 = 6.0 * 64.0;

/// Both players' Deployment Centers far off on a flat 40 × 30 map; player 1's Raptor ([`RAPTOR_ID`]), standing and
/// facing east at `raptor`, and player 2's Peacekeeper ([`PK_ID`]) at `pk`, spawned in tick 1, which has run;
/// auto-targeting off.
fn field(raptor: Vec2, pk: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 2))
        .start();
    t.without_auto_targeting();
    t.spawn_unit_at(1, RAPTOR, P1, raptor).spawn_unit_at(1, PK, P2, pk);
    t.run_until(1);
    assert_eq!(object(&t, RAPTOR_ID).facing, 0.0, "facing east");
    t
}

/// The missiles launched so far: (tick, shooter).
fn launches(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched { shooter, .. } => Some((*tick, shooter)),
            _ => None,
        })
        .collect()
}

/// The tick `id` died in, if it did.
fn died(t: &TestSim, id: ObjectId) -> Option<Tick> {
    (t.events().iter()).find_map(|(tick, e)| match *e {
        SimEvent::Died { id: dead, .. } if dead == id => Some(*tick),
        _ => None,
    })
}

/// One tick of a [`chase`]: what the Raptor did in it.
#[derive(Debug)]
struct Moment {
    tick: Tick,
    launched: bool,
    speed: f32,
    /// Its turret's angle off its hull's facing and off the bearing to the Peacekeeper, radians in `(-PI, PI]`.
    off_hull: f32,
    off_target: f32,
    /// The gap between its shape and the Peacekeeper's.
    gap: f32,
}

impl Moment {
    fn of(t: &TestSim, launched: bool) -> Moment {
        let (raptor, pk) = (object(t, RAPTOR_ID), object(t, PK_ID));
        let turret = turret_facing(raptor).expect("a turret");
        let bearing = math::vector_angle(pk.pos - raptor.pos);
        Moment {
            tick: t.sim().tick(),
            launched,
            speed: t.mover(RAPTOR_ID.0).momentum.length(),
            off_hull: math::wrap_angle(turret - raptor.facing),
            off_target: math::wrap_angle(turret - bearing),
            gap: shape(t, RAPTOR_ID).gap(&shape(t, PK_ID)),
        }
    }
}

/// [`field`] with the Raptor at (300, 1400) and the Peacekeeper `start` degrees north of its facing, 400 su off, in its
/// Range and sight: in tick 2 the Peacekeeper runs off `run` degrees north of east and the Raptor attacks it. Runs
/// until the Peacekeeper dies, within 200 ticks; returns the game and each tick of the chase.
fn chase(start: f32, run: f32) -> (TestSim, Vec<Moment>) {
    let from = Vec2::new(300.0, 1400.0);
    let pk = from + math::facing_vector(-start.to_radians()) * 400.0;
    let mut t = field(from, pk);
    let to = pk + math::facing_vector(-run.to_radians()) * 1600.0;
    t.at(2, P2, order(&[PK_ID], OrderKind::Move(at(to))));
    t.at(2, P1, order(&[RAPTOR_ID], OrderKind::Attack(PK_ID)));
    let mut moments = Vec::new();
    while died(&t, PK_ID).is_none() {
        assert!(t.sim().tick() < 200, "the Peacekeeper killed\n{}", t.ascii(None));
        let before = launches(&t).len();
        t.step();
        if t.sim().object(PK_ID).is_some() {
            moments.push(Moment::of(&t, launches(&t).len() > before));
        }
    }
    (t, moments)
}

/// The plan's acceptance: "The Raptor hits a running Peacekeeper at Range 6 every 32 frames, its turret tracking
/// while the hull drives". The Peacekeeper, 400 su ahead of it, runs off 10° north of east at its 4 su a tick; the
/// Raptor launches from where it stands in tick 7 (Aiming 4 and Firing 2 from the order's tick), then one missile
/// every 32 frames (Cooldown 4 and Reload 22 on), each with the Peacekeeper at its Range of 6 grid units (a gap of 384
/// su as its Aiming starts, a few su more as the missile leaves: the Peacekeeper runs on). It keeps up, driving after
/// the Peacekeeper (more than 300 su by the fourth launch) and turning after it, its turret on the Peacekeeper every
/// tick from its first Aiming on. The four missiles hit one every 32 frames, for 13 each, and the fourth kills it.
#[test]
fn it_hits_a_running_peacekeeper_from_range_6_every_32_frames() {
    let (t, moments) = chase(0.0, 10.0);
    let launched: Vec<Tick> = launches(&t).iter().map(|&(tick, _)| tick).collect();
    assert_eq!(launched[..4], [7, 39, 71, 103], "one every 32 frames");
    assert_eq!(
        hits_on(&t, PK_ID),
        [50, 82, 114, 146].map(|tick| (tick, RAPTOR_ID, HIT)),
        "each 32 frames after the one before"
    );
    assert_eq!(died(&t, PK_ID), Some(146), "the fourth kills it");
    for moment in moments.iter().filter(|moment| moment.launched) {
        assert!(
            (RANGE - 8.0..=RANGE + 8.0).contains(&moment.gap),
            "tick {}: launched {} su off",
            moment.tick,
            moment.gap
        );
    }
    let raptor = object(&t, RAPTOR_ID);
    assert!(raptor.pos.x > 300.0 + 300.0, "it drove after it: {}", raptor.pos);
    assert!(
        raptor.facing < -4.0 * DEGREE,
        "and turned after it: {}",
        raptor.facing.to_degrees()
    );
    for moment in &moments[1..] {
        assert!(
            moment.off_target.abs() <= DEGREE,
            "tick {}: its turret {}° off the Peacekeeper",
            moment.tick,
            moment.off_target.to_degrees()
        );
    }
    assert!(moments.iter().any(|moment| moment.speed >= 5.0), "it drove");
}

/// Its turret, tracking while the hull drives: the Peacekeeper 20° north of the Raptor's facing, 400 su off, runs off
/// 45° north of east. The Raptor's turret turns the 20° onto it at its 10° a tick and the first missile leaves from
/// where the Raptor stands, its hull still facing east, its turret 20° off it, inside its ±30°. Then it drives after
/// the Peacekeeper, its hull turning toward it, and its turret holds on it every tick, turned back the other way on
/// its hull as the hull turns under it: driving at 3 su a tick or more, its turret is 10° or more off its hull and on
/// the Peacekeeper. The second missile leaves while it still rolls (it fires on the move). Each missile leaves from its
/// Range, 32 or 33 frames after the one before (33 when the Peacekeeper is a hair out of range as the Reload ends),
/// and four hits kill it.
#[test]
fn its_turret_holds_its_target_while_the_hull_drives_and_turns_under_it() {
    let (t, moments) = chase(20.0, 45.0);
    let first = moments.iter().find(|moment| moment.launched).expect("a launch");
    assert_eq!(first.tick, 7);
    assert!(
        near(first.off_hull, -20.0 * DEGREE, 2.0 * DEGREE) && first.off_hull.abs() <= HALF_ARC,
        "its turret {}° off its hull",
        first.off_hull.to_degrees()
    );
    assert_eq!(first.speed, 0.0, "from where it stands");
    for moment in &moments[1..] {
        assert!(
            moment.off_target.abs() <= DEGREE,
            "tick {}: its turret {}° off the Peacekeeper",
            moment.tick,
            moment.off_target.to_degrees()
        );
        assert!(
            moment.off_hull.abs() <= HALF_ARC + DEGREE,
            "tick {}: in its arc",
            moment.tick
        );
    }
    let tracking = |moment: &Moment| moment.speed >= 3.0 && moment.off_hull.abs() >= 10.0 * DEGREE;
    assert!(moments.iter().any(tracking), "{moments:#?}");
    let launched: Vec<&Moment> = moments.iter().filter(|moment| moment.launched).collect();
    assert!(launched[1].speed > 0.0, "the second on the move: {:?}", launched[1]);
    for pair in launched.windows(2) {
        let apart = pair[1].tick - pair[0].tick;
        assert!((32..=33).contains(&apart), "{pair:?}");
    }
    for moment in &launched {
        assert!((RANGE - 12.0..=RANGE + 8.0).contains(&moment.gap), "{moment:?}");
    }
    assert_eq!(hits_on(&t, PK_ID).len(), 4);
    let facing = object(&t, RAPTOR_ID).facing;
    assert!(facing < -30.0 * DEGREE, "its hull turned: {}", facing.to_degrees());
}
