# 0434 — The Prism: panel, Thrust aiming and queueing, orders held during launch

- **Date**: 2026-11-26T19:50:00-05:00
- **Answers**: Q0168
- **Docs**: `factions/syndicate/units/prism.md > Placeholder stats`

## What changed and why
Answered from the docs and the standing permission on placeholders:
- **1 (a)**: panel **Q** Move, **R** Thrust, **X** Stop; **A** Attack while armed (0397/0399: A is Attack wherever a unit can attack).
- **2 (b)**: Thrust is **targeted** (click a point; always the full 3 GU toward it) and, as every command, **Shift-queues** (systems/controls/commands.md: commands are queued by holding Shift, hotkey + target click included). Each queued thrust spends its charge when it starts.
- **3 (a)**: orders given during the 4 s launch/unfold are **held** (last one wins), as in the vehicle deploy model.

## Implementation impact
2 differs from the placeholder (not queueable): thrusts now queue with Shift. 1 and 3 match.

## Verification
- An unarmed Prism shows Q, R, X; armed by a relay it also shows A.
- R, click a point 10 GU away: it thrusts 3 GU toward it. R + Shift-clicks on three points: three thrusts in a row, then R greys out.
- Give a Move while it is launching: it moves there once unfolded.
