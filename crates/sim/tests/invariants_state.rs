//! The invariants H1, B1 and A1 (slice M5-M1, `testing::invariants`): **negative tests**. Every game keeps them
//! (`TestSim` checks them after every tick, as it does the rest), so no game can show the checker failing. These tests
//! break each one by editing a saved state, as `invariants_overlap.rs` does, and ask the checker, which last saw the
//! state a tick before the edited one:
//! - H1 (`invariants/hit_points.rs`): HP above the MaxHP with the unit's Augmentations, on the map or held; HP at 0,
//!   a dead object still in the game; HP missing; Shields out of 0 to the MaxShields;
//! - B1 (`invariants/banks.rs`): a bank that wrapped below 0 or past its top, unless a `DebugCommand::SetResources` of
//!   the tick set the player's banks;
//! - A1 (`invariants/augmentations.rs`): an Augmentation on a kind that can't have it, on the map or held.
//!
//! The check of the rules' invariants alone (`Invariants::check_rules`, which the seed sweep's games run) finds the same.

use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    RecruitKind, Sim, SpawnAt, TickInputs, TilePos, Vec2,
};
use serde_json::{Value, json};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The objects of [`game`]: player 1's Armory and the Recruit inside it, a Peacekeeper and a Drone (MaxShields 10) of
/// player 2's, and two Recruits of player 1's, one with Enhanced Constitution (MaxHP 40 + 20) and one with Improved
/// Metabolism (MaxHP 40 − 10).
const ARMORY: u32 = 1;
const HELD: u32 = 2;
const PEACEKEEPER: u32 = 3;
const DRONE: u32 = 4;
const CONSTITUTION: u32 = 5;
const METABOLISM: u32 = 6;

