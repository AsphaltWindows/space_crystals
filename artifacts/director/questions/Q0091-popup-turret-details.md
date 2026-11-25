# Q0091 — Pop-up Laser Turret details the docs leave open: its Tunnel dying with a unit on top, a target while it retracts, a Tunnel-built turret's ground and tier, and when its site takes Tunnel Space

- **Date**: 2026-11-25T13:47:05-05:00
- **Blocking**: no. Proceeding as built (option (a) for each point) by slice M4-S2.
- **Docs**: `factions/syndicate/structures/laser_turret.md` > `Upright vs Pop-up (by location)`, `> Conversion`, `> Construction` and `> Placeholder stats`; `factions/syndicate/tunnels.md > Tunnel Space`.

## Question
M4-S2 built the pop-up turret by 0296–0298. Five details are left that the docs don't settle:
1. **A lowered pop-up whose Tunnel is destroyed while a unit stands on it.** Conversion makes it an upright turret, but "it can't rise while any unit stands over it". What is it until the spot is clear?
2. **A new target while it retracts.** Does it finish retracting before it rises again, or reverse at once?
3. **A Tunnel-built turret's ground.** It is "placed within that Tunnel's Tunnel Area like an expansion". Does it need only Drillable ground, like an expansion, or ground an upright turret could stand on?
4. **Its tier in the ExpandMenu.** The doc gives it no Tier Requirement. Can a Tier 1 Tunnel build it?
5. **When a turret site takes its ¼ Tunnel Space.** From the site's start, or at completion?

## Context
- **Point 1, as built:** it stays lowered (hidden from enemies, at 6/6, not blocking) until the spot is clear, then comes up as an upright turret. An enemy that stands on it keeps it hidden and down for as long as it stays. (An own idle unit is pushed off first, as for any rise.)
- **Point 2, as built:** it finishes retracting (up to 8 frames), then rises (8 frames). So up to 16 frames pass before it can aim, against 8 from fully lowered.
- **Point 3, as built:** Buildable and flat ground, as well as Drillable, inside the Tunnel's area. It comes up to the surface, and it becomes an upright turret if its Tunnel is destroyed, so it never stands where an upright one couldn't be built.
- **Point 4, as built:** Tier 1. Only the Optics Lab is required.
- **Point 5, as built:**
  - a Tunnel's turret site takes its ¼ from its first frame, like an expansion's site, since it is underground;
  - an Agent's site inside a Tunnel Area takes it at completion, when the turret becomes a pop-up. Until then it is a surface site, like a Tunnel site.

## Options
1. **Its Tunnel dies with a unit on top:**
   - (a) as built: hidden and at 6/6 until the spot is clear;
   - (b) visible at once, with its raised armor, but still down: not blocking, and not firing until the spot is clear;
   - (c) it comes up at once, and any unit over it, enemies included, is pushed off.
2. **A target while it retracts:**
   - (a) as built: it finishes retracting, then rises;
   - (b) it reverses at once, taking only the frames it had retracted to come back up.
3. **A Tunnel-built turret's ground:**
   - (a) as built: Buildable, flat and Drillable;
   - (b) Drillable only, like an expansion. Then a rule is needed for a turret that must become upright where an upright one couldn't be built.
4. **The tier:**
   - (a) as built: Tier 1;
   - (b) a Tier Requirement (which tier?).
5. **When its site takes Tunnel Space:**
   - (a) as built: a Tunnel's site from its start, an Agent's at completion;
   - (b) both at completion;
   - (c) both from the start.

Each option is a small change in the sim's turret code.
