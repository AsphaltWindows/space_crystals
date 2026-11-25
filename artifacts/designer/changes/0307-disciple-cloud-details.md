# 0307 — Disciple cloud: centered on the Disciple, ground objects only, shared buffer released every 0.5 s over the area

- **Date**: 2026-11-26T13:55:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/disciple.md > Spells > Cloud spell (name TBD)`

## What changed and why
User, refining 0306:
- **Position:** the cloud is centered on the Disciple, which can't move while channeling.
- **Scope:** it affects every **ground** object under it, ground units and buildings of any owner, not air units. **All anti-ground damage** on them, single-target and AoE, is absorbed.
- **Shared buffer:** all contiguously connected clouds share one damage buffer. Every **0.5 s (placeholder)** it is released as one AoE hit over the combined cloud area. Objects take shares by overlap area, armor applies once per release, and the share on empty area is lost.

## Implementation impact
Concept only, not for the current playtest. Still TBD:
- cloud size;
- drain rate;
- what ends the channel;
- pre-armor modifiers;
- what happens to the buffer when clouds split or vanish;
- name and hotkey.

## Verification
—
