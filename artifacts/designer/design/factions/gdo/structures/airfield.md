# Airfield
GDO aircraft production structure. Constructs aircraft and provides a landing/reloading zone for them.

## Faction - GlobalDefenseOrganization
## Constructed by - DeploymentCenter
## TechPrerequisite - player owns at least one Advanced Mechanics

## Produces (so far):
- Hornet: 150 Space Crystals, 320 frames (20 seconds) (placeholder).
- Dragonfly: 175 Space Crystals, 320 frames (20 seconds) (placeholder). Requires: player owns at least one Propulsion Labs.

## Aircraft Support
- **Limited ammo**: GDO aircraft carry limited ammo and **rearm at an Airfield** (the Airfield is their landing and reloading zone). Ammo per unit is on each aircraft (factions/gdo/units/README.md); return/rearm flow below.
- **Capacity: each Airfield supports at most 4 aircraft.** (User, 2026-11-23.)

### Assignment
Every GDO aircraft is **assigned** to one Airfield (rules: user, 2026-11-23):
- An aircraft is assigned to the Airfield that built it.
- An aircraft lands and reloads only at its **assigned** Airfield.
- An Airfield holds at most **4 assignments**.
- **Reassigning**: the player can move an aircraft's assignment to another Airfield, if that Airfield has room: its assignments plus the aircraft in its production queue must be **fewer than 4** (queued aircraft hold their slots, so reassigning into them is blocked). Input: select the aircraft and **right-click the other Airfield**. The aircraft is reassigned and flies there to land.
- **No manual unassigning**: the player cannot unassign an aircraft (so aircraft can't be freely shuffled around).
- **Airfield destroyed**: its aircraft become **unassigned**. An unassigned aircraft can't land to reload until the player reassigns it. **There is no automatic reassignment**, even if another Airfield has room. **Aircraft landed on it are destroyed with it** (user, 2026-11-26; Q0162): they don't take off by themselves. Designer's reading: an aircraft still **landing** (not yet down) counts as landed and is destroyed; one already **taking off** escapes.

### Production limit
An Airfield **can't queue another aircraft** if its assignments plus the aircraft already in its production queue would exceed 4. (E.g. 3 assigned + 1 queued = full.)

### Landing slots
An Airfield has **4 landing slots**, one per possible assignment, so all of its aircraft can be landed at the same time. **Layout** (designer placeholder, 2026-11-28; Q0189): a 2x2 grid, one slot near each corner of the footprint. **No free slot for a new aircraft** (Q0189; e.g. a slot taken by an aircraft reassigned elsewhere while landed here): the finished aircraft **waits inside** until a slot frees, as production behind a blocked exit (core/entities.md > Exit Doorway).

### Returning and rearming
- **Automatic return**: when an aircraft runs out of ammo, it automatically flies back to its assigned Airfield, lands and rearms.
- The automatic return **can be interrupted** by giving the aircraft any other order (e.g. to fly elsewhere). It then stays out of ammo until it returns.
- **Rearm** command (renamed from "Return to Base", user, 2026-11-28; Q0189: the name says it rearms, which is why a queued order waits for it): sends the aircraft to its assigned Airfield to land and rearm (hotkey **F**, placeholder). Right-clicking its own Airfield does the same.
- **Shift-queued orders wait for the full rearm** (user, 2026-11-28; Q0189): after a Rearm (or an automatic return), an order queued with Shift runs once rearming is **complete**. Only an order **given** during rearming (without Shift) takes off early, below.
- **Landed aircraft hold fire** (user, 2026-11-28; Q0189): they fire only in the air, even at enemies in range of their slot.
- **Landed aircraft are out of the air separation** (designer, 2026-11-28; Q0189): flying aircraft don't push landed ones off their slots, nor the reverse.
- After rearming, the aircraft **stays landed** at the Airfield. It doesn't go back to where it was unless ordered.
- **An order during rearming** (user, 2026-11-26; Q0162): the aircraft takes off at once **with the ammo rearmed so far** (a Hornet ordered off 5 s into its 10 s rearm leaves with 2 of 5 missiles).
- **New aircraft** (user, 2026-11-26; Q0162) appear **landed on a free slot**; they take off for the Airfield's rally point, and with no rally point they stay landed.
- **The Dragonfly** (a Glider) lands and takes off **vertically**, as the Specter at an air door (user, 2026-11-26; Q0162): it approaches its slot at IdleSpeed, stops over it while it lands (16 frames), and takes off straight up, leaving at IdleSpeed. Landed, it doesn't move.
- An unassigned aircraft has no base: it doesn't return automatically, and Rearm is unavailable until it is reassigned.
- **Rearm time is per unit of ammo**, and the value differs per aircraft type (RearmTimePerAmmo on each aircraft, factions/gdo/units/README.md). Rearming an aircraft takes RearmTimePerAmmo × missing ammo (user, 2026-11-23).
- **Landed aircraft are ground targets** (user, 2026-11-23): while parked on the Airfield, an aircraft can be hit only by anti-ground weapons (TargetDomain Ground or Universal, systems/combat.md). Air-only weapons (TargetDomain Air) can't hit it. Once it takes off it is an air target again. **During landing and take-off** (user, 2026-11-26; Q0162) it is an **air target**: anti-air can hit it until it has fully landed, and again from the moment it starts to take off (as the Specter at a Tunnel's air door).

Landing and take-off: **16 frames (1 s)** each (placeholder). RearmTimePerAmmo: on each aircraft.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 3x3; SymmetryType AAAA; MaxHP 500; PointArmor 1, FullArmor 9; SightRange 5 grid units; Groupable true; Power -30; BuildRadiusExtension 2 grid units.
- Panel: **Q** Hornet, **W** Dragonfly (greyed without Propulsion Labs), **X** Cancel Production, **C** Set Rally Point.
