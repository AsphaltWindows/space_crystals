//! What a unit whose turn is done leaves with (`factions/syndicate/structures/null_chamber.md > Null Chamber Exit
//! Buff`, `factions/syndicate/structures/null_chamber.md > Placeholder stats`; `network::services`): "**+2 grid units
//! SightRange**" in its eye and "**20%** damage-taken reduction" before armour while the buff lasts, its abrupt end
//! 20 s after the unit came out, its restart, by Side A or (M6-SD) a Tier 3 Tunnel's air exit door, and none without a
//! chamber of its owner's.

use sc_sim::combat::pre_armor_multiplier;
use sc_sim::vision::TILE_SIGHT_POINT;
use sc_sim::{VisibilityState, Vision};

use super::*;

const PK: ObjectKind = ObjectKind::Peacekeeper;
const PHANTOM: ObjectKind = ObjectKind::Phantom;

/// The Guard's SightRange, grid units (`factions/syndicate/units/guard.md`: "SightRange - 5"). The plan's acceptance
/// reads "its sight is 6 + 2": the Guard's is 5, so its buffed eye sees 7.
const GUARD_SIGHT: f32 = 5.0;
/// "**+2 grid units SightRange**".
const BONUS: f32 = 2.0;
/// The Phantom's (`factions/syndicate/units/phantom.md`: "SightRange 8 grid units").
const PHANTOM_SIGHT: f32 = 8.0;
/// A Peacekeeper's hit: "Damage: 10" (`factions/gdo/units/peacekeeper.md`), less a Guard's PointArmor 1; 10 × 0.8 − 1
/// on a buffed Guard ("damage taken = Attack Damage × (1 − reduction) − armor").
const PLAIN_HIT: f32 = 9.0;
const BUFFED_HIT: f32 = 7.0;

/// Asserts that unit `id`'s eye sees as far as `radius` grid units and no farther: in a vision of its own
/// (`Vision::update` on a copy, with no other object; flat ground, so no ElevationModifier), each tile whose sight
/// point ([`TILE_SIGHT_POINT`]) lies within `radius` of it is Visible, and each other not, but for those on the edge
/// (within a hundredth of a grid unit of it), which may go either way.
fn sees_as_far_as(t: &TestSim, id: ObjectId, radius: f32) {
    let sim = t.sim();
    let o = object(t, id);
    let mut vision = Vision::new(sim.map(), [P1]);
    vision.update(sim.map(), [o], sim.tick());
    let at = o.map_pos(sim.map());
    let mut seen = 0;
    for y in 0..sim.map().height() as i32 {
        for x in 0..sim.map().width() as i32 {
            let tile = TilePos::new(x, y);
            let point = Vec2::new(x as f32, y as f32) * SU_PER_GU + TILE_SIGHT_POINT;
            let far = point.distance(at) / SU_PER_GU;
            if (far - radius).abs() < 0.01 {
                continue;
            }
            let visible = vision.state(P1, tile) == VisibilityState::Visible;
            assert_eq!(
                visible,
                far < radius,
                "tick {}: {tile:?}, {far} GU from {id:?}",
                sim.tick()
            );
            seen += usize::from(visible);
        }
    }
    assert!(seen > 0, "{id:?} sees something");
}

/// [`with_chamber`] started, with a Guard sent in, done with its turn and ejected in the first tick it may leave: the
/// game, at the end of that tick, the Guard and the tick.
fn buffed_guard() -> (TestSim, ObjectId, Tick) {
    let mut t = started(with_chamber(), 1);
    let guard = guards_in(&mut t, 1)[0];
    let out = entered(&t, guard) + TURN;
    t.at(out, P1, eject(T1, GUARD));
    run_checked(&mut t, out);
    assert_eq!(left(&t, guard), out, "out in the first tick it may leave");
    (t, guard, out)
}

/// Player 2's Peacekeeper, spawned in the next tick at `at`, on Hold: its id.
fn peacekeeper(t: &mut TestSim, at: Vec2) -> ObjectId {
    let pk = id(next_id(t));
    let now = t.sim().tick();
    t.debug_at(now + 1, spawn(PK, P2, SpawnAt::Point(at)));
    t.at(now + 2, P2, order(&[pk], OrderKind::Hold));
    run_checked(t, now + 2);
    pk
}

/// The hits on `target` from the tick after `since` on: their damage.
fn hits_since(t: &TestSim, target: ObjectId, since: Tick) -> Vec<f32> {
    (hits_on(t, target).into_iter())
        .filter(|&(tick, ..)| tick > since)
        .map(|(.., damage)| damage)
        .collect()
}

