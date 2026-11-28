# Task M6-GM: The Twister's mines

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > GM: The Twister's mines`, with T89 and T90 (`> §D`).

Read them before you start. One design change since the plan replaces four of its placeholders (below).

## Goal
- **A Twister starts with 3 mines,** and buys more (F: 25 SC, 80 frames each), up to 3.
- **Laying.** It lays one anywhere (R): it drives there and stops for 16 frames.
- **A mine** is hidden underground. An enemy infantry unit or vehicle within 32 su sets it off, and 8 frames later it explodes for 200 over 64 su. The blast hurts own units too, but never air.
- **A mine is a deployed unit,** selectable by its owner, not a structure.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics doc** (0481): `systems/mechanics/spawned_objects.md`. The mine is a **Deployed object**: not a structure, hidden, with a trigger and an effect, and removed when destroyed. The Twister's mines are the doc's **stock and purchase**, which queues up to the free stock.
Read `factions/gdo/units/twister.md > Mines` and `> Placeholder stats` as they stand.
- **0426 (Q0160):** read `artifacts/designer/changes/0426-twister-mines.md`. Each part replaces a placeholder of D35:
  1. **A mine is "closer to a permanently deployed unit".** It is not a structure, so it doesn't count for the Defeat Condition, and its owner can select it. Enemies can select it only once they see it, which needs detection (not designed yet).
     - Designer placeholders: Unit Control 0, Groupable true, an empty command panel.
     - It needs a selection shape and an InfoPanel: 16×16, HP 40.
     - This replaces D35's "isn't selectable".
  2. **Hover units don't trigger mines:** the HoverVehicles (the Colonists' Drone) and the HoverCraft, as with sludge. A blast set off by something else still damages a HoverVehicle in its radius: it is a ground unit, and only air is exempt (designer's reading). This replaces D35's "every enemy ground-domain unit triggers it".
  3. **A mine destroyed by damage is removed without exploding.** (D35, now the design.)
  4. **Buy Mine presses queue up to the free stock,** each paid at its press, bought one after another, 80 frames (5 s) each. This replaces D35's "F, pressed while a purchase runs, is refused".
- **0267:** detection comes later. Area damage destroys mines now. An Attack on a mine needs a player to see it, so it waits for detection.
- **0441 (the Twister's Supply) and 0451 (parking):** M6-GF built both. Nothing of yours.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > GM`'s Code list. Since the plan:
- **M6-GK registered the mine** (`ObjectKind::Mine`, `Category::Mine`) with its numbers, and **M6-GF made the Twister live,** its R and F greyed `Planned` (`kinds/gdo/twister.rs`, `Planned::{LayMine, BuyMine}`). Replace them with the real commands.
- **Queued buying (0426 part 4) changes GD's `stock.rs`,** which the plan made read-only for you. It's yours now, for this rule alone:
  - Today a purchase is refused while one runs.
  - Make it a count of purchases waiting, each paid at its press, with a press refused only when the stock plus the purchases already under way would pass the type's max.
  - For the Drone Operator (max 1), that is the same as today's rule, and 0421's "refused while the drone is out" stays. So GD's tests and the golden `gdo_drones` must not move. Check both.
- **0457 (Q0182 part 3): the mine's sight and armour,** the user's numbers. Read `artifacts/designer/changes/0457-gdo-numbers-chopper-outpost-mine.md` and `factions/gdo/units/twister.md > Placeholder stats` (the mine).
  - **SightRange 1 grid unit,** and **PointArmor 12, FullArmor 12.** They replace M6-GK's placeholders of 0 and none in `defs/gdo/mine.rs`; drop those marks and cite 0457.
  - **The reason** (the user's): mines shouldn't be easy to clear. Weak hits do only the minimum damage against its 40 HP.
  - **0457's Verification:** a laid mine's owner sees a 1 GU circle around it. Peacekeepers shooting a revealed mine do only the minimum damage per hit.
  - **The 8-grenades note below** was worked out at armour 0. Work out the number again at 12/12, size the test by it, and give it in your report.
- **0457 part 4: the Twister's TargetType** (SingleTarget, its FullyConnected gun) is confirmed. Replace the "Placeholder (not in the design: the Twister's TargetType)" mark in `defs/gdo/units/twister.rs` with 0457's citation.
- **The mine as a deployed unit (0426 part 1):** an object its owner selects and sees in the InfoPanel, with no commands. It isn't a structure for the Defeat Condition (`Sim`'s defeat check), and it takes no Unit Control.
  - Decide how it fits the selection code (its own selection group, Groupable true) and say so in your report.
- **The trigger (0426 part 2):** every enemy ground unit except the hover bases. Read it by base, not by kind, so a later hover unit needs nothing.
  - "Ground" is the kind's Domain through M6-ST1's accessor (`UnitDef::domain()`, which `combat::kind_domain` reads), never the base's field: a surfaced Subterranean Transport triggers a mine, a burrowed one doesn't. Not `combat::object_domain`: a landed aircraft is neither infantry nor a vehicle, so it never sets one off.
- **The blast** goes through `combat/area.rs` (read-only): ground and underground objects in the radius, air never, friendly fire on.
- **Hiding:** the mine's arm in `vision/hidden.rs`. The enemy's vision never shows or records it. Its owner sees it.
- **New `Object` state,** if any (a mine's fuse, say): hash and save it only while present, under **Object tag 19** (the numbers below it are taken, unused or promised (`artifacts/director/hash-tags.md`)).
- **`OrderKind::LayMine`** hashes as **tag 23** in `orders/hash.rs`. Master's highest is 21 (GA's VehicleDeploy); 20 and 22 stay unused.
- **The ruleset (X1):** the mine's numbers are def numbers (`ObjectDef::mine`), by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > GM`'s Acceptance, with 0426's Verification in place of D35's cases:
- lay a mine and click it: it is selected and shows its HP. A player with only mines left (no buildings) is defeated;
- a Colonists Drone drives over a mine and nothing happens. A Recruit walks in: it explodes after 8 frames, and a Drone in the radius takes damage;
- a mine killed by area fire disappears without a blast; at the placeholder numbers, a Grenadier's Attack Ground takes about 8 grenades to destroy a mine (M6-Q's note), so the test must allow for that;
- a Twister with 0 mines: F pressed three times pays 75 SC, and three mines arrive at 5, 10 and 15 s.

