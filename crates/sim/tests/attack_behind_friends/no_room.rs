//! **No way round** (slice M3-G3; design change 0223, answering question Q0057; `systems/controls/behaviors.md >
//! AttackingObject > Blocked by its own side`; `docs/movement.md > 6.5`, rule 2). An attacker whose way to its
//! target is blocked by its own side, with no way round (a corridor, a passage the front rank fills):
//! - **farther than 3 grid units from the target**, follows the Failure rule (`systems/controls/behaviors.md >
//!   MovingToLocation > Failure`): its attempts at a way round are the leg's three, and it gives up when it is still
//!   stuck `STUCK_TICKS` after the last, about 7 s after its last progress, as any stuck unit does. Until 0223 it
//!   waited behind the blockers for as long as the target stood. An Attack then completes, and the unit is idle;
//! - **within 3 grid units**, out of range (melee units behind the front rank), the approach counts as arrived, and
//!   the unit gives the target up (0223 point 1, as built).
//!
//! An idle unit then takes the target again by itself after 32 ticks (`systems/controls/behaviors.md >
//! BaseAutoTargeting > Unreachable auto-targets`, design change 0181), and fires once it is in range. An AttackMove
//! treats the chase it gave up as one it can't finish (design change 0235, slice M3-L4): it walks on along its leg, and
//! takes the target again only within weapon range, or once it has moved or the known obstacles have changed
//! (`orders::reach`).
//!
//! The rewrites of slice M3-G2's tests of a unit with no room (`in_a_corridor_the_units_behind_wait_without_moving_
//! or_planning` and `in_a_narrow_passage_the_second_row_waits`) are here, as the tests of the same scenes under
//! the new rule.

use super::*;
use sc_sim::tuning::movement::{GIVE_UP_TICKS, STUCK_GRACE_TICKS};
use sc_sim::tuning::orders::GIVE_UP_RETRY_TICKS;
use sc_sim::{Faction, MoveStatus};

/// A corridor one tile wide leads east from player 1's Deployment Center (tiles (0, 1) to (3, 4)): tile row 3, y
/// from 192 to 256, x from 256 on, between water. A Guard fills it. Two Peacekeepers fit side by side, but not
/// beside one in the middle.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ....~~~~~~~~~~~~~~~~~~~~
    ....~~~~~~~~~~~~~~~~~~~~
    ........................
    ....~~~~~~~~~~~~~~~~~~~~
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ........................
    ........................
";

/// The corridor of [`CORRIDOR`], two tiles wide: five Peacekeepers abreast fill it.
const PASSAGE: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ....~~~~~~~~~~~~~~~~~~~~
    ........................
    ........................
    ....~~~~~~~~~~~~~~~~~~~~
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ........................
    ........................
";

const SABRE: ObjectKind = ObjectKind::SabreLegionnaire; // melee: Range 0

/// What became of one attacker of the building (`id(1)`), from the order on ([`watch`]).
#[derive(Clone, Debug, Default)]
struct Story {
    /// Each change of (whether it has an order, whether it is engaged on the building): its tick, and the new pair.
    flips: Vec<(Tick, bool, bool)>,
    /// Each failed move: its tick, the ticks since the unit's last progress, and the attempts at another route it
    /// had left.
    failed: Vec<(Tick, Tick, u8)>,
    /// The ticks its attacks on the building took effect, and whether it had an order then.
    shots: Vec<(Tick, bool)>,
    /// The longest it went without progress while it was on its way somewhere (`MoveStatus::Moving`): to the
    /// building, or past it on an AttackMove.
    longest_stall: Tick,
}

impl Story {
    /// The ticks it gave the building up on its order: engaged on it on its order one tick, and the next either no
    /// longer engaged (an AttackMove walks on) or no longer ordered (an Attack completes).
    fn gave_up(&self) -> Vec<Tick> {
        (self.flips.windows(2))
            .filter(|pair| pair[0].1 && pair[0].2 && !(pair[1].1 && pair[1].2))
            .map(|pair| pair[1].0)
            .collect()
    }

    /// The ticks it took the building up by itself after giving it up: with no order (idle), or on its AttackMove.
    fn took_it_again(&self) -> Vec<Tick> {
        let Some(&first) = self.gave_up().first() else {
            return Vec::new();
        };
        (self.flips.iter())
            .filter(|&&(tick, _, engaged)| tick >= first && engaged)
            .map(|&(tick, ..)| tick)
            .collect()
    }

