//! Attackers blocked by their own side (`systems/controls/behaviors.md > AttackingObject > Blocked by its own side`,
//! design change 0223; `docs/movement.md > 6.5`, rule 2). One test binary:
//! - here, **the way round** (slice M3-G2, `movement::round`): a unit behind its owner's units that stand firing
//!   gets into range, by `systems/units.md > UnitCollision > Ground Collision`, "Moving units must pathfind around
//!   occupied space", for a unit that approaches the target of its attack (`systems/controls/behaviors.md >
//!   AttackingObject`, `systems/controls/behaviors.md > AttackMovingToLocation`);
//! - `no_room`, **no way round** (slice M3-G3, design change 0223, answering question Q0057): the Failure rule, with
//!   no indefinite wait behind the blockers, and 0223's Verification.
//!
//! The bug (`docs/playtest/m3.md`): of four Peacekeepers that attack-moved in a column onto a building, the last
//! could stay behind the other three and never fire. The three stopped side by side, each in the tick its attack
//! reached the building, so a fraction of a step out of line; the fourth, queueing behind them, stepped to and fro
//! in the pocket between two of them for as long as the building stood.

mod no_room;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5
const DC: ObjectKind = ObjectKind::DeploymentCenter; // 4 × 4 tiles, 1000 HP: it outlasts the approach

/// The attackers are objects 3 and up: 1 is the target, 2 a far structure that keeps its owner in the game.
const FIRST: u32 = 3;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn gu(x: f32, y: f32) -> Vec2 {
    Vec2::new(x * 64.0, y * 64.0)
}

/// The first tick in which each of the `n` attackers' attacks took effect, `None` for one that never fired.
fn first_shots(t: &TestSim, n: u32) -> Vec<Option<Tick>> {
    (FIRST..FIRST + n)
        .map(|unit| {
            t.events()
                .iter()
                .find(|(_, e)| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == id(unit)))
                .map(|&(tick, _)| tick)
        })
        .collect()
}

fn move_failures(t: &TestSim) -> usize {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }))
        .count()
}

/// How the attackers stand when they are ordered.
#[derive(Copy, Clone, Debug)]
enum Formation {
    /// One behind the other, the first nearest the target, `spacing` su from centre to centre.
    Column,
    /// Side by side, facing the target.
    Line,
}