Also:
- **The Drone Operator's buying** is unchanged: its tests and `gdo_drones` don't move.
- **Golden** `gdo_mines`. No other golden should move.
- **Perf:** a new `mines_perf` with 100 mines under a 200-unit crowd. Record its mean and p99 beside `movement_perf`, against master's binary (`target/perf/<hash>`).
- **The computer players' games:** their units set off mines as any enemy does, but no game of theirs has a Twister, so the 192 rows stay identical.
- **QA screenshots:**
  - a minefield as its owner sees it, and the same ground as the enemy sees it;
  - a selected mine's InfoPanel;
  - the Twister's R and F live, with three buys queued.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > GM`'s Owns list, and:
- the queued-buying rule in `stock.rs` (part 4);
- the Twister's `planned` R and F in `kinds/gdo/twister.rs`, and `Planned::{LayMine, BuyMine}`;
- the mine's selection and InfoPanel in the client;
- the defeat check's line for mines.

Read-only: `combat/area.rs`, the rest of `stock.rs` and GD's drone (`tether.rs`, `sim/drones.rs`), and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- the stock and queued buying (and the Drone Operator unchanged);
- laying;
- the mine: hidden, selectable, not a structure;
- the trigger, the blast, and destroying one;
- the golden, perf, and the computer players' games;
- notes for ST2 (the burrowed Transport passes through mines) and for detection, and placeholders for the Director to file.
