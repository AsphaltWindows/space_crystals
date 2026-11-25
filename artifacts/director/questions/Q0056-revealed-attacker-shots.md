# Q0056 — Should a revealed attacker's shots be drawn from the attacker?

- **Date**: 2026-11-24T20:30:42-05:00
- **Blocking**: no — proceeding on assumption: as built, a shot is drawn only where it crosses tiles Visible to the player, even when its attacker is revealed (option A).
- **Docs**: `core/vision.md > Attacks from outside vision` (Shots are drawn where they travel through vision; The attacker is revealed)

## Question
When an attacker in the fog is revealed by its own shot, should that shot be drawn all the way from the attacker? Or should it be drawn only over the tiles Visible to the player, as now, which leaves a gap between the revealed unit and its shot?

## Context
The two rules, quoted from `core/vision.md > Attacks from outside vision`:
- "**Shots are drawn where they travel through vision**: a shot, tracer or projectile is visible to a player only along the part of its path that crosses tiles **Visible** to that player. A shot fired from the fog at your unit is seen coming out of the fog edge, which shows its direction."
- "**The attacker is revealed**: a unit or structure that attacks one of your units or structures while outside your vision becomes **visible to you** for **1.5 seconds (24 frames), placeholder**, after each shot ... Only the attacker itself is revealed, not the area around it."

M3-L1 built both rules as written. The revealed attacker is drawn in the fog, but its tracer starts only where Visible tiles begin.

In M3-K2's QA walkthrough, four Peacekeepers fire from the fog at a Bio Portal. Each one shows, but its tracer starts about 140 px short of it at the default zoom, so the shot looks as if it doesn't come from the unit.

## Options
- **A. As built.** Shots are drawn only over Visible tiles, revealed attacker or not. No change; the gap stays.
- **B. Draw a revealed attacker's shots along their whole path**, from the attacker to the target, while it is revealed. Other shots from the fog stay as now.
  - This is a small client change: the shot clipping treats the revealed attacker's own shots as visible.
  - It shows nothing the player doesn't already see, since the attacker is visible anyway.
- **C. Also reveal the ground under the attacker** (a small area), so that the shot is drawn from it. This changes vision: other objects there would show too, against "Only the attacker itself is revealed".
