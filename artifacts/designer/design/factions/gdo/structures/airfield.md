# Airfield
GDO aircraft production structure. Constructs aircraft and provides a landing/reloading zone for them.

## Faction - GlobalDefenseOrganization
## Constructed by - DeploymentCenter
## TechPrerequisite - player owns at least one Advanced Mechanics

## Produces (so far):
- Hornet: cost/time TBD.
- Dragonfly: cost/time TBD. Requires: player owns at least one Propulsion Labs.

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
- **Airfield destroyed**: its aircraft become **unassigned**. An unassigned aircraft can't land to reload until the player reassigns it. **There is no automatic reassignment**, even if another Airfield has room.

### Production limit
An Airfield **can't queue another aircraft** if its assignments plus the aircraft already in its production queue would exceed 4. (E.g. 3 assigned + 1 queued = full.)

### Landing slots
An Airfield has **4 landing slots**, one per possible assignment, so all of its aircraft can be landed at the same time.

### Returning and rearming
- **Automatic return**: when an aircraft runs out of ammo, it automatically flies back to its assigned Airfield, lands and rearms.
- The automatic return **can be interrupted** by giving the aircraft any other order (e.g. to fly elsewhere). It then stays out of ammo until it returns.
- **Return to Base** command: sends the aircraft to its assigned Airfield to land and rearm (hotkey TBD). Right-clicking its own Airfield does the same.
- After rearming, the aircraft **stays landed** at the Airfield. It doesn't go back to where it was unless ordered.
- An unassigned aircraft has no base: it doesn't return automatically, and Return to Base is unavailable until it is reassigned.
- **Rearm time is per unit of ammo**, and the value differs per aircraft type (RearmTimePerAmmo on each aircraft, factions/gdo/units/README.md). Rearming an aircraft takes RearmTimePerAmmo × missing ammo (user, 2026-11-23).
- **Landed aircraft are ground targets** (user, 2026-11-23): while parked on the Airfield, an aircraft can be hit only by anti-ground weapons (TargetDomain Ground or Universal, systems/combat.md). Air-only weapons (TargetDomain Air) can't hit it. Once it takes off it is an air target again.

TBD: RearmTimePerAmmo values; landing/take-off time; Return to Base hotkey/slot.

(Full stats TBD)