/// 40 × 12 open tiles with player 1's Deployment Center on tiles (15, 3) to (18, 6), x 960 to 1216 and y 192 to
/// 448, and `n` units of `kind` of player 2 east of it: a column along the row `y` from 26 GU on, or a line 27 GU
/// east. An AttackMove sends them to the ground just east of the building. For an explicit Attack they start where
/// they see it (21.5 GU; the order is on what its owner sees), which is in range for the first of them.
///
/// Returns the game after `ticks`, or as the building fell.
fn assault(kind: ObjectKind, n: u32, formation: Formation, spacing: f32, y: f32, attack: bool, ticks: Tick) -> TestSim {
    let row = ".".repeat(40);
    let map = vec![row; 12].join("\n");
    let mut builder = ScenarioBuilder::new(&map)
        .spawn(DC, Some(P1), SpawnAt::Tile(TilePos::new(15, 3)))
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(1, 9)));
    let x = if attack { 21.5 * 64.0 } else { 26.0 * 64.0 };
    for i in 0..n {
        let at = match formation {
            Formation::Column => Vec2::new(x + spacing * i as f32, y),
            Formation::Line => Vec2::new(x + 64.0, 40.0 + spacing * i as f32),
        };
        builder = builder.spawn(kind, Some(P2), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    let units: Vec<ObjectId> = (FIRST..FIRST + n).map(id).collect();
    let order = if attack {
        OrderKind::Attack(id(1))
    } else {
        OrderKind::AttackMove(gu(19.6, 4.5))
    };
    t.order_at(2, P2, &units, order, false);
    while t.sim().tick() < ticks && t.sim().object(id(1)).is_some() {
        t.step();
    }
    t
}

/// Every attacker of [`assault`] fired within `bound` ticks, and no move failed.
fn assert_all_fire(kind: ObjectKind, n: u32, formation: Formation, attack: bool, bound: Tick) {
    let spacings: &[f32] = if kind == PK {
        &[24.0, 36.0, 40.0, 64.0]
    } else {
        &[36.0, 40.0, 64.0]
    };
    for &spacing in spacings {
        for y in [288.0, 250.0, 200.0, 100.0] {
            let t = assault(kind, n, formation, spacing, y, attack, bound);
            let shots = first_shots(&t, n);
            assert!(
                shots.iter().all(Option::is_some),
                "{n} {kind:?} in a {formation:?}, spacing {spacing}, y {y}, attack {attack}: first shots {shots:?} by \
                 tick {}\n{}",
                t.sim().tick(),
                t.ascii(None)
            );
            assert_eq!(move_failures(&t), 0, "{kind:?} {formation:?} {spacing} {y} {attack}");
        }
    }
}

/// The bug: four Peacekeepers attack-move in a column onto a building, and each of them fires, in 16 layouts (how
/// far apart they walk, and which row they come along: straight at the building's east face, or past its corner).
/// Before the fix one of them never fired in 5 of the 16: the last in 3, the third in 2.
///
/// **The bound**, 200 ticks from the order: the last of them walks 450 su to where the first stopped (113 ticks at
/// 4 su), stands behind the row for `JAMMED_TICKS` (12), goes round at most half of it (three Peacekeepers, 40 su
/// each way with the leg out and the leg across: 20 ticks) and aims (4). The rest allows for a second attempt
/// when another unit took the place first. Measured: 135 at most.
#[test]
fn four_peacekeepers_in_a_column_all_fire_on_the_building() {
    assert_all_fire(PK, 4, Formation::Column, false, 200);
}

/// The same under an explicit Attack: the first three are in range where they stand, and the fourth, behind them,
/// goes round. Measured: 52 ticks at most.
#[test]
fn four_peacekeepers_in_a_column_all_fire_under_an_attack_order() {
    assert_all_fire(PK, 4, Formation::Column, true, 120);
}

/// Eight in a column: the row they form is longer than the building's side is in reach of, and the late ones lose
/// the place they went for to a unit that got there first. Such an attempt doesn't count against the leg's three
/// (`stuck::after_move`, rule 3), so each of them gets a place in the end, before the building falls (it stands
/// about 300 ticks under eight Peacekeepers). Measured: 255 ticks at most for an AttackMove, 206 for an Attack.
#[test]
fn eight_peacekeepers_in_a_column_all_fire_on_the_building() {
    assert_all_fire(PK, 8, Formation::Column, false, 320);
    assert_all_fire(PK, 8, Formation::Column, true, 320);
}

/// Guards are half as wide again and reach a grid unit less far, so their row is longer and nearer. Four in a
/// column, and eight under an Attack. (Of eight Guards on an AttackMove the last one is still on its way round the
/// row's far end when the building falls, in 1 of 12 layouts: there is no room nearer.) Measured: 143 and 188
/// ticks at most.
#[test]
fn guards_in_a_column_all_fire_on_the_building() {
    assert_all_fire(GUARD, 4, Formation::Column, false, 220);
    assert_all_fire(GUARD, 4, Formation::Column, true, 220);
    assert_all_fire(GUARD, 8, Formation::Column, true, 320);
}

/// A line of units has nobody in its way, and fires as it comes into range: the detour is no part of it.
#[test]
fn a_line_of_units_fires_as_it_arrives() {
    for (kind, bound) in [(PK, 100), (GUARD, 180)] {
        assert_all_fire(kind, 4, Formation::Line, false, bound);
        assert_all_fire(kind, 8, Formation::Line, false, bound + 80);
        assert_all_fire(kind, 8, Formation::Line, true, 60);
    }
}

/// A unit as the target (a Guard that doesn't fight back, 85 HP): four Peacekeepers in a column kill it within a
/// few volleys, and until then none of them fails its move or loses its order.
#[test]
fn a_column_on_a_unit_keeps_its_orders_until_the_target_falls() {
    for attack in [false, true] {
        let row = ".".repeat(40);
        let map = vec![row; 12].join("\n");
        let mut builder = ScenarioBuilder::new(&map).spawn(GUARD, Some(P1), SpawnAt::Point(gu(17.5, 4.5)));
        let x = if attack { 22.0 } else { 26.0 };
        for i in 0..4 {
            builder = builder.spawn(
                PK,
                Some(P2),
                SpawnAt::Point(gu(x, 4.5) + Vec2::new(36.0 * i as f32, 0.0)),
            );
        }
        let mut t = builder.start();
        let units: Vec<ObjectId> = (2..6).map(id).collect();
        let order = if attack {
            OrderKind::Attack(id(1))
        } else {
            OrderKind::AttackMove(gu(17.5, 4.5))
        };
        // The Guard only stands there: nobody acquires anything by itself until the attackers are ordered.
        t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
        t.order_at(2, P2, &units, order, false);
        t.debug_at(3, DebugCommand::SetAutoTargeting { on: !attack });
        while t.sim().object(id(1)).is_some() {
            assert!(t.sim().tick() < 400, "the Guard still stands\n{}", t.ascii(None));
            t.step();
            if t.sim().tick() > 2 && t.sim().object(id(1)).is_some() {
                for unit in 2..6 {
                    let orders = t.sim().object(id(unit)).unwrap().orders.as_ref().unwrap();
                    assert!(!orders.is_idle(), "#{unit} gave up in tick {}", t.sim().tick());
                }
            }
        }
        assert_eq!(move_failures(&t), 0);
    }
}
