//! Where a strafing pass ends (`super`; slice M5-N7, design change 0408, answering Q0149): "a strafing pass used to end
//! when the target's far edge left weapon range (edge-to-edge range), so the loop grew with the target's size and a
//! lone Sparrow lost Agents, vehicles and buildings beyond its 8 GU sight. Now the pass ends when the **target's
//! centre** is out of weapon range, measured from the glider's edge to the target's centre. The loop is the same size
//! whatever the target, and a lone Sparrow's sight 8 covers it" (`systems/controls/command_behaviors.md >
//! AttackingObject`, GliderMovement exception, step 3; `factions/cults/units/sparrow.md > Flight rules`: "Strafing").

use super::*;

/// The Sparrow's SightRange, 8 grid units (design change 0403).
const SIGHT: f32 = 8.0 * 64.0;

/// A lone Sparrow of player 1's (1), with no spotter, attacks player 2's `kind` (2), whose player plays `faction`, some
/// 412 su east of it and still (auto-targeting off): a unit at the centre of the tile (20, 20), a building on the tiles
/// round it, its footprint's centre the tile's or the corner beside it. The Attack is its order every tick until the
/// target dies, and it ends with the target (`current`): the Sparrow never loses it. Returns, for each pass after the
/// first, the farthest the Sparrow got from the target's centre between the pass before and that one, space units, with
/// the tick the target died.
fn passes_until_it_dies(kind: ObjectKind, faction: Faction) -> (Vec<f32>, Tick) {
    let at = match kind.def().tile_size() {
        Some((w, h)) => SpawnAt::Tile(sc_sim::TilePos::new(20 - (w as i32) / 2, 20 - (h as i32) / 2)),
        None => point(1312.0, 1312.0),
    };
    let mut t = quiet(
        flat(40, 40)
            .faction(P2, faction)
            .spawn(SPARROW, Some(P1), point(900.0, 1312.0))
            .spawn(kind, Some(P2), at)
            .start(),
    );
    let target = t.pos(2);
    order(&mut t, 1, P1, &[1], OrderKind::Attack(ObjectId(2)));
    // Each pass, its shots no more than 2 s apart: its last shot, and the farthest the Sparrow got from the target
    // between the pass before and this one.
    let mut passes: Vec<(Tick, f32)> = Vec::new();
    let (mut seen, mut farthest) = (0, 0.0_f32);
    loop {
        t.step();
        let tick = t.sim().tick();
        farthest = farthest.max(t.pos(1).distance(target));
        let shots = hits(&t);
        for &(at, attacker, target, _) in &shots[seen..] {
            assert_eq!((at, attacker, target), (tick, 1, 2), "{kind:?}");
            match passes.last_mut() {
                Some(pass) if tick - pass.0 <= 32 => pass.0 = tick,
                _ => passes.push((tick, std::mem::take(&mut farthest))),
            }
        }
        seen = shots.len();
        if t.sim().object(ObjectId(2)).is_none() {
            assert_eq!(current(&t, 1), None, "{kind:?}: its target died");
            let loops = passes[1..].iter().map(|&(_, farthest)| farthest).collect();
            return (loops, tick);
        }
        assert!(tick < 16 * 600, "{kind:?} lives on: {passes:?}");
        assert_eq!(
            current(&t, 1),
            Some(OrderKind::Attack(ObjectId(2))),
            "{kind:?}, tick {tick}: {passes:?}"
        );
    }
}

/// 0408's Verification: "A lone Sparrow with no spotter given a direct Attack on a Seeker, a Guard and a building: it
/// keeps making passes on each until the target dies, never losing it", and "The farthest point of its loop is about
/// the same (~500 su from the target's centre) for a Peacekeeper and a building". Each in a game of its own: a Seeker
/// (56 × 40, MaxHP 120), a Guard (36 × 36, MaxHP 80), a Barracks (3 × 2 tiles, 192 × 128 su, MaxHP 300) and a
/// Peacekeeper (24 × 24, MaxHP 50), all with PointArmor 1, 2 HP a hit: 10, 7, 20 and 5 passes. Every loop takes the
/// Sparrow 488 to 496 su from its target's centre, whatever the target (between 470 and 510 here), within its sight;
/// the Barracks' farthest within 10 su of the Peacekeeper's. (Before 0408, edge to edge, the loops grew with the
/// target: 497 to 502 su for the Peacekeeper, and the Sparrow lost the Seeker on its third loop, 512 su off.)
#[test]
fn a_lone_sparrow_s_loop_is_the_same_size_whatever_it_attacks_until_the_target_dies() {
    let mut farthest = Vec::new();
    for (kind, faction) in [
        (ObjectKind::Seeker, Faction::Cults),
        (ObjectKind::Guard, Faction::Syndicate),
        (ObjectKind::Barracks, Faction::Gdo),
        (ObjectKind::Peacekeeper, Faction::Gdo),
    ] {
        let (loops, died) = passes_until_it_dies(kind, faction);
        assert!(loops.len() >= 3, "{kind:?}: {loops:?}");
        for &far in &loops {
            assert!(
                (470.0..510.0).contains(&far),
                "{kind:?}: a loop {far} su off: {loops:?}"
            );
            assert!(far < SIGHT, "{kind:?}");
        }
        let most = loops.iter().copied().fold(0.0, f32::max);
        println!(
            "{kind:?}: {} passes until tick {died}, loops {loops:?}",
            loops.len() + 1
        );
        farthest.push((kind, most));
    }
    let of = |kind| farthest.iter().find(|&&(k, _)| k == kind).map(|&(_, far)| far).unwrap();
    let (building, peacekeeper) = (of(ObjectKind::Barracks), of(ObjectKind::Peacekeeper));
    assert!(near(building, peacekeeper, 10.0), "{building} and {peacekeeper}");
}
