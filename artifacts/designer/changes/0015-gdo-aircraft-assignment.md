# 0015 — GDO aircraft assignment to Airfields; fighter is a Glider with 5 missiles

- **Date**: 2026-11-23T14:07:42-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures.md > Airfield > Aircraft Support > Assignment`, `> Production limit` (new); `factions/gdo/units.md > Air-to-Air Fighter (name TBD)` (UnitBase, Ammo)

## What changed and why
The user defined how aircraft are tied to Airfields:
- Every GDO aircraft is **assigned** to one Airfield, which starts as the one that built it. It lands and reloads only there. Each Airfield holds **at most 4 assignments**.
- The player can **reassign** an aircraft to another Airfield that has fewer than 4 assignments. There is **no manual unassigning**, so aircraft can't be shuffled freely.
- **Airfield destroyed:** its aircraft become unassigned and can't land to reload until assigned again.
- **Production limit:** an Airfield can't queue an aircraft if assignments + queued would exceed 4.
- **Air-to-Air Fighter:** UnitBase **Glider**; ammo **5 missiles**, then it reloads at its assigned Airfield.

## Implementation impact
Extends 0014; still design stubs. Not ready to build until the rest is decided: the reassign command, whether unassigned aircraft auto-assign, queued aircraft vs reassignment overflow, the rearm flow, the hovercraft's ammo, stats and costs.

## Verification
(For later, once buildable.) An Airfield with 3 aircraft and 1 queued refuses to queue a 5th. Destroying an Airfield leaves its aircraft unable to reload until reassigned. Reassigning to a full Airfield (4 assignments) is refused.
