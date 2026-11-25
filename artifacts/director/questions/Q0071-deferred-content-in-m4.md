# Q0071 — Deferred Syndicate content: which items will get their numbers or design in time for M4?

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: M4 builds only what the design specifies today (the Tunnel economy and network, the Agent, the Hideout, and the Guard behind the Optics Lab). Every item below stays out of M4 until its numbers and rules arrive.
- **Docs**: `factions/syndicate/units.md > Grenadier`, `> Assault Bike`, `> Shadebringer`, `> Laser Battery`, `> Subterranean Transport`, `> Phantom`; `factions/syndicate/structures.md > Optics Lab` to `> Orbital Command`, `> Laser Turret`; `factions/syndicate/tunnels.md > Production and Tunnel Space`, `> Tunnel > SymmetryType - ABCD` (the Tier 3 air doors); `factions/gdo/units.md > Artillery Rig > Modes`, `> Twister > Mines`; `factions/colonists/units.md > Scout`; `systems/units.md > DrillUnit`, `> Underground Collision`; `systems/combat.md > AoEDamage`; design changes 0187–0196 and 0207–0228.

## Question
Which of the deferred items below will get their numbers or design soon enough to be planned into M4, and in what order should they come?

M4 is planned in six waves. An item whose numbers and rules arrive before the fifth wave starts can still become an M4 slice, scheduled after the expansions (wave 4) and the tech prerequisites (wave 5). Anything later moves to a later milestone. Each item lists what it is missing. **Gap** marks a rule that is missing or contradictory, found while reading 0207–0228.

1. **The Optics Lab.** Its placeholder stats are asked in Q0059, which is blocking. It now gates the Guard, the Laser Turret, the Laser Battery (0210) and the Phantom (0226).
2. **The Laser Turret** (upright and pop-up).
   - Its stats: "(Full stats TBD — cost, build time, HP, raised/underground armor, range, damage, rise/retract timing.)"
   - Its hotkey.
   - Whether a Tunnel building one uses its single operation slot: "Whether it occupies the Tunnel's single operation slot (like an expansion) is TBD."

   It doesn't need detection. Underground it is "hidden like an expansion", and nothing reveals it until detection exists.
3. **The Grenadier, with the Chemical Plant.**
   - Its numbers: "All numbers (HP, armor, sight, speed, TunnelSpaceCost, damage, AoE radius, range, projectile speed, attack timings, cost, build time): TBD."
   - The Chemical Plant's "(Full stats TBD)".

   It also needs combat work that no faction has yet: projectiles, area damage and Attack Ground.
   - **Gap (area damage underground).** The two docs disagree:
     - `systems/combat.md > AoEDamage` says "Ground AoE hits Ground and surfaced Underground units";
     - 0218 and 0219 (`systems/units.md > DrillUnit`) say that while a DrillUnit is underground and hidden, "area (AoE) damage still affects it".

     Which is right, and which AoE domains reach a DrillUnit underground: Ground, Universal, or both? Does area damage also reach hidden underground **structures**: Syndicate expansions, a pop-up Laser Turret while underground, and the GDO's mines?
4. **The Vehicle Assembly and the Assault Bike.**
   - The Assembly: "(Full stats TBD)", and "Costs, times and panel layout: TBD".
   - The Bike: "All numbers (HP, armor, sight, speed, turn radius, turret turn angle and rate, TunnelSpaceCost, damage, AoE radius, range, projectile speed, timings, cost, build time): TBD."

   Both also need vehicle movement (the fixed turn radius) and turret arcs. Neither is built for any faction yet.
5. **The Laser Battery** (0207–0214).
   - Its numbers: "All numbers (HP, armor, sight, speed, TunnelSpaceCost, battery, damage, range, timings, cost, build time): TBD."
   - Its deploy and undeploy times: "TBD: deploy/undeploy time."
   - Refract, "to be designed together with satellites".

   The GDO's Artillery Rig uses the same deploy model, and its times are TBD too ("values TBD"). It isn't built yet.
   - **To confirm.** The Deploy text marks two points as assumed:
     - undeploying works like deploying: it can't be cancelled, and Deploy is available again only once it completes;
     - a held order that the deployed unit can't carry out is dropped with the usual unavailable-command feedback.
6. **The Subterranean Transport** (0215–0222).
   - What the doc leaves open: "TBD: capacity (and whether it counts each infantry unit's TunnelSpaceCost); burrow/surface time; load/unload commands and the panel; all numbers."
   - Its name, which is tentative.
   - The Forge's "(Full stats TBD)". The Forge is its prerequisite (0221).
   - **To confirm** (marked assumed): the DrillUnit base, and no attack.

   It needs vehicle movement, the DrillUnit's underground movement, and a new loading and unloading mechanic. It also needs either detection (item 9) or a decision to build it without detection.
   - **Gap (Tunnel Space aboard).** The two docs disagree:
     - 0222 says that inside the network "only the transport's own Tunnel space counts";
     - `tunnels.md > Production and Tunnel Space` says a unit's Tunnel Space "counts as used from that moment — through production and for as long as the unit lives".

     Do infantry aboard stop counting against the cap? If so, only while the Transport is in the network, or on the surface too?
   - **Gap (underground structures).** Underground units "do not collide with each other" (`systems/units.md > Underground Collision`). Does a burrowed Transport also pass through underground structures: expansions, a pop-up Turret while underground, the GDO's mines? And does its Surface check ("on top of a building") count them as buildings? If a hidden enemy expansion stopped it from surfacing, that would reveal the expansion.
   - **Gap (surfacing among units).** The general surfacing rule has two parts the Transport's Surface text doesn't mention:
     - it refuses a spot where the Silhouette "would overlap an above-ground unit" (`> Underground Collision > Surfacing`);
     - it reserves the footprint once surfacing starts (`> Reserved while surfacing`).

     The Transport's Surface text names only buildings and terrain. Assumed: both general rules apply to it.
   - **Gap (no detection yet).** Until detection is designed, enemies can't see or target an underground Transport. The only counter is an area attack on a guessed spot (0218). Should it wait for detection, or be built under that rule?
