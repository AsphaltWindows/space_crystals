# 0017 — GDO aircraft: automatic return, Return to Base, landing slots, reassign input

- **Date**: 2026-11-23T14:12:24-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures.md > Airfield > Aircraft Support` (Assignment updated; new Landing slots, Returning and rearming)

## What changed and why
User decisions on the rearm flow:
- **Out of ammo:** the aircraft automatically flies back to its assigned Airfield, lands and rearms. **Any order interrupts** the return, and it stays empty until it returns.
- **Return to Base** command sends it home to land and rearm (hotkey TBD). Right-clicking its own Airfield does the same.
- **After rearming, it stays landed.** It doesn't go back to where it was unless ordered.
- **4 landing slots** per Airfield, so all assigned aircraft can land at once.
- **Reassign input:** select aircraft and right-click another Airfield. Allowed only if that Airfield's assignments + queued aircraft < 4, so queued aircraft hold their slots.
- **No automatic reassignment** of unassigned aircraft (after an Airfield is destroyed). An unassigned aircraft doesn't auto-return, and Return to Base is unavailable for it.

## Implementation impact
Extends 0014–0016; still stubs. TBD: rearm time, landing/take-off time, whether landed aircraft are air or ground targets, Return to Base panel slot.

## Verification
(For later.) A Hovercraft fires its 5th missile and heads home by itself. Ordering it elsewhere mid-way makes it comply while staying empty. Once home it lands, rearms and stays. Right-clicking a different Airfield with 2 assigned + 2 queued is refused. After its Airfield dies, the aircraft doesn't return when empty and Return to Base is disabled.