    /// The first tick its order ended.
    fn order_ended(&self) -> Option<Tick> {
        (self.flips.iter())
            .find(|&&(_, ordered, _)| !ordered)
            .map(|&(tick, ..)| tick)
    }
}

/// Steps `t` up to tick `until`, or until the building falls, and tells in `stories` what became of the attackers
/// `FIRST`, `FIRST + 1` and so on while it stood (as it falls, their orders end). Returns the paths planned
/// meanwhile: searches and flow fields.
fn watch(t: &mut TestSim, stories: &mut [Story], until: Tick) -> u32 {
    let mut planned = 0;
    while t.sim().tick() < until && t.sim().object(id(1)).is_some() {
        t.step();
        if t.sim().object(id(1)).is_none() {
            break;
        }
        let tick = t.sim().tick();
        let nav = t.sim().nav_counters();
        planned += nav.searches + nav.fields;
        for (unit, story) in (FIRST..).zip(stories.iter_mut()) {
            let Some(o) = t.sim().object(id(unit)) else {
                continue;
            };
            let ordered = o.orders.as_ref().is_some_and(|orders| !orders.is_idle());
            let engaged = o.attack.as_ref().and_then(|attack| attack.target) == Some(id(1));
            if story.flips.last().is_none_or(|&(_, o, e)| (o, e) != (ordered, engaged)) {
                story.flips.push((tick, ordered, engaged));
            }
            let mover = t.mover(unit);
            if mover.status == MoveStatus::Moving {
                story.longest_stall = story.longest_stall.max(tick - mover.last_progress);
            }
            for event in t.sim().events() {
                match *event {
                    SimEvent::MoveFailed { unit: failed } if failed == id(unit) => {
                        let mover = t.mover(unit);
                        story
                            .failed
                            .push((tick, tick - mover.last_progress, mover.repaths_left));
                    }
                    SimEvent::Attacked { attacker, target, .. } if attacker == id(unit) && target == id(1) => {
                        story.shots.push((tick, ordered));
                    }
                    _ => {}
                }
            }
        }
    }
    planned
}

/// `story`'s unit followed the Failure rule (`systems/controls/behaviors.md > MovingToLocation > Failure`), and
/// waited nowhere: it was never on its way for longer than `GIVE_UP_TICKS` without progress (with
/// `STUCK_GRACE_TICKS` more for a unit not touching its blocker then, `docs/movement.md > 6.5`), and each of its
/// failed moves came after the leg's three attempts at another route (here, at a way round), when it was still
/// stuck `STUCK_TICKS` after the last of them.
fn assert_failure_rule(unit: u32, story: &Story) {
    // (M4-W3) The grace four times over (once until design change 0323): each of the three attempts at another route
    // and the giving up waits up to `STUCK_GRACE_TICKS` for a unit that touches nobody in the tick it is due, and
    // each attempt is timed from the one before. A unit shuffling a step along its blocker touched it in every
    // tick; one that takes a contact step across the corridor, a whole width, has nobody ahead of it for some ticks
    // of the step (`docs/movement.md > 6.3`). Measured: 117 ticks at most. It still gives up by the Failure rule.
    assert!(
        story.longest_stall < GIVE_UP_TICKS + 4 * STUCK_GRACE_TICKS,
        "#{unit} went {} ticks without progress: {story:?}",
        story.longest_stall
    );
    for &(tick, waited, attempts_left) in &story.failed {
        assert!(
            attempts_left == 0 && waited <= GIVE_UP_TICKS + STUCK_GRACE_TICKS,
            "#{unit}'s move failed in tick {tick}, {waited} ticks after its last progress, with {attempts_left} \
             attempts left: {story:?}"
        );
    }
}

/// `map` with player 1's Deployment Center at the end of its corridor, and `n` units of `kind` of player 2 in it,
/// one behind the other along the row `y` from x `x`, `spacing` su apart, who are ordered to `order` in tick 2.
/// Returns the game in tick 2.
fn in_corridor(map: &str, kind: ObjectKind, n: u32, (x, y): (f32, f32), spacing: f32, order: OrderKind) -> TestSim {
    let mut builder = ScenarioBuilder::new(map)
        .spawn(DC, Some(P1), SpawnAt::Tile(TilePos::new(0, 1)))
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(1, 6)));
    for i in 0..n {
        builder = builder.spawn(kind, Some(P2), SpawnAt::Point(Vec2::new(x + spacing * i as f32, y)));
    }
    let mut t = builder.start();
    let units: Vec<ObjectId> = (FIRST..FIRST + n).map(id).collect();
    t.order_at(2, P2, &units, order, false);
    t.run_until(2);
    t
}

