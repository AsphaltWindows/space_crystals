# Q0075 — A revealed attacker shooting at a target the player can't see

- **Date**: 2026-11-24T23:08:54-05:00
- **Blocking**: no — proceeding on assumption: A (as built in M3-J3)
- **Docs**: `core/vision.md > Attacks from outside vision` (0206)

## Question
0206: "while an attacker in the fog is revealed to a player, that attacker's own shots are drawn from the attacker to the target, with no gap at the fog edge. This reveals nothing extra." When that revealed attacker shoots at a target the player can't see, should its shot still be drawn whole? Examples are a neutral, or a third player's unit, in the fog.

## Context
- **As built.** M3-J3 built the literal reading: the shot is drawn whole whenever its attacker is revealed.
- **The usual case.** The target is the player's own object, which is visible, so nothing extra is revealed.
- **This case.** If the target is hidden in the fog, the drawn shot shows where it is, which is something extra.
- **When it can happen.** Only when a revealed attacker shoots at a neutral or, in games of three or more players, at another player's object.

## Options
- **A. Literal, as built.** Drawn whole. A shot at a target in the fog shows the target's position.
- **B. 0206's exception applies only while the target is visible to the player, or is the player's own.** Otherwise the shot is clipped to Visible tiles, like any other shot from the fog. Nothing extra is revealed. This is a small change in the client's shot drawing.
