//! Shields in the game (slice M3-B; `systems/combat.md > Shields`, plan decision T18): the design's cases on
//! Colonists targets, two hits in one tick, the regeneration, the objects of no one, saves and the state hash, and the
//! Shields of a building being warped in (design changes 0110 and 0111). The resolution itself, case by case, is
//! `combat/damage.rs`'s unit tests. `TestSim` checks the movement invariants after every tick.

use sc_sim::testing::{ScenarioBuilder, TestSim};
const SHIELD_REGEN_PER_TICK: f32 = sc_sim::tuning::shields::BUILTIN.shield_regen_per_tick;
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, Sim, SimEvent, SpawnAt, Tick, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const RIFLE: ObjectKind = ObjectKind::RifleLegionnaire;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(sim: &Sim, n: u32) -> &Object {
    sim.object(id(n)).expect("the object exists")
}

/// `builder` with both players playing the Colonists (plan decision T27).
fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Faction::Colonists).faction(P2, Faction::Colonists)
}

/// `sim` saved, with object `n`'s `field` set to `value`, and loaded: a state no command makes on demand.
fn edited(sim: &Sim, n: u32, field: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o[field] = value;
    serde_json::from_value(state).unwrap()
}

/// One tick of `sim` with `player`'s `commands`.
fn step(sim: &mut Sim, player: PlayerId, commands: Vec<Command>) {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player, command })
        .collect();
    sim.step(&TickInputs { commands });
}

/// The hits on object `n` so far: tick, attacker, Shields and HP taken.
fn hits_on(t: &TestSim, n: u32) -> Vec<(Tick, ObjectId, f32, f32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                shield_damage,
            } if *target == id(n) => Some((*tick, *attacker, *shield_damage, *damage)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance, the design's first case in a game (`> Shield Damage Resolution`): a Sabre Legionnaire's
/// strike of 15 on a Legionnaire's 10 Shields (armour 0) takes them all and 5 HP. Regeneration isn't gated by damage
/// (`> ShieldRegenRate`: "continuously (not gated by a delay after taking damage)"): the tick of the strike ends with
/// 0.25 of them back. Every later strike takes what Shields have come back, and its overflow from the HP.
#[test]
fn a_sabre_strike_takes_a_legionnaires_shields_and_five_hp() {
    let mut t = colonists(ScenarioBuilder::new(&flat(16, 6)))
        .spawn(SABRE, Some(P1), SpawnAt::Point(Vec2::new(200.0, 192.0))) // 1
        .spawn(RIFLE, Some(P2), SpawnAt::Point(Vec2::new(300.0, 192.0))) // 2
        .start();
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    while hits_on(&t, 2).is_empty() {
        assert!(t.sim().tick() < 60, "the Sabre strikes");
        t.step();
    }
    let (_, attacker, shields, hp) = hits_on(&t, 2)[0];
    assert_eq!(
        (attacker, shields, hp),
        (id(1), 10.0, 5.0),
        "the design's 15 on 10 Shields"
    );
    let rifle = object(t.sim(), 2);
    assert_eq!((rifle.shields, rifle.hp), (Some(SHIELD_REGEN_PER_TICK), Some(65.0)));
    t.run(40);
    for &(tick, _, shields, hp) in &hits_on(&t, 2)[1..] {
        assert!(shields > 0.0 && shields < 10.0, "tick {tick}: what came back");
        assert_eq!(shields + hp, 15.0, "tick {tick}: the overflow on armour 0");
    }
}

/// The design's fourth case in a game: Shields that take a whole hit leave the HP alone, and armour is never
/// consulted, so the MinimumDamage doesn't apply either. A Guard's 18 (design change 0186) on a Robo Portal's 20
/// Shields (PointArmor 1) takes 18 Shields and no HP.
#[test]
fn shields_that_take_a_whole_hit_leave_the_hp_alone() {
    let mut t = colonists(ScenarioBuilder::new(&flat(16, 10)))
        .spawn(ObjectKind::RoboPortal, Some(P1), SpawnAt::Tile(TilePos::new(2, 3))) // 1
        .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(Vec2::new(500.0, 320.0))) // 2
        .start();
    t.without_auto_targeting()
        .order_at(2, P2, &[id(2)], OrderKind::Attack(id(1)), false);
    while hits_on(&t, 1).is_empty() {
        assert!(t.sim().tick() < 60, "the Guard shoots");
        t.step();
    }
    let (_, _, shields, hp) = hits_on(&t, 1)[0];
    assert_eq!((shields, hp), (18.0, 0.0));
    let portal = object(t.sim(), 1);
    assert_eq!(
        (portal.shields, portal.hp),
        (Some(2.0 + SHIELD_REGEN_PER_TICK), Some(600.0))
    );
}

