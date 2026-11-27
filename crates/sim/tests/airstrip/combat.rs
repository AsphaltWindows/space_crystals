//! The Sparrow in a fight (`super`; `factions/cults/units/sparrow.md`: "A **small glider** built at the Airstrip, with a
//! **machine-gun-style** attack that hits **both air and ground**"): out of the Airstrip, it strafes a group of Agents,
//! which can't hit it; and who can hit it at all (`systems/combat.md > ValidTarget`: an attack's TargetDomain).

use sc_sim::defs::{Domain, TargetDomain};

use super::*;

/// Three Agents of player 2's, the Syndicate's, south-west of the Airstrip.
const AGENTS: Vec2 = Vec2::new(300.0, 1000.0);

/// The plan's acceptance: "It never lands, and it strafes a group of Agents". Player 1's Airstrip trains a Sparrow,
/// auto-targeting on, which is given an Attack Move on three Agents as it comes out; the order waits for the end of its
/// takeoff run, and then it flies at them and strafes them, one after the other (its turret also fires at whatever
/// comes within its arc and Range between passes, as any turret does): it hits the Agents only, most of its hits on
/// strafing passes (`Mover::strafe`), and kills all three, flying all along (the Glider invariant G1 checks its speed
/// every tick) and never inside a building again. The Agents, whose attack is Ground only
/// (`factions/syndicate/units/agent.md`: "TargetDomain: Ground"), never hit it.
#[test]
fn a_sparrow_out_of_the_airstrip_strafes_a_group_of_agents_which_can_t_hit_it() {
    let scenario = ScenarioBuilder::new(&flat(30, 22))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Syndicate)
        .spawn(AIRSTRIP, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2));
    let (mut t, order) = filled_on(scenario, 1);
    let first = next_id(&t);
    for (dx, dy) in [(-40.0, 0.0), (40.0, 0.0), (0.0, 50.0)] {
        t.spawn_unit_at(t.sim().tick() + 1, ObjectKind::Agent, P2, AGENTS + Vec2::new(dx, dy));
    }
    let agents: Vec<ObjectId> = (first..first + 3).map(id).collect();
    let start = t.sim().tick() + 1;
    t.debug_at(start, DebugCommand::SetAutoTargeting { on: true });
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    let (_, sparrow) = until_produced(&mut t, FRAMES + 2);
    assert_eq!(sparrow, order[0]);
    t.order_at(t.sim().tick() + 1, P1, &[sparrow], OrderKind::AttackMove(AGENTS), false);
    let mut seen = hits(&t).len();
    let (mut fired, mut strafing, mut dead) = (0, 0, Vec::new());
    while dead.len() < agents.len() {
        assert!(t.sim().tick() < start + 16 * 240, "{dead:?} dead, {fired} hits");
        step_checked(&mut t);
        assert!(t.sim().held_unit(sparrow).is_none(), "never lands");
        let all = hits(&t);
        for &(_, attacker, target, _) in &all[seen..] {
            assert_eq!(attacker, sparrow, "only the Sparrow hits: {:?}", &all[seen..]);
            assert!(agents.contains(&target), "{target:?}");
            strafing += usize::from(t.mover(sparrow.0).strafe.is_some());
            fired += 1;
        }
        seen = all.len();
        for &agent in &agents {
            if t.sim().object(agent).is_none() && !dead.contains(&agent) {
                dead.push(agent);
            }
        }
    }
    assert!(fired >= 3 * 38, "75 HP, PointArmor 1: 2 HP a hit, {fired} hits");
    assert!(
        strafing * 2 > fired,
        "{strafing} of its {fired} hits on strafing passes"
    );
    println!(
        "{strafing} of its {fired} hits on strafing passes, until tick {}",
        t.sim().tick()
    );
    let o = t.sim().object(sparrow).expect("alive");
    assert_eq!(o.hp, Some(50.0), "unhurt");
}

