# 0177 — Attacks from outside vision: visible shot segment, attacker revealed 1.5 s

- **Date**: 2026-11-24T22:40:00-05:00
- **Answers**: Q0017 (point 1; with 0139, Q0017 done)
- **Docs**: `core/vision.md > Attacks from outside vision` (new)

## What changed and why
- Option B, generalised: shots/tracers/projectiles are drawn only along the part of their path over tiles Visible to the viewer, so fog shots appear from the fog edge.
- User addition: an attacker outside the victim's vision is **revealed to the victim for 1.5 s (24 frames, placeholder) after each shot**; continuous fire keeps it visible. Only the attacker, not the area. Revealed = fully visible: selectable and targetable. All attackers incl. artillery. Stealth interaction TBD.

## Implementation impact
- Per-player visibility gets a timed per-object reveal (attacker → victim's owner).
- Replaces the "hit flash only, no tracer" assumption.
- Auto-targeting/acquisition can use the revealed attacker like any visible unit.

## Verification
- An enemy on unseen high ground shoots your unit: the shot appears from the fog edge; the shooter is visible for ~1.5 s after each shot; you can right-click it to attack.
- It stops firing: it disappears 1.5 s later.