/// **0223's Verification**: "Four Peacekeepers in a 1-tile corridor attack a building: the front ones fire. The ones
/// behind with no way round stop after about 7 s, idle, and fire by themselves if the target is in range." Here
/// under an explicit Attack (an AttackMove: the next test).
///
/// The first stops in range and fires; the three behind it can't get past it (`CORRIDOR`). Each of them gives up
/// by the Failure rule, its Attack completing in the next tick, and is idle from then on: until 0223 it kept its
/// order and waited behind the first for as long as the building stood. Idle, it leaves the target it gave up alone
/// for `GIVE_UP_RETRY_TICKS` (design change 0181; slice M3-L4: until then it took it up again in the very tick its
/// Attack completed) and then goes for it again by itself, and gives it up again the same way. The first unit leaves
/// (it is removed in tick 301): the idle three walk up and fire, with no order, each once its next try has come.
#[test]
fn four_peacekeepers_attack_down_a_corridor_and_the_ones_behind_give_up() {
    let mut t = in_corridor(CORRIDOR, PK, 4, (540.0, 224.0), 40.0, OrderKind::Attack(id(1)));
    let mut stories = vec![Story::default(); 4];
    watch(&mut t, &mut stories, 300);
    let front = &stories[0];
    assert!(
        front.shots.len() > 10 && front.flips == [(3, true, true)],
        "the first fires, on its order: {front:?}"
    );
    for (unit, story) in (FIRST..).zip(&stories).skip(1) {
        assert_eq!(story.shots, [], "#{unit} can't get into range behind the first");
        assert_failure_rule(unit, story);
        let failed = story.failed.first().map(|&(tick, ..)| tick);
        let ended = story.order_ended();
        assert!(
            failed.is_some_and(|failed| ended == Some(failed + 1) && failed < 240),
            "#{unit} doesn't wait behind the first: its move fails, and then its Attack completes: {story:?}\n{}",
            t.ascii(None)
        );
    }
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, DebugCommand::Remove { id: id(FIRST) });
    watch(&mut t, &mut stories, gone + 100);
    for (unit, story) in (FIRST..).zip(&stories).skip(1) {
        assert!(
            story
                .flips
                .iter()
                .all(|&(tick, ordered, _)| !ordered || tick < story.order_ended().unwrap()),
            "#{unit} stays idle: {story:?}"
        );
        let ended = story.order_ended().unwrap();
        assert_eq!(
            story.took_it_again().first().copied(),
            Some(ended + GIVE_UP_RETRY_TICKS),
            "#{unit} takes it up again by itself 32 ticks after it gave it up: {story:?}"
        );
        assert!(
            matches!(story.shots.first(), Some(&(tick, false)) if tick > gone),
            "#{unit} fires by itself once there is room: {story:?}"
        );
    }
}