/// Plan decision T18 in a game: the hits of a tick are resolved as the list is applied, so the second meets the
/// Shields the first left. Player 2's Peacekeepers 2 and 3, placed alike on either side of player 1's Rifle
/// Legionnaire 1, shoot it in the same tick: Peacekeeper 2's 10 takes its 10 Shields, Peacekeeper 3's 10 then takes
/// 10 of its 70 HP.
#[test]
fn the_second_hit_of_a_tick_meets_the_shields_the_first_left() {
    let mut t = colonists(ScenarioBuilder::new(&flat(16, 10)))
        .spawn(RIFLE, Some(P1), SpawnAt::Point(Vec2::new(500.0, 320.0))) // 1
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(350.0, 260.0)),
        ) // 2
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(350.0, 380.0)),
        ) // 3
        .start();
    t.without_auto_targeting()
        .order_at(2, P2, &[id(2), id(3)], OrderKind::Attack(id(1)), false);
    while hits_on(&t, 1).is_empty() {
        assert!(t.sim().tick() < 60, "the Peacekeepers shoot");
        t.step();
    }
    let tick = t.sim().tick();
    assert_eq!(
        hits_on(&t, 1),
        [(tick, id(2), 10.0, 0.0), (tick, id(3), 0.0, 10.0)],
        "both in one tick, in attacker id order"
    );
    let rifle = object(t.sim(), 1);
    assert_eq!((rifle.shields, rifle.hp), (Some(SHIELD_REGEN_PER_TICK), Some(60.0)));
}

/// Design change 0147's Verification: "Hit a shielded Drone from behind for less than its Shields: Shields lose exactly
/// the damage. Hit it for more: only the overflow gets the rear × 1.25 (after armor)". Player 2's Peacekeeper shoots
/// player 1's Drone from behind (× 1.25): its first 10 take the Drone's 10 Shields and nothing else, where 12.5 would
/// have broken through. The second, 17 ticks later (the Peacekeeper's AimDuration 2, FiringDuration 1, CooldownDuration
/// 2 and ReloadDuration 12), meets the 4.25 Shields that came back, and its overflow of 5.75 takes 5.75 × 1.25 HP.
#[test]
fn directional_armour_touches_the_overflow_only() {
    let mut t = colonists(ScenarioBuilder::new(&flat(16, 10)))
        .spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(Vec2::new(500.0, 320.0))) // 1, facing east
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(350.0, 320.0)),
        ) // 2, behind it
        .start();
    t.without_auto_targeting()
        .order_at(2, P2, &[id(2)], OrderKind::Attack(id(1)), false);
    while hits_on(&t, 1).len() < 2 {
        assert!(t.sim().tick() < 80, "the Peacekeeper shoots twice");
        t.step();
    }
    let hits = hits_on(&t, 1);
    assert_eq!(hits[1].0 - hits[0].0, 17);
    assert_eq!((hits[0].2, hits[0].3), (10.0, 0.0));
    assert_eq!((hits[1].2, hits[1].3), (4.25, 5.75 * 1.25));
    assert_eq!(object(t.sim(), 1).hp, Some(20.0 - 7.1875));
}

