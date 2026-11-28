//! `factions/gdo/economy.md > SupplyTower > Landing and Attachment`, and the chopper's landing as an aircraft's
//! (`factions/gdo/units/supply_chopper.md > Unit Transport`, `factions/gdo/structures/airfield.md > Returning and
//! rearming`): eviction, a command breaking the attachment, the domains while landing and down, and the free repair on
//! its Tower, at the speed of Power (`factions/gdo/overview.md > Power`).

use sc_sim::{ProductionCommand, Sim, Target, TickInputs};

use super::*;

/// The station of these tests: on tiles (14, 8) to (15, 9), holding 3 Supplies.
const S: u32 = FIRST;

/// [`base`] with [`S`], delivering every 960 ticks, and `more`; a Supply Tower of player 1's placed on tiles (2, 8) to
/// (4, 10), with its free chopper.
fn field(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> (TestSim, u32, u32) {
    let mut t = base(|b| more(b.spawn_with(station(3, 960), tile(14, 8))));
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    (t, tower, chopper)
}

/// A chopper trained at `tower`, out over its centre: its id.
fn trained(t: &mut TestSim, tower: u32) -> u32 {
    let train = Command::Production {
        objects: vec![id(tower)],
        command: ProductionCommand::Train(CHOPPER),
    };
    next(t, train);
    let before = spawned(t).len();
    step_until(t, 2 * CHOPPER_TICKS, "trained", |t| spawned(t).len() > before);
    spawned(t)[before].1
}

fn move_to(units: &[u32], to: Vec2) -> Command {
    order(units, OrderKind::Move(Target::Point(to)))
}

/// "When a Supply Chopper attaches to a Supply Tower, it immediately evicts any existing landed Supply Chopper that is
/// not the attached chopper": the free chopper sent off (its attachment broken), a second chopper picks up at the
/// station and lands on the Tower to drop off; as it lands, the first is given an Attach to Tower there. It is attached
/// in that tick, and the landing one takes off at once, its load still on board, its Drop Off over; the attached one
/// lands. "When a Supply Tower has an attached Supply Chopper, no other Supply Chopper may land on it": the gate
/// refuses the second a Drop Off there now (`Reject::TowerTaken`).
#[test]
fn an_attaching_chopper_evicts_the_one_landing_on_its_tower() {
    let (mut t, tower, first) = field(|b| b);
    let second = trained(&mut t, tower);
    let away = spot(&t, tower) + Vec2::new(0.0, 400.0);
    next(&mut t, move_to(&[first], away));
    assert_eq!(attached(&t, first), None);
    next(&mut t, pick_up(&[second], S));
    step_until(&mut t, 600, "loaded", |t| carried(t, second) == 3);
    next(&mut t, drop_off(&[second], tower));
    step_until(&mut t, 600, "landing on the Tower", |t| landing_of(t, second).is_some());
    t.run(LANDING / 2);
    assert!(landing_of(&t, second).is_some_and(|landing| landing.at == id(tower) && !landing.descent.done()));
    next(&mut t, attach(&[first], tower));
    assert_eq!(attached(&t, first), Some(tower), "attached at once");
    assert_eq!(landing_of(&t, second), None, "evicted");
    assert!(t.mover(second).rise.is_some(), "it takes off");
    t.step();
    assert_eq!(
        (doing(&t, second), carried(&t, second)),
        (None, 3),
        "its Drop Off over, its load on board"
    );
    assert_eq!(banks(&t, P1).1, 0);
    assert_eq!(
        gate(&t, &drop_off(&[second], tower)),
        Err(Reject::TowerTaken(id(tower)))
    );
    step_until(&mut t, 600, "the attached one down", |t| down_on(t, first, tower));
    assert_eq!(landing_of(&t, second), None, "the other stays off");
}

/// "Any command given to an attached Supply Chopper breaks its attachment to the tower": a Stop to the free chopper
/// down on its Tower breaks it and lifts it off; the Tower has no chopper attached, so it takes no Schedule
/// Deliveries (`Reject::NoChopperAttached`). (The schedule's own orders keep it: `schedule`.)
#[test]
fn a_command_breaks_the_attachment() {
    let (mut t, tower, chopper) = field(|b| b);
    assert_eq!(attached(&t, chopper), Some(tower));
    next(&mut t, order(&[chopper], OrderKind::Stop));
    assert_eq!(attached(&t, chopper), None);
    assert_eq!(landing_of(&t, chopper), None, "it takes off");
    assert_eq!(gate(&t, &schedule(&[tower], S)), Err(Reject::NoChopperAttached));
    t.run(LANDING);
    assert_eq!(
        (landing_of(&t, chopper), attached(&t, chopper)),
        (None, None),
        "hovering, unattached"
    );
}

/// The chopper lands as an aircraft does (`factions/gdo/structures/airfield.md > Returning and rearming`): landing on
/// its Tower, player 2's Twister (air only) may attack it and their Peacekeeper (ground only) may not
/// (`Reject::CannotHit`); down, the other way round; taking off, an air target again.
#[test]
fn a_chopper_is_an_air_target_landing_and_a_ground_target_down() {
    let (mut t, tower, chopper) = field(|b| {
        b.spawn(
            ObjectKind::Twister,
            Some(P2),
            SpawnAt::Point(TilePos::new(5, 13).center()),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P2),
            SpawnAt::Point(TilePos::new(3, 13).center()),
        )
    });
    let (twister, peacekeeper) = (FIRST + 1, FIRST + 2);
    let attack = |units: &[u32]| order(units, OrderKind::Attack(id(chopper)));
    let cannot = Err(Reject::CannotHit(id(chopper)));
    assert_eq!(gate_of(&t, P2, &attack(&[twister])), cannot, "down");
    assert_eq!(gate_of(&t, P2, &attack(&[peacekeeper])), Ok(()));
    let away = spot(&t, tower) + Vec2::new(0.0, -200.0);
    next(&mut t, move_to(&[chopper], away));
    assert_eq!(gate_of(&t, P2, &attack(&[twister])), Ok(()), "taking off");
    assert_eq!(gate_of(&t, P2, &attack(&[peacekeeper])), cannot);
    t.run(80);
    next(&mut t, attach(&[chopper], tower));
    step_until(&mut t, 400, "landing", |t| landing_of(t, chopper).is_some());
    for _ in 0..LANDING {
        assert_eq!(
            gate_of(&t, P2, &attack(&[twister])),
            Ok(()),
            "landing, in tick {}",
            t.sim().tick()
        );
        assert_eq!(gate_of(&t, P2, &attack(&[peacekeeper])), cannot);
        t.step();
    }
    assert!(down_on(&t, chopper, tower));
    assert_eq!(gate_of(&t, P2, &attack(&[twister])), cannot, "down");
    assert_eq!(gate_of(&t, P2, &attack(&[peacekeeper])), Ok(()));
}