/// The same down the corridor under an AttackMove, and slice M3-G2's three Guards (`in_a_corridor_the_units_behind_
/// wait_without_moving_or_planning` until 0223): the first stops in range and fires, and the units behind can't get
/// past it. Each of them now gives up its chase by the Failure rule, the movement's part of 0223, which this test
/// is about: no indefinite wait in the mover.
///
/// **What an AttackMove then does** (design change 0235, answering question Q0073; slice M3-L4, `orders::reach`):
/// "A chase given up by the Failure rule puts the enemy in the reach rule's rejected set", and 0235's Verification
/// is this scene: "The rear ones give up once (about 7 s), walk on, and don't turn back while the front one still
/// fires." Each unit behind gives the building up once, on its AttackMove, and never takes it up again on it; it
/// walks on along its leg, which is blocked by the first unit too, so that its AttackMove ends by the Failure rule
/// in its turn. (Until 0235 it took the building up again after `GIVE_UP_RETRY_TICKS`, a lap of 116 ticks, and its
/// AttackMove never ended.) Idle then, it tries for the building by itself every 32 ticks or so (design change 0181).
///
/// And as in M3-G2's test they stay calm: once they have closed up (by tick 200), they don't step to and fro along
/// the corridor, only sideways across it, and they plan few paths, where the bound is one a second for each of
/// them. When the first unit leaves (removed in tick 601), the next walks up and fires.
#[test]
fn in_a_corridor_the_units_behind_give_up_the_chase() {
    let order = OrderKind::AttackMove(Vec2::new(280.0, 224.0));
    for (kind, n, x, spacing) in [(GUARD, 3, 560.0, 60.0), (PK, 4, 540.0, 40.0)] {
        let mut t = in_corridor(CORRIDOR, kind, n, (x, 224.0), spacing, order);
        let mut stories = vec![Story::default(); n as usize];
        // (M4-W3) Closed up by tick 240 (200 until design change 0323): the last Peacekeeper crosses the corridor
        // by its contact step and back before it closes the last 24 su, in tick 219.
        watch(&mut t, &mut stories, 240);
        let closed_up: Vec<f32> = (FIRST..FIRST + n).map(|unit| t.pos(unit).x).collect();
        let mut planned = 0;
        while t.sim().tick() < 600 {
            let next = t.sim().tick() + 1;
            planned += watch(&mut t, &mut stories, next);
            let now: Vec<f32> = (FIRST..FIRST + n).map(|unit| t.pos(unit).x).collect();
            // Within a Peacekeeper's width: once their AttackMoves have ended they are idle, and an idle unit steps
            // aside for one that tries for the building (`docs/movement.md > 6.6`).
            assert!(
                now.iter().zip(&closed_up).all(|(now, then)| (now - then).abs() < 24.0),
                "{kind:?}: tick {}: {now:?} {closed_up:?}\n{}",
                t.sim().tick(),
                t.ascii(None)
            );
        }
        assert!(planned < 400 / 16 * (n - 1), "{kind:?}: {planned} paths in 400 ticks");
        let front = &stories[0];
        assert!(
            front.shots.len() > 10 && front.gave_up().is_empty(),
            "{kind:?}: the first fires all along: {front:?}"
        );
        for (unit, story) in (FIRST..).zip(&stories).skip(1) {
            assert_eq!(
                story.shots,
                [],
                "{kind:?}: #{unit} can't get into range behind the first"
            );
            assert_failure_rule(unit, story);
            let gave_up = story.gave_up();
            let failed: Vec<Tick> = story.failed.iter().map(|&(tick, ..)| tick + 1).collect();
            assert!(
                gave_up.len() == 1 && gave_up[0] < 200 && failed.first() == Some(&gave_up[0]),
                "{kind:?}: #{unit} gives up its chase once, as its move fails, by tick 200: {story:?}"
            );
            // 0235: on its AttackMove it never takes the building up again; the AttackMove ends.
            let on_its_order =
                (story.flips.iter()).filter(|&&(tick, ordered, engaged)| ordered && engaged && tick >= gave_up[0]);
            assert_eq!(on_its_order.count(), 0, "{kind:?}: #{unit}: {story:?}");
            assert!(
                story
                    .order_ended()
                    .is_some_and(|ended| ended > gave_up[0] && ended < 400),
                "{kind:?}: #{unit}'s AttackMove ends: {story:?}"
            );
        }
        t.debug_at(601, DebugCommand::Remove { id: id(FIRST) });
        watch(&mut t, &mut stories, 700);
        let next = &stories[1];
        assert!(
            matches!(next.shots.first(), Some(&(tick, false)) if tick < 700),
            "{kind:?}: the next one walks up and fires, by itself: {next:?}"
        );
    }
}

/// **A narrow passage**: the corridor two tiles wide ([`PASSAGE`]), and eight Peacekeepers on an AttackMove onto the
/// building (`in_a_narrow_passage_the_second_row_waits` until 0223). The first row of five stops in range and fires;
/// the three behind have no room in range and no way round. They give up their chases by the Failure rule, rather
/// than wait until the building falls (in tick 427). Measured: they first gave up in ticks 168, 175 and 178.
#[test]
fn in_a_narrow_passage_the_second_row_gives_up_the_chase() {
    let order = OrderKind::AttackMove(Vec2::new(280.0, 192.0));
    let mut t = in_corridor(PASSAGE, PK, 8, (620.0, 192.0), 40.0, order);
    let mut stories = vec![Story::default(); 8];
    watch(&mut t, &mut stories, 900);
    let fired = stories.iter().filter(|story| !story.shots.is_empty()).count();
    assert_eq!(fired, 5, "five abreast are in range\n{}", t.ascii(None));
    for (unit, story) in (FIRST..).zip(&stories) {
        assert_failure_rule(unit, story);
        if story.shots.is_empty() {
            assert!(
                story.gave_up().first().is_some_and(|&first| first < 250),
                "#{unit} gives up its chase: {story:?}"
            );
        }
    }
}

