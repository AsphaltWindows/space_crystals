use super::*;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DC: ObjectKind = ObjectKind::DeploymentCenter;
const EF: ObjectKind = ObjectKind::ExtractionFacility;
const GENERATOR: ObjectKind = ObjectKind::Generator;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;

/// Player 1's Deployment Center (1) and Extraction Facility (2), and player 2's Barracks (3), as a game holds them.
fn objects() -> BTreeMap<ObjectId, Object> {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
        .spawn(DC, Some(P1), tile(1, 1))
        .spawn(EF, Some(P1), tile(6, 1))
        .spawn(ObjectKind::Barracks, Some(P2), tile(10, 1))
        .build();
    Sim::new(&scenario).objects().map(|o| (o.id, o.clone())).collect()
}

fn constructing(kind: ObjectKind, since: Tick) -> Option<Constructor> {
    Some(Constructor::Constructing {
        kind,
        since,
        progress: 0,
    })
}

/// Player 1's power `generated / consumed`.
fn power(generated: u32, consumed: u32) -> BTreeMap<PlayerId, PowerSummary> {
    BTreeMap::from([(P1, PowerSummary { generated, consumed })])
}

/// Runs the constructors' step in ticks `ticks` and lists the completions, with their ticks.
fn run(
    objects: &mut BTreeMap<ObjectId, Object>,
    power: &BTreeMap<PlayerId, PowerSummary>,
    ticks: std::ops::RangeInclusive<Tick>,
) -> Vec<(Tick, SimEvent)> {
    let mut completions = Vec::new();
    for tick in ticks {
        let mut events = Vec::new();
        step(objects, power, ObjectId(u32::MAX), tick, &mut events);
        completions.extend(events.into_iter().map(|e| (tick, e)));
    }
    completions
}

fn completed(constructor: u32, kind: ObjectKind) -> SimEvent {
    SimEvent::ConstructionCompleted {
        constructor: ObjectId(constructor),
        kind,
    }
}

/// `> DeploymentCenterInstanceState`: a constructor starts with nothing under construction and nothing ready; only the
/// types with a `Constructs` list are constructors.
#[test]
fn constructors_start_idle() {
    for &kind in ObjectKind::ALL {
        let expected = matches!(kind, DC | EF).then_some(Constructor::Idle);
        assert_eq!(constructor_for(kind), expected, "{kind:?}");
    }
    let objects = objects();
    let states: Vec<Option<Constructor>> = objects.values().map(|o| o.constructor).collect();
    assert_eq!(states, [Some(Constructor::Idle), Some(Constructor::Idle), None]);
}

/// CurrentConstruction, ConstructionProgress ("frames elapsed") and ReadyToPlace, from the one state.
#[test]
fn the_state_tells_what_is_built_and_what_is_ready() {
    let idle = Constructor::Idle;
    assert!(idle.is_idle());
    assert_eq!((idle.current(), idle.ready(), idle.frames_done()), (None, None, None));
    let building = Constructor::Constructing {
        kind: GENERATOR,
        since: 3,
        progress: 5 * RATIO_ONE + RATIO_ONE / 2,
    };
    assert!(!building.is_idle());
    assert_eq!(
        (building.current(), building.ready(), building.frames_done()),
        (Some(GENERATOR), None, Some(5))
    );
    let ready = Constructor::Ready { kind: GENERATOR };
    assert!(!ready.is_idle());
    assert_eq!(
        (ready.current(), ready.ready(), ready.frames_done()),
        (None, Some(GENERATOR), None)
    );
    assert_eq!(Constructor::default(), idle);
}

/// `> ObjectInterfaceState[DeploymentCenter] > BuildMenu`: a building on the list, by an idle constructor, if the bank
/// covers its cost; the first reason otherwise.
#[test]
fn a_construction_starts_on_the_list_when_idle_and_paid_for() {
    let mut objects = objects();
    let dc = objects[&ObjectId(1)].clone();
    let ef = objects[&ObjectId(2)].clone();
    let barracks = &objects[&ObjectId(3)];
    assert_eq!(
        can_start(&dc, GENERATOR, 100).map(|r| (r.crystals, r.ticks)),
        Ok((100, 240))
    );
    assert_eq!(can_start(&ef, PLATE, 75).map(|r| (r.crystals, r.ticks)), Ok((75, 400)));
    assert_eq!(can_start(&dc, GENERATOR, 99), Err(Reject::NotEnoughCrystals));
    assert_eq!(can_start(&dc, PLATE, 1000), Err(Reject::CannotConstruct(PLATE)));
    assert_eq!(can_start(&ef, GENERATOR, 1000), Err(Reject::CannotConstruct(GENERATOR)));
    assert_eq!(
        can_start(barracks, GENERATOR, 1000),
        Err(Reject::CannotConstruct(GENERATOR))
    );
    // "Other build options unavailable" while it constructs or holds a ready building.
    for state in [constructing(GENERATOR, 1), Some(Constructor::Ready { kind: GENERATOR })] {
        let busy = objects.get_mut(&ObjectId(1)).unwrap();
        busy.constructor = state;
        assert_eq!(
            can_start(busy, GENERATOR, 1000),
            Err(Reject::ConstructorBusy(ObjectId(1)))
        );
        assert_eq!(
            can_start(busy, PLATE, 0),
            Err(Reject::CannotConstruct(PLATE)),
            "the list first"
        );
        assert_eq!(
            can_start(busy, ObjectKind::Barracks, 0),
            Err(Reject::ConstructorBusy(ObjectId(1)))
        );
    }
}

