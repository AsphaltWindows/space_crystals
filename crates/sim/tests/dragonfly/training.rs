//! The Airfield's Dragonfly behind the Deployment Center's Propulsion Labs: `factions/gdo/structures/airfield.md >
//! Produces (so far)`: "Dragonfly: 175 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at
//! least one Propulsion Labs", and "**W** Dragonfly (greyed without Propulsion Labs)"; `factions/gdo/units/dragonfly.md
//! > Placeholder stats`: "Cost: Airfield, 175 Space Crystals **+ 4 Supplies**" (design change 0435);
//! `factions/gdo/structures/deployment_center.md > Constructs`: "Propulsion Labs: 200 Space Crystals, 400 frames (25
//! seconds) (placeholder). Requires: player owns at least one Airfield". A new Dragonfly appears landed on a free slot
//! of its Airfield (design change 0428 point 2) and, landed, doesn't move; with a rally point it takes off straight up
//! and leaves at IdleSpeed (`systems/mechanics/aircraft_landing.md > Assigned base`).

use sc_sim::aircraft::AircraftState;
use sc_sim::{BuildingOrientation, ConstructionCommand, Constructor, ProductionCommand};

use super::*;

const LABS: ObjectKind = ObjectKind::PropulsionLabs;
/// The Dragonfly's price and its training (module docs).
const COST: u32 = 175;
const SUPPLIES: u32 = 4;
const FRAMES: Tick = 320;
/// Propulsion Labs' price and its construction (module docs).
const LABS_COST: u32 = 200;
const LABS_FRAMES: Tick = 400;

/// The bank of [`labs`]: plenty of Space Crystals and Supplies.
const BANK: u32 = 5000;
const BANK_SUPPLIES: u32 = 40;

/// W at `airfield`: a Train of a Dragonfly.
fn train(airfield: u32) -> Command {
    Command::Production {
        objects: vec![id(airfield)],
        command: ProductionCommand::Train(DRAGONFLY),
    }
}

/// X at `airfield`: its Cancel Production.
fn cancel(airfield: u32) -> Command {
    Command::Production {
        objects: vec![id(airfield)],
        command: ProductionCommand::CancelProduction,
    }
}

fn construction(command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![id(1)],
        command,
    }
}

fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

fn supplies(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").supplies
}

fn queued(sim: &Sim, producer: u32) -> Vec<ObjectKind> {
    let production = object(sim, producer).production.as_ref().expect("a producer");
    production.queue.iter().filter_map(|entry| entry.unit()).collect()
}

/// The units produced so far: (tick, producer, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    let unit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } => Some((*tick, producer, unit)),
        _ => None,
    };
    t.events().iter().filter_map(unit).collect()
}

/// Steps until a unit comes out of `producer`, at most two of its trainings on: the tick and the unit.
fn until_produced(t: &mut TestSim, producer: u32) -> (Tick, u32) {
    let before = produced(t).len();
    let until = t.sim().tick() + 2 * FRAMES;
    loop {
        if let Some(&(tick, by, unit)) = produced(t).get(before) {
            assert_eq!(by, id(producer));
            return (tick, unit.0);
        }
        assert!(t.sim().tick() < until, "nothing out of #{producer} by tick {until}");
        t.step();
    }
}

/// [`field`] with a third Generator (7) for Propulsion Labs' "Power -20" (`factions/gdo/structures/propulsion_labs.md`)
/// and, if `built`, Propulsion Labs (8) on tiles (6, 7) to (7, 8), below the Deployment Center; and [`BANK`] and
/// [`BANK_SUPPLIES`] from tick 2.
fn labs(built: bool) -> TestSim {
    let mut t = field(|builder| {
        let builder = builder.spawn(ObjectKind::Generator, Some(P1), tile(17, 2));
        if built {
            builder.spawn(LABS, Some(P1), tile(6, 7))
        } else {
            builder
        }
    });
    bank(&mut t, BANK, BANK_SUPPLIES);
    t
}

/// The Deployment Center's R, Propulsion Labs, with an Airfield on the map, for 200 Space Crystals and 400 frames, and
/// placed: until then the Airfield's W is refused (`Reject::Requires`, the client's greyed button and its "Requires
/// Propulsion Labs"), from then on the gate takes it.
#[test]
fn the_deployment_centers_propulsion_labs_unlocks_the_airfields_dragonfly() {
    let mut t = labs(false);
    let requires = Err(Reject::Requires(LABS));
    assert_eq!(gate(t.sim(), P1, &train(A)), requires, "greyed");
    next(&mut t, P1, construction(ConstructionCommand::Construct(LABS)));
    let began = t.sim().tick();
    assert_eq!(crystals(t.sim()), BANK - LABS_COST);
    let constructor = |sim: &Sim| object(sim, 1).constructor.expect("a constructor");
    step_until(&mut t, LABS_FRAMES + 2, "Propulsion Labs", |sim| {
        constructor(sim).ready().is_some()
    });
    assert_eq!(t.sim().tick() - began + 1, LABS_FRAMES, "400 frames");
    assert_eq!(constructor(t.sim()), Constructor::Ready { kind: LABS });
    assert_eq!(gate(t.sim(), P1, &train(A)), requires, "not placed yet");
    let place = ConstructionCommand::Place {
        origin: TilePos::new(6, 7),
        orientation: BuildingOrientation::default(),
    };
    next(&mut t, P1, construction(place));
    let placed = t.sim().objects().find(|o| o.kind == LABS);
    assert_eq!(placed.map(|o| o.owner), Some(Some(P1)), "placed");
    assert_eq!(gate(t.sim(), P1, &train(A)), Ok(()));
    assert_eq!(gate(t.sim(), P1, &train(B)), Ok(()), "at every Airfield");
}

