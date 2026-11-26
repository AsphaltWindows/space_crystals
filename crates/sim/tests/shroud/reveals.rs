//! What an absorbed hit reveals (design change 0386, answering Q0143; `factions/cults/units/disciple.md > Spells >
//! Shroud`, Reveals): "An absorbed hit, whether a direct shot, a projectile or an area hit, counts as landing on the
//! object it was aimed at. It reveals the attacker to that object's owner for the usual 1.5 s. The cloud's later release
//! reveals nobody" (`core/vision.md > Attacks from outside vision`).
//!
//! The games are played on [`step`]: player 1's Disciple channels on low ground in the west, with player 1's Militant
//! under its cloud, and player 2 fires at the Militant from the high ground in the east, beyond player 1's sight.

use sc_sim::TilePos;
use sc_sim::tuning::vision::REVEAL_TICKS;

use super::*;

/// 24 × 9 tiles: level 0 in the west, a ramp on column 12, level 1 from column 13 on, as `attack_reveal.rs`'s step.
/// From level 0 a unit's sight reaches a grid unit less far onto level 1, and from level 1 its attack reaches a grid
/// unit farther down (`core/vision.md > ElevationModifier`).
fn step(enemy: Faction) -> ScenarioBuilder {
    let tiles = vec![".".repeat(24); 9].join("\n");
    let heights = vec![format!("{}{}", "0".repeat(13), "1".repeat(12)); 10].join("\n");
    ScenarioBuilder::new(&format!("{tiles}\n\n{heights}"))
        .faction(P1, Faction::Cults)
        .faction(P2, enemy)
}

/// The centre of tile (`x`, `y`).
fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

/// The objects of the games, by id: player 1's Disciple on tile (7, 4) and Militant on (9, 4), 128 su east of it,
/// under its cloud; player 2's attacker, on the high ground.
const DISCIPLE_ID: u32 = 1;
const VICTIM: u32 = 2;
const ATTACKER: u32 = 3;

/// Whether `player` sees object `id`, and whether it is revealed to them.
fn sees(t: &TestSim, player: PlayerId, id: u32) -> (bool, bool) {
    let o = t.sim().object(ObjectId(id)).expect("in the game");
    let revealed = t
        .sim()
        .vision()
        .player(player)
        .expect("a player")
        .is_revealed(ObjectId(id));
    (t.sim().is_visible(player, o), revealed)
}

/// Player 1's Disciple channels from tick 1; player 2's Peacekeeper on tile (14, 4) of the high ground, 5 grid units
/// from the Militant, out of player 1's sight there (the Disciple's reaches 5 grid units up to level 1, the
/// Militant's 4), but its attack reaches 5 grid units down. Player 2 also owns an Extraction Plate, on a patch on tile
/// (7, 5), under the cloud south of the Disciple: a building of its own with no sight (`factions/gdo/economy.md >
/// ExtractionPlate`: "SightRange - 0") that a release hits. The Peacekeeper's sight, 6 grid units down to level 0,
/// doesn't reach the Disciple's tile either, so player 2 doesn't see the Disciple.
fn shot_into_the_cloud() -> TestSim {
    let mut t = quiet(
        step(Faction::Gdo)
            .spawn(DISCIPLE, Some(P1), centre(7, 4))
            .spawn(MILITANT, Some(P1), centre(9, 4))
            .spawn(ObjectKind::Peacekeeper, Some(P2), centre(14, 4))
            .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(7, 5)))
            .spawn(ObjectKind::ExtractionPlate, Some(P2), SpawnAt::Tile(TilePos::new(7, 5)))
            .start(),
    );
    order(&mut t, 1, P1, &[DISCIPLE_ID], OrderKind::Shroud);
    t.run_until(1);
    assert_eq!(sees(&t, P1, ATTACKER), (false, false), "out of player 1's sight");
    assert_eq!(sees(&t, P2, DISCIPLE_ID), (false, false), "out of player 2's");
    assert!(sees(&t, P2, VICTIM).0, "the Peacekeeper sees its target");
    t
}