/// `> Cancellation`: the full cost while constructing, 75% rounded down once ready, nothing when idle.
#[test]
fn a_refund_follows_the_state() {
    let objects = objects();
    let with = |id: u32, state| {
        let mut o = objects[&ObjectId(id)].clone();
        o.constructor = state;
        refund(&o)
    };
    let ready = |kind| Some(Constructor::Ready { kind });
    assert_eq!(with(2, constructing(PLATE, 1)), 75);
    assert_eq!(with(2, ready(PLATE)), 56);
    assert_eq!(with(1, constructing(GENERATOR, 1)), 100);
    assert_eq!(with(1, ready(GENERATOR)), 75);
    assert_eq!(with(1, constructing(ObjectKind::Barracks, 1)), 200);
    assert_eq!(with(1, ready(ObjectKind::Barracks)), 150);
    assert_eq!(with(1, ready(EF)), 150);
    assert_eq!(with(1, Some(Constructor::Idle)), 0);
    assert_eq!(with(1, None), 0);
    assert_eq!(
        with(1, ready(PLATE)),
        0,
        "not on the Deployment Center's list: a hand-edited save"
    );
}

/// `> Generator`: 240 frames. A construction ordered in tick 10 counts its frames from tick 11 and is ready in tick
/// 250, with one event.
#[test]
fn a_construction_is_ready_after_its_frames() {
    let mut objects = objects();
    objects.get_mut(&ObjectId(1)).unwrap().constructor = constructing(GENERATOR, 10);
    let power = power(30, 21);
    assert_eq!(run(&mut objects, &power, 10..=249), []);
    assert_eq!(objects[&ObjectId(1)].constructor.unwrap().frames_done(), Some(239));
    assert_eq!(run(&mut objects, &power, 250..=1000), [(250, completed(1, GENERATOR))]);
    assert_eq!(
        objects[&ObjectId(1)].constructor,
        Some(Constructor::Ready { kind: GENERATOR })
    );
}

/// Design change 0089: at negative power the Extraction Facility builds at the ratio, 400 frames at 30/51 taking
/// exactly 680 ticks; the Deployment Center at full speed; with nothing generated, the facility stands still.
#[test]
fn the_facility_is_slowed_by_negative_power_and_the_center_is_not() {
    let mut objects = objects();
    objects.get_mut(&ObjectId(1)).unwrap().constructor = constructing(GENERATOR, 0);
    objects.get_mut(&ObjectId(2)).unwrap().constructor = constructing(PLATE, 0);
    let mut slowed = objects.clone();
    assert_eq!(
        run(&mut slowed, &power(30, 51), 1..=2000),
        [(240, completed(1, GENERATOR)), (680, completed(2, PLATE))]
    );
    let mut halted = objects.clone();
    assert_eq!(
        run(&mut halted, &power(0, 15), 1..=5000),
        [(240, completed(1, GENERATOR))]
    );
    assert_eq!(halted[&ObjectId(2)].constructor, constructing(PLATE, 0));
    assert_eq!(
        run(&mut objects, &power(30, 21), 1..=2000),
        [(240, completed(1, GENERATOR)), (400, completed(2, PLATE))]
    );
}

/// The spawn-tick rule (`ARCHITECTURE.md > Tick pipeline`) and ownerless objects: a constructor created this tick, or
/// owned by no one, builds nothing.
#[test]
fn new_and_ownerless_constructors_wait() {
    let mut objects = objects();
    objects.get_mut(&ObjectId(1)).unwrap().constructor = constructing(GENERATOR, 0);
    objects.get_mut(&ObjectId(2)).unwrap().constructor = constructing(PLATE, 0);
    objects.get_mut(&ObjectId(2)).unwrap().owner = None;
    let mut events = Vec::new();
    step(&mut objects, &power(30, 21), ObjectId(1), 1, &mut events);
    for id in [1, 2] {
        assert_eq!(
            objects[&ObjectId(id)].constructor.unwrap().frames_done(),
            Some(0),
            "#{id}"
        );
    }
    step(&mut objects, &power(30, 21), ObjectId(2), 2, &mut events);
    assert_eq!(objects[&ObjectId(1)].constructor.unwrap().frames_done(), Some(1));
    assert_eq!(objects[&ObjectId(2)].constructor.unwrap().frames_done(), Some(0));
    assert_eq!(events, []);
}

/// Plan decision T6: progress in fixed point, done when it reaches the frames, never overflowing.
#[test]
fn progress_saturates() {
    let longest = Recipe {
        kind: GENERATOR,
        crystals: 0,
        ticks: u32::MAX / RATIO_ONE,
    };
    let mut progress = longest.ticks * RATIO_ONE - RATIO_ONE;
    assert!(advance(&mut progress, RATIO_ONE, &longest));
    let mut almost = u32::MAX - 1;
    assert!(advance(&mut almost, RATIO_ONE, &longest));
    assert_eq!(almost, u32::MAX);
    let mut stuck = 0;
    assert!(!advance(&mut stuck, 0, &longest));
}
