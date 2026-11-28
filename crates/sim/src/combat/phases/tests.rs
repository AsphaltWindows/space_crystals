use std::collections::BTreeMap;

use super::*;
use crate::defs::{AttackType, FullyConnectedSubtype, ObjectKind, TargetDomain, TargetType};
use crate::ids::PlayerId;
use crate::math::PI;
use crate::object::Object;
use crate::vision::Vision;

const ME: ObjectId = ObjectId(1);
const FOE: ObjectId = ObjectId(2);
const MINE: PlayerId = PlayerId(1);

fn attrs(aim: u32, firing: u32, cooldown: u32, reload: u32) -> AttackAttrs {
    AttackAttrs {
        attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
        target_domain: TargetDomain::Ground,
        target_type: TargetType::SingleTarget,
        damage: 10.0,
        range: 4.0,
        min_range: 0.0,
        projectile_speed: None,
        aim_ticks: aim,
        firing_ticks: firing,
        cooldown_ticks: cooldown,
        reload_ticks: reload,
    }
}

/// A unit `id` of `kind`, `owner`'s, at `pos` facing east, with full HP.
fn object(id: ObjectId, kind: ObjectKind, owner: Option<PlayerId>, pos: Vec2) -> Object {
    Object {
        id,
        kind,
        owner,
        pos,
        prev_pos: pos,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        shields: kind.def().max_shields,
        placement: None,
        resource: None,
        orders: None,
        mover: None,
        attack: None,
        riot_shield: None,
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
        tunnel: None,
        construction: None,
        decay: None,
        popup: None,
        lineage: Vec::new(),
        augmentation: Default::default(),
        energy: None,
        kept_research: None,
        buffs: Vec::new(),
        stock: None,
        tether: None,
        drone: None,
        cooldown: None,
        aircraft: None,
        illusion_of: None,
        tanks: None,
    }
}

/// The shooter's object: a Peacekeeper (`ME`) at the origin facing east, `owner`'s.
fn me(owner: Option<PlayerId>) -> Object {
    object(ME, ObjectKind::Peacekeeper, owner, Vec2::ZERO)
}

/// [`me`], of player `MINE`, and a neutral Guard (`FOE`) 100 su away in direction `bearing`, which `ME` sees.
fn world(bearing: f32) -> (Map, Targets, Vision) {
    let map = Map::new(4, 4);
    let mut objects = BTreeMap::new();
    let foe = object(FOE, ObjectKind::Guard, None, math::facing_vector(bearing) * 100.0);
    for o in [me(Some(MINE)), foe] {
        objects.insert(o.id, o);
    }
    let targets = Targets::snapshot(&map, &objects);
    let mut vision = Vision::new(&map, [MINE]);
    vision.update(&map, objects.values(), 0);
    (map, targets, vision)
}

/// Runs `ticks` ticks of a sequence engaging `FOE`; the ticks (from 1) whose hits landed, and the final facing.
fn hit_ticks(attrs: &AttackAttrs, bearing: f32, turn_rate: f32, ticks: u32) -> (Vec<u32>, f32) {
    let (map, targets, vision) = world(bearing);
    let sight = vision.player(MINE);
    let mut shooter = Shooter::new(&map, &me(Some(MINE)), attrs, sight, 0.0);
    shooter.turn_left = turn_rate;
    let mut state = AttackState::default();
    state.engage(Some(FOE));
    let mut facing = 0.0;
    let mut landed = Vec::new();
    for tick in 1..=ticks {
        let mut volley = Volley::new(tick, false);
        run(
            &mut state,
            &mut facing,
            &shooter,
            &targets,
            &mut volley,
            &mut Vec::new(),
        );
        if !volley.hits.is_empty() {
            landed.push(tick);
        }
    }
    (landed, facing)
}

#[test]
fn each_phase_lasts_exactly_its_duration() {
    // Aim 2, Firing 1, Cooldown 2, Reload 12: hits in ticks 3, 20, 37 (a period of 17).
    assert_eq!(hit_ticks(&attrs(2, 1, 2, 12), 0.0, PI, 40).0, [3, 20, 37]);
    // Aim 2, Firing 1, Cooldown 1, Reload 4: a period of 8.
    assert_eq!(hit_ticks(&attrs(2, 1, 1, 4), 0.0, PI, 20).0, [3, 11, 19]);
    // Longer Firing: the hit lands at its end.
    assert_eq!(hit_ticks(&attrs(1, 3, 0, 0), 0.0, PI, 10).0, [4, 8]);
}

