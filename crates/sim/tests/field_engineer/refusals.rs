//! The gate's refusals of a Field Engineer's Repair (`Reject::CantRepair`), the repairers' one rule
//! (`sc_sim::repair::can_repair`: an object of its owner's, complete and below its MaxHP, of a kind its type repairs),
//! as for Recruits (`factions/gdo/units/field_engineer.md > Placeholder stats`: "Repair works through the same shared
//! rules as the Recruit's"; design change 0329's verification: "Refused: a site under construction, a target at full
//! HP, an enemy's target").

use sc_sim::SpawnAt;

use super::*;

// The objects of [`refusals`]' game, by id, after [`gdo`]'s.
/// Player 1's two Field Engineers, east of their Barracks.
const ENGINEERS: [u32; 2] = [5, 6];
/// Player 1's Recruit, which starts a Storage's site: the GDO's own buildings come complete out of their constructor,
/// so a site of player 1's takes another faction's worker (a scenario's spawn: nothing ties a worker to its owner's
/// faction).
const RECRUIT: u32 = 7;
/// Player 2's Barracks, on tiles (18, 9) to (20, 10): in the Field Engineers' sight.
const ENEMY_BARRACKS: u32 = 8;
/// Where the Recruit builds the Storage: tiles (14, 14) to (15, 15).
const SITE: TilePos = TilePos::new(14, 14);

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

/// [`gdo`]'s game with a Career Campus, [`ENGINEERS`], [`RECRUIT`] and [`ENEMY_BARRACKS`].
fn refusals() -> TestSim {
    gdo(|scenario| {
        scenario
            .spawn(FIELD_ENGINEER, Some(P1), point(16, 7))
            .spawn(FIELD_ENGINEER, Some(P1), point(15, 8))
            .spawn(ObjectKind::Recruit, Some(P1), point(16, 12))
            .spawn(ObjectKind::Barracks, Some(P2), tile(18, 9))
    })
}

/// What a Repair of `target` by the first Field Engineer gets from the gate in `sim`.
fn repairing(sim: &Sim, target: u32) -> Result<(), Reject> {
    gate(sim, repair(&[id(ENGINEERS[0])], target))
}

/// The Field Engineer's Repair is refused on its own Barracks at full HP, on player 2's damaged Barracks, which it
/// sees, on the site of player 1's Storage, which its Recruit builds, and on the other Field Engineer, damaged
/// (infantry is no vehicle: `sc_sim::repair::is_vehicle`); on its own damaged Barracks the gate takes it, and the
/// Field Engineer is at work on it.
#[test]
fn the_gate_refuses_a_field_engineers_repair_of_what_cant_be_repaired() {
    let t = refusals();
    assert_eq!(
        repairing(t.sim(), BARRACKS),
        Err(Reject::CantRepair(id(BARRACKS))),
        "at full HP"
    );
    let sim = damaged(t.sim(), ENEMY_BARRACKS, 150.0);
    let sim = damaged(&sim, ENGINEERS[1], 20.0);
    let mut sim = damaged(&sim, BARRACKS, 150.0);
    assert!(sim.is_visible(P1, sim.object(id(ENEMY_BARRACKS)).expect("player 2's Barracks")));
    let build = OrderKind::Build {
        kind: ObjectKind::Storage,
        origin: SITE,
        orientation: Default::default(),
    };
    let recruit = Command::Order {
        units: vec![id(RECRUIT)],
        kind: build,
        queue: false,
    };
    step(&mut sim, vec![recruit]);
    let mut site = None;
    for _ in 0..LONGEST {
        step(&mut sim, Vec::new());
        site = sim.objects().find(|o| o.is_site()).map(|o| o.id.0);
        if site.is_some() {
            break;
        }
    }
    let site = site.expect("the Recruit started the site");
    let refused = [
        (ENEMY_BARRACKS, "an enemy's building"),
        (site, "a site under construction"),
        (ENGINEERS[1], "infantry"),
    ];
    for (target, why) in refused {
        assert_eq!(repairing(&sim, target), Err(Reject::CantRepair(id(target))), "{why}");
    }
    assert_eq!(repairing(&sim, BARRACKS), Ok(()));
    let before = hp(&sim, BARRACKS);
    step(&mut sim, vec![repair(&[id(ENGINEERS[0])], BARRACKS)]);
    for _ in 0..LONGEST {
        if hp(&sim, BARRACKS) > before {
            break;
        }
        step(&mut sim, Vec::new());
    }
    assert!(hp(&sim, BARRACKS) > before, "repaired");
}
