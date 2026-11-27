# Q0187 — The Null Chamber's exit buff: its SightRange outside vision, and leaving with no chamber left

- **Date**: 2026-11-27T17:36:45-05:00
- **Blocking**: no. M6-SN is built on the assumptions below, and a sweep changes them if the answer differs:
  - (1) the +2 grid units widen only what the unit sees, not the range at which it picks targets;
  - (2) a unit whose turn is done leaves buffed only if its player still owns a working Null Chamber when it leaves.
- **Docs**:
  - `factions/syndicate/structures/null_chamber.md` (the exit buff; `> Placeholder stats`);
  - `systems/controls/command_behaviors.md > AttackMove` ("Scan for ValidTarget enemies within SightRange");
  - `factions/syndicate/tunnels.md` (the network).

## Question
1. **Does the buff's "+2 grid units SightRange" also widen the range at which the unit looks for targets?** AttackMove scans "within SightRange", and so do Patrol and an idle unit's automatic targeting.
2. **A unit finishes its 2 s turn, but its player has no working Null Chamber when it leaves** (the chamber was destroyed meanwhile). Does it leave buffed?

## Context
M6-SN built the Null Chamber.
- **On point 1,** the buff's extra sight is applied to the unit's vision only. The unit sees 2 grid units farther, so its owner sees more of the map. Its AttackMove, Patrol and idle targeting still use its unbuffed SightRange.
  - A literal reading of "Increased SightRange" would raise every use of the stat. A buffed unit would then also pick targets 2 grid units farther away (out to its weapon range, plus the existing path rule for enemies out of weapon range).
- **On point 2,** the doc says a unit "that leaves before its turn leaves unbuffed", but says nothing about a chamber lost after the turn is done.
  - As built, the finished turn is kept while the unit stays inside. It gives the buff only if a working chamber exists when the unit leaves, so a chamber rebuilt later restores the buff without a new turn.

## Options
**Point 1:**
- **(a)** Vision only (as built).
- **(b)** Every use of SightRange: vision, AttackMove, Patrol and idle targeting. This is a small change: one accessor for the buffed range, which those behaviours read.

**Point 2:**
- **(a)** A finished turn always buffs on leaving, chamber or not. This is simplest; the turn has been "paid for".
- **(b)** Buffed only if a working chamber exists when it leaves (as built).
- **(c)** Losing the last chamber clears every finished turn, so a rebuilt chamber starts the turns over.