#[test]
fn phases_may_last_no_time_at_all() {
    // No aim and no Firing: the hit lands in the tick the sequence starts, whose time goes to Cooldown. The
    // period is still the sum of the durations, 3 ticks.
    assert_eq!(hit_ticks(&attrs(0, 0, 1, 2), 0.0, PI, 9).0, [1, 4, 7]);
    // Every phase empty: still one attack per tick at most.
    assert_eq!(hit_ticks(&attrs(0, 0, 0, 0), 0.0, PI, 3).0, [1, 2, 3]);
}

#[test]
fn aiming_lasts_until_the_unit_faces_its_target() {
    // The target is due south (a quarter turn); at 0.5 rad per tick the turn takes 4 ticks, longer than the aim.
    let (landed, facing) = hit_ticks(&attrs(2, 1, 0, 30), PI / 2.0, 0.5, 10);
    assert_eq!(landed, [5], "aims 4 ticks, fires in the 5th");
    assert_eq!(facing, PI / 2.0);
    // Fast enough: the aim's duration decides.
    let (landed, facing) = hit_ticks(&attrs(2, 1, 0, 30), PI / 2.0, PI, 10);
    assert_eq!(landed, [3]);
    assert_eq!(facing, PI / 2.0);
}

/// (G) One turn budget per tick (`movement::Turns`): a unit of TurnRate 0.5 rad that phase 5d turned in the
/// tick its aim begins has only the rest for the aim, at a target a quarter turn away; in the ticks after, the
/// attack holds its base, 5d turns it no more, and the aim has the whole TurnRate.
#[test]
fn aiming_turns_only_by_what_the_move_left_of_the_turn_rate() {
    let (map, targets, vision) = world(PI / 2.0);
    let attrs = attrs(2, 1, 0, 30);
    // The facings after each of the first ticks, and the tick the hit landed in.
    let aim = |spent_by_the_move: f32| {
        let sight = vision.player(MINE);
        let mut shooter = Shooter::new(&map, &me(Some(MINE)), &attrs, sight, 0.0);
        let mut state = AttackState::default();
        state.engage(Some(FOE));
        let (mut facing, mut facings, mut landed) = (0.0, Vec::new(), None);
        for tick in 1..=8 {
            shooter.turn_left = if tick == 1 { 0.5 - spent_by_the_move } else { 0.5 };
            let mut volley = Volley::new(tick, false);
            run(
                &mut state,
                &mut facing,
                &shooter,
                &targets,
                &mut volley,
                &mut Vec::new(),
            );
            facings.push(facing);
            if !volley.hits.is_empty() {
                landed.get_or_insert(tick);
            }
        }
        (facings[..5].to_vec(), landed)
    };
    assert_eq!(aim(0.0), (vec![0.5, 1.0, 1.5, PI / 2.0, PI / 2.0], Some(5)));
    assert_eq!(aim(0.25), (vec![0.25, 0.75, 1.25, PI / 2.0, PI / 2.0], Some(5)));
    // The move took the whole TurnRate: the aim turns from the next tick on, and the hit lands a tick later.
    assert_eq!(aim(0.5), (vec![0.0, 0.5, 1.0, 1.5, PI / 2.0], Some(6)));
}

/// `systems/combat.md > ValidTarget` 2: no sequence starts at a target the unit's owner doesn't see, and an aim
/// is cancelled once the target is out of sight. The target stays engaged: the order waits for it.
#[test]
fn only_targets_the_owner_sees_are_attacked() {
    let (map, targets, seen) = world(0.0);
    let blind = Vision::new(&map, [MINE]);
    let attrs = attrs(2, 1, 0, 30);
    let sees = Shooter::new(&map, &me(Some(MINE)), &attrs, seen.player(MINE), 0.0);
    let doesnt = Shooter::new(&map, &me(Some(MINE)), &attrs, blind.player(MINE), 0.0);
    let ownerless = Shooter::new(&map, &me(None), &attrs, None, 0.0);
    let mut state = AttackState::default();
    state.engage(Some(FOE));
    let (mut facing, mut volley) = (0.0, Volley::new(0, false));
    for shooter in [&doesnt, &ownerless, &doesnt] {
        run(&mut state, &mut facing, shooter, &targets, &mut volley, &mut Vec::new());
        assert_eq!(state.phase, None, "no sequence starts");
    }
    assert_eq!(state.target, Some(FOE), "still engaged");
    run(&mut state, &mut facing, &sees, &targets, &mut volley, &mut Vec::new());
    assert_eq!(state.phase, Some(AttackPhase::Aiming { target: FOE }));
    run(&mut state, &mut facing, &doesnt, &targets, &mut volley, &mut Vec::new());
    assert_eq!(state.phase, None, "the aim is cancelled");
    assert_eq!(state.target, Some(FOE));
    assert!(volley.hits.is_empty());
}

mod tail_disjointed;