/// `> ShieldRegenRate`, the plan's acceptance "Regen": 0.25 a tick, so 0 to 10 in 40 ticks, and never above the
/// maximum. A neutral object's Shields regenerate too (design change 0146: "including neutral ones"; a defeated
/// player's objects become no one's, `core/game_rules.md > On Defeat`).
#[test]
fn shields_regenerate_from_nothing_to_full_in_40_ticks() {
    let t = colonists(ScenarioBuilder::new(&flat(12, 6)))
        .spawn(SABRE, Some(P1), SpawnAt::Point(Vec2::new(200.0, 192.0))) // 1
        .spawn(RIFLE, None, SpawnAt::Point(Vec2::new(500.0, 192.0))) // 2, no one's
        .start();
    let mut sim = edited(t.sim(), 1, "shields", serde_json::json!(0.0));
    sim = edited(&sim, 2, "shields", serde_json::json!(0.0));
    for tick in 1..=40 {
        step(&mut sim, P1, Vec::new());
        let expected = Some(tick as f32 * SHIELD_REGEN_PER_TICK);
        assert_eq!((object(&sim, 1).shields, object(&sim, 2).shields), (expected, expected));
    }
    assert_eq!(object(&sim, 1).shields, Some(10.0), "full after 40 ticks");
    step(&mut sim, P1, Vec::new());
    assert_eq!(object(&sim, 1).shields, Some(10.0), "and no more");
}

/// The plan's acceptance "Dead objects don't regenerate": the regeneration runs after phase 5e's dead have left the
/// game, so a unit killed in a tick ends it gone, its Shields with it. Player 2's two Peacekeepers kill player 1's
/// Drone (20 HP, 10 Shields).
#[test]
fn the_dead_are_gone_before_the_shields_regenerate() {
    let mut t = colonists(ScenarioBuilder::new(&flat(16, 10)))
        .spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(Vec2::new(500.0, 320.0))) // 1
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(350.0, 260.0)),
        ) // 2
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(350.0, 380.0)),
        ) // 3
        .start();
    t.without_auto_targeting()
        .order_at(2, P2, &[id(2), id(3)], OrderKind::Attack(id(1)), false);
    while t.sim().object(id(1)).is_some() {
        assert!(t.sim().tick() < 100, "the Peacekeepers kill the Drone");
        t.step();
    }
    let tick = t.sim().tick();
    assert!(
        t.events()
            .iter()
            .any(|(at, e)| *at == tick && matches!(e, SimEvent::Died { id: ObjectId(1), .. }))
    );
    assert!(hits_on(&t, 1).iter().all(|&(at, ..)| at <= tick));
}

/// `Object::shields` is state: hashed (`docs/architecture/replays-and-golden-hashes.md > Replays and golden hashes`) and saved. A state saved before
/// Shields existed, without the field, loads with full Shields; a GDO unit has none.
#[test]
fn shields_are_hashed_saved_and_full_in_an_older_save() {
    let mut t = colonists(ScenarioBuilder::new(&flat(12, 6)))
        .spawn(SABRE, Some(P1), SpawnAt::Point(Vec2::new(200.0, 192.0))) // 1
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(Vec2::new(500.0, 192.0)),
        ) // 2
        .start();
    t.run(3);
    let sim = t.sim();
    assert_eq!((object(sim, 1).shields, object(sim, 2).shields), (Some(10.0), None));
    let damaged = edited(sim, 1, "shields", serde_json::json!(9.75));
    assert_eq!(object(&damaged, 1).shields, Some(9.75));
    assert_ne!(damaged.state_hash(), sim.state_hash(), "the hash covers the Shields");
    let reloaded: Sim = serde_json::from_str(&serde_json::to_string(&damaged).unwrap()).unwrap();
    assert_eq!(reloaded.state_hash(), damaged.state_hash());
    let mut state = serde_json::to_value(&damaged).unwrap();
    for o in state["objects"].as_array_mut().unwrap() {
        o.as_object_mut().unwrap().remove("shields");
    }
    let older: Sim = serde_json::from_value(state).unwrap();
    assert_eq!(
        (object(&older, 1).shields, object(&older, 2).shields),
        (Some(10.0), None)
    );
}