/// Player 1 plays the Cults, player 2 the Colonists, on 30 × 22 open tiles; auto-targeting is off. The Recruit east of
/// the Armory's entrance (side B) walks in, and the two others get their Augmentations, in tick 1. Returns the game a
/// tick before and at the tick after the Recruit is inside.
fn game() -> (Sim, Sim) {
    let point = |x: f32, y: f32| SpawnAt::Point(Vec2::new(x, y));
    let mut t: TestSim = ScenarioBuilder::new(&vec![".".repeat(30); 22].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::Recruit, Some(P1), point(1150.0, 704.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(300.0, 300.0))
        .spawn(ObjectKind::Drone, Some(P2), point(300.0, 400.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(1150.0, 1000.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(1150.0, 1100.0))
        .start();
    t.without_auto_targeting();
    t.order_at(1, P1, &[id(HELD)], OrderKind::Enter(id(ARMORY)), false);
    for (unit, kind) in [
        (CONSTITUTION, RecruitKind::Constitution),
        (METABOLISM, RecruitKind::Metabolism),
    ] {
        let augmentation = Augmentation::of(kind);
        t.debug_at(
            1,
            DebugCommand::SetAugmentation {
                id: id(unit),
                augmentation,
            },
        );
    }
    while t.sim().held_unit(id(HELD)).is_none() {
        assert!(t.sim().tick() < 200, "the Recruit hasn't gone in");
        t.step();
    }
    let before = t.sim().clone();
    t.step();
    (before, t.sim().clone())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// `sim` as saved and loaded again, with `edit` applied to the saved state.
fn edited(sim: &Sim, edit: impl FnOnce(&mut Value)) -> Sim {
    let mut json = serde_json::to_value(sim).expect("a state serializes");
    edit(&mut json);
    serde_json::from_value(json).expect("the edited state loads")
}

/// Object `n`'s saved state, on the map or held.
fn object(json: &mut Value, n: u32) -> &mut Value {
    let on_map = json["objects"].as_array_mut().expect("the objects are a list");
    if let Some(i) = on_map.iter().position(|o| o["id"] == n) {
        return &mut json["objects"][i];
    }
    let held = json["held"].as_array_mut().expect("the held units are a list");
    let i = held
        .iter()
        .position(|h| h["object"]["id"] == n)
        .expect("the object is in the state");
    &mut json["held"][i]["object"]
}

/// Player `p`'s saved state.
fn player(json: &mut Value, p: PlayerId) -> &mut Value {
    let players = json["players"].as_array_mut().expect("the players are a list");
    players
        .iter_mut()
        .find(|o| o["id"] == p.0)
        .expect("the player is in the state")
}

/// `sim` with object `n`'s `field` at `value`.
fn with(sim: &Sim, n: u32, field: &str, value: Value) -> Sim {
    edited(sim, |json| object(json, n)[field] = value)
}

/// The invariant the checker, which last saw `before`, finds broken in `after`, a tick on with `inputs`; the same as
/// the check of the rules' invariants alone finds, which the seed sweep's games run (`Invariants::check_rules`).
fn broken(before: &Sim, after: &Sim, inputs: &TickInputs) -> Option<&'static str> {
    let all = Invariants::new(before).check(after, inputs).err();
    let rules = Invariants::new(before).check_rules(after, inputs).err();
    let [all, rules] = [all, rules].map(|result| result.map(|violation| violation.invariant));
    assert_eq!(rules, all, "the rules' check alone");
    all
}

/// [`broken`] for a tick without commands.
fn broken_alone(before: &Sim, after: &Sim) -> Option<&'static str> {
    broken(before, after, &TickInputs::default())
}

/// H1: HP is above 0 and at most the MaxHP with the unit's Augmentations, on the map and held, and the Shields within 0
/// and the MaxShields.
#[test]
fn hp_above_its_maximum_or_at_0_and_shields_out_of_range_break_h1() {
    let (before, after) = game();
    assert_eq!(broken_alone(&before, &after), None, "the game itself keeps them");
    let research = after.research();
    let max = |n: u32| {
        after
            .object(id(n))
            .and_then(|o| o.max_hp_with(research))
            .expect("destructible")
    };
    assert_eq!(
        (max(PEACEKEEPER), max(CONSTITUTION), max(METABOLISM)),
        (50.0, 60.0, 30.0)
    );
    // At its MaxHP, a unit keeps H1; a hair above it, it breaks it.
    let hp = |n: u32, hp: f32| with(&after, n, "hp", json!(hp));
    assert_eq!(broken_alone(&before, &hp(PEACEKEEPER, 50.0)), None);
    assert_eq!(broken_alone(&before, &hp(PEACEKEEPER, 50.5)), Some("H1"));
    // The MaxHP is the one with the unit's Augmentations: above the kind's 40 with Enhanced Constitution, and below it
    // with Improved Metabolism.
    assert_eq!(broken_alone(&before, &hp(CONSTITUTION, 60.0)), None);
    assert_eq!(broken_alone(&before, &hp(CONSTITUTION, 60.5)), Some("H1"));
    assert_eq!(broken_alone(&before, &hp(METABOLISM, 30.5)), Some("H1"));
    // A unit held in a building too.
    assert!(after.held_unit(id(HELD)).is_some());
    assert_eq!(broken_alone(&before, &hp(HELD, 40.5)), Some("H1"));
    // At 0 HP or less an object is dead, and leaves the game in the phase that killed it.
    assert_eq!(broken_alone(&before, &hp(PEACEKEEPER, 0.0)), Some("H1"));
    assert_eq!(broken_alone(&before, &hp(ARMORY, -1.0)), Some("H1"));
    // A destructible object has HP.
    let no_hp = with(&after, PEACEKEEPER, "hp", Value::Null);
    assert_eq!(broken_alone(&before, &no_hp), Some("H1"));
    // The Shields stay within 0 and the MaxShields.
    let shields = |value: f32| with(&after, DRONE, "shields", json!(value));
    assert_eq!(broken_alone(&before, &shields(0.0)), None);
    assert_eq!(broken_alone(&before, &shields(10.0)), None);
    assert_eq!(broken_alone(&before, &shields(10.25)), Some("H1"));
    assert_eq!(broken_alone(&before, &shields(-0.25)), Some("H1"));
}

/// B1: a bank that went below 0 (or past its top) in a tick wrapped, as a `u32` does, unless a SetResources of the
/// tick set the player's banks; the game's own changes keep it.
#[test]
fn a_bank_that_wrapped_breaks_b1_unless_set_by_a_debug_command() {
    let (before, after) = game();
    let crystals = before.player(P1).expect("in the game").crystals;
    let bank =
        |sim: &Sim, p: PlayerId, bank: &str, value: u32| edited(sim, |json| player(json, p)[bank] = json!(value));
    // 10 Space Crystals more than the bank holds, taken by an unchecked subtraction in a release build.
    let below_0 = bank(&after, P1, "crystals", crystals.wrapping_sub(crystals + 10));
    assert_eq!(broken_alone(&before, &below_0), Some("B1"));
    // A debug command may set any bank.
    let set = |player: PlayerId| PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::SetResources {
            player,
            crystals: u32::MAX - 9,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        }),
    };
    let inputs = |player: PlayerId| TickInputs {
        commands: vec![set(player)],
    };
    assert_eq!(broken(&before, &below_0, &inputs(P1)), None);
    assert_eq!(broken(&before, &below_0, &inputs(P2)), Some("B1"), "another player's");
    // A spend, or an income, of the game's.
    assert_eq!(broken_alone(&before, &bank(&after, P1, "crystals", 0)), None);
    assert_eq!(
        broken_alone(&before, &bank(&after, P1, "crystals", crystals + 5000)),
        None
    );
    // Every bank: Supplies past their top, from near `u32::MAX` to 3, and the Colonists' Essence below 0.
    let full = bank(&before, P2, "supplies", u32::MAX - 5);
    assert_eq!(broken_alone(&full, &bank(&after, P2, "supplies", 3)), Some("B1"));
    assert_eq!(
        broken_alone(&before, &bank(&after, P2, "essence", u32::MAX)),
        Some("B1")
    );
}

/// A1: only a kind that can be augmented carries an Augmentation, on the map and held.
#[test]
fn an_augmentation_on_a_kind_that_cant_have_one_breaks_a1() {
    let (before, after) = game();
    let constitution = json!({ "constitution": true });
    // A Peacekeeper, a GDO unit.
    let augmented = with(&after, PEACEKEEPER, "augmentation", constitution.clone());
    assert_eq!(broken_alone(&before, &augmented), Some("A1"));
    // A Recruit, Cults infantry, may have both (the one at 30 of its MaxHP 30 with Improved Metabolism, which both make
    // 50).
    let both = with(
        &after,
        METABOLISM,
        "augmentation",
        json!({ "constitution": true, "metabolism": true }),
    );
    assert_eq!(broken_alone(&before, &both), None);
    // The Recruit held in the Armory, made a Peacekeeper with its Augmentation.
    let held = edited(&after, |json| {
        let o = object(json, HELD);
        o["kind"] = json!("Peacekeeper");
        o["augmentation"] = constitution;
    });
    assert_eq!(broken_alone(&before, &held), Some("A1"));
}
