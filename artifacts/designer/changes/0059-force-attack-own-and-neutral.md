# 0059 — Explicit Attack may target own and neutral objects; right-click doesn't

- **Date**: 2026-11-23T20:26:12-05:00
- **Answers**: Q0014 (point 1; points 2 and 3 still open)
- **Docs**: `systems/controls/commands.md > Attack`

## What changed and why
User decision: **allow force-attack**. An explicit Attack order can target own and neutral destructible objects (e.g. clearing your own blocking building, destroying a neutral Syndicate expansion so the enemy can't claim it). Right-click is unchanged: attack enemies; move to or follow own and neutral objects. Auto-targeting never picks own or neutral objects.

## Implementation impact
Matches the current behaviour.

## Verification
- Press Attack, click an own building: units attack it. Right-click the same building: they move to it.
- Right-click a neutral expansion: units move to it. Attack command + click: they attack it.
