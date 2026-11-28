//! The Career Campus and its Field Engineers (`core/tech_and_research.md > Tech Prerequisites`, slice M4-K's one rule):
//! the Barracks' "Field Engineer [...] Requires: player owns at least one Career Campus"
//! (`factions/gdo/structures/barracks.md > Produces`), and the Career Campus's own TechPrerequisite, "player owns at
//! least one Barracks" (`factions/gdo/structures/career_campus.md`); and what it adds to its owner's build area.

use sc_sim::{ConstructionCommand, Constructor};

use super::*;

/// Player 1's Career Campus in [`with_campus`]'s game, on tiles (8, 7) to (9, 8).
const CAMPUS: u32 = 5;

/// [`gdo`]'s game with [`CAMPUS`].
fn with_campus() -> TestSim {
    gdo(|scenario| scenario.spawn(CAREER_CAMPUS, Some(P1), tile(8, 7)))
}

/// The Deployment Center's Construct of a Career Campus: BuildMenu\[Basic\]'s D.
fn construct_campus() -> Command {
    Command::Construction {
        objects: vec![id(DC)],
        command: ConstructionCommand::Construct(CAREER_CAMPUS),
    }
}

/// "Requires: player owns at least one Barracks" (`factions/gdo/structures/deployment_center.md > Constructs`): with
/// its only Barracks gone, player 1's Deployment Center is refused the Career Campus (`Reject::Requires`, the client's
/// "Requires a Barracks") and stays idle; with a Barracks it takes it, for 150 Space Crystals.
#[test]
fn the_career_campus_requires_a_barracks() {
    let mut t = gdo(|scenario| scenario);
    assert_eq!(gate(t.sim(), construct_campus()), Ok(()));
    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(BARRACKS) }));
    let refused = Err(Reject::Requires(ObjectKind::Barracks));
    assert_eq!(gate(t.sim(), construct_campus()), refused);
    next(&mut t, construct_campus());
    let dc = t.sim().object(id(DC)).expect("the Deployment Center");
    assert_eq!((dc.constructor, crystals(t.sim())), (Some(Constructor::Idle), BANK));

    let mut t = gdo(|scenario| scenario);
    next(&mut t, construct_campus());
    let dc = t.sim().object(id(DC)).expect("the Deployment Center");
    assert!(matches!(
        dc.constructor,
        Some(Constructor::Constructing {
            kind: CAREER_CAMPUS,
            ..
        })
    ));
    assert_eq!(crystals(t.sim()), BANK - CAREER_CAMPUS_COST);
}

/// `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`, "Losing a Prerequisite": "units already
/// queued or in production continue and complete normally; new ones cannot be queued until the player again owns a
/// qualifying prerequisite structure". Two Field Engineers queued, the first in production, and then the only Career
/// Campus destroyed: the Barracks is refused a third (E greyed again, "Requires a Career Campus"), and both queued
/// ones come out, for 75 Space Crystals each and 1 Supply (`factions/gdo/units/field_engineer.md > Placeholder stats`).
#[test]
fn field_engineers_queued_before_the_campus_is_lost_come_out() {
    let mut t = with_campus();
    next(&mut t, train());
    next(&mut t, train());
    let production = t.sim().object(id(BARRACKS)).and_then(|o| o.production.clone());
    let production = production.expect("a producer");
    assert_eq!(
        (production.in_production(), production.queue.len()),
        (Some(FIELD_ENGINEER), 2)
    );
    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(CAMPUS) }));
    assert!(!t.sim().requirement_met(P1, CAREER_CAMPUS));
    assert_eq!(gate(t.sim(), train()), Err(Reject::Requires(CAREER_CAMPUS)));
    let produced = until_produced(&mut t, 2);
    for (_, unit) in produced {
        let o = t.sim().object(unit).expect("out of the Barracks");
        assert_eq!((o.kind, o.owner), (FIELD_ENGINEER, Some(P1)));
    }
    assert_eq!(crystals(t.sim()), BANK - 2 * FIELD_ENGINEER_COST);
    assert_eq!(t.sim().player(P1).map(|p| p.supplies), Some(SUPPLIES - 2));
    let production = t.sim().object(id(BARRACKS)).and_then(|o| o.production.clone());
    assert_eq!(production.map(|p| p.queue.len()), Some(0), "no third");
}

/// "BuildRadiusExtension 2 grid units" (`factions/gdo/structures/career_campus.md`;
/// `factions/gdo/structures/README.md > GDOBuildArea`): a Career Campus of player 1's on tiles (24, 10) to (25, 11),
/// east of what their other buildings reach (tile 17), adds tiles (22, 8) to (27, 13) to their build area, its
/// footprint grown by 2 on every side, and not a tile more.
#[test]
fn the_career_campus_extends_the_build_area_by_2() {
    let t = gdo(|scenario| scenario.spawn(CAREER_CAMPUS, Some(P1), tile(24, 10)));
    let area = t.sim().build_area(P1);
    let edges = [
        ((22, 10), (21, 10)),
        ((27, 11), (28, 11)),
        ((24, 8), (24, 7)),
        ((25, 13), (25, 14)),
    ];
    for ((x, y), (out_x, out_y)) in edges {
        assert!(area.contains(TilePos::new(x, y)), "({x}, {y}) is in the area");
        assert!(!area.contains(TilePos::new(out_x, out_y)), "({out_x}, {out_y}) is not");
    }
    let without = gdo(|scenario| scenario);
    assert!(!without.sim().build_area(P1).contains(TilePos::new(22, 10)));
}
