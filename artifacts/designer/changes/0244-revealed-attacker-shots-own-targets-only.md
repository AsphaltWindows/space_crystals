# 0244 — Only shots at your own objects are drawn whole from a revealed attacker

- **Date**: 2026-11-25T10:35:00-05:00
- **Answers**: Q0075 (narrows 0206)
- **Docs**: `core/vision.md > Attacks from outside vision` (Shots are drawn where they travel through vision)

## What changed and why
User: a fully drawn shot reveals nothing only when **you** are the one attacked, because then both the attacker (revealed) and the target (yours) are visible. A revealed attacker's shots at anything else (a neutral, a third player's object) are drawn only over your Visible tiles, like any shot from the fog. This is stricter than option B, which would also have drawn whole shots at any target visible to you.

## Implementation impact
- Client shot clipping: draw the whole path only if the attacker is revealed to the viewer **and** the shot's target is owned by the viewer.
- Everything else is clipped to Visible tiles (changes M3-J3's literal reading).

## Verification
In a three-player game, a revealed enemy then shoots a third player's unit hidden in the fog. Only the part of the shot inside your vision is drawn.
