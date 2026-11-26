//! The gate's refusals of a Repair (`Reject::CantRepair`): of an object that no recipient can repair now
//! (`Sim::can_repair`: an object of its owner's, complete and below its MaxHP, of a kind its type repairs).

use super::*;

/// Where the Recruits build a Storage, west of where they stand: tiles (6, 14) to (8, 15).
const SITE: TilePos = TilePos::new(6, 14);

/// Design change 0329's verification, "Refused: a site under construction, a target at full HP, an enemy's target",
/// and what repairs nothing: the Armory at full HP; a Storage's site, which the Recruits build instead
/// (`factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed)"); player
/// 2's damaged Armory, which player 1's Recruits see; a damaged Recruit, infantry being no vehicle; and a Militant's
/// Repair, which repairs nothing. Of a Militant and a Recruit ordered together, the Recruit takes the Repair and the
/// Militant keeps its orders.
#[test]
fn the_gate_refuses_a_repair_of_what_cant_be_repaired() {
    let t = cults();
    let recruit = RECRUITS[0];
    assert_eq!(
        gate(t.sim(), repair(&[recruit], ARMORY)),
        Err(Reject::CantRepair(id(ARMORY))),
        "at full HP"
    );
    let sim = damaged(t.sim(), ARMORY, 150.0);
    let sim = damaged(&sim, ENEMY_ARMORY, 150.0);
    let mut sim = damaged(&sim, RECRUITS[1], 20.0);
    let build = OrderKind::Build {
        kind: ObjectKind::Storage,
        origin: SITE,
        orientation: Default::default(),
    };
    step(&mut sim, vec![order(&RECRUITS[2..], build, false)]);
    let mut site = None;
    for _ in 0..WALK {
        run(&mut sim, 1);
        site = sim.objects().find(|o| o.is_site()).map(|o| o.id.0);
        if site.is_some() {
            break;
        }
    }
    let site = site.expect("the Recruits started the site");
    let refused = [
        (&[recruit][..], ENEMY_ARMORY, "an enemy's building"),
        (&[recruit], RECRUITS[1], "infantry"),
        (&[recruit], site, "a site under construction"),
        (&[MILITANT], ARMORY, "a Militant repairs nothing"),
    ];
    for (units, target, why) in refused {
        assert_eq!(
            gate(&sim, repair(units, target)),
            Err(Reject::CantRepair(id(target))),
            "{why}"
        );
    }
    let both = repair(&[MILITANT, recruit], ARMORY);
    assert_eq!(gate(&sim, both.clone()), Ok(()));
    step(&mut sim, vec![both]);
    assert_eq!(current(&sim, recruit), Some(OrderKind::Repair(id(ARMORY))));
    assert_eq!(current(&sim, MILITANT), None, "the Militant keeps its orders: none");
}

/// "Agents can't repair expansions" (`factions/syndicate/structures/README.md > Tunnel Expansions`: "**Not repaired;
/// they regenerate**"; design change 0329's verification: "an expansion (for an Agent)"): a damaged Hideout is refused,
/// and so is the Laser Turret at full HP; the damaged Tunnel is taken.
#[test]
fn an_agent_repairs_no_expansion() {
    let t = syndicate();
    assert_eq!(
        gate(t.sim(), repair(&[AGENT], TURRET)),
        Err(Reject::CantRepair(id(TURRET))),
        "at full HP"
    );
    let sim = damaged(t.sim(), HIDEOUT, 200.0);
    let sim = damaged(&sim, T1, 300.0);
    assert_eq!(
        gate(&sim, repair(&[AGENT], HIDEOUT)),
        Err(Reject::CantRepair(id(HIDEOUT)))
    );
    assert_eq!(gate(&sim, repair(&[AGENT], T1)), Ok(()));
}