/// The plan's acceptance: "A Guard that spent 2 s in the network with a chamber leaves buffed: its sight is 6 + 2, and
/// a hit of 10 deals 10 × 0.8 − armour." Out in the first tick it may leave, it holds the buff for 20 s; its eye sees
/// 7 grid units (the Guard's SightRange is 5, not 6) from that tick, where a Guard that never went in sees 5; and a
/// Peacekeeper's hit of 10 takes 7 HP of it, 9 of the other.
#[test]
fn a_guard_2_s_inside_with_a_chamber_leaves_seeing_farther_and_taking_less_before_armour() {
    let (mut t, guard, out) = buffed_guard();
    assert_eq!(buff_until(&t, guard), Some(out + DURATION));
    assert_eq!(object(&t, guard).def().sight_range, GUARD_SIGHT);
    sees_as_far_as(&t, guard, GUARD_SIGHT + BONUS);
    let plain = id(next_id(&t));
    t.spawn_unit_at(out + 1, GUARD, P1, Vec2::new(1200.0, 500.0));
    run_checked(&mut t, out + 1);
    let pk = peacekeeper(&mut t, Vec2::new(1060.0, 400.0));
    sees_as_far_as(&t, plain, GUARD_SIGHT);
    let map = t.sim().map();
    assert_eq!(pre_armor_multiplier(map, object(&t, guard)), 0.8);
    assert_eq!(pre_armor_multiplier(map, object(&t, plain)), 1.0);
    let now = t.sim().tick();
    t.at(now + 1, P2, order(&[pk], OrderKind::Attack(guard)));
    step_until(&mut t, 80, "the Peacekeeper hits the buffed Guard", |t| {
        !hits_on(t, guard).is_empty()
    });
    assert_eq!(hits_on(&t, guard)[0].1, pk);
    assert_eq!(hits_since(&t, guard, 0), [BUFFED_HIT], "10 × 0.8 − 1");
    let now = t.sim().tick();
    t.at(now + 1, P2, order(&[pk], OrderKind::Attack(plain)));
    step_until(&mut t, 80, "the Peacekeeper hits the other", |t| {
        !hits_on(t, plain).is_empty()
    });
    assert_eq!(hits_since(&t, plain, 0), [PLAIN_HIT], "10 − 1");
    assert!(t.sim().tick() < out + DURATION, "all while the buff lasts");
}

/// The plan's acceptance: "The buff ends at once, 20 s after the unit leaves" ("ends abruptly"). Held until 20 s from
/// the tick its unit came out in, it works in the 319th tick after's hits, which come before the buffs' step, and is
/// dropped in that step, before the tick's vision (`crate::buffs`): at the end of the 318th the Guard's eye sees 7 grid
/// units and its pre-armour multiplier is 0.8, at the end of the 319th 5 and 1. A Peacekeeper firing on it across the
/// end takes 7 HP a hit until then, and 9 after.
#[test]
fn the_buff_ends_at_once_20_s_after_the_unit_came_out() {
    let (mut t, guard, out) = buffed_guard();
    let pk = peacekeeper(&mut t, Vec2::new(1060.0, 400.0));
    let fire = out + DURATION - 60;
    t.at(fire, P2, order(&[pk], OrderKind::Attack(guard)));
    let last = out + DURATION - 1;
    run_checked(&mut t, last - 1);
    assert_eq!(buff_until(&t, guard), Some(out + DURATION));
    sees_as_far_as(&t, guard, GUARD_SIGHT + BONUS);
    assert_eq!(pre_armor_multiplier(t.sim().map(), object(&t, guard)), 0.8);
    step_checked(&mut t);
    assert_eq!(t.sim().tick(), last);
    assert_eq!(object(&t, guard).buffs, []);
    sees_as_far_as(&t, guard, GUARD_SIGHT);
    assert_eq!(pre_armor_multiplier(t.sim().map(), object(&t, guard)), 1.0);
    run_checked(&mut t, out + DURATION + 60);
    let hits = hits_on(&t, guard);
    let (during, after): (Vec<_>, Vec<_>) = hits.iter().partition(|&&(tick, ..)| tick <= last);
    assert!(during.len() >= 2 && after.len() >= 2, "{hits:?}");
    assert!(during.iter().all(|&&(.., damage)| damage == BUFFED_HIT), "{hits:?}");
    assert!(after.iter().all(|&&(.., damage)| damage == PLAIN_HIT), "{hits:?}");
}

