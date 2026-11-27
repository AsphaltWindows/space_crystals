//! A unit that deploys or has deployed is "a fixed obstacle that never yields" (`m6-plan.md > GA`;
//! `systems/units.md > UnitCollision`: units that can't move never yield): it has an order while it deploys, and
//! deployed its type can't move (plan decision T87), so it takes no sidestep (`movement::yielding::can_yield`) and no
//! idle push (`movement::separate`).

use super::*;
use sc_sim::TilePos;

/// Acceptance, "Deployed, it never yields: a friendly unit goes round it": a Peacekeeper of the Rig's owner, sent from
/// 3 grid units west of it to 3 east, straight through it, gets there round it; the Rig doesn't move a hair, any
/// frame.
#[test]
fn a_friend_goes_round_the_deployed_rig() {
    let pk = id(2);
    let mut t = deployed(|b| b.spawn(PK, Some(P1), SpawnAt::Point(from_home(gu(-3.0), 0.0))));
    let at = place(&t, UNIT);
    let goal = from_home(gu(3.0), 0.0);
    give(&mut t, &[pk], move_to(goal), false);
    for _ in 0..300 {
        t.step();
        assert_eq!(place(&t, UNIT), at, "tick {}", t.sim().tick());
    }
    assert!(t.pos(pk.0).distance(goal) < 8.0, "round it: {}", t.pos(pk.0));
}

/// A corridor one tile wide between mountains, the Rig in it: tile row 4, the Rig (1) at tile (10, 4), facing east
/// along it, and player 1's Peacekeeper (2) at tile (5, 4), west of it.
fn corridor() -> TestSim {
    let wall = "M".repeat(20);
    let rows = [&wall, &wall, &wall, &wall, &".".repeat(20), &wall, &wall, &wall, &wall];
    let map = rows.map(|row| row.as_str()).join("\n");
    let mut t = ScenarioBuilder::new(&map)
        .spawn(RIG, Some(P1), SpawnAt::Point(TilePos::new(10, 4).center()))
        .spawn(PK, Some(P1), SpawnAt::Point(TilePos::new(5, 4).center()))
        .start();
    t.without_auto_targeting();
    t.run_until(1);
    t
}

/// "Nothing pushes it": in a corridor that the Rig fills, a friendly Peacekeeper sent past it, east, comes up against
/// it, and the Rig neither steps aside nor is pushed, deploying (it has an order) or deployed (it can't move); the
/// Peacekeeper stays on its side.
#[test]
fn nothing_pushes_a_deploying_or_deployed_rig() {
    let pk = id(2);
    let mut t = corridor();
    let at = place(&t, UNIT);
    let given = give(&mut t, &[UNIT], deploy(), false);
    give(&mut t, &[pk], move_to(TilePos::new(15, 4).center()), false);
    for _ in 0..400 {
        t.step();
        assert_eq!(place(&t, UNIT), at, "tick {}", t.sim().tick());
    }
    assert!(t.sim().tick() > given + FRAMES && kind_of(&t, UNIT) == DEPLOYED);
    assert!(t.pos(pk.0).x < at.0.x - RIG_HALF, "on its side: {}", t.pos(pk.0));
}
