# Task M6-S6b: a sweep: 0455 (Metabolism and Inspire as percentages, and the attack-speed rule), 0456 and 0457 (the air radii, and confirmed numbers), 0459 (vehicles drive clear of a Tunnel's exit), and 0464 (Mirage copies buffs; Shift+F queues)

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **This is the plan's S6b** (`artifacts/director/m6-plan.md > §C`: "Later [answers] go to S6a (W9) or S6b (W10)"), the second answer sweep. Later answers get their own sweep.
- **Five independent parts.** Commit each as its own finished step, 0455 first. Parts 2 and 4 are small.

## 1. Design change 0455 (answers Q0180): Metabolism and Inspire become percentages; one rule for attack-speed frames
Read `artifacts/designer/changes/0455-attack-and-movement-speed-percentages.md`, and the docs it lists:
- `systems/combat.md > AttackSpeed` (new): all of it;
- `factions/cults/augmentation.md > Increments` (the Metabolism increment, and who counts as unarmed);
- `factions/cults/units/prayer_mobile.md > Abilities > Inspire` (Stacks) and `> Placeholder stats` (Inspire).

**What changes:**
- **A Metabolism increment** gives **+15% movement speed** (MaxSpeed). For unarmed units, today only the Recruit, it also gives **+15% attack speed**. Increments add, from −45% (−3) to +90% (+6).
  - This replaces +0.5 su/frame and −1 frame of FiringDuration per increment.
  - The other infantry get only the movement part, as before: the Militant and the Gunner, which are armed, and the Disciple, which has no attack.
- **Inspire** gives ×1.25 movement speed to every own infantry unit, and ×1.25 attack speed to Recruits. It multiplies after the augmentations.
  - This replaces M5-N11's +1.5 su/frame and its cycle × 0.75 (with its rounding, Firing-only, and no floor).
- **The factor** = (1 + 0.15 × net Metabolism increments) × (1.25 if inspired).
  - **MaxSpeed** = base × factor, unrounded.
  - **The attack cycle** (Aim + Firing + Cooldown + Reload) = base cycle ÷ factor, **rounded once, to the nearest frame, halves up**.
  - **The phases share the new cycle in proportion to their base lengths.** Minimums: Firing at least 2 frames, and Aim, Cooldown and Reload at least 1. A phase at its minimum passes the rest of the change to the others. The shortest cycle is 5 frames.

**How the phases share the frames is the Director's call** (0455: "as long as they sum to the rounded cycle"). Build it this way, and cite it as the Director's choice in the code:
1. **The cycle, in integers.** Write the factor as a fraction: (100 + 15n)/100, times 5/4 if inspired. Then cycle = round-half-up(base cycle × 100 × 4 ÷ ((100 + 15n) × (5 if inspired, else 4))), done in integers: `(2 × num + den) / (2 × den)`. No floats here: the halves must round up exactly, and the sim is deterministic.
2. **Minimums first.** A phase whose exact proportional share is below its minimum gets its minimum and leaves the share-out. The others share the frames left, in proportion to their base lengths. Repeat until no share falls below its minimum.
3. **Largest remainder.** Each remaining phase gets the floor of its exact share. The frames left go one each to the phases with the largest remainders. On a tie, the earlier phase wins: Aim, Firing, Cooldown, Reload. Exact shares and remainders are integer fractions too.

