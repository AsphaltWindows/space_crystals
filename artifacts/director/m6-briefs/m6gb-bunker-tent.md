# Task M6-GB: The Bunker and the First Aid Tent

First read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md` (which starts from `m5-common.md`). Its rules apply to you; where this brief differs, this brief wins.
- **Your slice's spec:** `artifacts/director/m6-plan.md > §B > GB: The Bunker and the First Aid Tent`, with T99 (`> §D > T99`).

Read both before you start. Two design changes since the plan **replace its way of building** and one placeholder (below). Where they and the plan disagree, they win.

## Goal
- **The Field Engineer builds a Bunker or a First Aid Tent** on any Buildable ground (W Construct, then Q or W). It builds **as an Agent builds a Tunnel**: it goes inside the site, alone, until the building is complete.
- **The Bunker:**
  - up to 4 GDO infantry enter it and fire their own weapons from it;
  - A ejects all of them;
  - they come out if it is destroyed.
- **The First Aid Tent** heals own and allied infantry within 3 GU by 2 HP/s. Tents don't stack.

## Running beside you
Filled in at launch.

## The design: answers since the plan
- **The mechanics docs** (0475, 0476, 0477): `systems/mechanics/garrisoning.md` makes the Bunker a **Garrison structure**, a parameter set of the one garrisoning module the Cults Garrison uses. **0477: when a Bunker is destroyed, its occupants all come out at once inside its former 2x2 footprint,** spread over it, and units that overlap ignore each other until they separate. This replaces "around its footprint". `systems/mechanics/hp_restoration.md` makes the First Aid Tent a **Healing area**.
Read `factions/gdo/units/field_engineer.md > Building` (new since the plan), `factions/gdo/structures/bunker.md` and `factions/gdo/structures/first_aid_tent.md` as they stand.
- **0429 (Q0163): the Field Engineer builds inside the site, alone, like an Agent.** Read `artifacts/designer/changes/0429-field-engineer-builds-inside.md`. This **replaces** D37 and T99's building ("as a Recruit does: from outside, shared, `shares_builds: true`"). The flow, `field_engineer.md > Building`:
  1. **The order** is paid when it's given, and needs the Space Crystals then. It's refunded in full if the order ends before construction starts: another order, or the Engineer killed.
  2. **It walks to the site,** and goes inside: untargetable and unseen. The site follows the ConstructionHP Rule (10% HP at the start; damage taken stays), as the Agent's Tunnel does.
  3. **Completed:** the Engineer comes back out, and idles. It doesn't garrison the Bunker by itself. It comes out through the Bunker's door (side B), or at the Tent's edge on the side it walked in from (the designer's placeholder).
  4. **The site destroyed:** the Engineer comes out alive at the footprint's centre, and the cost is lost.
  5. **Cancelled** (X, on both buildings): the Engineer comes out at the centre, with **75% refunded, rounded down**: 75 of the Bunker's 100, and 56 of the Tent's 75.
  - While under construction the site does nothing but Cancel. It counts as a structure for the Defeat Condition.
  - **One Engineer per site,** no help. **The Field Engineer becomes Groupable false**, as the Agent is.
- **0430 (Q0164):** read `artifacts/designer/changes/0430-bunker-shield-tent-healing.md`.
  1. **A shielded Peacekeeper that enters a Bunker lowers its shield** (the 6-frame toggle), then fires from inside, and comes out unshielded. This **replaces** D38's "stays shielded inside, so it doesn't fire".
  2. **The Tent heals only infantry on the map,** not units inside a building. (D38, now the design.)
  3. **A Tent under construction heals nobody.** (D38, now the design.)
- **"No abilities from inside"** (`bunker.md > Garrison`): a Drone Operator can't launch its drone, a Rocketman can't Target Lock, and a Peacekeeper can't raise its Riot Shield. They only fire their weapons.
- **Units inside see with the Bunker's SightRange,** as the Garrison's do.
- **Anything the docs don't settle:** a placeholder, cited as m6-common says, and a line in your report. The Director files the question.

## Code
Follow `m6-plan.md > GB`'s Code list and T99, with 0429's building in place of T99's shared one:
- **Build on the Agent's site, not the Recruits'.** M4-E's `construction::site` holds the Agent inside the site (`network::Place::Site`, `Builder` in `construction/site.rs`, `sim/sites.rs`).
  - Its completion sends the builder into the network (`BuilderGoes::Network`).
  - The Engineer comes out on the map instead. Add a third `BuilderGoes` arm, out through the building's door (the Bunker's side B, F3's evacuation spots), or at its edge on the side it came from (the Tent).
  - Cancel and destruction already put the builder at the footprint's centre (`Sim::release_builders`). Check that the Engineer takes that path.
  - The Cults' 75% refund is `SitePrice::Crystals`'s `cancelled`. Check that the Bunker's 100 gives 75, and the Tent's 75 gives 56.
- **The Field Engineer's `WorkerDef`:** `builds: [Bunker, FirstAidTent]`, and `shares_builds: false`.
  - It gathers nothing, so `WorkerDef::crystals` becomes an `Option`, as T99 says. The Recruit's, the Agent's and the Drone's defs gain `Some(..)`.
  - Its Groupable becomes false (0429). Update its def's test and the controls doc.
- **The recipes:** move the Bunker's and the Tent's `RECIPE`s from `defs::PLANNED_RECIPES` into the Field Engineer's `builds`, and turn the Construct menu's `Planned` entries into their `Kind`s.
- **Placement:** any Buildable ground, with no build-area rule (`bunker.md > Placement`; the Worker-Built Structures rules, `core/entities.md > Placement Validation`).
- **The Bunker:**
  - F3's `holds`, with the door on B, admitting the owner's GDO infantry, up to 4;
  - N2's `occupants_fire`, with no stand-in weapon, no range bonus and no `bonus_for`;
  - entering lowers a raised Riot Shield (0430 part 1), and no ability is offered inside;
  - its panel: A Eject All, and X Cancel while it's being built.
- **The Tent:** `StructureDef::heals` (`heal.rs`), a step after the buffs: 0.125 HP a tick to each own or allied infantry unit on the map whose centre is within 3 GU of a complete Tent, the highest rate among the Tents, up to `max_hp_with`.
- **New `Object` state,** if any: hash and save it only while present, under **Object tag 20** (the numbers below it are taken, unused or promised: `artifacts/director/hash-tags.md`).
- **A new `OrderKind`,** if any, hashes as **tag 26** in `orders/hash.rs`. Master's highest is 21; the numbers between are unused or promised to other slices (`artifacts/director/hash-tags.md`).
- **The ruleset (X1):** the new def fields are structure, and the numbers are def numbers, by m6-common's checklist.
- **The controls doc:** regenerate it.

## Acceptance
`m6-plan.md > GB`'s Acceptance, with 0429's building in place of D37's:
- **0429's Verification:**
  - an Engineer builds a Bunker: it walks there and vanishes, the Bunker rises from 10% HP over 20 s, enemies can't target the Engineer, and it walks out of the door when done;
  - two Engineers selected make two groups. W, Q and a placement send only the active one. Tab, and the same for the second;
  - a site destroyed mid-build: the Engineer appears at its centre;
  - a cancelled Bunker: 75 SC back, and the Engineer at the centre. A cancelled Tent: 56 back;
  - an Engineer's build order cancelled before it arrives: the whole cost back.
- **Drop the plan's "Two Engineers build a Bunker in half the time"** (D37). Keep "A Peacekeeper standing on the site holds the start back", if the Worker-Built Structures rules still say so.
- **0430's Verification:**
  - a shielded Peacekeeper enters a Bunker next to enemies: after 6 frames it fires; ejected, it is unshielded;
  - wounded Peacekeepers in a Bunker beside a Tent don't heal; ejected, they heal at 2 HP/s;
  - a Tent mid-construction next to wounded infantry: no healing until it is complete.
- **The Defeat Condition:** a player whose only structure is a Bunker site isn't defeated.
- **Golden** `gdo_bunker`. No other golden should move. If the Field Engineer's Groupable or the optional `crystals` moves one, find out why.
- **The computer players' games** are unchanged: none builds these.
- **QA screenshots:** the Construct menu live; an Engineer's site under construction; a Bunker firing; the Tent's radius.
- **Checks:** m5-common's "Done means".

## What you own
`m6-plan.md > GB`'s Owns list, and:
- the new `BuilderGoes` arm and its completion in `sim/sites.rs`;
- the Bunker's and the Tent's lines in `defs::PLANNED_RECIPES`, and the Field Engineer's def, Construct menu and client kind.

Read-only: `holding.rs`, `combat/occupants.rs`, the Agent's and the Recruits' site rules in `construction/site.rs`, and every other slice's files (`m6-plan.md > §C`).

## Landed since the plan (filled in at launch)
TBD.

## Report
At most 12 lines:
- building: the site, completion, cancel and destruction;
- the Bunker: entering, firing, the shield, ejecting and evacuation;
- the Tent's healing;
- the golden, and the computer players' games;
- placeholders for the Director to file.