/// `sim` saved, with object `n`'s HP set to `hp`, and loaded: damage that no command deals on demand.
fn damaged(sim: &Sim, n: u32, hp: f32) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o["hp"] = serde_json::json!(hp);
    serde_json::from_value(state).unwrap()
}

/// The ticks the free chopper, down on its Tower with 1 HP of its 150, takes back to full, and the banks then, the
/// Tower's owner's Power as `consumers` leave it (more spawned as debug spawns).
fn repaired(consumers: &[(ObjectKind, TilePos)]) -> (Tick, (u32, u32), (u32, u32)) {
    let (mut t, tower, chopper) = field(|b| b);
    for &(kind, origin) in consumers {
        let spawn = DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at: SpawnAt::Tile(origin),
            orientation: Default::default(),
        };
        debug(&mut t, spawn);
    }
    let before = banks(&t, P1);
    let mut sim = damaged(t.sim(), chopper, 1.0);
    let hp = |sim: &Sim| sim.object(id(chopper)).and_then(|o| o.hp).expect("alive");
    let mut ticks = 0;
    while hp(&sim) < 150.0 {
        assert!(
            landing::down(sim.object(id(chopper)).unwrap()),
            "down on its Tower #{tower}"
        );
        sim.step(&TickInputs::default());
        ticks += 1;
        assert!(ticks < 400, "{} HP after {ticks} ticks", hp(&sim));
    }
    let player = sim.player(P1).unwrap();
    (ticks, before, (player.crystals, player.supplies))
}

/// A damaged chopper down on its Tower is repaired at no cost, at the repairers' rate (`factions/gdo/economy.md >
/// SupplyTower > Landing and Attachment`): from 1 HP, full in 80 ticks, the banks untouched. At the speed of Power
/// (`factions/gdo/overview.md > Power`), **the Director's reading** (the repair is one of the Tower's timers, as an
/// Airfield's rearm): with a Barracks and an Arsenal more, 50 Power generated for 85 consumed (59%), it takes 136.
#[test]
fn a_damaged_chopper_down_on_its_tower_is_repaired_free_slower_at_low_power() {
    let (full, before, after) = repaired(&[]);
    assert_eq!(full, 80, "5 s");
    assert_eq!(after, before, "at no cost");
    let consumers = [
        (ObjectKind::Barracks, TilePos::new(12, 2)),
        (ObjectKind::Arsenal, TilePos::new(12, 5)),
    ];
    let (low, before, after) = repaired(&consumers);
    assert_eq!(low, 136, "at 50 / 85");
    assert_eq!(after, before);
}