/// "**Refresh**: exiting the Tunnel Network again instantly resets the buff to full duration": the buffed Guard goes
/// back in, keeping its buff inside, takes its turn again from the tick it goes in and comes out with the buff
/// restarted, 20 s from then: one buff, not two. Then in again with no chamber left: no new turn, and it comes out
/// with the buff it had, which ends at its own end.
#[test]
fn leaving_again_restarts_the_buff_and_leaving_without_a_new_turn_keeps_the_old_one() {
    let (mut t, guard, out) = buffed_guard();
    t.at(out + 40, P1, order(&[guard], OrderKind::Enter(T1)));
    step_until(&mut t, 100, "back in", |t| t.sim().held_unit(guard).is_some());
    let came_in = entered(&t, guard);
    assert_eq!(turn(&t, guard), Some(1), "a new turn");
    assert_eq!(buff_until(&t, guard), Some(out + DURATION), "kept inside");
    let again = came_in + TURN;
    t.at(again, P1, eject(T1, GUARD));
    run_checked(&mut t, again);
    assert_eq!(left(&t, guard), again);
    assert_eq!(object(&t, guard).buffs.len(), 1);
    assert_eq!(buff_until(&t, guard), Some(again + DURATION), "restarted");
    // In again with the chamber gone, and out with no new turn.
    t.debug_at(again + 1, DebugCommand::Remove { id: id(2) });
    t.at(again + 10, P1, order(&[guard], OrderKind::Enter(T1)));
    step_until(&mut t, 100, "in once more", |t| t.sim().held_unit(guard).is_some());
    let last_in = entered(&t, guard);
    t.at(last_in + TURN, P1, eject(T1, GUARD));
    run_checked(&mut t, last_in + TURN);
    assert_eq!(left(&t, guard), last_in + TURN);
    assert_eq!(buff_until(&t, guard), Some(again + DURATION), "the old one");
    run_checked(&mut t, again + DURATION - 2);
    assert_eq!(buff_until(&t, guard), Some(again + DURATION));
    step_checked(&mut t);
    assert_eq!(buff_until(&t, guard), None, "at its own end");
}

/// "While the player owns at least one Null Chamber anywhere in the Tunnel Network": without one of its owner's, a
/// Guard 3 s inside takes no turn and leaves unbuffed, seeing 5 grid units, a hit taking it as ever; player 2's
/// chamber (the Syndicate too, with its own Tunnel) serves player 2's network only.
#[test]
fn without_a_chamber_of_its_owner_s_a_guard_leaves_unbuffed() {
    let scenario = ScenarioBuilder::new(&flat(64, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(12, 10))
        .spawn(TUNNEL, Some(P2), tile(44, 10))
        .spawn(CHAMBER, Some(P2), tile(42, 8));
    let mut t = started(scenario, 1);
    let guard = guards_in(&mut t, 1)[0];
    let out = entered(&t, guard) + 3 * 16;
    t.at(out, P1, eject(T1, GUARD));
    while t.sim().tick() < out - 1 {
        step_checked(&mut t);
        assert_eq!(turn(&t, guard), None, "tick {}: no turn", t.sim().tick());
    }
    step_checked(&mut t);
    assert_eq!(left(&t, guard), out);
    assert_eq!(buff_until(&t, guard), None);
    sees_as_far_as(&t, guard, GUARD_SIGHT);
    assert_eq!(pre_armor_multiplier(t.sim().map(), object(&t, guard)), 1.0);
}

/// (M6-SD) A Phantom, manned ("infantry, vehicles, and crewed aircraft"), comes in by a Tier 3 Tunnel's air entry
/// door, takes its turn as a Guard does, and leaves buffed by the air exit door ("every **manned** Syndicate unit that
/// exits the Tunnel Network through any Tunnel"): over the door, rising, with its buff for 20 s from the tick it comes
/// out, its eye seeing 8 + 2 grid units.
#[test]
fn a_phantom_out_of_the_air_exit_door_leaves_buffed() {
    let mut t = started(with_chamber(), 3);
    let phantom = id(next_id(&t));
    t.spawn_unit_at(2, PHANTOM, P1, Vec2::new(896.0, 300.0));
    t.at(3, P1, order(&[phantom], OrderKind::Enter(T1)));
    step_until(&mut t, 200, "the Phantom descends into the air entry door", |t| {
        t.sim().held_unit(phantom).is_some()
    });
    let came_in = entered(&t, phantom);
    assert_eq!(turn(&t, phantom), Some(1));
    let out = came_in + TURN;
    t.at(out, P1, eject(T1, PHANTOM));
    run_checked(&mut t, out);
    assert_eq!(left(&t, phantom), out);
    let o = object(&t, phantom);
    assert_eq!(o.pos, paths(&t, T1).air_exit, "over the air exit door");
    assert!(o.mover.as_ref().is_some_and(|mover| mover.rise.is_some()), "rising");
    assert_eq!(buff_until(&t, phantom), Some(out + DURATION));
    assert_eq!(o.def().sight_range, PHANTOM_SIGHT);
    sees_as_far_as(&t, phantom, PHANTOM_SIGHT + BONUS);
}