/// Design change 0386's Verification: "An unseen enemy shoots a unit standing under a Shroud cloud. The defender sees
/// the attacker for 1.5 s, although the damage is buffered." The Peacekeeper fires once at the Militant and is
/// stopped: its shot goes whole into the buffer, the Militant untouched, and player 1 sees the Peacekeeper, revealed,
/// at the end of the shot's tick and the `REVEAL_TICKS` − 1 after it, and then not: the release 8 ticks after the shot
/// doesn't renew the reveal.
#[test]
fn an_absorbed_shot_reveals_its_unseen_attacker_for_1_5_s() {
    let mut t = shot_into_the_cloud();
    order(&mut t, 2, P2, &[ATTACKER], OrderKind::Attack(ObjectId(VICTIM)));
    step_until(&mut t, 60, "the shot absorbed", |t| buffer(t, DISCIPLE_ID) > 0.0);
    let shot = t.sim().tick();
    assert_eq!(
        (hp(&t, VICTIM), hits_on(&t, VICTIM), buffer(&t, DISCIPLE_ID)),
        (40.0, vec![], 10.0),
        "the Peacekeeper's Damage, buffered"
    );
    order(&mut t, shot + 1, P2, &[ATTACKER], OrderKind::Stop);
    while t.sim().tick() < shot + REVEAL_TICKS + 8 {
        let tick = t.sim().tick();
        let revealed = tick < shot + REVEAL_TICKS;
        assert_eq!(sees(&t, P1, ATTACKER), (revealed, revealed), "tick {tick}, shot {shot}");
        t.step();
    }
    assert_eq!(releases(&t).len(), 1, "one release, at {}", shot + 8);
    assert_eq!(hits_on(&t, VICTIM), [(shot + 8, DISCIPLE_ID, 0.5)]);
}

/// "The cloud's later release reveals nobody." Its hits are the cloud's first member's, the Disciple's: player 2's
/// plate under the cloud takes its share, and player 2 still neither sees the Disciple nor has it revealed, at the
/// release or after it.
#[test]
fn the_release_reveals_nobody() {
    let plate = 5;
    let mut t = shot_into_the_cloud();
    order(&mut t, 2, P2, &[ATTACKER], OrderKind::Attack(ObjectId(VICTIM)));
    step_until(&mut t, 60, "the shot absorbed", |t| buffer(t, DISCIPLE_ID) > 0.0);
    let shot = t.sim().tick();
    order(&mut t, shot + 1, P2, &[ATTACKER], OrderKind::Stop);
    t.run_until(shot + 8);
    assert_eq!(releases(&t).len(), 1);
    let [(tick, attacker, _)] = hits_on(&t, plate)[..] else {
        panic!("the plate's share: {:?}", hits_on(&t, plate));
    };
    assert_eq!((tick, attacker), (shot + 8, DISCIPLE_ID), "by the Disciple");
    for _ in 0..REVEAL_TICKS {
        assert_eq!(sees(&t, P2, DISCIPLE_ID), (false, false), "tick {}", t.sim().tick());
        t.step();
    }
}

/// "Whether a direct shot, a projectile or an area hit": player 2's Hand of Heaven on tile (21, 4) of the high ground,
/// 12 grid units off and far out of player 1's sight, fires a missile at the ground under the Militant and is stopped.
/// Its blast lands on the Militant only, into the buffer; player 1 doesn't see the Hand of Heaven while the missile
/// flies, and sees it, revealed, from the tick the blast is absorbed for `REVEAL_TICKS` (design change 0293: a blast
/// reveals its attacker "when its damage lands, not at the throw").
#[test]
fn an_absorbed_blast_reveals_its_unseen_attacker_as_it_lands() {
    let mut t = quiet(
        step(Faction::Cults)
            .spawn(DISCIPLE, Some(P1), centre(7, 4))
            .spawn(MILITANT, Some(P1), centre(9, 4))
            .spawn(ObjectKind::HandOfHeaven, Some(P2), centre(21, 4))
            .start(),
    );
    order(&mut t, 1, P1, &[DISCIPLE_ID], OrderKind::Shroud);
    let under = TilePos::new(9, 4).center();
    order(&mut t, 2, P2, &[ATTACKER], OrderKind::AttackGround(under));
    let launched = |t: &TestSim| {
        (t.events().iter())
            .any(|(_, e)| matches!(e, SimEvent::ProjectileLaunched { shooter, .. } if *shooter == ObjectId(ATTACKER)))
    };
    step_until(&mut t, 400, "the launch", launched);
    let next = t.sim().tick() + 1;
    order(&mut t, next, P2, &[ATTACKER], OrderKind::Stop);
    while buffer(&t, DISCIPLE_ID) == 0.0 {
        assert_eq!(
            sees(&t, P1, ATTACKER),
            (false, false),
            "in flight, tick {}",
            t.sim().tick()
        );
        assert!(t.sim().tick() < 400, "the blast never came");
        t.step();
    }
    let blast = t.sim().tick();
    assert_eq!((hp(&t, VICTIM), hits_on(&t, VICTIM)), (40.0, vec![]), "absorbed");
    while t.sim().tick() < blast + REVEAL_TICKS + 8 {
        let tick = t.sim().tick();
        let revealed = tick < blast + REVEAL_TICKS;
        assert_eq!(
            sees(&t, P1, ATTACKER),
            (revealed, revealed),
            "tick {tick}, blast {blast}"
        );
        t.step();
    }
    assert_eq!(releases(&t).len(), 1, "one release");
}