7. **The Shadebringer, with the Vault** (0189–0196).
   - Its numbers: "All numbers (HP, armor, sight, speed, turn radius, TunnelSpaceCost, X, Y, Z, smoke duration, cost, build time) and hotkeys: TBD."
   - Its toggle rules. 0189 says not to build it "until the rules are settled".
   - The Vault's stats.

   Its smoke would be the game's first sight-blocking mechanic.
8. **The other expansions.** Each is "(Full stats TBD)":
   - the Forge, now the Transport's prerequisite;
   - the Null Chamber, whose exit buff is also open: "TBD: duration; SightRange bonus; damage-reduction percentage; whether the buff ends abruptly or decays gradually; whether the buff ... must first be unlocked by research";
   - the Aerospace Assembly, which produces the Phantom (item 11);
   - the Orbital Command.
9. **Detection.** "Detection itself is not designed yet (detectors: which units/structures, range; to be designed, likely together with stealth such as the Colonists Scout's)."
   - It is what would reveal every hidden thing:
     - Syndicate expansions;
     - the pop-up Turret while underground;
     - DrillUnits underground (0219);
     - the GDO's mines;
     - the Scout, if it gets stealth.
   - Needed:
     - which units and structures detect, and at what range;
     - whether detecting needs sight as well;
     - whether a detected object is remembered once it is no longer detected;
     - how long it stays revealed.
10. **Research and notifications.**
    - Whether the Optics Lab "also hosts laser research", and Supplies spent on research.
    - The notification system, which the blocked production queue needs (`tunnels.md > Production and Tunnel Space`: "notification system not yet designed").
11. **The Phantom** (0226–0228), with aircraft and the Tier 3 air doors.
    - Its numbers: "TBD: all numbers (HP, armor, sight, speed and handling, TunnelSpaceCost, damage, range, timings, cost, build time); hotkeys."
    - Its exact turret cone: "moderate, 60–90°", "the **total** cone, i.e. 30–45° to each side of its facing; exact value TBD".
    - The Aerospace Assembly's "(Full stats TBD)". It is a Tier 3 expansion behind a Forge.
    - The air doors' rules, asked in Q0064 point 11.
    - **To confirm** (marked assumed): Groupable; a FullyConnected laser, "like other Syndicate lasers"; SingleTarget.

    It needs air units, which no faction has yet (air movement and collision), and turret arcs.

**Proposal:**
- Settle item 1 now (Q0059).
- Then items 2 and 3, in that order, if their numbers can arrive before M4's fifth wave:
  - the Laser Turret is the Syndicate's only static defence, and it uses machinery M4 builds anyway: expansions, and Agent-built structures;
  - the Grenadier is Tier 1 infantry, but it first needs a combat spike for projectiles and area damage. Its numbers and the area-damage gap therefore help most if they come early.
- Items 4–6 wait for vehicle movement, after M4. Item 6 also waits for detection (item 9), or for a decision on that gap.
- Item 11 waits for air units, after M4.
- Items 7, 8 and 10 come after that.

## Context
M4 builds what the design specifies: the Tunnel economy and network, the Agent, the Hideout, Agent-built Tunnels, expansions and upgrades, losing Tunnels, and the Guard behind the Optics Lab (Q0059). Everything above is deferred because it is TBD.

Design changes 0207–0228 arrived while M4 was being planned:
- the Laser Battery, and its deploy model (0210–0214);
- the Subterranean Transport, which burrows and surfaces on the same model (0215–0222);
- every DrillUnit hidden underground unless detected (0219);
- the Phantom, an aircraft that uses the Tier 3 air doors (0224, 0226–0228).

All of these wait for vehicles or aircraft. The gaps above were found while folding them into the plan. None blocks M4, but each blocks its own item.

## Options
- **A. As proposed.** Items 1–3 are planned into M4 if their numbers arrive before its fifth wave, and the rest go to later milestones. M4 then gains up to three slices: the Laser Turret, a combat spike, and the Grenadier.
- **B. Placeholders for items you name** (as 0166 did for the Colonists), built in M4 and tuned later. Their gaps still need answers. The vehicles (items 4–6) would still wait for vehicle movement, and the Phantom (item 11) for air units. M4 includes neither.
- **C. Another set or order** that you give.
- **D. Nothing more in M4.** It ends with the Tunnel economy and the Guard, and everything above moves to later milestones.

Whichever you choose, the gaps in items 3, 5 and 6 can be answered whenever convenient.