/// A 24 × 16 flat map for two Colonists players: player 1's Beacon 1 at (8, 6), funded, warps in a Robo Portal
/// from tick 1 on: a Robo Portal of 600 HP and 20 Shields in 640 frames, from a Beacon of 120 HP and 10 Shields.
/// Player 2's Beacon 2 keeps them in the game.
fn warping_portal() -> TestSim {
    let mut t = colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(ObjectKind::Beacon, Some(P1), SpawnAt::Tile(TilePos::new(8, 6)))
        .spawn(ObjectKind::Beacon, Some(P2), SpawnAt::Tile(TilePos::new(16, 12)))
        .start();
    let funds = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    let warp_in = ConstructionCommand::WarpIn {
        kind: ObjectKind::RoboPortal,
        orientation: BuildingOrientation::default(),
    };
    t.debug_at(1, funds).at(
        1,
        P1,
        Command::Construction {
            objects: vec![id(1)],
            command: warp_in,
        },
    );
    t
}

/// The brief's acceptance "During a transformation" (design change 0110: "Shields regenerate normally up to the current
/// maximum"; "current += (newMax − oldMax) each frame"; design change 0146): undamaged, the portal's Shields are its
/// interpolated maximum all the way, 15 at 50%, 20 at the end. Emptied at 50%, each tick they regenerate 0.25, up to
/// the maximum then, and follow its rise of 1/64 a frame, so they gain 17/64 a tick until they meet it, 60 ticks on,
/// and from then on stay exactly at it, never above. Every value is a whole number of 64ths, so the arithmetic is
/// exact.
#[test]
fn shields_follow_their_maximum_through_a_warp_in_and_regenerate_up_to_it() {
    let mut t = warping_portal();
    for tick in [1, 160, 320] {
        t.run_until(tick);
        let o = object(t.sim(), 1);
        let max = 10.0 + 10.0 * tick as f32 / 640.0;
        assert_eq!((o.max_shields(), o.shields), (Some(max), Some(max)), "tick {tick}");
    }
    assert_eq!(object(t.sim(), 1).shields, Some(15.0), "the plan's 50%");
    let mut sim = edited(t.sim(), 1, "shields", serde_json::json!(0.0));
    for k in 1..=320 {
        step(&mut sim, P1, Vec::new());
        let o = object(&sim, 1);
        let max = 15.0 + k as f32 / 64.0;
        let expected = (k as f32 * 17.0 / 64.0).min(max);
        assert_eq!(
            (o.max_shields(), o.shields),
            (Some(max), Some(expected)),
            "{k} ticks after"
        );
    }
    let o = object(&sim, 1);
    assert!(!o.is_transforming());
    assert_eq!(o.shields, Some(20.0));
}

/// The brief's acceptance "Cancel", with design change 0111's Verification and Shields added ("Cancel a Warp In at
/// 50% HP → Beacon 60/120"; "Shields by the same fraction (no minimum needed)"): the Robo Portal half warped in has
/// 360 HP and 15 Shields at most. With 180 HP and 7.5 Shields it goes back to a Beacon of 60 HP and 5 Shields; with
/// 0.75 Shields, to 0.5, below the 1 HP keeps; with none, to none. The tick of the cancel then regenerates 0.25.
#[test]
fn a_cancel_keeps_the_fraction_of_the_shields() {
    let mut t = warping_portal();
    t.run_until(320);
    let o = object(t.sim(), 1);
    assert_eq!((o.max_hp(), o.max_shields()), (Some(360.0), Some(15.0)));
    let cancel = Command::Construction {
        objects: vec![id(1)],
        command: ConstructionCommand::CancelTransform,
    };
    for (shields, after) in [(7.5, 5.0), (0.75, 0.5), (0.0, 0.0)] {
        let sim = edited(t.sim(), 1, "hp", serde_json::json!(180.0));
        let mut sim = edited(&sim, 1, "shields", serde_json::json!(shields));
        step(&mut sim, P1, vec![cancel.clone()]);
        let o = object(&sim, 1);
        assert_eq!(
            (o.kind, o.hp, o.max_shields(), o.shields),
            (
                ObjectKind::Beacon,
                Some(60.0),
                Some(10.0),
                Some(after + SHIELD_REGEN_PER_TICK)
            ),
            "{shields} of 15"
        );
    }
}
