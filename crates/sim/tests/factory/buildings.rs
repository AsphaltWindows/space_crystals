//! The Deployment Center's three new buildings (`factions/gdo/structures/deployment_center.md > Constructs`, design
//! change 0366): "Factory: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at least
//! one Barracks", "Advanced Mechanics: 200 Space Crystals, 400 frames (25 seconds) (placeholder). Requires: player
//! owns at least one Factory" and "Industrial Powerplant: 250 Space Crystals, 480 frames (30 seconds) (placeholder).
//! Requires: player owns at least one Advanced Mechanics", each the TechPrerequisite of its own page
//! (`core/tech_and_research.md > Tech Prerequisites`); and the Powerplant's "Power +60"
//! (`factions/gdo/structures/industrial_powerplant.md`).

use sc_sim::{BuildingOrientation, ConstructionCommand, Constructor, PowerSummary};

use super::*;

const DC: ObjectId = ObjectId(1);
const MECHANICS: ObjectKind = ObjectKind::AdvancedMechanics;
const POWERPLANT: ObjectKind = ObjectKind::IndustrialPowerplant;

fn construction(command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![DC],
        command,
    }
}

fn construct(kind: ObjectKind) -> Command {
    construction(ConstructionCommand::Construct(kind))
}

/// The Deployment Center's constructor state.
fn constructor(t: &TestSim) -> Constructor {
    object(t, DC).constructor.expect("a constructor")
}

/// The constructions completed so far: (tick, building).
fn completions(t: &TestSim) -> Vec<(Tick, ObjectKind)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ConstructionCompleted { kind, .. } => Some((*tick, kind)),
            _ => None,
        })
        .collect()
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
}

/// The chain of TechPrerequisites, each refused without the one before it (`Reject::Requires`, the client's "Requires
/// a ..."), the Deployment Center left idle and the bank whole; and taken with it, for its cost, the Deployment
/// Center constructing it: the Factory needs a Barracks, Advanced Mechanics a Factory, the Industrial Powerplant
/// Advanced Mechanics. Another player's doesn't count: with the Barracks gone and a Factory of player 2's on the map,
/// player 1 is refused all three.
#[test]
fn each_building_needs_the_one_before_it() {
    let steps = [
        (FACTORY, ObjectKind::Barracks, 250),
        (MECHANICS, FACTORY, 200),
        (POWERPLANT, MECHANICS, 250),
    ];
    // `base` has the Barracks; each step adds the building the next one needs, player 1's own.
    let with = |i: usize| {
        let mut scenario = base();
        for (j, &(before, _, _)) in steps[..i].iter().enumerate() {
            scenario = scenario.spawn(before, Some(P1), tile(10 + 4 * j as i32, 20));
        }
        scenario
    };
    for (i, &(kind, needs, cost)) in steps.iter().enumerate() {
        // The one it needs is gone, and player 2 has one.
        let mut t = started(with(i).spawn(needs, Some(P2), tile(30, 4)), 0);
        if i == 0 {
            next(&mut t, Command::Debug(DebugCommand::Remove { id: BARRACKS }));
        } else {
            let own = (t.sim().objects()).find(|o| o.kind == needs && o.owner == Some(P1));
            let own = own.expect("player 1's").id;
            next(&mut t, Command::Debug(DebugCommand::Remove { id: own }));
        }
        assert_eq!(gate(&t, P1, construct(kind)), Err(Reject::Requires(needs)), "{kind:?}");
        next(&mut t, construct(kind));
        assert_eq!((constructor(&t), bank(&t).0), (Constructor::Idle, BANK), "{kind:?}");

        let mut t = started(with(i), 0);
        assert!(t.sim().requirement_met(P1, needs), "{kind:?}");
        assert_eq!(gate(&t, P1, construct(kind)), Ok(()), "{kind:?}");
        next(&mut t, construct(kind));
        assert!(
            matches!(constructor(&t), Constructor::Constructing { kind: k, .. } if k == kind),
            "{kind:?}: {:?}",
            constructor(&t)
        );
        assert_eq!(bank(&t).0, BANK - cost, "{kind:?}");
    }
    let mut t = started(base().spawn(FACTORY, Some(P2), tile(30, 4)), 0);
    next(&mut t, Command::Debug(DebugCommand::Remove { id: BARRACKS }));
    for (kind, needs) in [
        (FACTORY, ObjectKind::Barracks),
        (MECHANICS, FACTORY),
        (POWERPLANT, MECHANICS),
    ] {
        assert_eq!(gate(&t, P1, construct(kind)), Err(Reject::Requires(needs)), "{kind:?}");
    }
}

/// Their times (`docs/architecture/economy.md > Economy > Recipe timing`): ordered in tick 2, the Factory is ready at the end of
/// tick 481 (480 frames, the command's own tick the first), and so is the Industrial Powerplant; Advanced Mechanics,
/// in 400 frames, at the end of tick 401. Each Deployment Center builds at full speed (its construction is never
/// slowed by Power, design change 0089).
#[test]
fn they_take_their_frames() {
    for (kind, frames) in [(FACTORY, 480), (MECHANICS, 400), (POWERPLANT, 480)] {
        let mut t = started(
            base()
                .spawn(FACTORY, Some(P1), tile(16, 10))
                .spawn(MECHANICS, Some(P1), tile(16, 16)),
            0,
        );
        next(&mut t, construct(kind));
        t.run_until(2 + frames);
        assert_eq!(completions(&t), [(1 + frames, kind)], "{kind:?}");
        assert_eq!(constructor(&t), Constructor::Ready { kind }, "{kind:?}");
    }
}

/// "Power +60" (`factions/gdo/structures/industrial_powerplant.md`), in the Power summary (`Sim::power`): player 1
/// starts with 90 generated (the Deployment Center's 30 and three Generators' 20 each) and 80 used (the Barracks' and
/// the Factory's 30 each, Advanced Mechanics' 20); the Deployment Center constructs an Industrial Powerplant and it is
/// placed, and from that tick the summary reads 150 generated and 80 used. Advanced Mechanics' −20 and the Factory's
/// −30 are counted from the start.
#[test]
fn the_industrial_powerplant_adds_60_to_the_power_summary() {
    let mut t = started(
        base()
            .spawn(ObjectKind::Generator, Some(P1), tile(8, 8))
            .spawn(FACTORY, Some(P1), tile(16, 10))
            .spawn(MECHANICS, Some(P1), tile(16, 16)),
        0,
    );
    let summary = |generated, consumed| PowerSummary { generated, consumed };
    assert_eq!(t.sim().power(P1), summary(90, 80));
    next(&mut t, construct(POWERPLANT));
    while constructor(&t).ready().is_none() {
        assert!(t.sim().tick() < 600, "{:?}", constructor(&t));
        t.step();
    }
    let place = ConstructionCommand::Place {
        origin: TilePos::new(7, 2),
        orientation: BuildingOrientation::default(),
    };
    assert_eq!(gate(&t, P1, construction(place)), Ok(()));
    next(&mut t, construction(place));
    let plant = (t.sim().objects()).find(|o| o.kind == POWERPLANT);
    assert_eq!(plant.map(|o| o.owner), Some(Some(P1)), "placed");
    assert_eq!(t.sim().power(P1), summary(150, 80));
}