**Worked examples** for your tests (the Recruit's base is 2/8/1/1 = 12):
- **+6 Metabolism, inspired, cycle 5:** the shares are 0.83/3.33/0.42/0.42. Aim, Cooldown and Reload are below 1, so each gets 1. Firing gets the other 2, which is its minimum. Result: 1/2/1/1.
- **0 Metabolism, inspired, cycle 10:** Cooldown and Reload (0.83 each) get 1 each. Aim and Firing share 8 as 1.6/6.4: floors 1/6, and the frame left goes to Aim (0.6 beats 0.4). Result: 2/6/1/1.
- **−3 Metabolism, cycle 22:** the shares are 3.67/14.67/1.83/1.83, none below its minimum. Floors 3/14/1/1 leave 3 frames: one each to Cooldown and Reload (0.83), and one to Aim (0.67, tied with Firing, and first). Result: 4/14/2/2.

**Where:**
- `augmentation.rs` and `augmentation/stats.rs` (M5-N1a's augmented stats) for the increments;
- `buffs.rs`'s `inspired` and `defs/cults/units/prayer_mobile.rs` for Inspire;
- one shared function for the frames, which both use. It doesn't belong to either: `combat` is a good home. A later attack-speed effect should be one more term in the factor.

**Numbers:** 15%, ×1.25 and the minimums are def numbers or tunables by m6-common's checklist, each with its citation (0455's are placeholders).

**Acceptance (0455's Verification):**
1. **The Recruit's cycle table,** every row, plain and inspired: −3: 22 (17); −1: 14 (11); 0: 12 (10); +2: 9 (7); +4: 8 (6); +6: 6 (5). Assert the phases too, and that they sum to the cycle.
2. **MaxSpeed:** a Level 2 Improved Metabolism Recruit moves at 7.6 su/frame, and at 9.5 inspired. A plain Militant inspired moves 25% faster.
3. **An armed unit's attack** is unchanged by Metabolism and by Inspire: a Militant's cycle with +6 Metabolism, inspired, is its base cycle.
4. **The tests pinning M5-N11's Inspire placeholders** (×0.75, Firing-only, no floor) are rewritten to 0455, citing it.

**Goldens and the computer players:**
- A golden with an augmented Recruit, or an inspired unit, may move.
  - Check that only 0455 moved it, say which, and re-bless it.
  - Report any golden that moves without one. None should.
- If any of the 192 computer-player rows moves, say which and why. The Cults' computer player may augment Recruits or cast Inspire.

## 2. Design changes 0456 (answers Q0181) and 0457 (Q0182): the air units' SeparationRadius, and confirmed numbers
Read `artifacts/designer/changes/0456-air-separation-radii.md`.
- **The values already match:** M6-GK and M6-SK registered them as the Silhouette's side + 8 su (the Hornet 56, the Supply Chopper 68, the drone 32, the Phantom 52, the Prism 48).
- **Replace the "Placeholder (Q0181)" marks** in their def modules and tests with 0456's citation. The files are `defs/gdo/units/{hornet,supply_chopper,drone_operator}.rs` and `defs/syndicate/units/{phantom,prism}.rs`. Grep for Q0181 across the workspace for any others.
- The values stay placeholders for the numbers pass, as 0456 says. Keep the word "placeholder" with the 0456 citation.
- **0456's Verification,** if no test covers it yet: two of each of the five, flown onto one spot, push apart until their centres are about the radius apart. One test over the five kinds is enough.
- **0457 (Q0182 parts 1 and 4) confirms two more of M6-GK's numbers.** Read `artifacts/designer/changes/0457-gdo-numbers-chopper-outpost-mine.md`, and replace their marks with 0457's citation:
  - the Supply Chopper's UnitControlCost, 2 (`defs/gdo/units/supply_chopper.rs`); it stays a designer placeholder;
  - the Raptor's TargetType, SingleTarget (`defs/gdo/units/raptor.rs`).
  - **The other Q0182 marks** belong to the slices that own their files: the mine's and the Twister's to M6-GM, the Hornet's to M6-GH2 (0458 changes it), the Outpost's to M6-GP, and the Dragonfly's to M6-GL. Leave them.

## 3. Design change 0459 (answers Q0183): vehicles ejected from a Tunnel drive clear of the exit
Read `artifacts/designer/changes/0459-tunnel-exits-vehicles-drive-clear.md`, and `factions/syndicate/tunnels.md > Tunnel Structure Sides > Side A` ("Vehicles clear the exit").

**What changes:**
- **An ejected wheeled or tracked vehicle with nowhere to go** (no rally point, no order) no longer stops at the end of the exit path. It drives to a stop point **3 GU straight out from the end of the exit path** (a placeholder).
- **If that point is taken,** it parks at the nearest free spot around it, by 0451's rule. Idle vehicles don't yield (0440), so this keeps the exit path clear, and every ejected vehicle gets out.
- **Unchanged:**
  - auto-ejected units, which still go to their rally point, parking around it when it is taken (0451; M6-GF built it for the pool's vehicles, `sim/holding.rs`);
  - infantry, which stop at the end of the path and step aside;
  - aircraft at the Tier 3 doors.
- **Not a rally point:** the player can't set it. Tunnels' rally points stay on production expansions.

**Where:**
- the ejection: `sim/network/ejects.rs` (M6-SN changed it before you; read its Built entry) and the exit path's end (`network/paths.rs`);
- the parking: reuse M6-GF's `sim/producers/park.rs` (`park_out_of`, `parking_spot`) rather than a second search. It is GF's, merged, so you may widen it for a Tunnel's exit;
- the 3 GU: a tunable, with 0459's citation, by m6-common's checklist;
- **Q0183's pins:** M6-SV pinned the stall (of five Bikes ejected with no order, two come out). Rewrite them to 0459.

**Acceptance (0459's Verification):**
1. Five Assault Bikes in the network, ejected from a Tier 2 Tunnel with no orders: all five come out and cluster about 3 GU out from the exit path. None stays inside.
2. Five infantry (Guards, say) ejected the same way stop at the end of the path and step aside, as before.
- A golden in which a vehicle is ejected with no order may move. Check that only 0459 moved it, and re-bless it. If any of the 192 rows moves, say which and why.

## 4. A client bug: a Shroud cloud's soft edge is clipped on its west and north sides
M6-CM found it, and left it: `crates/game/src/shroud_view.rs`'s `Area::of` floors both `at − r` and `at + r`. When a circle's edge lies on a tile line, the cloud's soft edge is clipped flat on its west and north sides.
- **Fix the bounds** so that the area covers the whole circle and its soft edge on every side. Round the low bound down and the high bound up, or add the soft edge's margin; your call.
- **A client test** with a circle whose edge lies exactly on a tile line, west and north: the area covers it.
- **A QA screenshot,** before and after, of a cloud whose edge lies on a tile line.

## 5. Design change 0464 (answers Q0188): Mirage copies the parent's buffs, and Shift+F queues
Read `artifacts/designer/changes/0464-mirage-copies-buffs-and-shift-queues.md`, and `factions/cults/units/disciple.md > Spells > Mirage` as it stands (its "Health" and "Shift-queueing" lines). M6-CM built Mirage on two assumptions, which 0464 changes. Read CM's Built entry in `artifacts/director/plan.md` first.

**1. Illusions copy the parent's timed buffs.**
- "Each timed buff the parent has at the cast (e.g. Inspire, a Shrine's regeneration) is copied to every illusion **for the time it has left**, so they keep matching it."
- Every `Buff` is timed (`buffs.rs`: `{ ability, until }`), so a copy keeps the parent's `until`. Copy them where `Sim::cast_mirage` (`sim/mirage.rs`) makes the illusions.
- Part 1 of this brief makes Inspire a percentage. Write this test against the rule as part 1 leaves it.

**2. Shift+F queues a Mirage step.**
- Today Mirage is an ability (`Command::Ability(Ability::Mirage)`, `sim/modes.rs`'s `use_ability`), cast at once, and Shift is ignored.
- **Plain F is unchanged:** it casts at once, and ends a Shroud channel.
- **Shift+F adds a Mirage step to the unit's order queue.** For example, a Move and then a Shift+F cast on arrival.
  - When its turn comes, it casts if the unit has enough energy.
  - If not, it is dropped "with the usual refusal feedback". Reuse M6-GA's `SimEvent::OrderDropped` path if it fits, and say what the player sees.
  - The step takes no time: the queue goes on to the next order in the same tick, or the next.
- **Behind a Shroud channel,** a queued Mirage waits for the channel to end, as every Shift-queued order does. The doc notes that in practice it is cast from Shroud with plain F. Don't add a special case.
- **The gate at the press:** it refuses an illusion's Shift+F as it refuses F (`IllusionsCantCast`). Whether it refuses a Shift+F for too little energy *now* is the doc's "when its turn comes" question: accept it at the press, and check the energy when its turn comes.
- **How the step is modelled is your call.** A new `OrderKind` (say `OrderKind::Mirage`) hashes as **tag 39** (your tag, below). The client's Shift+F must send it.
- **The controls doc:** regenerate it.

**Acceptance (0464's Verification):**
1. Inspire a Disciple, and cast Mirage 4 s later. All five move at the inspired speed, and the illusions' buff ends in the same tick as the parent's.
2. Order a Move, then Shift+F. Mirage casts when the Disciple arrives.
3. With too little energy at that point, nothing is cast, and the player gets the refusal feedback.
- Also: plain F during a Move still casts at once; a Shift+F given by an illusion is refused; and `cults_mirage` doesn't move. If it does, say why.

## Hash tags
Part 5's Mirage step, if it is a new `OrderKind`, hashes as **tag 39** in `orders/hash.rs`. Otherwise you should need no new state. If you do, new `Object` state hashes and saves only while present, under **Object tag 31**, and a Behaviour under **Behaviour tag 30** (`artifacts/director/hash-tags.md`).

## Running beside you
Filled in at launch.

## What you own
- the Metabolism increment's effects (`augmentation.rs`, `augmentation/stats.rs`) and Inspire's (`buffs.rs`'s `inspired`, the Prayer-mobile's def), and the new attack-speed function;
- the tests that pin them;
- the Q0181 lines in the five def modules, and the Chopper's and the Raptor's Q0182 lines (0457);
- the Tunnel's stop point for ejected vehicles (0459): the vehicle arm of `sim/network/ejects.rs`, the parking's widening, and their tests;
- `shroud_view.rs`'s `Area::of` and its test;
- the Mirage step (0464): the buff copy in `sim/mirage.rs`, the queued step and its arm in the orders, the gate's Shift+F, and the client's Shift+F.

Read-only: every other slice's files (`m6-plan.md > §C`).

## Checks
m5-common's "Done means".

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 14 lines:
- 0455: the factor, the frames' share-out, the table, MaxSpeed, and what moved (goldens, computer-player rows);
- 0456 and 0457: the marks, and the Verification test;
- 0459: the stop point, the parking around it, and what moved;
- the Shroud cloud's edge: the fix;
- 0464: the buff copy, how the Mirage step is modelled, and the feedback when it is dropped;
- anything left for the Director.