/// 40 × 12 open tiles with player 1's Deployment Center on tiles (15, 3) to (18, 6), and eight Sabre Legionnaires
/// of player 2 (Colonists) east of it, as in [`super::assault`]: a column along the row `y`, or a line. They are
/// ordered to attack it (from 21.5 GU, where they see it) or to attack-move to the ground just east of it.
fn sabres(formation: Formation, spacing: f32, y: f32, attack: bool) -> TestSim {
    let row = ".".repeat(40);
    let map = vec![row; 12].join("\n");
    let mut builder = ScenarioBuilder::new(&map)
        .faction(P2, Faction::Colonists)
        .spawn(DC, Some(P1), SpawnAt::Tile(TilePos::new(15, 3)))
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(1, 9)));
    let x = if attack { 21.5 * 64.0 } else { 26.0 * 64.0 };
    for i in 0..8 {
        let at = match formation {
            Formation::Column => Vec2::new(x + spacing * i as f32, y),
            Formation::Line => Vec2::new(x + 64.0, 40.0 + spacing * i as f32),
        };
        builder = builder.spawn(SABRE, Some(P2), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    let units: Vec<ObjectId> = (FIRST..FIRST + 8).map(id).collect();
    let order = if attack {
        OrderKind::Attack(id(1))
    } else {
        OrderKind::AttackMove(gu(19.6, 4.5))
    };
    t.order_at(2, P2, &units, order, false);
    t.run_until(2);
    t
}

/// **0223's Verification**: "Eight Sabre Legionnaires on a Deployment Center: the blocked ones give up, then
/// re-acquire on their own." (0223 point 1, as built.) Under an Attack and an AttackMove, in three layouts. The
/// front rank strikes; the Sabres behind it, within 3 grid units of the building and out of range, give it up (an
/// Attack completes; an AttackMove walks on). Idle after an Attack, each of them takes it up again by itself
/// before the building falls. On an AttackMove the chase given up is one the unit can't finish (design change 0235:
/// "an AttackMove treats it as a failed chase"): it walks on along its leg, and engages the building again on its
/// order only where it strikes it, within weapon range. None of them waits behind the others
/// ([`assert_failure_rule`]).
#[test]
fn eight_sabres_on_a_deployment_center_give_up_and_take_it_up_again() {
    for attack in [true, false] {
        for (formation, spacing, y) in [
            (Formation::Column, 40.0, 288.0),
            (Formation::Column, 64.0, 250.0),
            (Formation::Line, 80.0, 0.0),
        ] {
            let mut t = sabres(formation, spacing, y, attack);
            let mut stories = vec![Story::default(); 8];
            watch(&mut t, &mut stories, 600);
            let layout = format!("attack {attack}, {formation:?} {spacing} {y}");
            assert!(t.sim().object(id(1)).is_none(), "{layout}: the building still stands");
            let mut blocked = 0;
            for (unit, story) in (FIRST..).zip(&stories) {
                assert_failure_rule(unit, story);
                if story.gave_up().is_empty() {
                    assert!(
                        !story.shots.is_empty(),
                        "{layout}: #{unit} neither strikes nor gives up: {story:?}"
                    );
                    continue;
                }
                blocked += 1;
                if attack {
                    assert!(
                        !story.took_it_again().is_empty(),
                        "{layout}: #{unit} doesn't take it up again: {story:?}"
                    );
                } else {
                    let gave_up = story.gave_up()[0];
                    let on_its_order = (story.flips.iter())
                        .filter(|&&(tick, ordered, engaged)| ordered && engaged && tick >= gave_up)
                        .count();
                    assert!(
                        on_its_order == 0 || !story.shots.is_empty(),
                        "{layout}: #{unit} takes it up again out of its reach: {story:?}"
                    );
                }
            }
            assert!(blocked >= 2, "{layout}: {blocked} gave up");
        }
    }
}