/// "**W** Dragonfly" with 3 Supplies: the gate refuses it (`Reject::InsufficientSupplies`), and with 174 Space
/// Crystals (`Reject::InsufficientCrystals`), the bank keeping everything. With 175 and 4 the Airfield takes both as it
/// queues the Dragonfly; a cancel (X) gives both back.
#[test]
fn the_dragonfly_costs_175_crystals_and_4_supplies() {
    let mut t = labs(true);
    bank(&mut t, COST, SUPPLIES - 1);
    assert_eq!(gate(t.sim(), P1, &train(A)), Err(Reject::InsufficientSupplies));
    bank(&mut t, COST - 1, SUPPLIES);
    assert_eq!(gate(t.sim(), P1, &train(A)), Err(Reject::InsufficientCrystals));
    bank(&mut t, COST, SUPPLIES);
    next(&mut t, P1, train(A));
    assert_eq!(queued(t.sim(), A), [DRAGONFLY]);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (0, 0), "both paid");
    next(&mut t, P1, cancel(A));
    assert_eq!(queued(t.sim(), A), []);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (COST, SUPPLIES), "both back");
}

/// The Dragonfly comes out 320 frames after its W, landed on the first slot of its Airfield (design change 0428 point
/// 2), assigned to it with its missile, and, with no rally point, stays there without moving ("Landed, it doesn't
/// move"). The next, with a rally point set on the ground, appears on the next free slot, facing it; it takes off
/// straight up and leaves from IdleSpeed, flies there and circles it, as a Glider at its goal does.
#[test]
fn a_new_dragonfly_appears_landed_and_takes_off_straight_up_for_its_rally_point() {
    let mut t = labs(true);
    next(&mut t, P1, train(A));
    let queued_at = t.sim().tick();
    let (tick, first) = until_produced(&mut t, A);
    assert_eq!(tick, queued_at + FRAMES - 1, "320 frames");
    let o = object(t.sim(), first);
    assert_eq!(
        (o.kind, o.owner, o.pos),
        (DRAGONFLY, Some(P1), A_SLOTS[0]),
        "on the first slot"
    );
    assert_eq!(
        (o.aircraft, o.landing),
        (
            Some(AircraftState {
                ammo: AMMO,
                base: Some(id(A)),
            }),
            Some(sc_sim::landing::down_on(id(A), LANDING)),
        ),
        "landed, full"
    );
    for _ in 0..40 {
        t.step();
        let f = frame(t.sim(), first);
        assert_eq!(
            (f.pos, f.speed, f.down),
            (A_SLOTS[0], 0.0, true),
            "it stays there, still"
        );
        assert_eq!(mover(t.sim(), first).velocity, Vec2::ZERO);
    }

    let rally = A_AT + Vec2::new(0.0, 640.0);
    let set_rally = Command::Production {
        objects: vec![id(A)],
        command: ProductionCommand::SetRally(Target::Point(rally)),
    };
    next(&mut t, P1, set_rally);
    next(&mut t, P1, train(A));
    let (_, second) = until_produced(&mut t, A);
    let o = object(t.sim(), second);
    assert_eq!(o.pos, A_SLOTS[1], "the next free slot");
    let south = math::vector_angle(rally - A_SLOTS[1]);
    assert!(
        math::wrap_angle(o.facing - south).abs() < 1e-4,
        "facing its rally point: {}",
        o.facing
    );
    assert_eq!(doing(t.sim(), second), Some(OrderKind::Move(Target::Point(rally))));
    assert!(landing_of(t.sim(), second).is_some(), "landed");
    // Its rally point's order lifts it off in its first tick.
    t.step();
    rises_off(&mut t, second, A_SLOTS[1]);
    step_until(&mut t, 400, "circling its rally point", |sim| {
        mover(sim, second).orbit == Some(rally)
    });
    for _ in 0..64 {
        t.step();
        flies_or_is_held(frame(t.sim(), second));
        assert!(
            t.pos(second).distance(rally) < 3.0 * 64.0,
            "round its rally point: {}",
            t.pos(second)
        );
    }
    assert_eq!(base(t.sim(), second), Some(A), "still assigned to its Airfield");
    assert_eq!(t.pos(first), A_SLOTS[0], "the first still landed");
}