/// Player 2's `kind` (faction `faction`), on Hold, under player 1's Sparrow, which holds there, circling it, both with
/// auto-targeting on: the Sparrow 64 su from it, within any attack's Range. A Laser Turret is player 2's on the east
/// edge of its Tunnel's area (`factions/syndicate/structures/laser_turret.md`: it rises and fires while its Tunnel
/// stands). Returns the hits the attacker landed on the Sparrow in 20 s, and its right-click on it.
fn under_a_sparrow(kind: ObjectKind, faction: Faction) -> (usize, Option<OrderKind>) {
    // The popup tests' layout: the Tunnel on tiles (4, 8) to (7, 11), the turret on (9, 10), its centre `at`.
    let at = Vec2::new(9.5 * 64.0, 10.5 * 64.0);
    let turret = kind == ObjectKind::LaserTurret;
    let mut scenario = ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Cults)
        .faction(P2, faction)
        .spawn(SPARROW, Some(P1), SpawnAt::Point(at + Vec2::new(0.0, -IDLE_RADIUS)))
        .spawn(ObjectKind::Storage, Some(P1), tile(24, 2));
    scenario = if turret {
        scenario
            .spawn(ObjectKind::Tunnel, Some(P2), tile(4, 8))
            .spawn(kind, Some(P2), tile(9, 10))
    } else {
        scenario.spawn(kind, Some(P2), SpawnAt::Point(at))
    };
    let mut t = scenario.start();
    let (sparrow, attacker) = (id(1), if turret { id(4) } else { id(3) });
    t.order_at(1, P1, &[sparrow], OrderKind::Hold, false);
    if !turret {
        t.order_at(1, P2, &[attacker], OrderKind::Hold, false);
    }
    t.run(2);
    let right_click = t.sim().default_order(attacker, Target::Object(sparrow));
    let mut landed = 0;
    let mut seen = 0;
    for _ in 0..16 * 20 {
        if t.sim().object(sparrow).is_none() || t.sim().object(attacker).is_none() {
            break;
        }
        step_checked(&mut t);
        let all = hits(&t);
        landed += all[seen..]
            .iter()
            .filter(|&&(_, by, target, _)| (by, target) == (attacker, sparrow))
            .count();
        seen = all.len();
    }
    (landed, right_click)
}

/// Who can hit it (the plan's acceptance): "Militants, Guards and the Laser Turret hit it. Peacekeepers, Legionnaires
/// and Agents can't (their TargetDomain is Ground)", and (slice M5-N6) the Seeker ("it hits a Sparrow (Universal)":
/// `factions/cults/units/seeker.md`, "TargetDomain: **Universal** (ground and air)"), its turret on it. Each under a
/// Sparrow circling it on Hold for 20 s: the first four, whose attacks are Universal, land hits on it, and their
/// right-click on it is an Attack; the others, Ground only, land none, and their right-click on it follows it
/// (`systems/controls/commands.md`: a unit that can't attack a target moves to it). The Rocketman comes with slice K.
#[test]
fn militants_guards_the_laser_turret_and_the_seeker_hit_it_and_ground_attacks_can_t() {
    let cases = [
        (ObjectKind::Militant, Faction::Cults, true),
        (ObjectKind::Guard, Faction::Syndicate, true),
        (ObjectKind::LaserTurret, Faction::Syndicate, true),
        (ObjectKind::Seeker, Faction::Cults, true),
        (ObjectKind::Peacekeeper, Faction::Gdo, false),
        (ObjectKind::SabreLegionnaire, Faction::Colonists, false),
        (ObjectKind::RifleLegionnaire, Faction::Colonists, false),
        (ObjectKind::Agent, Faction::Syndicate, false),
    ];
    for (kind, faction, hits_air) in cases {
        let def = kind.def();
        let attack = (def.unit.as_ref().and_then(|unit| unit.attack.as_ref()))
            .or_else(|| def.structure.as_ref().and_then(|structure| structure.attack.as_ref()));
        let domain = attack.map(|attack| attack.target_domain);
        let expected = if hits_air {
            TargetDomain::Universal
        } else {
            TargetDomain::Ground
        };
        assert_eq!(domain, Some(expected), "{kind:?}");
        assert!(sc_sim::combat::domain_allows(expected, Domain::Air) == hits_air);
        let (landed, right_click) = under_a_sparrow(kind, faction);
        if hits_air {
            assert!(landed >= 3, "{kind:?}: {landed} hits");
            if kind != ObjectKind::LaserTurret {
                assert_eq!(right_click, Some(OrderKind::Attack(id(1))), "{kind:?}");
            }
        } else {
            assert_eq!(landed, 0, "{kind:?}");
            assert_eq!(right_click, Some(OrderKind::Move(Target::Object(id(1)))), "{kind:?}");
        }
    }
}
