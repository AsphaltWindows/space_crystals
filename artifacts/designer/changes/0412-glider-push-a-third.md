# 0412 — Gliders push apart at a third of the HoverCraft strength

- **Date**: 2026-11-26T16:20:00-05:00
- **Answers**: Q0152
- **Docs**: `systems/units.md > Air Collision` ("Gliders push apart slowly")

## What changed and why
The user chose to try **a third** of the HoverCraft repulsion strength for gliders (was a quarter, 0405), a little faster spreading while attack runs still barely bend. The strength is the same in every activity (not option d). 0405's Verification "drift apart over a few seconds" is replaced: spreading is expected to take roughly 10 s or more.

## Implementation impact
Change the glider repulsion factor from 0.25 to 1/3 of the HoverCraft strength. SeparationRadius values (0405) unchanged. Still a placeholder for playtests.

## Verification
- Six Sparrows sent to one point drift apart while circling and end about 48 su apart, somewhat faster than at a quarter (the Director can report the new time).
- Two Sparrows passing side by side on one target still land their hits near the target's centre; runs are only slightly bent.
